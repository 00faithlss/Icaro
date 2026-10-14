// SPDX-License-Identifier: GPL-3.0-only
//! Caché acotada de metadatos del cliente y anfitrión; nunca guarda errores.
use super::Error;
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::BTreeMap, path::Path, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};

pub(super) const TTL: Duration = Duration::from_secs(3600);
pub(super) const NEGATIVE_TTL: Duration = Duration::from_secs(600);
const MAX_ENTRIES: usize = 2048;
const MAX_BYTES: usize = 32 * 1024 * 1024;

struct Entry {
    expires: Instant,
    body: Vec<u8>,
}
#[derive(Default)]
pub(super) struct Cache {
    entries: BTreeMap<String, Entry>,
    bytes: usize,
}
impl Cache {
    pub fn cargar(path: &Path) -> Self {
        let cargar = || -> Option<Self> {
            let meta = std::fs::symlink_metadata(path).ok()?;
            if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 64 * 1024 * 1024 { return None; }
            let data: serde_json::Value = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
            if data["schema"].as_u64()? != 1 { return None; }
            let ahora = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
            let mut cache = Self::default();
            let entries = data["entries"].as_array()?;
            if entries.len() > MAX_ENTRIES { return None; }
            for item in entries {
                let restante = item["expires"].as_u64()?.saturating_sub(ahora);
                if restante == 0 || restante > TTL.as_secs() { continue; }
                cache.insert(item["key"].as_str()?.to_owned(), &item["value"], Duration::from_secs(restante)).ok()?;
            }
            Some(cache)
        };
        cargar().unwrap_or_default()
    }
    pub fn snapshot(&self) -> serde_json::Value {
        let ahora = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let entries: Vec<_> = self.entries.iter().filter_map(|(key, entry)| {
            let restante = entry.expires.checked_duration_since(Instant::now())?.as_secs();
            if restante == 0 { return None; }
            let value: serde_json::Value = serde_json::from_slice(&entry.body).ok()?;
            Some(serde_json::json!({"key":key,"expires":ahora+restante,"value":value}))
        }).collect();
        serde_json::json!({"schema":1,"entries":entries})
    }
    pub fn get<T: DeserializeOwned>(&mut self, key: &str) -> Option<T> {
        if self.entries.get(key)?.expires <= Instant::now() {
            self.remove(key);
            return None;
        }
        serde_json::from_slice(&self.entries[key].body).ok()
    }
    pub fn insert<T: Serialize>(&mut self, key: String, value: &T, ttl: Duration) -> Result<(), Error> {
        let body = serde_json::to_vec(value).map_err(|_| Error::Response("Modrinth"))?;
        if body.len() > MAX_BYTES { return Ok(()); }
        self.remove(&key);
        while self.entries.len() >= MAX_ENTRIES || self.bytes + body.len() > MAX_BYTES {
            let oldest = self.entries.iter().min_by_key(|(_, e)| e.expires).map(|(k, _)| k.clone());
            let Some(oldest) = oldest else { break; };
            self.remove(&oldest);
        }
        self.bytes += body.len();
        self.entries.insert(key, Entry { expires: Instant::now() + ttl, body });
        Ok(())
    }
    fn remove(&mut self, key: &str) {
        if let Some(old) = self.entries.remove(key) { self.bytes -= old.body.len(); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distingue_negativos_caduca_y_limita_memoria() {
        let mut cache = Cache::default();
        cache.insert("negativo".into(), &None::<String>, TTL).unwrap();
        assert_eq!(cache.get::<Option<String>>("negativo"), Some(None));
        cache.entries.get_mut("negativo").unwrap().expires = Instant::now();
        assert_eq!(cache.get::<Option<String>>("negativo"), None);
        assert_eq!(cache.bytes, 0);
        for n in 0..MAX_ENTRIES + 1 { cache.insert(n.to_string(), &n, TTL).unwrap(); }
        assert_eq!(cache.entries.len(), MAX_ENTRIES);
        assert!(cache.get::<usize>("0").is_none());
        let body = "x".repeat(MAX_BYTES / 2);
        cache.insert("grande1".into(), &body, TTL).unwrap();
        cache.insert("grande2".into(), &body, TTL).unwrap();
        assert!(cache.bytes <= MAX_BYTES);
        assert!(cache.get::<String>("grande1").is_none());
        assert!(cache.get::<String>("grande2").is_some());
    }
}
