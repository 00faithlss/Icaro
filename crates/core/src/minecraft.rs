// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Instalación Vanilla desde los manifiestos oficiales, con caché verificada.
//! El juego del usuario nunca se sustituye: binarios, índices y nativos viven
//! exclusivamente dentro de `.icaro-minecraft`, junto a `instance.json`.

use crate::instancias::{Almacen, Instancia};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::Duration,
};
use tokio_util::sync::CancellationToken;

const METADATA_LIMIT: u64 = 8 * 1024 * 1024;
const FILE_LIMIT: u64 = 1024 * 1024 * 1024;
const MAX_FILES: usize = 100_000;
const MARKER: &str = "instalado.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instalado {
    pub version: String,
    pub main_class: String,
    pub classpath: Vec<PathBuf>,
    pub natives: PathBuf,
    pub assets: PathBuf,
    pub asset_index: String,
    pub java_major: u32,
    pub java_component: String,
    pub game_arguments: Vec<String>,
    pub jvm_arguments: Vec<String>,
}

/// Los contadores representan archivos verificados, incluidos los reutilizados.
#[derive(Debug, Clone)]
pub struct Progreso {
    pub etapa: String,
    pub completados: u64,
    pub total: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("La instalación fue cancelada. Los archivos verificados se conservaron.")]
    Cancelada,
    #[error("Esta instancia ya se está preparando.")]
    Ocupada,
    #[error("No se pudo leer o guardar la instalación de Minecraft.")]
    Disco,
    #[error("El manifiesto oficial contiene datos inválidos o rutas no permitidas.")]
    Datos,
    #[error("No se pudo conectar con los servidores de Minecraft. Revisa tu conexión.")]
    Red,
    #[error("Minecraft respondió con HTTP {0}. Inténtalo de nuevo.")]
    Http(u16),
    #[error("El archivo descargado no coincide con el tamaño o SHA-1 oficial.")]
    Integridad,
    #[error("Esta versión o plataforma todavía no está soportada por Ícaro: {0}.")]
    NoSoportada(String),
    #[error("La versión no aparece en el catálogo oficial de Minecraft.")]
    Version,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Descarga {
    url: String,
    sha1: String,
    size: u64,
    #[serde(default)]
    path: String,
    #[serde(default)]
    id: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Version {
    id: String,
    #[serde(rename = "type", default)]
    kind: String,
    main_class: String,
    #[serde(default)]
    inherits_from: Option<String>,
    downloads: BTreeMap<String, Descarga>,
    libraries: Vec<Libreria>,
    asset_index: Descarga,
    #[serde(default)]
    java_version: Option<JavaVersion>,
    #[serde(default)]
    arguments: Option<Argumentos>,
    #[serde(default)]
    minecraft_arguments: Option<String>,
    #[serde(default)]
    logging: BTreeMap<String, Logging>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JavaVersion {
    major_version: u32,
    component: String,
}
#[derive(Debug, Deserialize)]
struct Logging {
    argument: String,
    file: Descarga,
}
#[derive(Debug, Deserialize)]
struct Libreria {
    name: String,
    #[serde(default)]
    rules: Option<Vec<Regla>>,
    #[serde(default)]
    downloads: LibraryDownloads,
    #[serde(default)]
    natives: BTreeMap<String, String>,
    #[serde(default)]
    extract: Extraer,
}
#[derive(Debug, Default, Deserialize)]
struct LibraryDownloads {
    artifact: Option<Descarga>,
    #[serde(default)]
    classifiers: BTreeMap<String, Descarga>,
}
#[derive(Debug, Default, Deserialize)]
struct Extraer {
    #[serde(default)]
    exclude: Vec<String>,
}
#[derive(Debug, Deserialize)]
struct Regla {
    action: String,
    os: Option<Os>,
    #[serde(default)]
    features: BTreeMap<String, bool>,
}
#[derive(Debug, Deserialize)]
struct Os {
    name: Option<String>,
    arch: Option<String>,
    version: Option<String>,
    #[serde(rename = "versionRange")]
    version_range: Option<Rango>,
}
#[derive(Debug, Deserialize)]
struct Rango {
    min: Option<String>,
    max: Option<String>,
}
#[derive(Debug, Deserialize)]
struct Argumentos {
    game: Vec<Argumento>,
    jvm: Vec<Argumento>,
}
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Argumento {
    Texto(String),
    Condicional {
        rules: Option<Vec<Regla>>,
        value: ValorArgumento,
    },
}
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ValorArgumento {
    Texto(String),
    Lista(Vec<String>),
}
#[derive(Debug, Deserialize)]
struct Indice {
    objects: BTreeMap<String, Objeto>,
    #[serde(default, rename = "virtual")]
    virtual_: bool,
    #[serde(default)]
    map_to_resources: bool,
}
#[derive(Debug, Deserialize)]
struct Objeto {
    hash: String,
    size: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Archivo {
    descarga: Descarga,
    ruta: PathBuf,
    etapa: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct Marcador {
    schema: u32,
    sistema: String,
    instalado: Instalado,
    archivos: Vec<Archivo>,
    nativos: Vec<(PathBuf, u64, String)>,
}

fn cancelado(cancel: &CancellationToken) -> Result<(), Error> {
    if cancel.is_cancelled() {
        Err(Error::Cancelada)
    } else {
        Ok(())
    }
}
fn plataforma() -> Result<&'static str, Error> {
    if std::env::consts::ARCH != "x86_64" {
        return Err(Error::NoSoportada("se requiere un sistema x64".into()));
    }
    match std::env::consts::OS {
        "windows" => Ok("windows"),
        "linux" => Ok("linux"),
        os => Err(Error::NoSoportada(os.into())),
    }
}
fn version_sistema() -> String {
    #[cfg(windows)]
    {
        use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};
        let key = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion");
        if let Ok(key) = key {
            let major: u32 = key.get_value("CurrentMajorVersionNumber").unwrap_or(10);
            let minor: u32 = key.get_value("CurrentMinorVersionNumber").unwrap_or(0);
            let build: String = key.get_value("CurrentBuildNumber").unwrap_or_default();
            return format!("{major}.{minor}.{build}");
        }
        String::new()
    }
    #[cfg(not(windows))]
    {
        std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .unwrap_or_default()
            .trim()
            .into()
    }
}
fn version_partes(version: &str) -> Vec<u64> {
    version.split('.').map(|s| s.parse().unwrap_or(0)).collect()
}
fn permitida(rules: Option<&[Regla]>, os: &str, os_version: &str) -> Result<bool, Error> {
    let Some(rules) = rules else { return Ok(true) };
    let mut allowed = false;
    for rule in rules {
        if !matches!(rule.action.as_str(), "allow" | "disallow") {
            return Err(Error::Datos);
        }
        if rule.features.values().any(|v| *v) {
            continue;
        }
        if let Some(condition) = &rule.os {
            if condition.name.as_deref().is_some_and(|n| n != os)
                || condition
                    .arch
                    .as_deref()
                    .is_some_and(|a| !matches!(a, "x86_64" | "amd64"))
            {
                continue;
            }
            if let Some(pattern) = &condition.version {
                if pattern.len() > 256 {
                    return Err(Error::Datos);
                }
                let regex = regex::Regex::new(pattern).map_err(|_| Error::Datos)?;
                if !regex.is_match(os_version) {
                    continue;
                }
            }
            if let Some(range) = &condition.version_range {
                if range
                    .min
                    .as_ref()
                    .is_some_and(|min| version_partes(os_version) < version_partes(min))
                    || range
                        .max
                        .as_ref()
                        .is_some_and(|max| version_partes(os_version) >= version_partes(max))
                {
                    continue;
                }
            }
        }
        allowed = rule.action == "allow";
    }
    Ok(allowed)
}
fn argumentos(items: &[Argumento], os: &str, version: &str) -> Result<Vec<String>, Error> {
    let mut result = vec![];
    for item in items {
        match item {
            Argumento::Texto(text) => result.push(text.clone()),
            Argumento::Condicional { rules, value }
                if permitida(rules.as_deref(), os, version)? =>
            {
                match value {
                    ValorArgumento::Texto(text) => result.push(text.clone()),
                    ValorArgumento::Lista(values) => result.extend(values.iter().cloned()),
                }
            }
            _ => {}
        }
    }
    if result.len() > 1024 || result.iter().any(|s| s.len() > 16384 || s.contains('\0')) {
        return Err(Error::Datos);
    }
    Ok(result)
}
fn hash_valido(hash: &str) -> bool {
    hash.len() == 40
        && hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn nombre_valido(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        && value != "."
        && value != ".."
}
fn relativa(value: &str) -> Result<PathBuf, Error> {
    if value.is_empty()
        || value.len() > 1024
        || value.contains(['\\', ':', '\0'])
        || value.starts_with('/')
        || value
            .split('/')
            .any(|s| s.is_empty() || s == "." || s == ".." || s.ends_with(['.', ' ']))
    {
        return Err(Error::Datos);
    }
    let path = PathBuf::from(value);
    if path
        .components()
        .any(|p| !matches!(p, Component::Normal(_)))
    {
        return Err(Error::Datos);
    }
    Ok(path)
}
fn enlace(meta: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        meta.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        meta.file_type().is_symlink()
    }
}
/// Comprueba cada segmento antes de crear carpetas o abrir destinos de escritura.
fn segura(raiz: &Path, ruta: &Path, crear: bool) -> Result<(), Error> {
    let relative = ruta.strip_prefix(raiz).map_err(|_| Error::Datos)?;
    let root_meta = fs::symlink_metadata(raiz).map_err(|_| Error::Disco)?;
    if !root_meta.is_dir() || enlace(&root_meta) {
        return Err(Error::Datos);
    }
    let mut current = raiz.to_owned();
    let components: Vec<_> = relative.components().collect();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(part) = component else {
            return Err(Error::Datos);
        };
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(meta) => {
                if enlace(&meta) || (index + 1 < components.len() && !meta.is_dir()) {
                    return Err(Error::Datos);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if crear && index + 1 < components.len() {
                    if let Err(error) = fs::create_dir(&current) {
                        if error.kind() != std::io::ErrorKind::AlreadyExists {
                            return Err(Error::Disco);
                        }
                        let meta = fs::symlink_metadata(&current).map_err(|_| Error::Disco)?;
                        if !meta.is_dir() || enlace(&meta) {
                            return Err(Error::Datos);
                        }
                    }
                }
            }
            Err(_) => return Err(Error::Disco),
        }
    }
    Ok(())
}
fn oficial(value: &str) -> Result<reqwest::Url, Error> {
    let url = reqwest::Url::parse(value).map_err(|_| Error::Datos)?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !matches!(
            url.host_str(),
            Some(
                "piston-meta.mojang.com"
                    | "piston-data.mojang.com"
                    | "launchermeta.mojang.com"
                    | "launcher.mojang.com"
                    | "libraries.minecraft.net"
                    | "resources.download.minecraft.net"
            )
        )
    {
        return Err(Error::Datos);
    }
    Ok(url)
}
fn validar_descarga(download: &Descarga) -> Result<(), Error> {
    oficial(&download.url)?;
    if !hash_valido(&download.sha1) || download.size > FILE_LIMIT {
        return Err(Error::Datos);
    }
    Ok(())
}
fn sha1_archivo(path: &Path, cancel: &CancellationToken) -> Result<(u64, String), Error> {
    let mut file = File::open(path).map_err(|_| Error::Disco)?;
    let mut hash = Sha1::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut len = 0u64;
    loop {
        cancelado(cancel)?;
        let n = file.read(&mut buffer).map_err(|_| Error::Disco)?;
        if n == 0 {
            break;
        }
        len += n as u64;
        if len > FILE_LIMIT {
            return Err(Error::Datos);
        }
        hash.update(&buffer[..n]);
    }
    Ok((len, format!("{:x}", hash.finalize())))
}
fn verificado(
    root: &Path,
    path: &Path,
    download: &Descarga,
    cancel: &CancellationToken,
) -> Result<bool, Error> {
    segura(root, path, false)?;
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(_) => return Err(Error::Disco),
    };
    if !meta.is_file() || enlace(&meta) {
        return Err(Error::Datos);
    }
    if meta.len() != download.size {
        return Ok(false);
    }
    Ok(sha1_archivo(path, cancel)?.1 == download.sha1)
}
fn random() -> Result<String, Error> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| Error::Disco)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
struct Temporal(PathBuf);
impl Temporal {
    fn crear(path: PathBuf) -> Result<(Self, File), Error> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|_| Error::Disco)?;
        Ok((Self(path), file))
    }
}
impl Drop for Temporal {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
async fn descargar(
    client: &reqwest::Client,
    root: &Path,
    file: &Archivo,
    cancel: &CancellationToken,
) -> Result<(), Error> {
    descargar_observada(client, root, file, cancel, || {})
        .await
        .map(|_| ())
}

/// Solo anuncia una descarga cuando el archivo falta o no pasa la verificación.
async fn descargar_observada(
    client: &reqwest::Client,
    root: &Path,
    file: &Archivo,
    cancel: &CancellationToken,
    al_descargar: impl Fn(),
) -> Result<bool, Error> {
    validar_descarga(&file.descarga)?;
    cancelado(cancel)?;
    if verificado(root, &file.ruta, &file.descarga, cancel)? {
        return Ok(false);
    }
    al_descargar();
    segura(root, &file.ruta, true)?;
    let (temporary, mut disk) = Temporal::crear(
        file.ruta
            .parent()
            .ok_or(Error::Datos)?
            .join(format!(".icaro-{}.tmp", random()?)),
    )?;
    let mut response = tokio::select! {
        _ = cancel.cancelled() => return Err(Error::Cancelada),
        result = client.get(oficial(&file.descarga.url)?).send() => result.map_err(|_| Error::Red)?,
    };
    if !response.status().is_success() {
        return Err(Error::Http(response.status().as_u16()));
    }
    if response
        .content_length()
        .is_some_and(|len| len != file.descarga.size)
    {
        return Err(Error::Integridad);
    }
    let mut hash = Sha1::new();
    let mut written = 0u64;
    loop {
        let chunk = tokio::select! {
            _ = cancel.cancelled() => return Err(Error::Cancelada),
            chunk = response.chunk() => chunk.map_err(|_| Error::Red)?,
        };
        let Some(chunk) = chunk else { break };
        written += chunk.len() as u64;
        if written > file.descarga.size {
            return Err(Error::Integridad);
        }
        disk.write_all(&chunk).map_err(|_| Error::Disco)?;
        hash.update(&chunk);
    }
    if written != file.descarga.size || format!("{:x}", hash.finalize()) != file.descarga.sha1 {
        return Err(Error::Integridad);
    }
    disk.sync_all().map_err(|_| Error::Disco)?;
    drop(disk);
    cancelado(cancel)?;
    segura(root, &file.ruta, false)?;
    fs::rename(&temporary.0, &file.ruta).map_err(|_| Error::Disco)?;
    Ok(true)
}
async fn bytes_oficiales(
    client: &reqwest::Client,
    url: &str,
    limit: usize,
    cancel: &CancellationToken,
) -> Result<Vec<u8>, Error> {
    let response = tokio::select! {
        _ = cancel.cancelled() => return Err(Error::Cancelada),
        response = client.get(oficial(url)?).send() => response.map_err(|_| Error::Red)?,
    };
    if !response.status().is_success() {
        return Err(Error::Http(response.status().as_u16()));
    }
    tokio::select! {
        _ = cancel.cancelled() => Err(Error::Cancelada),
        bytes = crate::http::body_bytes(response, "Minecraft", limit) => bytes.map_err(|_| Error::Datos),
    }
}
fn leer_json<T: for<'de> Deserialize<'de>>(root: &Path, path: &Path) -> Result<T, Error> {
    segura(root, path, false)?;
    let meta = fs::symlink_metadata(path).map_err(|_| Error::Disco)?;
    if !meta.is_file() || meta.len() > METADATA_LIMIT || enlace(&meta) {
        return Err(Error::Datos);
    }
    let bytes = fs::read(path).map_err(|_| Error::Disco)?;
    serde_json::from_slice(&bytes).map_err(|_| Error::Datos)
}
fn escribir_json<T: Serialize>(root: &Path, path: &Path, value: &T) -> Result<(), Error> {
    segura(root, path, true)?;
    let bytes = serde_json::to_vec(value).map_err(|_| Error::Datos)?;
    if bytes.len() > METADATA_LIMIT as usize {
        return Err(Error::Datos);
    }
    let (temporary, mut file) = Temporal::crear(
        path.parent()
            .ok_or(Error::Datos)?
            .join(format!(".icaro-{}.tmp", random()?)),
    )?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| Error::Disco)?;
    drop(file);
    fs::rename(&temporary.0, path).map_err(|_| Error::Disco)
}
static INSTALANDO: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();
struct Guardia(PathBuf);
impl Guardia {
    fn adquirir(path: &Path) -> Result<Self, Error> {
        let mut set = INSTALANDO
            .get_or_init(Default::default)
            .lock()
            .map_err(|_| Error::Ocupada)?;
        if !set.insert(path.to_owned()) {
            return Err(Error::Ocupada);
        }
        Ok(Self(path.to_owned()))
    }
}
impl Drop for Guardia {
    fn drop(&mut self) {
        if let Ok(mut set) = INSTALANDO.get_or_init(Default::default).lock() {
            set.remove(&self.0);
        }
    }
}

