use anyhow::{Context, Result};
use pwd::Passwd;
use std::fs;
use tracing::{info, warn};
use zbus::{Connection, proxy, zvariant::OwnedObjectPath};

#[proxy(
    default_path = "/org/freedesktop/Accounts",
    default_service = "org.freedesktop.Accounts",
    interface = "org.freedesktop.Accounts"
)]
pub trait AccountsService {
    fn list_cached_users(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
}

#[proxy(
    default_service = "org.freedesktop.Accounts",
    default_path = "/org/freedesktop/Accounts",
    interface = "org.freedesktop.Accounts.User"
)]
pub trait User {
    #[zbus(property)]
    fn user_name(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn real_name(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn shell(&self) -> zbus::Result<String>;
}

#[derive(Debug, Clone)]
pub struct SystemUser {
    pub user_name: String,
    pub real_name: String,
}

impl SystemUser {
    pub async fn all() -> Result<Vec<Self>> {
        // 1. Try D-Bus (AccountsService) first as it has better metadata
        match Self::from_dbus().await {
            Ok(users) if !users.is_empty() => {
                info!("Found {} users via D-Bus", users.len());
                return Ok(users);
            }
            Ok(_) => warn!("D-Bus returned empty user list, falling back to passwd"),
            Err(e) => warn!("D-Bus error: {:?}, falling back to passwd", e),
        }

        // 2. Fallback to /etc/passwd using the pwd crate
        let users = Self::from_passwd();
        info!("Found {} users via /etc/passwd", users.len());
        Ok(users)
    }

    async fn from_dbus() -> Result<Vec<Self>> {
        let conn = Connection::system()
            .await
            .context("failed to connect to system D-Bus")?;
        let accounts_proxy = AccountsServiceProxy::new(&conn)
            .await
            .context("failed to create AccountsService proxy")?;

        let user_paths = accounts_proxy
            .list_cached_users()
            .await
            .context("failed to list cached users")?;

        let mut users = Vec::new();

        for user_path in user_paths {
            let user_proxy = UserProxy::builder(&conn).path(user_path)?.build().await?;

            let user_name = user_proxy.user_name().await.unwrap_or_default();
            let real_name = user_proxy.real_name().await.unwrap_or_default();

            if user_name.is_empty() {
                continue;
            }

            users.push(Self {
                user_name,
                real_name,
            });
        }

        users.sort_by(|a, b| a.user_name.cmp(&b.user_name));
        Ok(users)
    }

    fn from_passwd() -> Vec<Self> {
        let mut users = Vec::new();
        let normal_user = NormalUser::load();

        for entry in Passwd::iter() {
            if normal_user.is_normal_user(entry.uid) {
                let user_name = entry.name;

                let real_name = if let Some(gecos) = entry.gecos {
                    if gecos.is_empty() {
                        user_name.clone()
                    } else {
                        gecos.split(',').next().unwrap_or(&gecos).to_string()
                    }
                } else {
                    user_name.clone()
                };

                users.push(Self {
                    user_name,
                    real_name,
                });
            }
        }

        users.sort_by(|a, b| a.user_name.cmp(&b.user_name));
        users
    }
}

struct NormalUser {
    uid_min: u32,
    uid_max: u32,
}

impl Default for NormalUser {
    fn default() -> Self {
        Self {
            uid_min: 1000,
            uid_max: 60000,
        }
    }
}

impl NormalUser {
    pub fn load() -> Self {
        let mut min = None;
        let mut max = None;

        if let Ok(content) = fs::read_to_string("/etc/login.defs") {
            for line in content.lines().map(str::trim) {
                if line.starts_with("UID_MIN") {
                    if let Some(val) = line.split_whitespace().nth(1) {
                        min = val.parse().ok();
                    }
                } else if line.starts_with("UID_MAX") {
                    if let Some(val) = line.split_whitespace().nth(1) {
                        max = val.parse().ok();
                    }
                }
            }
        }

        let def = Self::default();
        Self {
            uid_min: min.unwrap_or(def.uid_min),
            uid_max: max.unwrap_or(def.uid_max),
        }
    }

    pub fn is_normal_user(&self, uid: u32) -> bool {
        uid >= self.uid_min && uid <= self.uid_max
    }
}

#[derive(Debug, Clone)]
pub struct SystemSession {
    pub name: String,
    pub exec: String,
}

impl SystemSession {
    pub fn all() -> Vec<Self> {
        let mut sessions = Vec::new();
        let xdg_data_dirs = std::env::var("XDG_DATA_DIRS")
            .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());

        for data_dir in xdg_data_dirs.split(':') {
            if data_dir.is_empty() {
                continue;
            }
            let base_path = std::path::Path::new(data_dir);
            Self::scan_dir(&base_path.join("wayland-sessions"), &mut sessions);
            Self::scan_dir(&base_path.join("xsessions"), &mut sessions);
        }

        let greetd_env = std::path::Path::new("/etc/greetd/environments");
        if greetd_env.exists() {
            if let Ok(content) = fs::read_to_string(greetd_env) {
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        sessions.push(Self {
                            name: line.to_string(),
                            exec: line.to_string(),
                        });
                    }
                }
            }
        }

        sessions.sort_by(|a, b| a.name.cmp(&b.name));
        sessions.dedup_by(|a, b| a.name == b.name);
        sessions
    }

    fn scan_dir(path: &std::path::Path, sessions: &mut Vec<Self>) {
        if !path.exists() || !path.is_dir() {
            return;
        }
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.extension().is_some_and(|ext| ext == "desktop") {
                    if let Some(session) = Self::parse_desktop_file(&entry_path) {
                        sessions.push(session);
                    }
                }
            }
        }
    }

    fn parse_desktop_file(path: &std::path::Path) -> Option<Self> {
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
            (Some(n), Some(e)) => Some(Self { name: n, exec: e }),
            _ => None,
        }
    }
}
