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
pub fn get_cache_dir() -> PathBuf {
    let uid = rustix::process::getuid().as_raw();
    if uid == 0 {
        PathBuf::from(config::CACHE_DIR)
    } else {
        PathBuf::from(format!(".cache/{}", config::GREETER_NAME))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cache {
    /// The last user who logged in
    pub last_user: Option<String>,
    /// Maps username to their last used session (compositor name) using LRU
    pub user_to_last_sess: LruWrapper<String, String>,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            last_user: None,
            user_to_last_sess: LruWrapper::new(CACHE_LIMIT),
        }
    }
}

impl Cache {
    pub fn load() -> Self {
        let path = Self::get_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cache) = toml::from_str(&content) {
                    return cache;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::get_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(content) = toml::to_string_pretty(self) {
            let _ = fs::write(path, content);
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

    fn get_path() -> PathBuf {
        get_cache_dir().join("state.toml")
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