fn nuevo_archivo(download: &Descarga, path: PathBuf, stage: &str) -> Result<Archivo, Error> {
    validar_descarga(download)?;
    Ok(Archivo {
        descarga: download.clone(),
        ruta: path,
        etapa: stage.into(),
    })
}
fn nativo_otro_arch(name: &str) -> bool {
    name.contains("natives-")
        && (name.ends_with("-arm64") || name.ends_with("-aarch64") || name.ends_with("-x86"))
}

/// Instala o repara todos los archivos necesarios y publica el marcador al final.
pub async fn instalar(
    store: &Almacen,
    instance: &Instancia,
    cancel: CancellationToken,
    progress: impl Fn(Progreso) + Send + Sync,
) -> Result<Instalado, Error> {
    if instance.loader != "vanilla" {
        return Err(Error::NoSoportada(
            "el loader de esta instancia aún no se instala".into(),
        ));
    }
    let os = plataforma()?;
    let os_version = version_sistema();
    if !nombre_valido(&instance.minecraft) {
        return Err(Error::Datos);
    }
    cancelado(&cancel)?;
    let folder = store
        .carpeta(&instance.id)
        .map_err(|_| Error::Disco)?
        .canonicalize()
        .map_err(|_| Error::Disco)?;
    let game = store
        .juego(&instance.id)
        .map_err(|_| Error::Disco)?
        .canonicalize()
        .map_err(|_| Error::Disco)?;
    let _lock = Guardia::adquirir(&folder)?;
    let root = folder.join(".icaro-minecraft");
    segura(&folder, &root, false)?;
    if !root.exists() {
        fs::create_dir(&root).map_err(|_| Error::Disco)?
    }
    if !fs::symlink_metadata(&root)
        .map_err(|_| Error::Disco)?
        .is_dir()
    {
        return Err(Error::Datos);
    }
    let marker_path = root.join(MARKER);
    progress(Progreso {
        etapa: "Comprobando archivos de Minecraft".into(),
        completados: 0,
        total: 0,
    });
    let client = reqwest::Client::builder()
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(180))
        .user_agent(crate::http::USER_AGENT)
        .build()
        .map_err(|_| Error::Red)?;

    // La copia del catálogo solo identifica metadatos cuyo SHA-1 se vuelve a comprobar.
    let manifest_path = root.join("catalogo.json");
    let cached: Option<crate::manifest::VersionManifest> = leer_json(&root, &manifest_path).ok();
    let cached_entry = cached
        .as_ref()
        .and_then(|m| m.versions.iter().find(|v| v.id == instance.minecraft))
        .cloned();
    let fresh = fs::metadata(&manifest_path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|date| date.elapsed().ok())
        .is_some_and(|age| age.as_secs() < 24 * 60 * 60);
    let entry = if fresh && cached_entry.is_some() {
        cached_entry.unwrap()
    } else {
        let fetched = bytes_oficiales(
            &client,
            crate::manifest::VERSION_MANIFEST_URL,
            METADATA_LIMIT as usize,
            &cancel,
        )
        .await;
        match fetched {
            Ok(bytes) => {
                let manifest =
                    crate::manifest::VersionManifest::parse(&bytes).map_err(|_| Error::Datos)?;
                let entry = manifest
                    .find(&instance.minecraft)
                    .map_err(|_| Error::Version)?
                    .clone();
                escribir_json(&root, &manifest_path, &manifest)?;
                entry
            }
            Err(Error::Cancelada) => return Err(Error::Cancelada),
            Err(error) => cached_entry.ok_or(error)?,
        }
    };
    oficial(&entry.url)?;
    if !hash_valido(&entry.sha1) {
        return Err(Error::Datos);
    }
    let version_path = root
        .join("versions")
        .join(&instance.minecraft)
        .join("version.json");
    let mut version_file = None;
    if let Ok(meta) = fs::symlink_metadata(&version_path) {
        segura(&root, &version_path, false)?;
        if meta.is_file()
            && meta.len() <= METADATA_LIMIT
            && !enlace(&meta)
            && sha1_archivo(&version_path, &cancel)?.1 == entry.sha1
        {
            version_file = Some(nuevo_archivo(
                &Descarga {
                    url: entry.url.clone(),
                    sha1: entry.sha1.clone(),
                    size: meta.len(),
                    path: String::new(),
                    id: String::new(),
                },
                version_path.clone(),
                "Metadatos",
            )?);
        }
    }
    let version_file = match version_file {
        Some(file) => file,
        None => {
            let bytes =
                bytes_oficiales(&client, &entry.url, METADATA_LIMIT as usize, &cancel).await?;
            if crate::manifest::sha1_hex(&bytes) != entry.sha1 {
                return Err(Error::Integridad);
            }
            let download = Descarga {
                url: entry.url.clone(),
                sha1: entry.sha1.clone(),
                size: bytes.len() as u64,
                path: String::new(),
                id: String::new(),
            };
            segura(&root, &version_path, true)?;
            let (temporary, mut file) = Temporal::crear(
                version_path
                    .parent()
                    .ok_or(Error::Datos)?
                    .join(format!(".icaro-{}.tmp", random()?)),
            )?;
            file.write_all(&bytes)
                .and_then(|_| file.sync_all())
                .map_err(|_| Error::Disco)?;
            drop(file);
            fs::rename(&temporary.0, &version_path).map_err(|_| Error::Disco)?;
            nuevo_archivo(&download, version_path.clone(), "Metadatos")?
        }
    };
    let version: Version = leer_json(&root, &version_path)?;
    if version.id != instance.minecraft
        || version.inherits_from.is_some()
        || !nombre_valido(&version.asset_index.id)
        || version.main_class.len() > 256
        || !version
            .main_class
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'$'))
    {
        return Err(Error::Datos);
    }
    if version.libraries.len() > 4096 || version.asset_index.size > METADATA_LIMIT {
        return Err(Error::Datos);
    }
    let mut files = vec![version_file];
    let assets = root.join("assets");
    let index_path = assets
        .join("indexes")
        .join(format!("{}.json", version.asset_index.id));
    let index_file = nuevo_archivo(
        &version.asset_index,
        index_path.clone(),
        "Índice de recursos",
    )?;
    progress(Progreso {
        etapa: "Preparando los recursos de Minecraft".into(),
        completados: 0,
        total: 0,
    });
    descargar(&client, &root, &index_file, &cancel).await?;
    files.push(index_file);
    let index: Indice = leer_json(&root, &index_path)?;
    if index.objects.len() > MAX_FILES {
        return Err(Error::Datos);
    }
    let mut hashes = BTreeSet::new();
    for (name, object) in &index.objects {
        relativa(name)?;
        if !hash_valido(&object.hash) {
            return Err(Error::Datos);
        }
        if hashes.insert(object.hash.clone()) {
            let path = assets
                .join("objects")
                .join(&object.hash[..2])
                .join(&object.hash);
            let download = Descarga {
                url: format!(
                    "https://resources.download.minecraft.net/{}/{}",
                    &object.hash[..2],
                    object.hash
                ),
                sha1: object.hash.clone(),
                size: object.size,
                path: String::new(),
                id: String::new(),
            };
            files.push(nuevo_archivo(&download, path, "Descargando recursos")?);
        }
    }
    let client_file = nuevo_archivo(
        version
            .downloads
            .get("client")
            .ok_or_else(|| Error::NoSoportada("la versión no ofrece cliente".into()))?,
        root.join("versions").join(&version.id).join("client.jar"),
        "Descargando Minecraft",
    )?;
    let mut classpath = vec![];
    let mut native_archives: Vec<(PathBuf, Vec<String>)> = vec![];
    for library in &version.libraries {
        if !permitida(library.rules.as_deref(), os, &os_version)? || nativo_otro_arch(&library.name)
        {
            continue;
        }
        if let Some(artifact) = &library.downloads.artifact {
            let path = root.join("libraries").join(relativa(&artifact.path)?);
            files.push(nuevo_archivo(
                artifact,
                path.clone(),
                "Descargando librerías",
            )?);
            classpath.push(path);
        }
        if let Some(classifier) = library.natives.get(os) {
            let classifier = classifier.replace("${arch}", "64");
            let download = library
                .downloads
                .classifiers
                .get(&classifier)
                .ok_or(Error::Datos)?;
            let path = root.join("libraries").join(relativa(&download.path)?);
            files.push(nuevo_archivo(
                download,
                path.clone(),
                "Descargando componentes nativos",
            )?);
            native_archives.push((path, library.extract.exclude.clone()));
        }
    }
    classpath.push(client_file.ruta.clone());
    files.push(client_file);
    let mut jvm = if let Some(args) = &version.arguments {
        argumentos(&args.jvm, os, &os_version)?
    } else {
        vec![
            "-Djava.library.path=${natives_directory}".into(),
            "-cp".into(),
            "${classpath}".into(),
        ]
    };
    let mut game_arguments = if let Some(args) = &version.arguments {
        argumentos(&args.game, os, &os_version)?
    } else {
        version
            .minecraft_arguments
            .as_ref()
            .ok_or_else(|| {
                Error::NoSoportada(
                    "esta versión usa un lanzamiento anterior a Minecraft 1.6".into(),
                )
            })?
            .split_whitespace()
            .map(str::to_owned)
            .collect()
    };
    if let Some(logging) = version.logging.get("client") {
        let log = root
            .join("assets")
            .join("log_configs")
            .join(relativa(&logging.file.id)?);
        jvm.push(logging.argument.replace("${path}", &log.to_string_lossy()));
        files.push(nuevo_archivo(
            &logging.file,
            log,
            "Configurando los registros",
        )?);
    }
    let mut unique = BTreeMap::<PathBuf, Archivo>::new();
    for file in files {
        if let Some(previous) = unique.insert(file.ruta.clone(), file.clone()) {
            if previous.descarga.sha1 != file.descarga.sha1
                || previous.descarga.size != file.descarga.size
            {
                return Err(Error::Datos);
            }
        }
    }
    let files: Vec<_> = unique.into_values().collect();
    if files.len() > MAX_FILES
        || files
            .iter()
            .try_fold(0u64, |sum, file| sum.checked_add(file.descarga.size))
            .is_none_or(|size| size > 16 * 1024 * 1024 * 1024)
    {
        return Err(Error::Datos);
    }
    let total = files.len() as u64;
    let mut completed = 0u64;
    let downloads_cancel = cancel.child_token();
    let mut tasks = tokio::task::JoinSet::new();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut remaining = files.iter().cloned();
    loop {
        while tasks.len() < 8 {
            let Some(file) = remaining.next() else { break };
            progress(Progreso {
                etapa: "Verificando archivos de Minecraft".into(),
                completados: completed,
                total,
            });
            let client = client.clone();
            let root = root.clone();
            let token = downloads_cancel.clone();
            let tx = tx.clone();
            tasks.spawn(async move {
                let result = descargar_observada(&client, &root, &file, &token, || {
                    let _ = tx.send((format!("Descargando {}", file.etapa), completed));
                })
                .await;
                (file.etapa, result)
            });
        }
        let result = tokio::select! {
            aviso = rx.recv() => {
                if let Some((etapa, completados)) = aviso {
                    progress(Progreso { etapa, completados, total });
                }
                continue;
            }
            result = tasks.join_next() => result,
        };
        let Some(result) = result else {
            break;
        };
        let (stage, result) = result.map_err(|_| Error::Disco)?;
        let descargado = match result {
            Ok(descargado) => descargado,
            Err(error) => {
                downloads_cancel.cancel();
                while tasks.join_next().await.is_some() {}
                return Err(error);
            }
        };
        completed += 1;
        progress(Progreso {
            etapa: if descargado {
                format!("Descargando {stage}")
            } else {
                "Verificando archivos de Minecraft".into()
            },
            completados: completed,
            total,
        });
    }
    cancelado(&cancel)?;
    progress(Progreso {
        etapa: "Preparando componentes nativos".into(),
        completados: completed,
        total,
    });
    let previous: Option<Marcador> = leer_json(&root, &marker_path).ok();
    let reuse = previous.filter(|marker| {
        marker.schema == 1
            && marker.sistema == format!("{os}-x86_64")
            && marker.instalado.version == version.id
            && marker.instalado.natives.parent() == Some(root.as_path())
            && segura(&root, &marker.instalado.natives, false).is_ok()
            && marker.instalado.natives.is_dir()
            && marker.archivos.len() == files.len()
            && marker.archivos.iter().zip(&files).all(|(old, current)| {
                old.ruta == current.ruta && old.descarga.sha1 == current.descarga.sha1
            })
            && marker.nativos.iter().all(|(path, size, hash)| {
                path.starts_with(&marker.instalado.natives)
                    && segura(&root, path, false).is_ok()
                    && sha1_archivo(path, &cancel)
                        .is_ok_and(|actual| actual.0 == *size && actual.1 == *hash)
            })
    });
    let (natives, natives_files) = if let Some(previous) = reuse {
        (previous.instalado.natives, previous.nativos)
    } else {
        let natives = root.join(format!("natives-{}", random()?));
        fs::create_dir(&natives).map_err(|_| Error::Disco)?;
        match extraer_nativos(&native_archives, &natives, &cancel) {
            Ok(files) => (natives, files),
            Err(error) => {
                // Solo se elimina la carpeta aleatoria creada en esta operación.
                if natives.parent() == Some(root.as_path()) {
                    let _ = fs::remove_dir_all(&natives);
                }
                return Err(error);
            }
        }
    };
    let game_assets = if index.virtual_ {
        assets.join("virtual").join(&version.asset_index.id)
    } else {
        assets.clone()
    };
    if index.virtual_ || index.map_to_resources {
        progress(Progreso {
            etapa: "Preparando recursos de esta versión".into(),
            completados: completed,
            total,
        });
        materializar(&root, &assets, &index, &game_assets, &game, &cancel)?;
    }
    for argument in &mut game_arguments {
        *argument = argument
            .replace("${version_type}", &version.kind)
            .replace("${game_assets}", &game_assets.to_string_lossy());
    }
    let java = version.java_version.unwrap_or(JavaVersion {
        major_version: 8,
        component: "jre-legacy".into(),
    });
    if !matches!(java.major_version, 8 | 16 | 17 | 21 | 25) || !nombre_valido(&java.component) {
        return Err(Error::NoSoportada(format!("Java {}", java.major_version)));
    }
    let installed = Instalado {
        version: version.id,
        main_class: version.main_class,
        classpath,
        natives,
        assets,
        asset_index: version.asset_index.id,
        java_major: java.major_version,
        java_component: java.component,
        game_arguments,
        jvm_arguments: jvm,
    };
    cancelado(&cancel)?;
    escribir_json(
        &root,
        &marker_path,
        &Marcador {
            schema: 1,
            sistema: format!("{os}-x86_64"),
            instalado: installed.clone(),
            archivos: files,
            nativos: natives_files,
        },
    )?;
    progress(Progreso {
        etapa: "Minecraft preparado".into(),
        completados: total,
        total,
    });
    Ok(installed)
}

