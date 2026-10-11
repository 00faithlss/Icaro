// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Mods de Modrinth: dependencias, hashes y sustitución del conjunto preparado.
use crate::{
    instancias::Almacen,
    modrinth::{Catalog, Dependency, File as ModFile, Version},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    io::Write,
    path::{Path, PathBuf},
};

const META: &str = ".icaro-mods.json";
const MAX_FILE: u64 = 128 * 1024 * 1024;

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Red(#[from] crate::modrinth::Error),
    #[error("No se pudieron leer o guardar los mods. Los archivos actuales se conservan.")]
    Disco,
    #[error("Los archivos o metadatos de mods no son válidos.")]
    Datos,
    #[error("Otra operación de mods está en curso. Inténtalo de nuevo al terminar.")]
    Ocupada,
    #[error("No hay una versión compatible de {0} para esta instancia.")]
    Compatible(String),
    #[error("Hay requisitos incompatibles entre los mods: {0}.")]
    Conflicto(String),
    #[error("El archivo descargado no coincide con su tamaño o SHA-512.")]
    Hash,
    #[error("Hay un archivo modificado o un nombre ocupado: {0}. Se conservó el archivo actual.")]
    Archivo(String),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instalado {
    pub project_id: String,
    pub nombre: String,
    pub version_id: String,
    pub version: String,
    #[serde(default)]
    pub publicada: String,
    pub archivo: String,
    pub sha512: String,
    pub activo: bool,
    pub dependencias: Vec<Dependency>,
}
fn nombre_valido(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 240
        && !s.starts_with('.')
        && !s.ends_with('.')
        && !s.ends_with(' ')
        && !s
            .chars()
            .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
        && !matches!(
            s.split('.')
                .next()
                .unwrap_or("")
                .to_ascii_uppercase()
                .as_str(),
            "CON"
                | "PRN"
                | "AUX"
                | "NUL"
                | "COM1"
                | "COM2"
                | "COM3"
                | "COM4"
                | "COM5"
                | "COM6"
                | "COM7"
                | "COM8"
                | "COM9"
                | "LPT1"
                | "LPT2"
                | "LPT3"
                | "LPT4"
                | "LPT5"
                | "LPT6"
                | "LPT7"
                | "LPT8"
                | "LPT9"
        )
}
fn enlace(m: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        m.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        m.file_type().is_symlink()
    }
}
fn regular(path: &Path, directory: bool) -> Result<fs::Metadata, Error> {
    let m = fs::symlink_metadata(path).map_err(|_| Error::Disco)?;
    if enlace(&m) || (directory && !m.is_dir()) || (!directory && !m.is_file()) {
        return Err(Error::Datos);
    }
    Ok(m)
}
fn hash(bytes: &[u8]) -> String {
    Sha512::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn ruta_mod(dir: &Path, m: &Instalado) -> PathBuf {
    dir.join(if m.activo {
        m.archivo.clone()
    } else {
        format!("{}.disabled", m.archivo)
    })
}
pub(crate) fn guardia(game: &Path) -> Result<fs::File, Error> {
    regular(game, true)?;
    let path = game.join(".icaro-mods.lock");
    if path.exists() {
        regular(&path, false)?;
    }
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|_| Error::Disco)?;
    file.try_lock().map_err(|_| Error::Ocupada)?;
    Ok(file)
}
fn recuperar(game: &Path) -> Result<(), Error> {
    let mods = game.join("mods");
    let backup = game.join(".icaro-mods-backup");
    if !mods.exists() && backup.exists() {
        regular(&backup, true)?;
        fs::rename(backup, mods).map_err(|_| Error::Disco)?;
    }
    Ok(())
}
fn registros(game: &Path) -> Result<Vec<Instalado>, Error> {
    let dir = game.join("mods");
    if !dir.exists() {
        return Ok(vec![]);
    }
    regular(&dir, true)?;
    let path = dir.join(META);
    if !path.exists() {
        return Ok(vec![]);
    }
    if regular(&path, false)?.len() > 2 * 1024 * 1024 {
        return Err(Error::Datos);
    }
    let data: Vec<Instalado> = serde_json::from_slice(&fs::read(path).map_err(|_| Error::Disco)?)
        .map_err(|_| Error::Datos)?;
    let mut projects = BTreeSet::new();
    let mut names = BTreeSet::new();
    if data.len() > 512
        || data.iter().any(|m| {
            !nombre_valido(&m.archivo)
                || !m.archivo.ends_with(".jar")
                || !projects.insert(m.project_id.clone())
                || !names.insert(m.archivo.to_lowercase())
        })
    {
        return Err(Error::Datos);
    }
    Ok(data)
}
pub fn inventario(store: &Almacen, id: &str) -> Result<Vec<Instalado>, Error> {
    let game = store.juego(id).map_err(|_| Error::Disco)?;
    let _lock = guardia(&game)?;
    recuperar(&game)?;
    let dir = game.join("mods");
    let mut mods = registros(&game)?;
    let mut known = BTreeSet::new();
    for m in &mut mods {
        let active = dir.join(&m.archivo);
        let disabled = dir.join(format!("{}.disabled", m.archivo));
        if active.exists() {
            regular(&active, false)?;
            m.activo = true;
        } else if disabled.exists() {
            regular(&disabled, false)?;
            m.activo = false;
        } else {
            return Err(Error::Archivo(m.archivo.clone()));
        }
        known.insert(m.archivo.clone());
    }
    if dir.exists() {
        for e in fs::read_dir(dir).map_err(|_| Error::Disco)? {
            let e = e.map_err(|_| Error::Disco)?;
            let name = e.file_name().to_string_lossy().into_owned();
            let base = name.strip_suffix(".disabled").unwrap_or(&name);
            if base.ends_with(".jar") && !known.contains(base) {
                regular(&e.path(), false)?;
                if !nombre_valido(base) {
                    return Err(Error::Datos);
                }
                mods.push(Instalado {
                    project_id: String::new(),
                    nombre: base.into(),
                    version_id: String::new(),
                    version: "Archivo local".into(),
                    publicada: String::new(),
                    archivo: base.into(),
                    sha512: String::new(),
                    activo: name == base,
                    dependencias: vec![],
                });
            }
        }
    }
    Ok(mods)
}
pub trait Fuente: Sync {
    fn version(&self, id: &str)
        -> impl std::future::Future<Output = Result<Version, Error>> + Send;
    fn versiones(
        &self,
        project: &str,
        mc: &str,
        loader: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Version>, Error>> + Send;
    fn nombre(
        &self,
        project: &str,
    ) -> impl std::future::Future<Output = Result<String, Error>> + Send;
    fn archivo(
        &self,
        file: &ModFile,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, Error>> + Send;
}
impl Fuente for Catalog {
    async fn version(&self, id: &str) -> Result<Version, Error> {
        Ok(self.version(id).await?)
    }
    async fn versiones(&self, p: &str, mc: &str, l: &str) -> Result<Vec<Version>, Error> {
        Ok(self.versions(p, Some(mc), Some(l)).await?)
    }
    async fn nombre(&self, p: &str) -> Result<String, Error> {
        Ok(self.project(p).await?.title)
    }
    async fn archivo(&self, f: &ModFile) -> Result<Vec<u8>, Error> {
        Ok(self.download_mod(f).await?)
    }
}
pub fn compatible(v: &Version, mc: &str, loader: &str) -> bool {
    loader != "vanilla"
        && v.game_versions.iter().any(|x| x == mc)
        && v.loaders.iter().any(|x| x.eq_ignore_ascii_case(loader))
}
fn principal(v: &Version) -> Result<&ModFile, Error> {
    let f = v
        .files
        .iter()
        .find(|f| f.primary)
        .or(v.files.first())
        .ok_or(Error::Datos)?;
    if !nombre_valido(&f.filename)
        || !f.filename.ends_with(".jar")
        || f.size == 0
        || f.size > MAX_FILE
        || f.hashes.sha512.len() != 128
        || !f.hashes.sha512.bytes().all(|b| b.is_ascii_hexdigit())
        || !crate::modrinth::supported_image(&f.url)
    {
        return Err(Error::Datos);
    }
    Ok(f)
}
fn ultima(mut versions: Vec<Version>) -> Option<Version> {
    versions.sort_by(|a, b| b.date_published.cmp(&a.date_published));
    versions
        .iter()
        .find(|v| v.version_type == "release")
        .cloned()
        .or_else(|| versions.into_iter().next())
}
pub async fn actualizaciones(
    source: &impl Fuente,
    installed: &[Instalado],
    mc: &str,
    loader: &str,
) -> Result<Vec<Version>, Error> {
    let mut result = vec![];
    for m in installed.iter().filter(|m| !m.project_id.is_empty()) {
        let versions = source.versiones(&m.project_id, mc, loader).await?;
        if let Some(v) = ultima(
            versions
                .into_iter()
                .filter(|v| {
                    v.project_id == m.project_id
                        && compatible(v, mc, loader)
                        && v.version_type == "release"
                })
                .collect(),
        ) {
            let published = if m.publicada.is_empty() {
                source.version(&m.version_id).await?.date_published
            } else {
                m.publicada.clone()
            };
            if v.id != m.version_id && v.date_published > published {
                result.push(v);
            }
        }
    }
    Ok(result)
}
pub async fn resolver(
    source: &impl Fuente,
    roots: Vec<Version>,
    installed: &[Instalado],
    mc: &str,
    loader: &str,
) -> Result<Vec<Version>, Error> {
    let mut selected: BTreeMap<String, Version> = BTreeMap::new();
    let mut queue = VecDeque::new();
    for v in roots {
        if !compatible(&v, mc, loader) {
            return Err(Error::Compatible(v.name));
        }
        if let Some(prior) = selected.insert(v.project_id.clone(), v.clone()) {
            if prior.id != v.id {
                return Err(Error::Conflicto(v.name));
            }
        }
        queue.push_back(v);
    }
    let mut visited = BTreeSet::new();
    while let Some(v) = queue.pop_front() {
        if !visited.insert(v.id.clone()) {
            continue;
        }
        if visited.len() > 128 {
            return Err(Error::Datos);
        }
        for dep in v
            .dependencies
            .iter()
            .filter(|d| d.dependency_type == "required")
        {
            let candidate = if let Some(id) = &dep.version_id {
                let pinned = source.version(id).await?;
                if pinned.id != *id
                    || dep
                        .project_id
                        .as_ref()
                        .is_some_and(|p| *p != pinned.project_id)
                {
                    return Err(Error::Datos);
                }
                pinned
            } else if let Some(p) = &dep.project_id {
                if let Some(existing) = selected.get(p) {
                    existing.clone()
                } else if let Some(existing) = installed.iter().find(|m| m.project_id == *p) {
                    let version = source.version(&existing.version_id).await?;
                    if version.id != existing.version_id {
                        return Err(Error::Datos);
                    }
                    version
                } else {
                    ultima(
                        source
                            .versiones(p, mc, loader)
                            .await?
                            .into_iter()
                            .filter(|v| v.project_id == *p && compatible(v, mc, loader))
                            .collect(),
                    )
                    .ok_or_else(|| Error::Compatible(p.clone()))?
                }
            } else {
                return Err(Error::Compatible(
                    dep.file_name
                        .clone()
                        .unwrap_or("Dependencia externa".into()),
                ));
            };
            if !compatible(&candidate, mc, loader) {
                return Err(Error::Compatible(candidate.name));
            }
            if dep
                .project_id
                .as_ref()
                .is_some_and(|p| *p != candidate.project_id)
            {
                return Err(Error::Datos);
            }
            if let Some(prior) = selected.get(&candidate.project_id) {
                if prior.id != candidate.id {
                    return Err(Error::Conflicto(candidate.name));
                }
            } else {
                selected.insert(candidate.project_id.clone(), candidate.clone());
                queue.push_back(candidate);
            }
        }
    }
    for v in selected.values() {
        for dep in v
            .dependencies
            .iter()
            .filter(|d| d.dependency_type == "incompatible")
        {
            let matches = |project: &str, version: &str| {
                (dep.project_id.is_some() || dep.version_id.is_some())
                    && dep.project_id.as_deref().is_none_or(|p| p == project)
                    && dep.version_id.as_deref().is_none_or(|id| id == version)
            };
            let conflict = selected
                .values()
                .any(|other| matches(&other.project_id, &other.id))
                || installed.iter().any(|other| {
                    !selected.contains_key(&other.project_id)
                        && matches(&other.project_id, &other.version_id)
                });
            if conflict {
                return Err(Error::Conflicto(v.name.clone()));
            }
        }
    }
    for old in installed
        .iter()
        .filter(|m| !selected.contains_key(&m.project_id))
    {
        for dep in &old.dependencias {
            let project = if dep.dependency_type == "required" && dep.project_id.is_none() {
                if let Some(id) = &dep.version_id {
                    let version = source.version(id).await?;
                    if version.id != *id {
                        return Err(Error::Datos);
                    }
                    Some(version.project_id)
                } else {
                    None
                }
            } else {
                dep.project_id.clone()
            };
            if selected.values().any(|v| {
                let same = project.as_deref().is_some_and(|p| p == v.project_id)
                    || dep.version_id.as_deref().is_some_and(|id| id == v.id);
                (dep.dependency_type == "required"
                    && same
                    && dep.version_id.as_deref().is_some_and(|id| id != v.id))
                    || (dep.dependency_type == "incompatible"
                        && same
                        && dep.version_id.as_deref().is_none_or(|id| id == v.id))
            }) {
                return Err(Error::Conflicto(old.nombre.clone()));
            }
        }
    }
    Ok(selected.into_values().collect())
}
fn borrar_preparacion(game: &Path, path: &Path) -> Result<(), Error> {
    regular(path, true)?;
    let root = fs::canonicalize(game).map_err(|_| Error::Disco)?;
    let resolved = fs::canonicalize(path).map_err(|_| Error::Disco)?;
    if resolved.parent() != Some(root.as_path())
        || !resolved
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .starts_with(".icaro-mods-")
    {
        return Err(Error::Datos);
    }
    fs::remove_dir_all(resolved).map_err(|_| Error::Disco)
}
fn copiar(
    source: &Path,
    target: &Path,
    depth: usize,
    total: &mut u64,
    count: &mut usize,
) -> Result<(), Error> {
    if depth > 16 {
        return Err(Error::Datos);
    }
    regular(source, true)?;
    for entry in fs::read_dir(source).map_err(|_| Error::Disco)? {
        let entry = entry.map_err(|_| Error::Disco)?;
        *count += 1;
        if *count > 4096 {
            return Err(Error::Datos);
        }
        let meta = fs::symlink_metadata(entry.path()).map_err(|_| Error::Disco)?;
        if enlace(&meta) {
            return Err(Error::Datos);
        }
        let dest = target.join(entry.file_name());
        if meta.is_dir() {
            fs::create_dir(&dest).map_err(|_| Error::Disco)?;
            copiar(&entry.path(), &dest, depth + 1, total, count)?;
        } else if meta.is_file() {
            *total = total.saturating_add(meta.len());
            if *total > 2 * 1024 * 1024 * 1024 {
                return Err(Error::Datos);
            }
            fs::copy(entry.path(), dest).map_err(|_| Error::Disco)?;
        } else {
            return Err(Error::Datos);
        }
    }
    Ok(())
}
fn comprobar_actual(dir: &Path, m: &Instalado) -> Result<(), Error> {
    let path = ruta_mod(dir, m);
    if regular(&path, false)?.len() > MAX_FILE {
        return Err(Error::Datos);
    }
    let bytes = fs::read(path).map_err(|_| Error::Disco)?;
    if hash(&bytes) != m.sha512.to_lowercase() {
        return Err(Error::Archivo(m.archivo.clone()));
    }
    Ok(())
}
fn guardar_archivo(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    let mut f = fs::File::create(path).map_err(|_| Error::Disco)?;
    f.write_all(bytes)
        .and_then(|_| f.sync_all())
        .map_err(|_| Error::Disco)
}
pub async fn instalar(
    source: &impl Fuente,
    store: &Almacen,
    id: &str,
    roots: Vec<Version>,
    mc: &str,
    loader: &str,
    progress: impl Fn(&str) + Send,
) -> Result<Vec<Instalado>, Error> {
    let game = store.juego(id).map_err(|_| Error::Disco)?;
    let _lock = guardia(&game)?;
    recuperar(&game)?;
    let dir = game.join("mods");
    let mut installed = registros(&game)?;
    for m in &mut installed {
        m.activo = dir.join(&m.archivo).exists();
        comprobar_actual(&dir, m)?;
    }
    progress("Resolviendo dependencias obligatorias…");
    let mut active: BTreeSet<String> = roots
        .iter()
        .filter(|v| {
            installed
                .iter()
                .find(|m| m.project_id == v.project_id)
                .is_none_or(|m| m.activo)
        })
        .map(|v| v.project_id.clone())
        .collect();
    let plan = resolver(source, roots, &installed, mc, loader).await?;
    loop {
        let before = active.len();
        for v in plan
            .iter()
            .filter(|v| active.contains(&v.project_id))
            .cloned()
            .collect::<Vec<_>>()
        {
            for dep in v
                .dependencies
                .iter()
                .filter(|d| d.dependency_type == "required")
            {
                if let Some(target) = plan.iter().find(|p| {
                    dep.project_id
                        .as_deref()
                        .is_some_and(|id| id == p.project_id)
                        || dep.version_id.as_deref().is_some_and(|id| id == p.id)
                }) {
                    active.insert(target.project_id.clone());
                }
            }
        }
        if active.len() == before {
            break;
        }
    }
    let mut random = [0u8; 16];
    getrandom::fill(&mut random).map_err(|_| Error::Disco)?;
    let stage = game.join(format!(
        ".icaro-mods-next-{}",
        random
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    ));
    fs::create_dir(&stage).map_err(|_| Error::Disco)?;
    let result = async {
        if dir.exists() {
            copiar(&dir, &stage, 0, &mut 0, &mut 0)?;
        }
        let mut occupied: BTreeSet<String> =
            installed.iter().map(|m| m.archivo.to_lowercase()).collect();
        for v in plan {
            if let Some(m) = installed
                .iter_mut()
                .find(|m| m.project_id == v.project_id && m.version_id == v.id)
            {
                if !m.activo && active.contains(&v.project_id) {
                    let source = ruta_mod(&stage, m);
                    let target = stage.join(&m.archivo);
                    if target.exists() {
                        return Err(Error::Archivo(m.archivo.clone()));
                    }
                    fs::rename(source, target).map_err(|_| Error::Disco)?;
                    m.activo = true;
                }
                continue;
            }
            let file = principal(&v)?.clone();
            let old = installed.iter().position(|m| m.project_id == v.project_id);
            if occupied.contains(&file.filename.to_lowercase())
                && old.is_none_or(|i| {
                    installed[i].archivo.to_lowercase() != file.filename.to_lowercase()
                })
            {
                return Err(Error::Archivo(file.filename.clone()));
            }
            let activo =
                active.contains(&v.project_id) || old.map_or(true, |i| installed[i].activo);
            let destination = stage.join(if activo {
                file.filename.clone()
            } else {
                format!("{}.disabled", file.filename)
            });
            if destination.exists()
                && old.is_none_or(|i| ruta_mod(&stage, &installed[i]) != destination)
            {
                return Err(Error::Archivo(file.filename.clone()));
            }
            progress(&format!("Descargando {}…", v.name));
            let bytes = source.archivo(&file).await?;
            if bytes.len() as u64 != file.size || hash(&bytes) != file.hashes.sha512.to_lowercase()
            {
                return Err(Error::Hash);
            }
            let name = source.nombre(&v.project_id).await?;
            if let Some(i) = old {
                occupied.remove(&installed[i].archivo.to_lowercase());
                fs::remove_file(ruta_mod(&stage, &installed[i])).map_err(|_| Error::Disco)?;
                installed.remove(i);
            }
            guardar_archivo(&destination, &bytes)?;
            occupied.insert(file.filename.to_lowercase());
            installed.push(Instalado {
                project_id: v.project_id,
                nombre: name,
                version_id: v.id,
                version: v.version_number,
                publicada: v.date_published,
                archivo: file.filename.clone(),
                sha512: file.hashes.sha512.clone(),
                activo,
                dependencias: v.dependencies,
            });
        }
        guardar_archivo(
            &stage.join(META),
            &serde_json::to_vec_pretty(&installed).map_err(|_| Error::Datos)?,
        )?;
        progress("Guardando los mods de la instancia…");
        let backup = game.join(".icaro-mods-backup");
        if backup.exists() {
            borrar_preparacion(&game, &backup)?;
        }
        if dir.exists() {
            regular(&dir, true)?;
            fs::rename(&dir, &backup).map_err(|_| Error::Disco)?;
        }
        if fs::rename(&stage, &dir).is_err() {
            if backup.exists() {
                let _ = fs::rename(&backup, &dir);
            }
            return Err(Error::Disco);
        }
        Ok(installed)
    }
    .await;
    if stage.exists() {
        let _ = borrar_preparacion(&game, &stage);
    }
    result
}

pub fn alternar(store: &Almacen, id: &str, filename: &str) -> Result<(), Error> {
    if !nombre_valido(filename) {
        return Err(Error::Datos);
    }
    let game = store.juego(id).map_err(|_| Error::Disco)?;
    let _lock = guardia(&game)?;
    recuperar(&game)?;
    let dir = game.join("mods");
    let active = dir.join(filename);
    let disabled = dir.join(format!("{filename}.disabled"));
    let (source, target) = if active.exists() {
        (active, disabled)
    } else {
        (disabled, active)
    };
    regular(&source, false)?;
    if target.exists() {
        return Err(Error::Archivo(filename.into()));
    }
    fs::rename(source, target).map_err(|_| Error::Disco)
}

/// Ticket de una retirada reversible. La ruta no se acepta desde la interfaz.
pub struct Retirados {
    instancia: String,
    carpeta: PathBuf,
    mods: Vec<Instalado>,
}

fn guardar_registros(game: &Path, list: &[Instalado]) -> Result<(), Error> {
    let bytes = serde_json::to_vec_pretty(list).map_err(|_| Error::Datos)?;
    let dir = game.join("mods");
    let temp = dir.join(".icaro-mods.json.next");
    if temp.exists() {
        regular(&temp, false)?;
    }
    fs::write(&temp, bytes).map_err(|_| Error::Disco)?;
    fs::rename(temp, dir.join(".icaro-mods.json")).map_err(|_| Error::Disco)
}

pub fn retirar(store: &Almacen, id: &str, files: &BTreeSet<String>) -> Result<Retirados, Error> {
    if files.is_empty() {
        return Err(Error::Datos);
    }
    let inventory = inventario(store, id)?;
    let game = store.juego(id).map_err(|_| Error::Disco)?;
    let _lock = guardia(&game)?;
    let manifest = registros(&game)?;
    let selected: Vec<_> = inventory
        .into_iter()
        .filter(|m| files.contains(&m.archivo))
        .collect();
    if selected.len() != files.len() {
        return Err(Error::Datos);
    }
    let root = game.join(".icaro-mods-trash");
    if !root.exists() {
        fs::create_dir(&root).map_err(|_| Error::Disco)?;
    }
    regular(&root, true)?;
    let mut random = [0u8; 16];
    getrandom::fill(&mut random).map_err(|_| Error::Datos)?;
    let trash = root.join(hash(&random));
    fs::create_dir(&trash).map_err(|_| Error::Disco)?;
    let dir = game.join("mods");
    let mut moved: Vec<(PathBuf, PathBuf)> = vec![];
    let result = (|| {
        for item in &selected {
            let src = ruta_mod(&dir, item);
            regular(&src, false)?;
            let dst = trash.join(src.file_name().ok_or(Error::Datos)?);
            fs::rename(&src, &dst).map_err(|_| Error::Disco)?;
            moved.push((src, dst));
        }
        let kept: Vec<_> = manifest
            .into_iter()
            .filter(|m| !files.contains(&m.archivo))
            .collect();
        guardar_registros(&game, &kept)
    })();
    if let Err(e) = result {
        for (src, dst) in moved.into_iter().rev() {
            let _ = fs::rename(dst, src);
        }
        return Err(e);
    }
    Ok(Retirados {
        instancia: id.into(),
        carpeta: trash,
        mods: selected,
    })
}

pub fn restaurar(store: &Almacen, id: &str, ticket: &Retirados) -> Result<(), Error> {
    if ticket.instancia != id {
        return Err(Error::Datos);
    }
    let game = store.juego(id).map_err(|_| Error::Disco)?;
    let _lock = guardia(&game)?;
    regular(&ticket.carpeta, true)?;
    let parent = ticket.carpeta.parent().ok_or(Error::Datos)?;
    regular(parent, true)?;
    if parent.canonicalize().map_err(|_| Error::Disco)?
        != game
            .join(".icaro-mods-trash")
            .canonicalize()
            .map_err(|_| Error::Disco)?
    {
        return Err(Error::Datos);
    }
    let dir = game.join("mods");
    let mut manifest = registros(&game)?;
    for item in &ticket.mods {
        if dir.join(&item.archivo).exists()
            || dir.join(format!("{}.disabled", item.archivo)).exists()
            || manifest.iter().any(|m| {
                m.archivo == item.archivo
                    || (!item.project_id.is_empty() && m.project_id == item.project_id)
            })
        {
            return Err(Error::Archivo(item.archivo.clone()));
        }
        regular(&ruta_mod(&ticket.carpeta, item), false)?;
    }
    let mut moved = vec![];
    for item in &ticket.mods {
        let src = ruta_mod(&ticket.carpeta, item);
        let dst = ruta_mod(&dir, item);
        if fs::rename(&src, &dst).is_err() {
            for (s, d) in moved.into_iter().rev() {
                let _ = fs::rename(d, s);
            }
            return Err(Error::Disco);
        }
        moved.push((src, dst));
        if !item.project_id.is_empty() {
            manifest.push(item.clone());
        }
    }
    if let Err(e) = guardar_registros(&game, &manifest) {
        for (s, d) in moved.into_iter().rev() {
            let _ = fs::rename(d, s);
        }
        return Err(e);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Prueba(PathBuf);
    impl Prueba {
        fn nueva() -> Self {
            let mut bytes = [0; 16];
            getrandom::fill(&mut bytes).unwrap();
            Self(std::env::temp_dir().join(format!("icaro-mods-test-{}", hash(&bytes))))
        }
        fn instancia(&self) -> (Almacen, crate::instancias::Instancia) {
            let store = Almacen::nuevo(&self.0);
            let i = store
                .crear_con_loader("Mundo", "1.21.1", "fabric", 0)
                .unwrap();
            (store, i)
        }
    }
    impl Drop for Prueba {
        fn drop(&mut self) {
            if self.0.parent() == Some(std::env::temp_dir().as_path())
                && self
                    .0
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("icaro-mods-test-")
            {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
    }
    struct Fake {
        versions: Vec<Version>,
        bad: bool,
        downloads: AtomicUsize,
    }
    impl Fuente for Fake {
        async fn version(&self, id: &str) -> Result<Version, Error> {
            self.versions
                .iter()
                .find(|v| v.id == id)
                .cloned()
                .ok_or(Error::Datos)
        }
        async fn versiones(&self, project: &str, _: &str, _: &str) -> Result<Vec<Version>, Error> {
            Ok(self
                .versions
                .iter()
                .filter(|v| v.project_id == project)
                .cloned()
                .collect())
        }
        async fn nombre(&self, project: &str) -> Result<String, Error> {
            Ok(format!("Nombre {project}"))
        }
        async fn archivo(&self, file: &ModFile) -> Result<Vec<u8>, Error> {
            self.downloads.fetch_add(1, Ordering::Relaxed);
            Ok(if self.bad {
                b"bad".to_vec()
            } else {
                file.filename.as_bytes().to_vec()
            })
        }
    }
    fn version(project: &str, id: &str, dependencies: Vec<Dependency>) -> Version {
        let filename = format!("{project}-{id}.jar");
        serde_json::from_value(serde_json::json!({"id":format!("{project}{id}"),"project_id":project,"name":id,"version_number":id,"version_type":"release","game_versions":["1.21.1"],"loaders":["fabric"],"files":[{"filename":filename,"url":format!("https://cdn.modrinth.com/data/{project}/versions/{id}/{filename}"),"size":filename.len(),"primary":true,"hashes":{"sha512":hash(filename.as_bytes())}}],"dependencies":dependencies,"date_published":format!("2026-10-{id}T00:00:00Z")})).unwrap()
    }
    fn required(project: &str, id: Option<&str>) -> Dependency {
        Dependency {
            project_id: Some(project.into()),
            version_id: id.map(|id| format!("{project}{id}")),
            dependency_type: "required".into(),
            file_name: None,
        }
    }
    fn fake(versions: Vec<Version>) -> Fake {
        Fake {
            versions,
            bad: false,
            downloads: AtomicUsize::new(0),
        }
    }
    #[tokio::test]
    async fn retirar_y_deshacer_preserva_mods_manuales_y_no_pisa_archivos_nuevos() {
        let p = Prueba::nueva();
        let (store, i) = p.instancia();
        let v = version("mod", "01", vec![]);
        let source = fake(vec![v.clone()]);
        instalar(&source, &store, &i.id, vec![v], "1.21.1", "fabric", |_| {})
            .await
            .unwrap();
        let game = store.juego(&i.id).unwrap();
        let dir = game.join("mods");
        fs::write(dir.join("manual.jar"), b"manual").unwrap();
        let before = inventario(&store, &i.id).unwrap();
        let files = before.iter().map(|m| m.archivo.clone()).collect();
        let ticket = retirar(&store, &i.id, &files).unwrap();
        assert!(inventario(&store, &i.id).unwrap().is_empty());
        fs::write(dir.join("manual.jar"), b"nuevo").unwrap();
        assert!(restaurar(&store, &i.id, &ticket).is_err());
        assert_eq!(fs::read(dir.join("manual.jar")).unwrap(), b"nuevo");
        assert!(!dir.join("mod-01.jar").exists());
        fs::rename(dir.join("manual.jar"), dir.join("nuevo.jar")).unwrap();
        restaurar(&store, &i.id, &ticket).unwrap();
        let restored = inventario(&store, &i.id).unwrap();
        assert_eq!(restored.len(), 3);
        assert!(restored.iter().any(|m| m.project_id == "mod"));
        assert_eq!(fs::read(dir.join("manual.jar")).unwrap(), b"manual");
    }
    #[tokio::test]
    async fn required_dependencies_install_once_and_disabled_requirements_become_active() {
        let p = Prueba::nueva();
        let (store, i) = p.instancia();
        let dep = version("dep", "01", vec![]);
        let root = version(
            "root",
            "01",
            vec![
                required("dep", None),
                required("dep", Some("01")),
                Dependency {
                    dependency_type: "optional".into(),
                    ..required("optional", None)
                },
            ],
        );
        let source = fake(vec![dep.clone(), root.clone()]);
        instalar(
            &source,
            &store,
            &i.id,
            vec![dep],
            "1.21.1",
            "fabric",
            |_| {},
        )
        .await
        .unwrap();
        alternar(&store, &i.id, "dep-01.jar").unwrap();
        let game = store.juego(&i.id).unwrap();
        fs::write(game.join("mods/manual.jar"), b"local").unwrap();
        instalar(
            &source,
            &store,
            &i.id,
            vec![root],
            "1.21.1",
            "fabric",
            |_| {},
        )
        .await
        .unwrap();
        let reopened = inventario(&Almacen::nuevo(&p.0), &i.id).unwrap();
        assert_eq!(source.downloads.load(Ordering::Relaxed), 2);
        assert_eq!(reopened.len(), 3);
        assert!(
            reopened
                .iter()
                .find(|m| m.project_id == "dep")
                .unwrap()
                .activo
        );
        assert_eq!(
            reopened
                .iter()
                .find(|m| m.project_id == "root")
                .unwrap()
                .nombre,
            "Nombre root"
        );
        assert_eq!(fs::read(game.join("mods/manual.jar")).unwrap(), b"local");
        assert!(!game.join("mods/optional-01.jar").exists());
    }
    #[tokio::test]
    async fn failed_update_keeps_the_entire_old_set_and_modified_files_are_preserved() {
        let p = Prueba::nueva();
        let (store, i) = p.instancia();
        let old = version("root", "01", vec![]);
        let new = version("root", "02", vec![required("dep", None)]);
        let dep = version("dep", "01", vec![]);
        let mut source = fake(vec![old.clone(), new.clone(), dep]);
        instalar(
            &source,
            &store,
            &i.id,
            vec![old],
            "1.21.1",
            "fabric",
            |_| {},
        )
        .await
        .unwrap();
        let dir = store.juego(&i.id).unwrap().join("mods");
        let metadata = fs::read(dir.join(META)).unwrap();
        source.bad = true;
        assert!(matches!(
            instalar(
                &source,
                &store,
                &i.id,
                vec![new.clone()],
                "1.21.1",
                "fabric",
                |_| {}
            )
            .await,
            Err(Error::Hash)
        ));
        assert_eq!(fs::read(dir.join(META)).unwrap(), metadata);
        assert_eq!(fs::read(dir.join("root-01.jar")).unwrap(), b"root-01.jar");
        assert!(!dir.join("root-02.jar").exists());
        source.bad = false;
        fs::write(dir.join("root-01.jar"), b"user change").unwrap();
        assert!(matches!(
            instalar(
                &source,
                &store,
                &i.id,
                vec![new],
                "1.21.1",
                "fabric",
                |_| {}
            )
            .await,
            Err(Error::Archivo(_))
        ));
        assert_eq!(fs::read(dir.join("root-01.jar")).unwrap(), b"user change");
    }
    #[tokio::test]
    async fn incompatible_versions_conflicting_pins_cycles_and_unsafe_names() {
        let mut wrong = version("root", "01", vec![]);
        wrong.loaders = vec!["forge".into()];
        let source = fake(vec![]);
        assert!(matches!(
            resolver(&source, vec![wrong], &[], "1.21.1", "fabric").await,
            Err(Error::Compatible(_))
        ));
        let a = version("a", "01", vec![required("b", Some("01"))]);
        let b = version("b", "01", vec![required("a", Some("01"))]);
        // Version IDs are globally unique in Modrinth.
        let mut a = a;
        a.id = "a01".into();
        a.dependencies[0].version_id = Some("b01".into());
        let mut b = b;
        b.id = "b01".into();
        b.dependencies[0].version_id = Some("a01".into());
        let mut c = version("c", "01", vec![required("b", Some("02"))]);
        c.id = "c01".into();
        let mut b2 = version("b", "02", vec![]);
        b2.id = "b02".into();
        let source = fake(vec![a.clone(), b, c.clone(), b2]);
        assert_eq!(
            resolver(&source, vec![a.clone()], &[], "1.21.1", "fabric")
                .await
                .unwrap()
                .len(),
            2
        );
        assert!(matches!(
            resolver(&source, vec![a, c], &[], "1.21.1", "fabric").await,
            Err(Error::Conflicto(_))
        ));
        for s in [
            "../mod.jar",
            "..\\mod.jar",
            "CON.jar",
            "mods:secret.jar",
            "mod.jar ",
            ".hidden.jar",
        ] {
            assert!(!nombre_valido(s));
        }
        let p = Prueba::nueva();
        let (store, i) = p.instancia();
        let mut bad = version("x", "01", vec![]);
        bad.files[0].filename = "../outside.jar".into();
        assert!(matches!(
            instalar(
                &source,
                &store,
                &i.id,
                vec![bad],
                "1.21.1",
                "fabric",
                |_| {}
            )
            .await,
            Err(Error::Datos)
        ));
        assert!(!store.juego(&i.id).unwrap().join("mods").exists());
    }
    #[tokio::test]
    async fn updating_a_library_respects_a_version_only_pin_of_an_installed_mod() {
        let p = Prueba::nueva();
        let (store, i) = p.instancia();
        let dep = version("dep", "01", vec![]);
        let mut pin = required("dep", Some("01"));
        pin.project_id = None;
        let parent = version("parent", "01", vec![pin]);
        let update = version("dep", "02", vec![]);
        let source = fake(vec![dep, parent.clone(), update.clone()]);
        instalar(
            &source,
            &store,
            &i.id,
            vec![parent],
            "1.21.1",
            "fabric",
            |_| {},
        )
        .await
        .unwrap();
        let installed = inventario(&store, &i.id).unwrap();
        assert!(matches!(
            resolver(&source, vec![update], &installed, "1.21.1", "fabric").await,
            Err(Error::Conflicto(_))
        ));
    }
    #[tokio::test]
    async fn newer_stable_updates_do_not_downgrade_newer_beta_and_lock_serializes_writes() {
        let p = Prueba::nueva();
        let (store, i) = p.instancia();
        let old = version("x", "01", vec![]);
        let mut newest = version("x", "03", vec![]);
        newest.version_type = "beta".into();
        let release = version("x", "02", vec![]);
        let source = fake(vec![old.clone(), newest.clone(), release]);
        instalar(
            &source,
            &store,
            &i.id,
            vec![old],
            "1.21.1",
            "fabric",
            |_| {},
        )
        .await
        .unwrap();
        assert_eq!(
            actualizaciones(
                &source,
                &inventario(&store, &i.id).unwrap(),
                "1.21.1",
                "fabric"
            )
            .await
            .unwrap()[0]
                .id,
            "x02"
        );
        instalar(
            &source,
            &store,
            &i.id,
            vec![newest],
            "1.21.1",
            "fabric",
            |_| {},
        )
        .await
        .unwrap();
        assert!(actualizaciones(
            &source,
            &inventario(&store, &i.id).unwrap(),
            "1.21.1",
            "fabric"
        )
        .await
        .unwrap()
        .is_empty());
        let _lock = guardia(&store.juego(&i.id).unwrap()).unwrap();
        assert!(matches!(
            alternar(&store, &i.id, "x-03.jar"),
            Err(Error::Ocupada)
        ));
    }
    #[tokio::test]
    #[ignore = "Consulta Modrinth y descarga mods a una instancia temporal aislada"]
    async fn modrinth_real_required_dependencies() {
        let p = Prueba::nueva();
        let (store, i) = p.instancia();
        let catalog = Catalog::new().unwrap();
        let root = ultima(
            catalog
                .versions("reeses-sodium-options", Some("1.21.1"), Some("fabric"))
                .await
                .unwrap()
                .into_iter()
                .filter(|v| compatible(v, "1.21.1", "fabric") && v.version_type == "release")
                .collect(),
        )
        .unwrap();
        assert!(root
            .dependencies
            .iter()
            .any(|d| d.dependency_type == "required"));
        let result = instalar(
            &catalog,
            &store,
            &i.id,
            vec![root],
            "1.21.1",
            "fabric",
            |_| {},
        )
        .await
        .unwrap();
        assert!(result.len() >= 2);
        assert!(result.iter().any(|m| m.nombre == "Sodium"));
        for m in result {
            comprobar_actual(&store.juego(&i.id).unwrap().join("mods"), &m).unwrap();
        }
    }
}
