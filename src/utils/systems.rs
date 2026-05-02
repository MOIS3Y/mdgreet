use crate::config;
use anyhow::{Context, Result};
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

        // Filter human users (usually >= 1000)
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

    // Sort by login
    users.sort_by(|a, b| a.login.cmp(&b.login));

    Ok(users)
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