fn extraer_nativos(
    archives: &[(PathBuf, Vec<String>)],
    target: &Path,
    cancel: &CancellationToken,
) -> Result<Vec<(PathBuf, u64, String)>, Error> {
    let mut result = vec![];
    let mut seen = HashSet::new();
    let mut expanded = 0u64;
    for (path, excluded) in archives {
        cancelado(cancel)?;
        let file = File::open(path).map_err(|_| Error::Disco)?;
        let mut archive = zip::ZipArchive::new(file).map_err(|_| Error::Datos)?;
        if archive.len() > 10_000 {
            return Err(Error::Datos);
        }
        for i in 0..archive.len() {
            cancelado(cancel)?;
            let mut member = archive.by_index(i).map_err(|_| Error::Datos)?;
            let name = member.name().map_err(|_| Error::Datos)?.into_owned();
            if name.starts_with("META-INF/")
                || excluded.iter().any(|s| name.starts_with(s))
                || member.is_dir()
            {
                continue;
            }
            let relative = relativa(&name)?;
            if member
                .unix_mode()
                .is_some_and(|mode| mode & 0o170000 == 0o120000)
                || !seen.insert(relative.clone())
            {
                return Err(Error::Datos);
            }
            expanded = expanded.checked_add(member.size()).ok_or(Error::Datos)?;
            if member.size() > 128 * 1024 * 1024 || expanded > 512 * 1024 * 1024 {
                return Err(Error::Datos);
            }
            let destination = target.join(relative);
            segura(target, &destination, true)?;
            let mut output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&destination)
                .map_err(|_| Error::Disco)?;
            let mut total = 0u64;
            let mut hash = Sha1::new();
            let mut buffer = [0u8; 64 * 1024];
            loop {
                cancelado(cancel)?;
                let n = member.read(&mut buffer).map_err(|_| Error::Datos)?;
                if n == 0 {
                    break;
                }
                total += n as u64;
                if total > member.size() {
                    return Err(Error::Datos);
                }
                output.write_all(&buffer[..n]).map_err(|_| Error::Disco)?;
                hash.update(&buffer[..n]);
            }
            if total != member.size() {
                return Err(Error::Integridad);
            }
            output.sync_all().map_err(|_| Error::Disco)?;
            result.push((destination, total, format!("{:x}", hash.finalize())));
        }
    }
    Ok(result)
}

