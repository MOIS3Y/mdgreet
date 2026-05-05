use crate::config;
use lru::LruCache as OrigLruCache;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{MapAccess, Visitor},
    ser::SerializeMap,
};
use std::fs;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::path::PathBuf;

const CACHE_LIMIT: usize = 100;
/// Returns the base directory for all cache files (state, images, etc.)
pub fn get_cache_dir(config: &config::GreeterConfig) -> PathBuf {
    config
        .cache
        .path
        .clone()
        .unwrap_or_else(|| PathBuf::from(format!("/var/cache/{}", config::GREETER_NAME)))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cache {
    /// The last user who logged in
    pub last_user: Option<String>,
    /// Maps username to their last used session (compositor name) using LRU
    pub user_to_last_sess: LruWrapper<String, String>,
    #[serde(skip)]
    path: PathBuf,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            last_user: None,
            user_to_last_sess: LruWrapper::new(CACHE_LIMIT),
            path: PathBuf::new(),
        }
    }
}
impl Cache {
    pub fn load(config: &config::GreeterConfig) -> Self {
        let path = get_cache_dir(config).join("state.toml");
        let mut cache = if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                toml::from_str(&content).unwrap_or_default()
            } else {
                Self::default()
            }
        } else {
            Self::default()
        };
        cache.path = path;
        cache
    }

    pub fn save(&self) {
        if self.path.as_os_str().is_empty() {
            return;
        }
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(content) = toml::to_string_pretty(self) {
            let _ = fs::write(&self.path, content);
        }
    }

    pub fn set_last_user(&mut self, user: String) {
        self.last_user = Some(user);
    }

    pub fn set_last_session(&mut self, user: String, session: String) {
        self.user_to_last_sess.put(user, session);
    }

    pub fn get_last_session(&mut self, user: &str) -> Option<&String> {
        self.user_to_last_sess.get(&user.to_string())
    }
}

/// Wrapper for lru::LruCache to support Serde serialization
#[derive(Debug, Clone)]
pub struct LruWrapper<K: Hash + Eq, V>(OrigLruCache<K, V>);

impl<K: Hash + Eq, V> LruWrapper<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self(OrigLruCache::new(NonZeroUsize::new(capacity).unwrap()))
    }

    pub fn put(&mut self, key: K, value: V) {
        self.0.put(key, value);
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.0.get(key)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<K: Serialize + Hash + Eq, V: Serialize> Serialize for LruWrapper<K, V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self.0.iter() {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl<'de, K: Deserialize<'de> + Hash + Eq, V: Deserialize<'de>> Deserialize<'de>
    for LruWrapper<K, V>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LruVisitor<K, V>(std::marker::PhantomData<(K, V)>);

        impl<'de, K: Deserialize<'de> + Hash + Eq, V: Deserialize<'de>> Visitor<'de> for LruVisitor<K, V> {
            type Value = LruWrapper<K, V>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map for LRU cache")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut lru = LruWrapper::new(CACHE_LIMIT);
                while let Some((key, value)) = access.next_entry()? {
                    lru.put(key, value);
                }
                Ok(lru)
            }
        }

        deserializer.deserialize_map(LruVisitor(std::marker::PhantomData))
    }
}
