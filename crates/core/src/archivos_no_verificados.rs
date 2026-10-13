// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Revisión de procedencia de JAR locales, utilizable antes o después de importar.
//! Una ausencia en Modrinth no es evidencia de malware. La confianza se aplica
//! al contenido SHA-512 exacto y desaparece si el archivo se sustituye.

use crate::{
    instancias::Almacen,
    modrinth::{Catalog, Version},
    mods,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_JAR: u64 = 128 * 1024 * 1024;
const MAX_CACHE: u64 = 2 * 1024 * 1024;
const CACHE: &str = ".icaro-verificacion.json";

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("No se pudo revisar el archivo local. Sus datos se conservan.")]
    Disco,
    #[error("El archivo cambió desde su revisión. Revísalo otra vez antes de continuar.")]
    Cambio,
    #[error("El archivo o los datos de verificación no son válidos.")]
    Datos,
    #[error(transparent)]
    Mods(#[from] mods::Error),
    #[error(transparent)]
    Red(#[from] crate::modrinth::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Procedencia {
    Modrinth { proyecto: String, version: String },
    Confiado,
    Desconocida,
}

#[derive(Debug, Clone)]
pub struct Archivo {
    pub nombre: String,
    pub bytes: u64,
    pub sha512: String,
    pub activo: bool,
    pub procedencia: Procedencia,
}

#[derive(Debug, Clone, Default)]
pub struct Informe {
    pub archivos: Vec<Archivo>,
    /// Una consulta fallida sigue permitiendo revisar y confiar en los archivos.
    pub error_red: Option<String>,
}
impl Informe {
    pub fn pendientes(&self) -> impl Iterator<Item = &Archivo> {
        self.archivos
            .iter()
            .filter(|a| a.procedencia == Procedencia::Desconocida)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Entrada {
    #[serde(default)]
    confiado: bool,
    #[serde(default)]
    consultado: u64,
    proyecto: Option<String>,
    version: Option<String>,
}
type Cache = BTreeMap<String, Entrada>;

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
fn regular(path: &Path, dir: bool) -> Result<fs::Metadata, Error> {
    let m = fs::symlink_metadata(path).map_err(|_| Error::Disco)?;
    if enlace(&m) || (dir && !m.is_dir()) || (!dir && !m.is_file()) {
        return Err(Error::Datos);
    }
    Ok(m)
}
fn digest(path: &Path) -> Result<(u64, String), Error> {
    let size = regular(path, false)?.len();
    if size > MAX_JAR {
        return Err(Error::Datos);
    }
    let mut file = fs::File::open(path)
        .map_err(|_| Error::Disco)?
        .take(MAX_JAR + 1);
    let mut hash = Sha512::new();
    let mut bytes = 0;
    let mut buffer = [0; 64 * 1024];
    loop {
        let n = file.read(&mut buffer).map_err(|_| Error::Disco)?;
        if n == 0 {
            break;
        }
        bytes += n as u64;
        if bytes > MAX_JAR {
            return Err(Error::Datos);
        }
        hash.update(&buffer[..n]);
    }
    if bytes != size {
        return Err(Error::Cambio);
    }
    Ok((
        bytes,
        hash.finalize().iter().map(|b| format!("{b:02x}")).collect(),
    ))
}
fn ahora() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn leer_cache(game: &Path) -> Result<Cache, Error> {
    let path = game.join(CACHE);
    if !path.exists() {
        return Ok(Default::default());
    }
    if regular(&path, false)?.len() > MAX_CACHE {
        return Err(Error::Datos);
    }
    let data = fs::read(path).map_err(|_| Error::Disco)?;
    if data.len() as u64 > MAX_CACHE {
        return Err(Error::Datos);
    }
    let cache: Cache = serde_json::from_slice(&data).map_err(|_| Error::Datos)?;
    if cache.len() > 4096
        || cache
            .keys()
            .any(|h| h.len() != 128 || !h.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err(Error::Datos);
    }
    Ok(cache)
}
fn guardar_cache(game: &Path, cache: &Cache) -> Result<(), Error> {
    let mut nonce = [0; 16];
    getrandom::fill(&mut nonce).map_err(|_| Error::Disco)?;
    let temp = game.join(format!(
        ".verificacion-{}.tmp",
        nonce.iter().map(|b| format!("{b:02x}")).collect::<String>()
    ));
    let path = game.join(CACHE);
    if path.exists() {
        regular(&path, false)?;
    }
    let bytes = serde_json::to_vec_pretty(cache).map_err(|_| Error::Datos)?;
    if bytes.len() as u64 > MAX_CACHE {
        return Err(Error::Datos);
    }
    let result = (|| {
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|_| Error::Disco)?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|_| Error::Disco)?;
        drop(file);
        fs::rename(&temp, path).map_err(|_| Error::Disco)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}
fn ruta(game: &Path, a: &Archivo) -> Result<PathBuf, Error> {
    if a.nombre.is_empty()
        || a.nombre.starts_with('.')
        || !a.nombre.ends_with(".jar")
        || a.nombre
            .chars()
            .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
    {
        return Err(Error::Datos);
    }
    let dir = game.join("mods");
    regular(&dir, true)?;
    let path = dir.join(if a.activo {
        a.nombre.clone()
    } else {
        format!("{}.disabled", a.nombre)
    });
    let (_, hash) = digest(&path)?;
    if hash != a.sha512 {
        return Err(Error::Cambio);
    }
    Ok(path)
}

pub trait Fuente: Sync {
    fn versiones(
        &self,
        hashes: &[String],
    ) -> impl std::future::Future<Output = Result<BTreeMap<String, Version>, Error>> + Send;
}
impl Fuente for Catalog {
    async fn versiones(&self, hashes: &[String]) -> Result<BTreeMap<String, Version>, Error> {
        Ok(self.versions_from_hashes(hashes).await?)
    }
}

pub async fn buscar_modrinth(nombre: &str) -> Result<(), Error> {
    if nombre.is_empty() || nombre.len() > 240 || nombre.chars().any(char::is_control) {
        return Err(Error::Datos);
    }
    let mut url = reqwest::Url::parse("https://modrinth.com/mods").map_err(|_| Error::Datos)?;
    url.query_pairs_mut()
        .append_pair("q", nombre.strip_suffix(".jar").unwrap_or(nombre));
    Ok(crate::modrinth::open_link(url.as_str()).await?)
}

/// Llamar desde una tarea en segundo plano; calcula el hash en streaming y
/// consulta en un solo lote. La caché se reutiliza exclusivamente por SHA-512.
pub async fn revisar<F: Fuente>(fuente: &F, store: &Almacen, id: &str) -> Result<Informe, Error> {
    let inventory = mods::inventario(store, id)?;
    let game = store.juego(id).map_err(|_| Error::Disco)?;
    let (mut cache, mut archivos) = {
        let _lock = mods::guardia(&game)?;
        let cache = leer_cache(&game)?;
        let mut archivos = vec![];
        for m in inventory {
            let dir = game.join("mods");
            regular(&dir, true)?;
            let path = dir.join(if m.activo {
                m.archivo.clone()
            } else {
                format!("{}.disabled", m.archivo)
            });
            let (bytes, sha512) = digest(&path)?;
            let procedencia = if !m.project_id.is_empty()
                && !m.version_id.is_empty()
                && sha512 == m.sha512.to_lowercase()
            {
                Procedencia::Modrinth {
                    proyecto: m.project_id,
                    version: m.version_id,
                }
            } else {
                Procedencia::Desconocida
            };
            archivos.push(Archivo {
                nombre: m.archivo,
                bytes,
                sha512,
                activo: m.activo,
                procedencia,
            });
        }
        (cache, archivos)
    };
    let now = ahora();
    let mut pendientes = vec![];
    for a in &mut archivos {
        if a.procedencia != Procedencia::Desconocida {
            continue;
        }
        match cache.get(&a.sha512) {
            Some(e) if e.confiado => a.procedencia = Procedencia::Confiado,
            Some(e) if now.saturating_sub(e.consultado) < 86400 && e.consultado > 0 => {
                if let (Some(proyecto), Some(version)) = (&e.proyecto, &e.version) {
                    a.procedencia = Procedencia::Modrinth {
                        proyecto: proyecto.clone(),
                        version: version.clone(),
                    };
                }
            }
            _ => {
                if !pendientes.contains(&a.sha512) {
                    pendientes.push(a.sha512.clone());
                }
            }
        }
    }
    let mut error_red = None;
    if !pendientes.is_empty() {
        match fuente.versiones(&pendientes).await {
            Ok(resultados) => {
                if resultados.iter().any(|(h, v)| {
                    !pendientes.contains(h)
                        || !v
                            .files
                            .iter()
                            .any(|f| f.hashes.sha512.eq_ignore_ascii_case(h))
                }) {
                    return Err(Error::Datos);
                }
                for hash in pendientes {
                    let version = resultados.get(&hash);
                    cache.insert(
                        hash.clone(),
                        Entrada {
                            confiado: false,
                            consultado: now,
                            proyecto: version.map(|v| v.project_id.clone()),
                            version: version.map(|v| v.id.clone()),
                        },
                    );
                    if let Some(v) = version {
                        for a in archivos.iter_mut().filter(|a| a.sha512 == hash) {
                            a.procedencia = Procedencia::Modrinth {
                                proyecto: v.project_id.clone(),
                                version: v.id.clone(),
                            };
                        }
                    }
                }
                let _lock = mods::guardia(&game)?;
                for a in &archivos {
                    ruta(&game, a)?;
                }
                // Una confianza dada mientras la consulta viajaba no se pierde.
                for (h, e) in leer_cache(&game)? {
                    if e.confiado {
                        for a in archivos
                            .iter_mut()
                            .filter(|a| a.sha512 == h && a.procedencia == Procedencia::Desconocida)
                        {
                            a.procedencia = Procedencia::Confiado;
                        }
                        cache.insert(h, e);
                    }
                }
                guardar_cache(&game, &cache)?;
            }
            Err(e) => error_red = Some(e.to_string()),
        }
    }
    let _lock = mods::guardia(&game)?;
    for a in &archivos {
        ruta(&game, a)?;
    }
    Ok(Informe {
        archivos,
        error_red,
    })
}

pub fn confiar(store: &Almacen, id: &str, archivo: &Archivo) -> Result<(), Error> {
    let game = store.juego(id).map_err(|_| Error::Disco)?;
    let _lock = mods::guardia(&game)?;
    ruta(&game, archivo)?;
    let mut cache = leer_cache(&game)?;
    cache.entry(archivo.sha512.clone()).or_default().confiado = true;
    guardar_cache(&game, &cache)
}

/// Desactiva, sin borrar y sin acusar al archivo; útil cuando un importador
/// recibe una decisión o evidencia concreta de su proveedor.
pub fn desactivar(store: &Almacen, id: &str, archivo: &Archivo) -> Result<(), Error> {
    let game = store.juego(id).map_err(|_| Error::Disco)?;
    let _lock = mods::guardia(&game)?;
    let source = ruta(&game, archivo)?;
    if !archivo.activo {
        return Ok(());
    }
    let target = source.with_file_name(format!("{}.disabled", archivo.nombre));
    if target.exists() {
        return Err(Error::Datos);
    }
    fs::rename(source, target).map_err(|_| Error::Disco)
}

/// Retirada reversible que conserva también el registro de un mod modificado.
pub struct Retirado(mods::Retirados);
impl std::fmt::Debug for Retirado {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Retirado")
    }
}
pub fn retirar(store: &Almacen, id: &str, archivo: &Archivo) -> Result<Retirado, Error> {
    Ok(Retirado(mods::retirar_con_hash(
        store,
        id,
        &std::collections::BTreeSet::from([archivo.nombre.clone()]),
        &BTreeMap::from([(archivo.nombre.clone(), archivo.sha512.clone())]),
    )?))
}
pub fn restaurar(store: &Almacen, id: &str, ticket: &Retirado) -> Result<(), Error> {
    Ok(mods::restaurar(store, id, &ticket.0)?)
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
            Self(std::env::temp_dir().join(format!(
                "icaro-verificacion-test-{}",
                bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
            )))
        }
        fn instancia(&self) -> (Almacen, String, PathBuf) {
            let store = Almacen::nuevo(&self.0);
            let i = store.crear("Prueba", "1.21.4", 0).unwrap();
            let dir = store.juego(&i.id).unwrap().join("mods");
            fs::create_dir(&dir).unwrap();
            (store, i.id, dir)
        }
    }
    impl Drop for Prueba {
        fn drop(&mut self) {
            if self.0.parent() == Some(std::env::temp_dir().as_path()) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
    }
    struct FuentePrueba {
        llamada: AtomicUsize,
        verificado: bool,
        falla: bool,
        cambiar: Option<PathBuf>,
    }
    impl FuentePrueba {
        fn nueva(verificado: bool) -> Self {
            Self {
                llamada: AtomicUsize::new(0),
                verificado,
                falla: false,
                cambiar: None,
            }
        }
    }
    impl Fuente for FuentePrueba {
        async fn versiones(&self, hashes: &[String]) -> Result<BTreeMap<String, Version>, Error> {
            self.llamada.fetch_add(1, Ordering::SeqCst);
            if let Some(path) = &self.cambiar {
                fs::write(path, b"changed in transit").unwrap();
            }
            if self.falla {
                return Err(Error::Red(crate::modrinth::Error::Network("Modrinth")));
            }
            let mut result = BTreeMap::new();
            if self.verificado {
                for h in hashes {
                    let v = serde_json::from_value(serde_json::json!({
                        "id":"v1", "project_id":"project", "name":"V1", "version_number":"1", "version_type":"release",
                        "game_versions":["1.21.4"], "loaders":["fabric"], "files":[{"filename":"local.jar",
                        "url":"https://cdn.modrinth.com/data/project/versions/v1/local.jar", "size":4, "primary":true,
                        "hashes":{"sha512":h}}], "dependencies":[], "changelog":null
                    })).unwrap();
                    result.insert(h.clone(), v);
                }
            }
            Ok(result)
        }
    }
    #[tokio::test]
    async fn matching_hash_is_verified_and_batch_results_are_cached() {
        let p = Prueba::nueva();
        let (store, id, dir) = p.instancia();
        fs::write(dir.join("local.jar"), b"local").unwrap();
        fs::write(dir.join("second.jar.disabled"), b"local2").unwrap();
        let source = FuentePrueba::nueva(true);
        let a = revisar(&source, &store, &id).await.unwrap();
        assert_eq!(a.archivos.len(), 2);
        assert_eq!(a.pendientes().count(), 0);
        assert!(a
            .archivos
            .iter()
            .all(|a| matches!(a.procedencia, Procedencia::Modrinth { .. })));
        revisar(&source, &store, &id).await.unwrap();
        assert_eq!(source.llamada.load(Ordering::SeqCst), 1);
    }
    #[tokio::test]
    async fn trust_is_for_exact_contents_and_never_hides_replacement_file() {
        let p = Prueba::nueva();
        let (store, id, dir) = p.instancia();
        fs::write(dir.join("local.jar"), b"local").unwrap();
        let source = FuentePrueba::nueva(false);
        let a = revisar(&source, &store, &id)
            .await
            .unwrap()
            .archivos
            .remove(0);
        confiar(&store, &id, &a).unwrap();
        assert_eq!(
            revisar(&source, &store, &id).await.unwrap().archivos[0].procedencia,
            Procedencia::Confiado
        );
        assert_eq!(source.llamada.load(Ordering::SeqCst), 1);
        fs::write(dir.join("local.jar"), b"new contents").unwrap();
        assert!(matches!(confiar(&store, &id, &a), Err(Error::Cambio)));
        assert_eq!(
            revisar(&source, &store, &id)
                .await
                .unwrap()
                .pendientes()
                .count(),
            1
        );
        assert_eq!(source.llamada.load(Ordering::SeqCst), 2);
    }
    #[tokio::test]
    async fn outage_does_not_claim_malware_or_disable_local_mods() {
        let p = Prueba::nueva();
        let (store, id, dir) = p.instancia();
        fs::write(dir.join("local.jar"), b"local").unwrap();
        let mut source = FuentePrueba::nueva(false);
        source.falla = true;
        let report = revisar(&source, &store, &id).await.unwrap();
        assert!(report.error_red.is_some());
        assert_eq!(report.pendientes().count(), 1);
        assert!(report.archivos[0].activo);
        assert!(dir.join("local.jar").exists());
        assert!(!dir.join("local.jar.disabled").exists());
    }
    #[tokio::test]
    async fn file_change_during_query_cannot_get_verified_or_trusted() {
        let p = Prueba::nueva();
        let (store, id, dir) = p.instancia();
        fs::write(dir.join("local.jar"), b"local").unwrap();
        let mut source = FuentePrueba::nueva(true);
        source.cambiar = Some(dir.join("local.jar"));
        assert!(matches!(
            revisar(&source, &store, &id).await,
            Err(Error::Cambio)
        ));
        assert!(!store.juego(&id).unwrap().join(CACHE).exists());
    }
    #[tokio::test]
    async fn disable_and_reversible_removal_preserve_bytes_and_reject_name_reuse() {
        let p = Prueba::nueva();
        let (store, id, dir) = p.instancia();
        fs::write(dir.join("local.jar"), b"local").unwrap();
        let source = FuentePrueba::nueva(false);
        let a = revisar(&source, &store, &id)
            .await
            .unwrap()
            .archivos
            .remove(0);
        desactivar(&store, &id, &a).unwrap();
        assert_eq!(fs::read(dir.join("local.jar.disabled")).unwrap(), b"local");
        let disabled = revisar(&source, &store, &id)
            .await
            .unwrap()
            .archivos
            .remove(0);
        let ticket = retirar(&store, &id, &disabled).unwrap();
        assert!(!dir.join("local.jar.disabled").exists());
        fs::write(dir.join("local.jar"), b"new").unwrap();
        assert!(restaurar(&store, &id, &ticket).is_err());
        assert_eq!(fs::read(dir.join("local.jar")).unwrap(), b"new");
        fs::rename(dir.join("local.jar"), dir.join("new.jar")).unwrap();
        restaurar(&store, &id, &ticket).unwrap();
        assert_eq!(fs::read(dir.join("local.jar.disabled")).unwrap(), b"local");
    }

    #[tokio::test]
    async fn stale_removal_rejects_new_contents_and_modified_registered_mod_keeps_metadata_on_undo()
    {
        let p = Prueba::nueva();
        let (store, id, dir) = p.instancia();
        fs::write(dir.join("local.jar"), b"local").unwrap();
        let source = FuentePrueba::nueva(false);
        let old = revisar(&source, &store, &id)
            .await
            .unwrap()
            .archivos
            .remove(0);
        fs::write(dir.join("local.jar"), b"modified").unwrap();
        assert!(retirar(&store, &id, &old).is_err());
        assert_eq!(fs::read(dir.join("local.jar")).unwrap(), b"modified");
        let registro = mods::Instalado {
            project_id: "project".into(),
            nombre: "Local".into(),
            version_id: "v1".into(),
            version: "1".into(),
            publicada: String::new(),
            archivo: "local.jar".into(),
            sha512: old.sha512,
            activo: true,
            dependencias: vec![],
        };
        fs::write(
            dir.join(".icaro-mods.json"),
            serde_json::to_vec(&vec![registro.clone()]).unwrap(),
        )
        .unwrap();
        let reviewed = revisar(&source, &store, &id)
            .await
            .unwrap()
            .archivos
            .remove(0);
        assert_eq!(reviewed.procedencia, Procedencia::Desconocida);
        let ticket = retirar(&store, &id, &reviewed).unwrap();
        assert!(mods::inventario(&store, &id).unwrap().is_empty());
        restaurar(&store, &id, &ticket).unwrap();
        let restored = mods::inventario(&store, &id).unwrap().remove(0);
        assert_eq!(restored.project_id, registro.project_id);
        assert_eq!(restored.sha512, registro.sha512);
        assert_eq!(fs::read(dir.join("local.jar")).unwrap(), b"modified");
        let again = retirar(&store, &id, &reviewed).unwrap();
        let backup = again.0.carpeta_recuperada().unwrap().join("local.jar");
        fs::write(&backup, b"tampered backup").unwrap();
        assert!(restaurar(&store, &id, &again).is_err());
        assert!(!dir.join("local.jar").exists());
        assert!(backup.exists());
    }
}