fn materializar(
    root: &Path,
    assets: &Path,
    index: &Indice,
    virtual_root: &Path,
    game: &Path,
    cancel: &CancellationToken,
) -> Result<(), Error> {
    for (name, object) in &index.objects {
        cancelado(cancel)?;
        let relative = relativa(name)?;
        let source = assets
            .join("objects")
            .join(&object.hash[..2])
            .join(&object.hash);
        for (base, destination) in [
            (root, index.virtual_.then(|| virtual_root.join(&relative))),
            (
                game,
                index
                    .map_to_resources
                    .then(|| game.join("resources").join(&relative)),
            ),
        ] {
            let Some(destination) = destination else {
                continue;
            };
            segura(base, &destination, true)?;
            if destination.exists() {
                let meta = fs::symlink_metadata(&destination).map_err(|_| Error::Disco)?;
                if !meta.is_file() || enlace(&meta) {
                    return Err(Error::Datos);
                }
                // Minecraft anterior a 1.7 puede contener recursos personalizados del usuario.
                if base == game {
                    continue;
                }
                if sha1_archivo(&destination, cancel)?.1 == object.hash {
                    continue;
                }
            }
            let (temporary, mut file) = Temporal::crear(
                destination
                    .parent()
                    .ok_or(Error::Datos)?
                    .join(format!(".icaro-{}.tmp", random()?)),
            )?;
            let mut input = File::open(&source).map_err(|_| Error::Disco)?;
            std::io::copy(&mut input, &mut file).map_err(|_| Error::Disco)?;
            file.sync_all().map_err(|_| Error::Disco)?;
            drop(file);
            cancelado(cancel)?;
            segura(base, &destination, false)?;
            // Las copias privadas se reparan; nunca se sustituye un recurso del usuario.
            if base == game {
                // Publicación exclusiva: ni siquiera una creación simultánea del
                // juego puede ser sustituida por la copia de recursos legacy.
                match fs::hard_link(&temporary.0, &destination) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(_) => return Err(Error::Disco),
                }
            } else {
                fs::rename(&temporary.0, &destination).map_err(|_| Error::Disco)?;
            }
        }
    }
    Ok(())
}

