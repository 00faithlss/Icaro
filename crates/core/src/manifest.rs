// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Manifiesto global de versiones de Mojang y descarga verificada del JSON de una versión.

use serde::Deserialize;
use sha1::{Digest, Sha1};

pub const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error de red: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON inválido: {0}")]
    Json(#[from] serde_json::Error),
    #[error("la versión {0} no existe en el manifiesto")]
    VersionNotFound(String),
    #[error("SHA-1 incorrecto: esperado {expected}, obtenido {actual}")]
    HashMismatch { expected: String, actual: String },
}

#[derive(Debug, Deserialize)]
pub struct VersionManifest {
    pub latest: Latest,
    pub versions: Vec<VersionEntry>,
}

#[derive(Debug, Deserialize)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Deserialize)]
pub struct VersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    pub sha1: String,
}

impl VersionManifest {
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn find(&self, id: &str) -> Result<&VersionEntry, Error> {
        self.versions
            .iter()
            .find(|v| v.id == id)
            .ok_or_else(|| Error::VersionNotFound(id.to_owned()))
    }
}

pub fn sha1_hex(bytes: &[u8]) -> String {
    Sha1::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub async fn fetch_manifest(client: &reqwest::Client) -> Result<VersionManifest, Error> {
    let bytes = client
        .get(VERSION_MANIFEST_URL)
        .send()
        .await?
        .bytes()
        .await?;
    VersionManifest::parse(&bytes)
}

/// Descarga el JSON de una versión y verifica su SHA-1 contra el manifiesto.
pub async fn fetch_version_json(
    client: &reqwest::Client,
    entry: &VersionEntry,
) -> Result<Vec<u8>, Error> {
    let bytes = client.get(&entry.url).send().await?.bytes().await?;
    let actual = sha1_hex(&bytes);
    if actual != entry.sha1 {
        return Err(Error::HashMismatch {
            expected: entry.sha1.clone(),
            actual,
        });
    }
    Ok(bytes.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "latest": {"release": "1.21.1", "snapshot": "1.21.1"},
        "versions": [
            {"id": "1.21.1", "type": "release", "url": "https://example.com/1.21.1.json",
             "sha1": "da39a3ee5e6b4b0d3255bfef95601890afd80709"}
        ]
    }"#;

    #[test]
    fn parsea_y_encuentra_version() {
        let m = VersionManifest::parse(SAMPLE.as_bytes()).unwrap();
        assert_eq!(m.find("1.21.1").unwrap().kind, "release");
        assert!(matches!(m.find("0.0"), Err(Error::VersionNotFound(_))));
    }

    #[test]
    fn sha1_de_vacio() {
        assert_eq!(sha1_hex(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }
}
