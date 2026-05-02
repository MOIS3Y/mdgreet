use crate::config;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use zbus::{Connection, proxy};

#[proxy(
    interface = "org.freedesktop.Accounts",
    default_service = "org.freedesktop.Accounts",
    default_path = "/org/freedesktop/Accounts"
)]
trait Accounts {
    fn list_cached_users(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.Accounts.User",
    default_service = "org.freedesktop.Accounts"
)]
trait User {
    #[zbus(property)]
    fn user_name(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn real_name(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn home_directory(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn icon_file(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn uid(&self) -> zbus::Result<u64>;
}

#[derive(Debug, Clone)]
pub struct SystemUser {
    pub login: String,
    pub pretty_name: String,
    #[allow(dead_code)]
    pub home_dir: PathBuf,
    #[allow(dead_code)]
    pub avatar_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct SystemSession {
    pub name: String,
    pub exec: String,
}

pub async fn get_users() -> Result<Vec<SystemUser>> {
    let conn = Connection::system()
        .await
        .context("systems: failed to connect to system D-Bus")?;
    let accounts_proxy = AccountsProxy::new(&conn).await?;

    let user_paths = accounts_proxy
        .list_cached_users()
        .await
        .context("systems: failed to list cached users")?;

    let mut users = Vec::new();

    for path in user_paths {
        let user_proxy = UserProxy::builder(&conn).path(path)?.build().await?;

        let uid = user_proxy.uid().await.unwrap_or(0);

        if uid >= 1000 && uid < 65534 {
            let login = user_proxy.user_name().await.unwrap_or_default();
            let pretty_name = user_proxy.real_name().await.unwrap_or_default();
            let home_dir = PathBuf::from(user_proxy.home_directory().await.unwrap_or_default());
            let icon_file = user_proxy
                .icon_file()
                .await
                .ok()
                .filter(|s| !s.is_empty())
                .map(PathBuf::from);

            users.push(SystemUser {
                login,
                pretty_name,
                home_dir,
                avatar_path: icon_file,
            });
        }
    }

    users.sort_by(|a, b| a.login.cmp(&b.login));
    Ok(users)
}

pub fn get_sessions() -> Vec<SystemSession> {
    let mut sessions = Vec::new();

    // NixOS and other XDG compliant systems use XDG_DATA_DIRS
    let xdg_data_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());

    for data_dir in xdg_data_dirs.split(':') {
        if data_dir.is_empty() {
            continue;
        }
        let base_path = std::path::Path::new(data_dir);

        // Scan Wayland sessions
        scan_session_dir(&base_path.join("wayland-sessions"), &mut sessions);
        // Scan X11 sessions
        scan_session_dir(&base_path.join("xsessions"), &mut sessions);
    }

    // Also check greetd environments (standard greetd location)
    let greetd_env = std::path::Path::new("/etc/greetd/environments");
    if greetd_env.exists() {
        if let Ok(content) = fs::read_to_string(greetd_env) {
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    sessions.push(SystemSession {
                        name: line.to_string(),
                        exec: line.to_string(),
                    });
                }
            }
        }
    }

    // Sort and remove duplicates
    sessions.sort_by(|a, b| a.name.cmp(&b.name));
    sessions.dedup_by(|a, b| a.name == b.name);

    sessions
}

fn scan_session_dir(path: &std::path::Path, sessions: &mut Vec<SystemSession>) {
    if !path.exists() || !path.is_dir() {
        return;
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.extension().is_some_and(|ext| ext == "desktop") {
                if let Some(session) = parse_desktop_file(&entry_path) {
                    sessions.push(session);
                }
            }
        }
    }
}

fn parse_desktop_file(path: &std::path::Path) -> Option<SystemSession> {
    let content = fs::read_to_string(path).ok()?;
    let mut name = None;
    let mut exec = None;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("Name=") && name.is_none() {
            name = Some(line.replace("Name=", ""));
        } else if line.starts_with("Exec=") && exec.is_none() {
            exec = Some(line.replace("Exec=", ""));
        }
    }

    match (name, exec) {
        (Some(n), Some(e)) => Some(SystemSession { name: n, exec: e }),
        _ => None,
    }
}

#[allow(dead_code)]
pub fn get_state_path() -> PathBuf {
    let uid = unsafe { libc::getuid() };
    let base = if uid == 0 {
        PathBuf::from(config::CACHE_DIR)
    } else {
        PathBuf::from(".cache")
    };
    base.join("state.json")
}