/// Consulta rápida para las tarjetas; antes de lanzar `instalar` comprueba SHA-1.
pub fn listo(store: &Almacen, instance: &Instancia) -> bool {
    if instance.loader != "vanilla" {
        return false;
    }
    let Ok(folder) = store.carpeta(&instance.id).and_then(|path| {
        path.canonicalize()
            .map_err(|_| crate::instancias::Error::Disco)
    }) else {
        return false;
    };
    let root = folder.join(".icaro-minecraft");
    let Ok(marker): Result<Marcador, _> = leer_json(&folder, &root.join(MARKER)) else {
        return false;
    };
    let Ok(os) = plataforma() else { return false };
    if marker.schema != 1
        || marker.sistema != format!("{os}-x86_64")
        || marker.instalado.version != instance.minecraft
        || marker.archivos.is_empty()
    {
        return false;
    }
    marker.archivos.iter().all(|file| {
        segura(&root, &file.ruta, false).is_ok()
            && fs::symlink_metadata(&file.ruta).is_ok_and(|meta| {
                meta.is_file() && !enlace(&meta) && meta.len() == file.descarga.size
            })
    }) && segura(&root, &marker.instalado.natives, false).is_ok()
        && marker.instalado.natives.is_dir()
        && marker.nativos.iter().all(|(path, size, _)| {
            segura(&root, path, false).is_ok()
                && fs::metadata(path).is_ok_and(|meta| meta.is_file() && meta.len() == *size)
        })
}

