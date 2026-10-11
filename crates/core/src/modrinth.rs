// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Catálogo público: cliente independiente de las credenciales de Minecraft.

use crate::http;
pub use crate::http::Error;
use reqwest::{Client, Url};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const API: &str = "https://api.modrinth.com/v2/";
const BODY_LIMIT: usize = 4 * 1024 * 1024;
pub const PAGE_SIZE: u32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Mod,
    Modpack,
    Resourcepack,
}
impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mod => "mod",
            Self::Modpack => "modpack",
            Self::Resourcepack => "resourcepack",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Search {
    pub query: String,
    pub kind: Kind,
    pub minecraft: Option<String>,
    pub loader: Option<String>,
    pub offset: u32,
}

fn search_url(search: &Search) -> Url {
    let mut facets = vec![vec![format!("project_type:{}", search.kind.as_str())]];
    if let Some(version) = &search.minecraft {
        facets.push(vec![format!("versions:{version}")]);
    }
    if let Some(loader) = &search.loader {
        facets.push(vec![format!("categories:{}", loader.to_lowercase())]);
    }
    let mut url = Url::parse(API)
        .expect("URL constante")
        .join("search")
        .expect("Ruta constante");
    url.query_pairs_mut().extend_pairs([
        ("query", search.query.as_str()),
        (
            "facets",
            &serde_json::to_string(&facets).expect("Facetas de texto"),
        ),
        ("offset", &search.offset.to_string()),
        ("limit", &PAGE_SIZE.to_string()),
        ("index", "relevance"),
    ]);
    url
}

#[derive(Debug, Clone, Deserialize)]
pub struct Results {
    pub hits: Vec<Hit>,
    pub offset: u32,
    pub limit: u32,
    pub total_hits: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Hit {
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub license: License,
    pub downloads: u64,
}
#[derive(Debug, Clone, Deserialize)]
pub struct License {
    pub id: String,
    pub name: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Version {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub version_number: String,
    pub version_type: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub files: Vec<File>,
    pub dependencies: Vec<Dependency>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct File {
    pub filename: String,
    pub url: String,
    pub size: u64,
    pub primary: bool,
    pub hashes: Hashes,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Hashes {
    pub sha512: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Dependency {
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub dependency_type: String,
}

fn project_url(id: &str) -> Result<Url, Error> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
    {
        return Err(Error::Response("Modrinth"));
    }
    Url::parse(API)
        .expect("URL constante")
        .join(&format!("project/{id}"))
        .map_err(|_| Error::Response("Modrinth"))
}

#[derive(Clone)]
pub struct Catalog {
    client: Client,
    cooldown: Arc<Mutex<Option<Instant>>>,
}
impl Catalog {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            client: http::client()?,
            cooldown: Arc::new(Mutex::new(None)),
        })
    }
    async fn json<T: serde::de::DeserializeOwned>(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<T, Error> {
        if let Some(deadline) = *self
            .cooldown
            .lock()
            .map_err(|_| Error::Response("Modrinth"))?
        {
            if let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
                return Err(Error::RateLimited(remaining.as_secs() + 1));
            }
        }
        let result = http::json(request, "Modrinth", BODY_LIMIT).await;
        if let Err(Error::RateLimited(seconds)) = &result {
            let wait = Duration::from_secs((*seconds).min(86400));
            *self
                .cooldown
                .lock()
                .map_err(|_| Error::Response("Modrinth"))? = Some(Instant::now() + wait);
        }
        result
    }
    pub async fn search(&self, search: &Search) -> Result<Results, Error> {
        if search.query.len() > 4096 {
            return Err(Error::Response("Modrinth"));
        }
        self.json(self.client.get(search_url(search))).await
    }
    pub async fn project(&self, id: &str) -> Result<Project, Error> {
        self.json(self.client.get(project_url(id)?)).await
    }
    pub async fn versions(
        &self,
        id: &str,
        minecraft: Option<&str>,
        loader: Option<&str>,
    ) -> Result<Vec<Version>, Error> {
        let mut url = project_url(id)?;
        url.set_path(&format!("{}/version", url.path()));
        if let Some(mc) = minecraft {
            url.query_pairs_mut()
                .append_pair("game_versions", &serde_json::json!([mc]).to_string());
        }
        if let Some(loader) = loader {
            url.query_pairs_mut().append_pair(
                "loaders",
                &serde_json::json!([loader.to_lowercase()]).to_string(),
            );
        }
        let versions: Vec<Version> = self.json(self.client.get(url)).await?;
        Ok(versions
            .into_iter()
            .filter(|v| {
                minecraft.is_none_or(|mc| v.game_versions.iter().any(|x| x == mc))
                    && loader.is_none_or(|loader| {
                        v.loaders.iter().any(|x| x.eq_ignore_ascii_case(loader))
                    })
            })
            .collect())
    }
}

pub async fn open_project(id: &str) -> Result<(), Error> {
    project_url(id)?;
    let url = format!("https://modrinth.com/project/{id}");
    tokio::task::spawn_blocking(move || {
        webbrowser::open(&url).map_err(|_| Error::Network("el navegador"))
    })
    .await
    .map_err(|_| Error::Network("el navegador"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn cooldown_blocks_queries_before_a_network_request() {
        let catalog = Catalog::new().unwrap();
        *catalog.cooldown.lock().unwrap() = Some(Instant::now() + Duration::from_secs(30));
        let search = Search {
            query: "sodium".into(),
            kind: Kind::Mod,
            minecraft: None,
            loader: None,
            offset: 0,
        };
        assert!(
            matches!(catalog.clone().search(&search).await, Err(Error::RateLimited(seconds)) if (1..=31).contains(&seconds))
        );
    }
    #[test]
    fn facets_are_encoded_and_filters_are_conjoined() {
        let search = Search {
            query: "sodium & iris".into(),
            kind: Kind::Mod,
            minecraft: Some("1.21.4".into()),
            loader: Some("Fabric".into()),
            offset: 20,
        };
        let url = search_url(&search);
        let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(pairs["query"], "sodium & iris");
        assert_eq!(pairs["offset"], "20");
        let facets: serde_json::Value = serde_json::from_str(&pairs["facets"]).unwrap();
        assert_eq!(
            facets,
            serde_json::json!([
                ["project_type:mod"],
                ["versions:1.21.4"],
                ["categories:fabric"]
            ])
        );
        assert!(!url.as_str().contains("token"));
    }
    #[test]
    fn rejects_paths_hosts_and_query_injection_in_ids() {
        for id in [
            "",
            "../version",
            "https://attacker.example",
            "abc?token=secret",
            "abc#fragment",
            "a/b",
        ] {
            assert!(project_url(id).is_err());
        }
        assert_eq!(
            project_url("fabric-api").unwrap().host_str(),
            Some("api.modrinth.com")
        );
    }
}