/// Recupera la descripción local del lanzamiento sin red. `instalar` debe
/// volver a verificar los hashes antes de ejecutar el proceso.
pub fn leer(store: &Almacen, instance: &Instancia) -> Option<Instalado> {
    if !listo(store, instance) {
        return None;
    }
    let folder = store.carpeta(&instance.id).ok()?.canonicalize().ok()?;
    let marker: Marcador =
        leer_json(&folder, &folder.join(".icaro-minecraft").join(MARKER)).ok()?;
    Some(marker.instalado)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn temp() -> PathBuf {
        let path = std::env::temp_dir().join(format!("icaro-minecraft-test-{}", random().unwrap()));
        fs::create_dir(&path).unwrap();
        path
    }
    #[test]
    fn reglas_de_so_arch_features_y_ultimo_match() {
        let rules: Vec<Regla> = serde_json::from_str(r#"[{"action":"allow"},{"action":"disallow","os":{"name":"osx"}},{"action":"disallow","features":{"is_demo_user":true}}]"#).unwrap();
        assert!(permitida(Some(&rules), "windows", "10.0.26100").unwrap());
        assert!(!permitida(Some(&rules), "osx", "15.0").unwrap());
        let args: Vec<Argumento> = serde_json::from_str(r#"["-cp","${classpath}",{"rules":[{"action":"allow","os":{"arch":"x86"}}],"value":"-Xss1M"},{"rules":[{"action":"allow","features":{"has_custom_resolution":true}}],"value":["--width","${resolution_width}"]}]"#).unwrap();
        assert_eq!(
            argumentos(&args, "windows", "10.0.26100").unwrap(),
            vec!["-cp", "${classpath}"]
        );
    }
    #[test]
    fn rutas_y_urls_rechazan_escape() {
        for path in [
            "../secret",
            "a/../secret",
            "/tmp/x",
            "C:/x",
            "a\\x",
            "a//x",
            "a/CON:foo",
        ] {
            assert!(relativa(path).is_err(), "{path}")
        }
        for url in [
            "http://libraries.minecraft.net/x",
            "https://libraries.minecraft.net.evil.test/x",
            "https://piston-data.mojang.com/x?token=x",
            "https://user@libraries.minecraft.net/x",
        ] {
            assert!(oficial(url).is_err(), "{url}")
        }
        assert!(oficial("https://libraries.minecraft.net/com/mojang/x.jar").is_ok());
    }
    #[test]
    fn cache_se_reutiliza_solo_con_tamano_y_hash() {
        let path = temp();
        let file = path.join("client.jar");
        fs::write(&file, b"good").unwrap();
        let download = Descarga {
            url: "https://piston-data.mojang.com/a".into(),
            sha1: crate::manifest::sha1_hex(b"good"),
            size: 4,
            path: String::new(),
            id: String::new(),
        };
        assert!(verificado(&path, &file, &download, &CancellationToken::new()).unwrap());
        fs::write(&file, b"evil").unwrap();
        assert!(!verificado(&path, &file, &download, &CancellationToken::new()).unwrap());
        let token = CancellationToken::new();
        token.cancel();
        assert!(matches!(sha1_archivo(&file, &token), Err(Error::Cancelada)));
        fs::remove_dir_all(path).unwrap();
    }

    #[tokio::test]
    async fn archivo_guardado_no_anuncia_descarga_ni_consulta_red() {
        let path = temp();
        let ruta = path.join("client.jar");
        fs::write(&ruta, b"good").unwrap();
        let file = nuevo_archivo(
            &Descarga {
                url: "https://piston-data.mojang.com/a".into(),
                sha1: crate::manifest::sha1_hex(b"good"),
                size: 4,
                path: String::new(),
                id: String::new(),
            },
            ruta,
            "Cliente de Minecraft",
        )
        .unwrap();
        let client = crate::http::client().unwrap();
        let descargado =
            descargar_observada(&client, &path, &file, &CancellationToken::new(), || {
                panic!("Una caché válida solo se prepara")
            })
            .await
            .unwrap();
        assert!(!descargado);
        fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn zip_nativo_rechaza_traversal_y_no_sobrescribe() {
        let path = temp();
        let archive = path.join("natives.jar");
        let mut zip = zip::ZipWriter::new(File::create(&archive).unwrap());
        zip.start_file("../escape.dll", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"bad").unwrap();
        zip.finish().unwrap();
        let target = path.join("out");
        fs::create_dir(&target).unwrap();
        assert!(extraer_nativos(
            &[(archive.clone(), vec![])],
            &target,
            &CancellationToken::new()
        )
        .is_err());
        assert!(!path.join("escape.dll").exists());
        let mut zip = zip::ZipWriter::new(File::create(&archive).unwrap());
        zip.start_file("native.dll", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"native").unwrap();
        zip.finish().unwrap();
        fs::write(target.join("native.dll"), b"keep").unwrap();
        assert!(extraer_nativos(&[(archive, vec![])], &target, &CancellationToken::new()).is_err());
        assert_eq!(fs::read(target.join("native.dll")).unwrap(), b"keep");
        fs::remove_dir_all(path).unwrap();
    }

    #[tokio::test]
    #[ignore = "Consulta manifiestos oficiales; no instala ni ejecuta el juego"]
    async fn metadatos_oficiales_actuales_y_legacy() {
        let client = crate::http::client().unwrap();
        let token = CancellationToken::new();
        let bytes = bytes_oficiales(
            &client,
            crate::manifest::VERSION_MANIFEST_URL,
            METADATA_LIMIT as usize,
            &token,
        )
        .await
        .unwrap();
        let manifest = crate::manifest::VersionManifest::parse(&bytes).unwrap();
        for id in [
            manifest.latest.release.as_str(),
            "1.21.1",
            "1.12.2",
            "1.6.4",
        ] {
            let entry = manifest.find(id).unwrap();
            let bytes = bytes_oficiales(&client, &entry.url, METADATA_LIMIT as usize, &token)
                .await
                .unwrap();
            assert_eq!(crate::manifest::sha1_hex(&bytes), entry.sha1);
            let version: Version = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(version.id, id);
            validar_descarga(version.downloads.get("client").unwrap()).unwrap();
            let assets = bytes_oficiales(
                &client,
                &version.asset_index.url,
                METADATA_LIMIT as usize,
                &token,
            )
            .await
            .unwrap();
            assert_eq!(assets.len() as u64, version.asset_index.size);
            assert_eq!(crate::manifest::sha1_hex(&assets), version.asset_index.sha1);
            let index: Indice = serde_json::from_slice(&assets).unwrap();
            for name in index.objects.keys() {
                relativa(name).unwrap();
            }
            for os in ["windows", "linux"] {
                if let Some(args) = &version.arguments {
                    argumentos(&args.game, os, "10.0.26100").unwrap();
                    argumentos(&args.jvm, os, "10.0.26100").unwrap();
                }
                for lib in &version.libraries {
                    if permitida(lib.rules.as_deref(), os, "10.0.26100").unwrap() {
                        if let Some(download) = &lib.downloads.artifact {
                            validar_descarga(download).unwrap();
                            relativa(&download.path).unwrap();
                        }
                        if let Some(classifier) = lib.natives.get(os) {
                            let native = lib
                                .downloads
                                .classifiers
                                .get(&classifier.replace("${arch}", "64"))
                                .unwrap();
                            validar_descarga(native).unwrap();
                            relativa(&native.path).unwrap();
                        }
                    }
                }
            }
        }
    }

    #[tokio::test]
    #[ignore = "Descarga Minecraft 1.21.1 completo desde Mojang en una carpeta temporal aislada; no inicia el juego ni usa cuentas"]
    async fn instala_vanilla_real_y_reutiliza_archivos_verificados() {
        struct Limpieza(PathBuf);
        impl Drop for Limpieza {
            fn drop(&mut self) {
                // temp() creó y devolvió exclusivamente esta carpeta de prueba.
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let root = Limpieza(temp());
        let store = Almacen::nuevo(root.0.join("instances"));
        let instance = store.crear("Vanilla aislada", "1.21.1", 0).unwrap();
        let before = fs::read(store.carpeta(&instance.id).unwrap().join("instance.json")).unwrap();
        let user_file = store.juego(&instance.id).unwrap().join("options.txt");
        fs::write(&user_file, b"usuario:conservar\n").unwrap();
        let progress = std::sync::Arc::new(Mutex::new(None));
        let observer = progress.clone();
        let first = instalar(&store, &instance, CancellationToken::new(), move |p| {
            *observer.lock().unwrap() = Some(p);
        })
        .await
        .unwrap();
        assert!(listo(&store, &instance));
        assert_eq!(first.version, "1.21.1");
        assert_eq!(first.java_major, 21);
        assert_eq!(first.java_component, "java-runtime-delta");
        assert_eq!(leer(&store, &instance).unwrap().natives, first.natives);
        let client = first.classpath.last().unwrap();
        assert_eq!(client.file_name().unwrap(), "client.jar");
        let mut zip = zip::ZipArchive::new(File::open(client).unwrap()).unwrap();
        assert!(zip.by_name("net/minecraft/client/main/Main.class").is_ok());
        drop(zip);
        let native = first
            .classpath
            .iter()
            .find(|path| {
                let name = path.file_name().unwrap().to_string_lossy();
                name.contains(if cfg!(windows) {
                    "natives-windows"
                } else {
                    "natives-linux"
                }) && !name.contains("-arm64")
                    && !name.contains("-x86")
            })
            .unwrap();
        let mut zip = zip::ZipArchive::new(File::open(native).unwrap()).unwrap();
        assert!((0..zip.len()).any(|i| {
            let member = zip.by_index(i).unwrap();
            let name = member.name().unwrap();
            name.ends_with(if cfg!(windows) { ".dll" } else { ".so" })
        }));
        drop(zip);
        assert!(first.natives.is_dir());
        let index: Indice = leer_json(
            first.assets.parent().unwrap(),
            &first
                .assets
                .join("indexes")
                .join(format!("{}.json", first.asset_index)),
        )
        .unwrap();
        assert!(!index.objects.is_empty());
        let object = index.objects.values().next().unwrap();
        let resource = first
            .assets
            .join("objects")
            .join(&object.hash[..2])
            .join(&object.hash);
        let actual = sha1_archivo(&resource, &CancellationToken::new()).unwrap();
        assert_eq!(actual, (object.size, object.hash.clone()));
        let client_time = fs::metadata(client).unwrap().modified().unwrap();
        let resource_time = fs::metadata(&resource).unwrap().modified().unwrap();
        let done = progress.lock().unwrap().clone().unwrap();
        assert!(done.total > 100);
        assert_eq!(done.completados, done.total);
        let second = instalar(&store, &instance, CancellationToken::new(), |_| {})
            .await
            .unwrap();
        assert_eq!(second.natives, first.natives);
        assert_eq!(second.classpath, first.classpath);
        assert_eq!(
            fs::metadata(client).unwrap().modified().unwrap(),
            client_time
        );
        assert_eq!(
            fs::metadata(resource).unwrap().modified().unwrap(),
            resource_time
        );
        assert_eq!(fs::read(&user_file).unwrap(), b"usuario:conservar\n");
        assert_eq!(
            fs::read(store.carpeta(&instance.id).unwrap().join("instance.json")).unwrap(),
            before
        );
        // Un archivo alterado con el mismo tamaño tampoco pasa la verificación.
        let mut jar = fs::read(client).unwrap();
        jar[0] ^= 0xff;
        fs::write(client, jar).unwrap();
        let repaired = instalar(&store, &instance, CancellationToken::new(), |_| {})
            .await
            .unwrap();
        assert!(listo(&store, &instance));
        assert_eq!(repaired.natives, first.natives);
        let mut zip = zip::ZipArchive::new(File::open(client).unwrap()).unwrap();
        assert!(zip.by_name("net/minecraft/client/main/Main.class").is_ok());
    }
}
