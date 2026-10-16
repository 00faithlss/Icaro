// SPDX-License-Identifier: GPL-3.0-only
//! Índice del anfitrión y sincronización de archivos exactos de Modrinth.
use super::*;
use crate::{instancias::Instancia, modrinth::Catalog};
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::RwLock,
};
use tokio_util::sync::CancellationToken;
const SYNC: &str = ".icaro-sync.json";
const MAX_INDEX: usize = 1024 * 1024;
pub const RUTA: &str = "/icaro/v1/mods";
pub mod dedicado;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Indice {
    pub esquema: u32,
    pub minecraft: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub mods: Vec<Entrada>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub archivos: Vec<ArchivoPropio>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub avisos: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchivoPropio {
    pub id: String,
    pub nombre: String,
    pub version: String,
    pub archivo: String,
    pub bytes: u64,
    pub sha512: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entrada {
    pub proyecto: String,
    pub version: String,
    pub sha512: String,
}
impl Indice {
    pub fn validar(&self) -> Result<(), String> {
        let identifier = |s: &str| {
            !s.is_empty()
                && s.len() <= 128
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        };
        let mut projects = BTreeSet::new();
        let mut filenames = BTreeSet::new();
        if !matches!(self.esquema, 1 | 2)
            || (self.esquema == 1 && !self.archivos.is_empty())
            || self.mods.len() + self.archivos.len() > 512
            || self.avisos.len() > 1024
            || self
                .avisos
                .iter()
                .any(|s| s.len() > 1024 || s.chars().any(char::is_control))
            || self.archivos.iter().any(|a| {
                !identifier(&a.id)
                    || !a.id.starts_with("icaro-host-")
                    || !projects.insert(&a.id)
                    || !nombre_valido(&a.archivo)
                    || !a.archivo.to_ascii_lowercase().ends_with(".jar")
                    || !filenames.insert(a.archivo.to_lowercase())
                    || a.bytes == 0
                    || a.bytes > MAX_FILE
                    || a.nombre.is_empty()
                    || a.nombre.len() > 240
                    || a.nombre.chars().any(char::is_control)
                    || a.version.len() > 128
                    || a.version.chars().any(char::is_control)
                    || a.sha512.len() != 128
                    || !a.sha512.bytes().all(|b| b.is_ascii_hexdigit())
            })
            || !crate::minecraft::version_minecraft_valida(&self.minecraft)
            || !matches!(
                self.loader.as_str(),
                "fabric" | "quilt" | "forge" | "neoforge"
            )
            || self
                .loader_version
                .as_ref()
                .is_none_or(|s| !crate::minecraft::version_minecraft_valida(s))
            || self.mods.len() > 512
            || self.mods.iter().any(|m| {
                !identifier(&m.proyecto)
                    || !identifier(&m.version)
                    || !projects.insert(&m.proyecto)
                    || m.sha512.len() != 128
                    || !m.sha512.bytes().all(|b| b.is_ascii_hexdigit())
            })
        {
            return Err("El índice de mods del anfitrión no es válido.".into());
        }
        Ok(())
    }
    pub fn revision(&self) -> Result<String, String> {
        self.validar()?;
        Ok(hash(
            &serde_json::to_vec(self).map_err(|_| "No se pudo leer el índice.")?,
        ))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Gestionados {
    origen: String,
    proyectos: BTreeSet<String>,
    #[serde(default)]
    archivos: Vec<ArchivoPropio>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Tipo {
    Agregar,
    Actualizar,
    Quitar,
}
#[derive(Debug, Clone)]
pub struct Cambio {
    pub proyecto: String,
    pub tipo: Tipo,
    pub nombre: String,
    pub antes: String,
    pub despues: String,
    pub bytes: u64,
}
#[derive(Debug, Clone)]
pub struct Plan {
    pub indice: Indice,
    pub cambios: Vec<Cambio>,
    pub conservados: Vec<String>,
    pub conflictos: Vec<String>,
    reemplazos: BTreeSet<String>,
    reemplazos_autorizados: bool,
    origen: String,
    huella: Vec<(String, String)>,
    actuales: Vec<Instalado>,
    destinos: Vec<Version>,
    quitar: BTreeSet<String>,
    locales: Vec<ArchivoPropio>,
    quitar_locales: Vec<ArchivoPropio>,
    revision_origen: String,
    recuperadas: BTreeSet<String>,
}
impl Plan {
    pub fn revision_origen(&self) -> &str {
        &self.revision_origen
    }
    pub fn autorizar_reemplazos(&mut self, autorizar: bool) {
        self.reemplazos_autorizados = autorizar;
    }
    pub fn puede_aplicar(&self) -> bool {
        self.conflictos.is_empty() || self.reemplazos_autorizados
    }
}

// Corrección acotada a la versión cuyos metadatos omiten un requisito del cliente.
// Fuente: https://modrinth.com/mod/litemoretica (Litematica solo es obligatoria en cliente).
fn requiere_litematica(v: &Version) -> bool {
    v.project_id == "OYR5DMCo" && v.id == "nhi2RkZ9"
}

fn dependencias_cliente(v: &Version) -> Vec<Dependency> {
    let mut deps: Vec<_> = v.dependencies.iter()
        .filter(|d| d.dependency_type == "required").cloned().collect();
    if requiere_litematica(v) && !deps.iter().any(|d| d.project_id.as_deref() == Some("bEpr0Arc")) {
        deps.push(Dependency { project_id: Some("bEpr0Arc".into()), version_id: None,
            dependency_type: "required".into(), file_name: None });
    }
    deps
}

// Los índices antiguos pueden transportar un mod como JAR propio y omitir sus
// bibliotecas. Recuperarlas por hash conserva la identidad exacta del mod raíz.
async fn completar_dependencias_propias(source: &impl Fuente, index: &mut Indice) -> Result<BTreeSet<String>, String> {
    let hashes = index.archivos.iter().map(|a| a.sha512.to_lowercase()).collect::<Vec<_>>();
    let conocidas = if hashes.is_empty() { BTreeMap::new() } else {
        source.versiones_por_hash(&hashes).await.map_err(err)?
    };
    let mut cubiertos = index.mods.iter().map(|m| m.proyecto.clone()).collect::<BTreeSet<_>>();
    let mut pendientes = vec![];
    for archivo in &index.archivos {
        if let Some(version) = conocidas.get(&archivo.sha512.to_lowercase()) {
            if !version.files.iter().any(|f| f.hashes.sha512.eq_ignore_ascii_case(&archivo.sha512)) {
                return Err("Modrinth no confirma el hash del JAR del anfitrión.".into());
            }
            cubiertos.insert(version.project_id.clone());
            pendientes.push(version.clone());
        }
    }
    // También reparar índices nuevos que transportan esta versión por catálogo.
    // Los demás proyectos conservan la validación estricta de su índice exacto.
    for entry in index.mods.iter().filter(|m| m.proyecto == "OYR5DMCo" && m.version == "nhi2RkZ9") {
        let v = source.version(&entry.version).await.map_err(err)?;
        if v.id != entry.version || v.project_id != entry.proyecto
            || !v.files.iter().any(|f| f.hashes.sha512.eq_ignore_ascii_case(&entry.sha512)) {
            return Err("Modrinth no confirma el mod que requiere dependencias de cliente.".into());
        }
        pendientes.push(v);
    }
    let mut recuperadas = BTreeSet::new();
    while let Some(version) = pendientes.pop() {
        for dep in &dependencias_cliente(&version) {
            // Los pins de un mod transportado como propio no sustituyen el
            // conjunto que el anfitrión ya proporciona (por ejemplo Fabric API).
            if let Some(project) = dep.project_id.as_ref().filter(|p| cubiertos.contains(*p)) {
                if recuperadas.contains(project) && dep.version_id.as_ref().is_some_and(|pin| {
                    index.mods.iter().any(|m| &m.proyecto == project && &m.version != pin)
                }) {
                    return Err(format!("Dos mods del anfitrión requieren versiones distintas de la biblioteca {project}."));
                }
                continue;
            }
            let pinned = if let Some(id) = &dep.version_id {
                let v = source.version(id).await.map_err(err)?;
                if v.id != *id || dep.project_id.as_ref().is_some_and(|p| p != &v.project_id) {
                    return Err("Modrinth devolvió otra dependencia del archivo del anfitrión.".into());
                }
                Some(v)
            } else { None };
            let project = dep.project_id.as_ref().or_else(|| pinned.as_ref().map(|v| &v.project_id))
                .ok_or("Una dependencia del anfitrión no tiene identidad de Modrinth.")?.clone();
            if cubiertos.contains(&project) {
                if recuperadas.contains(&project) && pinned.as_ref().is_some_and(|v| {
                    index.mods.iter().any(|m| m.proyecto == project && m.version != v.id)
                }) {
                    return Err(format!("Dos mods del anfitrión requieren versiones distintas de la biblioteca {project}."));
                }
                continue;
            }
            let candidate = if let Some(v) = pinned { v } else {
                let mut versiones = source.versiones(&project, &index.minecraft, &index.loader).await.map_err(err)?;
                versiones.sort_by(|a,b| b.date_published.cmp(&a.date_published).then(a.id.cmp(&b.id)));
                versiones.into_iter().find(|v| v.project_id == project && v.version_type == "release"
                    && compatible(v, &index.minecraft, &index.loader))
                    .ok_or_else(|| format!("Falta una dependencia compatible del anfitrión: {project}."))?
            };
            if !compatible(&candidate, &index.minecraft, &index.loader) {
                return Err("La dependencia del archivo del anfitrión no coincide con Minecraft o loader.".into());
            }
            let file = candidate.files.iter().find(|f| f.primary && f.filename.ends_with(".jar"))
                .or_else(|| candidate.files.iter().find(|f| f.filename.ends_with(".jar")))
                .ok_or("La dependencia del anfitrión no ofrece un JAR.")?;
            index.mods.push(Entrada { proyecto: candidate.project_id.clone(), version: candidate.id.clone(), sha512: file.hashes.sha512.clone() });
            index.validar()?;
            cubiertos.insert(candidate.project_id.clone());
            recuperadas.insert(candidate.project_id.clone());
            pendientes.push(candidate);
        }
    }
    if !recuperadas.is_empty() {
        index.avisos.push("Se completaron dependencias obligatorias omitidas por el índice del anfitrión.".into());
    }
    index.validar()?;
    Ok(recuperadas)
}
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn huella_dir(
    dir: &Path,
    base: &Path,
    depth: usize,
    result: &mut Vec<(String, String)>,
    total: &mut u64,
) -> Result<(), String> {
    regular(dir, true).map_err(err)?;
    if depth > 8 {
        return Err("La carpeta de mods contiene demasiados niveles.".into());
    }
    for entry in fs::read_dir(dir).map_err(err)? {
        let path = entry.map_err(err)?.path();
        let meta = fs::symlink_metadata(&path).map_err(err)?;
        if enlace(&meta) {
            return Err("La carpeta de mods contiene enlaces no permitidos.".into());
        }
        if meta.is_dir() {
            huella_dir(&path, base, depth + 1, result, total)?;
        } else {
            regular(&path, false).map_err(err)?;
            *total = total
                .checked_add(meta.len())
                .ok_or("La carpeta de mods es demasiado grande.")?;
            if result.len() >= 2048 || meta.len() > MAX_FILE || *total > 4 * 1024 * 1024 * 1024 {
                return Err("La carpeta de mods supera los límites de sincronización.".into());
            }
            result.push((
                path.strip_prefix(base)
                    .map_err(err)?
                    .to_string_lossy()
                    .into_owned(),
                hash_archivo(&path).map_err(err)?,
            ));
        }
    }
    Ok(())
}
pub fn huella(store: &Almacen, id: &str) -> Result<Vec<(String, String)>, String> {
    let game = store.juego(id).map_err(err)?;
    let _lock = guardia(&game).map_err(err)?;
    recuperar(&game).map_err(err)?;
    fingerprint(&game.join("mods"))
}
pub(super) fn fingerprint(dir: &Path) -> Result<Vec<(String, String)>, String> {
    let mut out = vec![];
    if dir.exists() {
        huella_dir(dir, dir, 0, &mut out, &mut 0)?;
    }
    out.sort();
    Ok(out)
}
/// Publica solo mods comunes a cliente/servidor identificados por contenido.
pub async fn publicar(
    catalog: &Catalog,
    store: &Almacen,
    instance: &Instancia,
) -> Result<Indice, String> {
    let before = huella(store, &instance.id)?;
    let active: Vec<_> = before
        .iter()
        .filter(|(n, _)| n.ends_with(".jar") && !n.contains(['/', '\\']))
        .collect();
    if active.len() > 512 {
        return Err("La instancia supera 512 mods.".into());
    }
    let hashes = active.iter().map(|(_, h)| h.clone()).collect::<Vec<_>>();
    let versions = catalog.versions_from_hashes(&hashes).await.map_err(err)?;
    let mut entries = vec![];
    let mut candidatos = BTreeMap::new();
    let mut necesarios = BTreeSet::new();
    let mut projects = BTreeSet::new();
    for (name, sha) in active {
        let v = versions.get(sha).ok_or_else(|| {
            format!(
                "{name} no está identificado en Modrinth. El índice no se publicará incompleto."
            )
        })?;
        if !compatible(v, &instance.minecraft, &instance.loader) {
            return Err(format!(
                "{name} no corresponde al Minecraft y loader compartidos."
            ));
        }
        let p = catalog.project(&v.project_id).await.map_err(err)?;
        if !projects.insert(v.project_id.clone()) {
            return Err(
                "Hay dos archivos del mismo proyecto en la instancia del anfitrión.".into(),
            );
        }
        if p.client_side != "unsupported" && p.server_side != "unsupported" {
            necesarios.insert(v.project_id.clone());
        }
        candidatos.insert(
            v.project_id.clone(),
            (
                v.clone(),
                Entrada {
                    proyecto: v.project_id.clone(),
                    version: v.id.clone(),
                    sha512: sha.clone(),
                },
            ),
        );
    }
    // Un mod común puede necesitar una dependencia que solo se ejecute en el cliente.
    loop {
        let previous = necesarios.len();
        for project in necesarios.clone() {
            for dep in candidatos[&project]
                .0
                .dependencies
                .iter()
                .filter(|d| d.dependency_type == "required")
            {
                let candidate = candidatos.values().find(|(v, _)| {
                    dep.version_id.as_ref().map_or_else(
                        || dep.project_id.as_ref() == Some(&v.project_id),
                        |id| id == &v.id,
                    )
                });
                let (v, _) = candidate
                    .ok_or("Falta una dependencia obligatoria en la instancia del anfitrión.")?;
                necesarios.insert(v.project_id.clone());
            }
        }
        if previous == necesarios.len() {
            break;
        }
    }
    for project in necesarios {
        entries.push(candidatos.remove(&project).unwrap().1);
    }
    entries.sort_by(|a, b| a.proyecto.cmp(&b.proyecto));
    let index = Indice {
        esquema: 1,
        minecraft: instance.minecraft.clone(),
        loader: instance.loader.clone(),
        loader_version: crate::versiones_instancia::leer_loader(store, instance),
        mods: entries,
        archivos: vec![],
        avisos: vec![],
    };
    index.validar()?;
    if before != huella(store, &instance.id)? {
        return Err("Los mods del anfitrión cambiaron durante la publicación.".into());
    }
    Ok(index)
}
pub async fn preparar(
    source: &impl Fuente,
    store: &Almacen,
    instance: &Instancia,
    mut index: Indice,
    origen: String,
) -> Result<Plan, String> {
    index.validar()?;
    let revision_origen = index.revision()?;
    if index.minecraft != instance.minecraft
        || index.loader != instance.loader
        || index.loader_version != crate::versiones_instancia::leer_loader(store, instance)
    {
        return Err("La instancia debe usar el mismo Minecraft, loader y versión del loader que el anfitrión. Cámbialos antes de sincronizar.".into());
    }
    let recuperadas = completar_dependencias_propias(source, &mut index).await?;
    let fingerprint = huella(store, &instance.id)?;
    let mut installed = inventario(store, &instance.id).map_err(err)?;
    for m in &mut installed {
        let filename = if m.activo {
            m.archivo.clone()
        } else {
            format!("{}.disabled", m.archivo)
        };
        let sha = fingerprint
            .iter()
            .find(|(n, _)| n == &filename)
            .ok_or("Los mods cambiaron durante la consulta.")?
            .1
            .clone();
        if !m.sha512.is_empty() && m.sha512.to_lowercase() != sha {
            return Err(format!(
                "{} fue modificado. Revísalo antes de sincronizar.",
                m.archivo
            ));
        }
        m.sha512 = sha;
    }
    let dir = store.juego(&instance.id).map_err(err)?.join("mods");
    // Los JAR importados pueden no tener identidad de catálogo todavía.
    // Una consulta por lote permite reconocer versiones anteriores sin adivinar por nombre.
    if index.mods.iter().any(|e| !installed.iter().any(|m| m.project_id == e.proyecto)) {
        let hashes: Vec<_> = installed.iter().filter(|m| m.activo && m.project_id.is_empty())
            .map(|m| m.sha512.clone()).collect();
        if !hashes.is_empty() {
            let reconocidas = source.versiones_por_hash(&hashes).await.map_err(err)?;
            for m in &mut installed {
                if m.project_id.is_empty() {
                    if let Some(v) = reconocidas.get(&m.sha512).filter(|v| v.files.iter().any(|f| f.hashes.sha512.eq_ignore_ascii_case(&m.sha512))) {
                        m.project_id = v.project_id.clone();
                        m.version_id = v.id.clone();
                        m.version = v.version_number.clone();
                    }
                }
            }
        }
    }
    let mut identidades = BTreeMap::new();
    if matches!(instance.loader.as_str(), "fabric" | "quilt") {
        for m in &installed {
            if let Some(id) = identidad_fabric(&ruta_mod(&dir, m))? {
                identidades.insert(m.archivo.clone(), id);
            }
        }
    }
    let mut locales_previos = vec![];
    let managed = if dir.join(SYNC).exists() {
        if regular(&dir.join(SYNC), false).map_err(err)?.len() > MAX_INDEX as u64 {
            return Err("El registro de sincronización no es válido.".into());
        }
        let saved: Gestionados = serde_json::from_slice(&fs::read(dir.join(SYNC)).map_err(err)?)
            .map_err(|_| "El registro de sincronización no es válido.")?;
        if saved.origen == origen {
            let mut check = index.clone();
            check.mods.clear();
            check.avisos.clear();
            check.esquema = 2;
            check.archivos = saved.archivos.clone();
            check.validar()?;
            locales_previos = saved.archivos;
            saved.proyectos
        } else {
            BTreeSet::new()
        }
    } else {
        BTreeSet::new()
    };
    let wanted = index
        .mods
        .iter()
        .map(|m| m.proyecto.clone())
        .collect::<BTreeSet<_>>();
    let mut quitar = managed
        .difference(&wanted)
        .cloned()
        .collect::<BTreeSet<_>>();
    for project in quitar.clone() {
        if source.solo_cliente(&project).await.map_err(err)? {
            quitar.remove(&project);
        }
    }
    let mut changes = vec![];
    let mut versions = vec![];
    let mut conflictos = vec![];
    let (locales, mut quitar_locales) = preparar_locales(
        &index,
        &installed,
        &locales_previos,
        &quitar,
        &mut changes,
        &mut conflictos,
        &identidades,
    )?;
    if !locales.is_empty() {
        direccion_archivos(&origen)?;
    }
    quitar.retain(|id| {
        !installed.iter().any(|m| {
            &m.project_id == id
                && locales
                    .iter()
                    .any(|a| a.sha512.eq_ignore_ascii_case(&m.sha512))
        })
    });
    let ids = index.mods.iter().map(|m| m.version.clone()).collect::<Vec<_>>();
    let verificadas = source.versiones_exactas(&ids).await.map_err(err)?;
    for entry in &index.mods {
        let mut v = verificadas.get(&entry.version).cloned().ok_or("Modrinth no confirmó una versión del anfitrión.")?;
        if v.id != entry.version
            || v.project_id != entry.proyecto
            || !compatible(&v, &index.minecraft, &index.loader)
        {
            return Err("Modrinth no confirma la versión solicitada por el anfitrión.".into());
        }
        let f = v
            .files
            .iter()
            .find(|f| f.hashes.sha512.eq_ignore_ascii_case(&entry.sha512))
            .cloned()
            .ok_or("Modrinth no confirma el archivo solicitado por el anfitrión.")?;
        if !nombre_valido(&f.filename) || !f.filename.ends_with(".jar") || f.size > MAX_FILE {
            return Err("El archivo solicitado no es válido.".into());
        }
        v.files = vec![f.clone()];
        if locales
            .iter()
            .any(|a| a.archivo.eq_ignore_ascii_case(&f.filename))
        {
            return Err("Un archivo propio y uno de Modrinth usan el mismo nombre.".into());
        }
        let old = installed
            .iter()
            .find(|m| m.project_id == entry.proyecto)
            .or_else(|| {
                installed
                    .iter()
                    .find(|m| m.sha512.eq_ignore_ascii_case(&entry.sha512))
            });
        if old.is_none_or(|m| !m.activo || !m.sha512.eq_ignore_ascii_case(&entry.sha512)) {
            changes.push(Cambio {
                proyecto: entry.proyecto.clone(),
                tipo: if old.is_some() {
                    Tipo::Actualizar
                } else {
                    Tipo::Agregar
                },
                nombre: source.nombre(&entry.proyecto).await.map_err(err)?,
                antes: old.map(|m| m.version.clone()).unwrap_or_default(),
                despues: v.version_number.clone(),
                bytes: if old.is_some_and(|m| m.sha512.eq_ignore_ascii_case(&entry.sha512)) {
                    0
                } else {
                    f.size
                },
            });
        }
        versions.push(v);
    }
    // Dependencias obligatorias también deben estar fijadas en el índice, nunca elegir la última.
    let resolved = resolver(
        source,
        versions.clone(),
        &[],
        &index.minecraft,
        &index.loader,
    )
    .await
    .map_err(err)?;
    if resolved.iter().any(|v| {
        !versions
            .iter()
            .any(|p| p.id == v.id && p.project_id == v.project_id)
    }) {
        return Err("Al índice del anfitrión le falta una dependencia obligatoria exacta.".into());
    }
    for old in installed.iter().filter(|m| quitar.contains(&m.project_id)) {
        changes.push(Cambio {
            proyecto: old.project_id.clone(),
            tipo: Tipo::Quitar,
            nombre: old.nombre.clone(),
            antes: old.version.clone(),
            despues: String::new(),
            bytes: 0,
        });
    }
    let reemplazos = reconciliar_identidades(&installed, &identidades, &locales, &versions, &mut changes, &mut conflictos)?;
    quitar_locales.retain(|a| !reemplazos.contains(&a.archivo));
    let keep = installed
        .iter()
        .filter(|m| {
            !reemplazos.contains(&m.archivo) && !conflictos.contains(&m.archivo) && !locales
                .iter()
                .any(|a| a.archivo.eq_ignore_ascii_case(&m.archivo))
                && !quitar_locales
                    .iter()
                    .any(|a| a.archivo.eq_ignore_ascii_case(&m.archivo))
                && !quitar.contains(&m.project_id)
                && !wanted.contains(&m.project_id)
                && !index
                    .mods
                    .iter()
                    .any(|e| e.sha512.eq_ignore_ascii_case(&m.sha512))
        })
        .map(|m| m.nombre.clone())
        .collect();
    if fingerprint != huella(store, &instance.id)? {
        return Err("Los mods cambiaron durante la consulta. Comprueba de nuevo.".into());
    }
    Ok(Plan {
        indice: index,
        cambios: changes,
        conservados: keep,
        conflictos,
        reemplazos,
        reemplazos_autorizados: false,
        origen,
        huella: fingerprint,
        actuales: installed,
        destinos: versions,
        quitar,
        locales,
        quitar_locales,
        revision_origen,
        recuperadas,
    })
}
fn identidad_fabric(path: &Path) -> Result<Option<String>, String> {
    Ok(
        dedicado::descriptor_fabric(path, &mut vec![])?.and_then(|d| {
            d.get("id")
                .and_then(|v| v.as_str())
                .filter(|id| !id.is_empty())
                .map(str::to_owned)
        }),
    )
}

// La identidad del descriptor prevalece sobre el nombre y la versión declarada:
// dos JAR pueden anunciar la misma versión y contener registros diferentes.
fn reconciliar_identidades(
    installed: &[Instalado],
    identidades: &BTreeMap<String, String>,
    locales: &[ArchivoPropio],
    versiones: &[Version],
    cambios: &mut Vec<Cambio>,
    conflictos: &mut Vec<String>,
) -> Result<BTreeSet<String>, String> {
    let mut destinos = Vec::new();
    for local in locales {
        let id = installed
            .iter()
            .find(|m| m.sha512.eq_ignore_ascii_case(&local.sha512))
            .and_then(|m| identidades.get(&m.archivo))
            .cloned()
            .or_else(|| local.id.strip_prefix("icaro-host-").map(str::to_owned));
        if let Some(id) = id {
            destinos.push((
                local.id.clone(),
                local.archivo.clone(),
                local.sha512.clone(),
                id,
            ));
        }
    }
    for version in versiones {
        let file = &version.files[0];
        let exacto = installed
            .iter()
            .find(|m| m.sha512.eq_ignore_ascii_case(&file.hashes.sha512));
        let anterior = exacto.or_else(|| {
            installed
                .iter()
                .find(|m| m.project_id == version.project_id)
        });
        if let Some(id) = anterior.and_then(|m| identidades.get(&m.archivo)) {
            destinos.push((
                version.project_id.clone(),
                exacto
                    .map(|m| m.archivo.clone())
                    .unwrap_or_else(|| file.filename.clone()),
                file.hashes.sha512.clone(),
                id.clone(),
            ));
        }
    }
    let mut requeridos = BTreeMap::new();
    let mut reemplazos = BTreeSet::new();
    for (proyecto, archivo, sha, id) in destinos {
        if requeridos
            .insert(id.clone(), sha.clone())
            .is_some_and(|otra| otra != sha)
        {
            return Err(format!(
                "El anfitrión publica dos JAR distintos del mismo mod {id}. Revisa el servidor."
            ));
        }
        for actual in installed.iter().filter(|m| {
            (m.activo || m.archivo.eq_ignore_ascii_case(&archivo))
                && identidades.get(&m.archivo) == Some(&id)
                && !(m.archivo.eq_ignore_ascii_case(&archivo)
                    && m.sha512.eq_ignore_ascii_case(&sha))
        }) {
            if !reemplazos.insert(actual.archivo.clone()) {
                continue;
            }
            conflictos.retain(|n| n != &actual.archivo);
            if let Some(cambio) = cambios.iter_mut().find(|c| c.proyecto == proyecto) {
                if cambio.tipo == Tipo::Agregar {
                    cambio.tipo = Tipo::Actualizar;
                }
                if cambio.antes.is_empty() {
                    cambio.antes = actual.version.clone();
                }
            } else {
                cambios.push(Cambio {
                    proyecto: format!("icaro-reemplazo-{}", hash(actual.archivo.as_bytes())),
                    tipo: Tipo::Quitar,
                    nombre: format!("{id} · copia anterior: {}", actual.archivo),
                    antes: actual.version.clone(),
                    despues: String::new(),
                    bytes: 0,
                });
            }
        }
    }
    Ok(reemplazos)
}

fn preparar_locales(
    index: &Indice,
    installed: &[Instalado],
    previos: &[ArchivoPropio],
    quitar: &BTreeSet<String>,
    changes: &mut Vec<Cambio>,
    conflictos: &mut Vec<String>,
    identidades: &BTreeMap<String, String>,
) -> Result<(Vec<ArchivoPropio>, Vec<ArchivoPropio>), String> {
    let mut wanted = vec![];
    for entry in &index.archivos {
        let previous = previos.iter().find(|a| a.id == entry.id);
        let current = installed.iter().find(|m| m.sha512.eq_ignore_ascii_case(&entry.sha512)).or_else(|| previous
            .and_then(|p| {
                installed
                    .iter()
                    .find(|m| m.archivo.eq_ignore_ascii_case(&p.archivo))
            }))
            .or_else(|| {
                installed
                    .iter()
                    .find(|m| m.sha512.eq_ignore_ascii_case(&entry.sha512))
            })
            .or_else(|| {
                installed
                    .iter()
                    .find(|m| m.archivo.eq_ignore_ascii_case(&entry.archivo))
            })
            .or_else(|| {
                installed.iter().find(|m| m.activo && identidades.get(&m.archivo)
                    .is_some_and(|id| entry.id.strip_prefix("icaro-host-") == Some(id.as_str())))
            });
        if let Some(m) = current {
            let same = m.sha512.eq_ignore_ascii_case(&entry.sha512);
            let owned = previous.is_some_and(|p| {
                p.archivo.eq_ignore_ascii_case(&m.archivo)
                    && p.sha512.eq_ignore_ascii_case(&m.sha512)
            });
            if !same && !owned && !quitar.contains(&m.project_id) {
                conflictos.push(m.archivo.clone());
            }
        }
        let mut local = entry.clone();
        if installed.iter().any(|m| {
            m.archivo.eq_ignore_ascii_case(&entry.archivo)
                && current.is_none_or(|c| c.archivo != m.archivo)
                && !identidades.get(&m.archivo).is_some_and(|id| entry.id.strip_prefix("icaro-host-") == Some(id.as_str()))
        }) {
            return Err(format!(
                "El nombre {} está ocupado por otro archivo local.",
                entry.archivo
            ));
        }
        if current.is_some_and(|m| m.sha512.eq_ignore_ascii_case(&entry.sha512)) {
            local.archivo = current.unwrap().archivo.clone();
        }
        if current.is_none_or(|m| !m.activo || !m.sha512.eq_ignore_ascii_case(&entry.sha512)) {
            changes.push(Cambio {
                proyecto: entry.id.clone(),
                tipo: if current.is_some() {
                    Tipo::Actualizar
                } else {
                    Tipo::Agregar
                },
                nombre: format!("{} · archivo del anfitrión", entry.nombre),
                antes: previous.map(|p| p.version.clone()).unwrap_or_default(),
                despues: entry.version.clone(),
                bytes: if current.is_some_and(|m| m.sha512.eq_ignore_ascii_case(&entry.sha512)) {
                    0
                } else {
                    entry.bytes
                },
            });
        }
        wanted.push(local);
    }
    let mut removed = vec![];
    for previous in previos {
        if wanted
            .iter()
            .any(|a| a.id == previous.id && a.archivo.eq_ignore_ascii_case(&previous.archivo))
            || index
                .mods
                .iter()
                .any(|m| m.sha512.eq_ignore_ascii_case(&previous.sha512))
        {
            continue;
        }
        if let Some(current) = installed
            .iter()
            .find(|m| m.archivo.eq_ignore_ascii_case(&previous.archivo))
        {
            if !current.sha512.eq_ignore_ascii_case(&previous.sha512) {
                return Err(format!(
                    "{} fue modificado. No se eliminará.",
                    previous.archivo
                ));
            }
            if !wanted
                .iter()
                .any(|a| a.archivo.eq_ignore_ascii_case(&previous.archivo))
            {
                removed.push(previous.clone());
                if !wanted.iter().any(|a| a.id == previous.id) {
                    changes.push(Cambio {
                        proyecto: previous.id.clone(),
                        tipo: Tipo::Quitar,
                        nombre: previous.nombre.clone(),
                        antes: previous.version.clone(),
                        despues: String::new(),
                        bytes: 0,
                    });
                }
            }
        }
    }
    if wanted.iter().any(|a| {
        index
            .mods
            .iter()
            .any(|e| e.sha512.eq_ignore_ascii_case(&a.sha512))
    }) {
        return Err("El índice repite un archivo propio y uno de Modrinth.".into());
    }
    Ok((wanted, removed))
}
fn direccion_archivos(origen: &str) -> Result<SocketAddr, String> {
    let addr: SocketAddr = origen
        .parse()
        .map_err(|_| "Origen de archivos propios inválido.".to_string())?;
    if addr.port() < 2
        || endpoint(&SocketAddr::new(addr.ip(), addr.port() - 1).to_string())? != addr
    {
        return Err("Origen de archivos propios inválido.".into());
    }
    Ok(addr)
}
async fn descargar_propio(origen: &str, file: &ArchivoPropio) -> Result<Vec<u8>, String> {
    let addr = direccion_archivos(origen)?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(err)?;
    let mut response = client
        .get(format!(
            "http://{addr}/icaro/v1/files/{}",
            file.sha512.to_lowercase()
        ))
        .send()
        .await
        .map_err(|_| format!("No se pudo descargar {} del anfitrión.", file.archivo))?;
    if !response.status().is_success() {
        return Err(
            "El archivo del anfitrión cambió o no está disponible. Comprueba de nuevo.".into(),
        );
    }
    if response
        .content_length()
        .is_some_and(|n| n != file.bytes || n > MAX_FILE)
    {
        return Err("Tamaño de archivo del anfitrión inválido.".into());
    }
    let mut bytes = vec![];
    while let Some(chunk) = response.chunk().await.map_err(err)? {
        if bytes.len() as u64 + chunk.len() as u64 > file.bytes {
            return Err("El archivo supera el tamaño publicado.".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    if bytes.len() as u64 != file.bytes || hash(&bytes) != file.sha512.to_lowercase() {
        return Err("El archivo del anfitrión no coincide con su tamaño o SHA-512.".into());
    }
    Ok(bytes)
}
pub async fn aplicar(
    source: &impl Fuente,
    store: &Almacen,
    instance: &Instancia,
    plan: &Plan,
    cancel: CancellationToken,
    progress: impl Fn(&str, bool) + Send,
) -> Result<(), String> {
    let game = store.juego(&instance.id).map_err(err)?;
    if !plan.puede_aplicar() {
        return Err("Hay mods locales distintos de los del anfitrión. Revisa los conflictos y autoriza su respaldo antes de sincronizar.".into());
    }
    let _lock = guardia(&game).map_err(err)?;
    recuperar(&game).map_err(err)?;
    let dir = game.join("mods");
    if cancel.is_cancelled() || fingerprint(&dir)? != plan.huella {
        return Err("El plan fue cancelado o los mods cambiaron. Comprueba de nuevo.".into());
    }
    let current = store
        .cargar()
        .map_err(err)?
        .instancias
        .into_iter()
        .find(|i| i.id == instance.id)
        .ok_or("La instancia ya no existe.")?;
    if current.minecraft != plan.indice.minecraft
        || current.loader != plan.indice.loader
        || crate::versiones_instancia::leer_loader(store, &current) != plan.indice.loader_version
    {
        return Err("El perfil de la instancia cambió. Comprueba de nuevo.".into());
    }
    // Si este origen ya gestiona exactamente el conjunto verificado, no hay que
    // copiar ni publicar de nuevo toda la carpeta antes de cada partida.
    if plan.cambios.is_empty() && dir.join(SYNC).exists() {
        let saved: Gestionados = serde_json::from_slice(&fs::read(dir.join(SYNC)).map_err(err)?)
            .map_err(|_| "El registro de sincronización no es válido.")?;
        let wanted = plan.indice.mods.iter()
            .map(|m| m.proyecto.clone()).collect::<BTreeSet<_>>();
        if saved.origen == plan.origen
            && saved.proyectos == wanted
            && saved.archivos == plan.locales
        {
            if cancel.is_cancelled() {
                return Err("Sincronización cancelada. Los mods anteriores se conservan.".into());
            }
            return Ok(());
        }
    }
    let mut random = [0u8; 16];
    getrandom::fill(&mut random).map_err(err)?;
    let stage = game.join(format!(
        ".icaro-mods-next-{}",
        random
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    ));
    fs::create_dir(&stage).map_err(err)?;
    let result = async {
        if dir.exists() { copiar(&dir,&stage,0,&mut 0,&mut 0).map_err(err)?; }
        let respaldos: BTreeSet<_> = plan.conflictos.iter().chain(plan.reemplazos.iter()).cloned().collect();
        // El respaldo es independiente de la carpeta temporal de recuperación.
        // Permanece disponible incluso después de otras sincronizaciones.
        if !respaldos.is_empty() {
            let respaldo = game.join(format!(".icaro-mods-conflictos-{}", random.iter().map(|b| format!("{b:02x}")).collect::<String>()));
            fs::create_dir(&respaldo).map_err(err)?;
            for nombre in &respaldos {
                let anterior = plan.actuales.iter().find(|m| &m.archivo == nombre).ok_or("El conflicto ya no existe.")?;
                let ruta = ruta_mod(&stage, anterior);
                let destino = respaldo.join(ruta.file_name().ok_or("Nombre de mod inválido.")?);
                fs::copy(&ruta, &destino).map_err(err)?;
                if hash_archivo(&destino).map_err(err)? != anterior.sha512 { return Err("No se pudo verificar el respaldo del mod local.".into()); }
                fs::OpenOptions::new().write(true).open(&destino).and_then(|f| f.sync_all()).map_err(err)?;
                fs::remove_file(ruta).map_err(err)?;
            }
        }
        let actuales: Vec<_> = plan.actuales.iter().filter(|m| !respaldos.contains(&m.archivo)).cloned().collect();
        let wanted=plan.indice.mods.iter().map(|m|m.proyecto.clone()).collect::<BTreeSet<_>>();
        let mut records=actuales.iter().filter(|m|!plan.quitar.contains(&m.project_id) && !wanted.contains(&m.project_id)).cloned().collect::<Vec<_>>();
        for old in actuales.iter().filter(|m|plan.quitar.contains(&m.project_id)) { progress(&old.project_id,false); fs::remove_file(ruta_mod(&stage,old)).map_err(err)?; progress(&old.project_id,true); }
        for v in &plan.destinos {
            progress(&v.project_id,false);
            if cancel.is_cancelled() { return Err("Sincronización cancelada. Los mods anteriores se conservan.".into()); }
            let file=&v.files[0];
            let old=actuales.iter().find(|m|m.sha512.eq_ignore_ascii_case(&file.hashes.sha512)).or_else(||actuales.iter().find(|m|m.project_id == v.project_id));
            let same=old.is_some_and(|m|m.sha512.eq_ignore_ascii_case(&file.hashes.sha512));
            let filename=if same {old.unwrap().archivo.clone()} else {file.filename.clone()};
            let target=stage.join(&filename);
            if actuales.iter().any(|m|!plan.quitar.contains(&m.project_id) && m.archivo.eq_ignore_ascii_case(&filename) && old.is_none_or(|o|o.archivo != m.archivo)) { return Err(format!("El nombre {filename} está ocupado por un archivo local.")); }
            if (target.exists() || stage.join(format!("{filename}.disabled")).exists()) && old.is_none_or(|m|m.archivo.to_lowercase() != filename.to_lowercase()) { return Err(format!("El nombre {filename} está ocupado por un archivo local.")); }
            if same {
                if !old.unwrap().activo { fs::rename(ruta_mod(&stage,old.unwrap()),&target).map_err(err)?; }
            } else {
                let bytes=tokio::select! { _=cancel.cancelled()=>return Err("Sincronización cancelada. Los mods anteriores se conservan.".into()), r=source.archivo(file)=>r.map_err(err)? };
                if bytes.len() as u64 != file.size || hash(&bytes) != file.hashes.sha512.to_lowercase() { return Err("El mod descargado no coincide con su tamaño o SHA-512.".into()); }
                if let Some(old)=old { fs::remove_file(ruta_mod(&stage,old)).map_err(err)?; }
                guardar_archivo(&target,&bytes).map_err(err)?;
            }
            if plan.recuperadas.contains(&v.project_id)
                    && matches!(instance.loader.as_str(), "fabric" | "quilt")
                    && dedicado::descriptor_fabric(&target, &mut vec![])?
                        .is_some_and(|d| d.get("environment").and_then(|v| v.as_str()) == Some("server"))
                {
                    return Err(format!("La dependencia {} declara que solo admite servidor. El conjunto anterior se conserva.", v.project_id));
            }
            records.retain(|m|m.archivo != filename && m.project_id != v.project_id);
            let nombre=plan.cambios.iter().find(|c|c.proyecto==v.project_id).map(|c|c.nombre.clone()).or_else(||old.map(|m|m.nombre.clone())).unwrap_or_else(||v.name.clone());
            records.push(Instalado { project_id:v.project_id.clone(), nombre, version_id:v.id.clone(), version:v.version_number.clone(), publicada:v.date_published.clone(), archivo:filename, sha512:file.hashes.sha512.clone(), activo:true, dependencias:v.dependencies.clone() });
            progress(&v.project_id,true);
        }
        records.retain(|m|!m.project_id.is_empty());
        guardar_archivo(&stage.join(META),&serde_json::to_vec_pretty(&records).map_err(err)?).map_err(err)?;
        for old in &plan.quitar_locales {
            progress(&old.id,false);
            let path = stage.join(&old.archivo);
            let disabled = stage.join(format!("{}.disabled",old.archivo));
            if path.exists() { fs::remove_file(path).map_err(err)?; }
            else if disabled.exists() { fs::remove_file(disabled).map_err(err)?; }
            records.retain(|m| !m.archivo.eq_ignore_ascii_case(&old.archivo));
            progress(&old.id,true);
        }
        for local in &plan.locales {
            progress(&local.id,false);
            if cancel.is_cancelled() { return Err("Sincronización cancelada. Los mods anteriores se conservan.".into()); }
            let target=stage.join(&local.archivo);
            let disabled=stage.join(format!("{}.disabled",local.archivo));
            let same=if target.exists() { hash_archivo(&target).map_err(err)? == local.sha512.to_lowercase() }
                else if disabled.exists() { hash_archivo(&disabled).map_err(err)? == local.sha512.to_lowercase() } else { false };
            if same { if !target.exists() {fs::rename(&disabled,&target).map_err(err)?;} }
            else {
                let bytes=tokio::select!{_=cancel.cancelled()=>return Err("Sincronización cancelada. Los mods anteriores se conservan.".into()),r=descargar_propio(&plan.origen,local)=>r?};
                if target.exists(){fs::remove_file(&target).map_err(err)?;}
                if disabled.exists(){fs::remove_file(&disabled).map_err(err)?;}
                guardar_archivo(&target,&bytes).map_err(err)?;
            }
            records.retain(|m|!m.archivo.eq_ignore_ascii_case(&local.archivo));
            progress(&local.id,true);
        }
        if matches!(instance.loader.as_str(), "fabric" | "quilt") {
            let mut requeridos = BTreeSet::new();
            for local in &plan.locales {
                if let Some(id) = identidad_fabric(&stage.join(&local.archivo))? { requeridos.insert(id); }
            }
            for registro in records.iter().filter(|r| wanted.contains(&r.project_id)) {
                if let Some(id) = identidad_fabric(&stage.join(&registro.archivo))? { requeridos.insert(id); }
            }
            let mut vistos = BTreeSet::new();
            for entrada in fs::read_dir(&stage).map_err(err)? {
                let path = entrada.map_err(err)?.path();
                if path.extension().is_some_and(|e| e == "jar") {
                    if let Some(id) = identidad_fabric(&path)? {
                        if requeridos.contains(&id) && !vistos.insert(id.clone()) {
                            return Err(format!("Quedaron dos versiones activas del mod {id}. No se publicó la sincronización."));
                        }
                    }
                }
            }
        }
        guardar_archivo(&stage.join(META),&serde_json::to_vec_pretty(&records).map_err(err)?).map_err(err)?;
        guardar_archivo(&stage.join(SYNC),&serde_json::to_vec(&Gestionados {origen:plan.origen.clone(), proyectos:wanted, archivos:plan.locales.clone()}).map_err(err)?).map_err(err)?;
        if cancel.is_cancelled() || fingerprint(&dir)? != plan.huella { return Err("Los mods cambiaron o se canceló la sincronización. El conjunto anterior se conserva.".into()); }
        let backup=game.join(".icaro-mods-backup");
        if backup.exists() { borrar_preparacion(&game,&backup).map_err(err)?; }
        if dir.exists() { fs::rename(&dir,&backup).map_err(err)?; }
        if fs::rename(&stage,&dir).is_err() { if backup.exists(){let _=fs::rename(&backup,&dir);} return Err("No se pudo publicar la sincronización. El conjunto anterior se conserva.".into()); }
        if backup.exists() { let _=borrar_preparacion(&game,&backup); }
        Ok(())
    }.await;
    if stage.exists() {
        let _ = borrar_preparacion(&game, &stage);
    }
    result
}
pub fn endpoint(address: &str) -> Result<SocketAddr, String> {
    let d = crate::servidores::direccion(address)?;
    let ip = d
        .host
        .parse::<IpAddr>()
        .map_err(|_| "Usa la IP privada de ZeroTier para consultar la sincronización.")?;
    let private = match ip {
        IpAddr::V4(ip) => ip.is_private() || ip.is_loopback(),
        IpAddr::V6(ip) => ip.is_loopback() || ip.segments()[0] & 0xfe00 == 0xfc00,
    };
    if !private {
        return Err("La sincronización del anfitrión requiere una IP privada de ZeroTier.".into());
    }
    Ok(SocketAddr::new(
        ip,
        d.puerto
            .checked_add(1)
            .ok_or("El puerto de sincronización no es válido.")?,
    ))
}
pub async fn consultar(addr: SocketAddr, cancel: CancellationToken) -> Result<Indice, String> {
    if addr.port() < 2 {
        return Err("Puerto de sincronización inválido.".into());
    }
    endpoint(&SocketAddr::new(addr.ip(), addr.port() - 1).to_string())?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(err)?;
    let work = async {
        let mut response=client.get(format!("http://{addr}{RUTA}")).send().await.map_err(|_|format!("No responde el servicio de mods en {addr}. El anfitrión debe ejecutar icaro-sync-host y permitir ese puerto en su cortafuegos. Estar conectado a ZeroTier no inicia este servicio."))?;
        if !response.status().is_success() {
            return Err("El anfitrión todavía no tiene un índice de mods disponible.".into());
        }
        if response
            .content_length()
            .is_some_and(|n| n > MAX_INDEX as u64)
        {
            return Err("El índice supera el tamaño permitido.".into());
        }
        let mut bytes = vec![];
        while let Some(chunk) = response.chunk().await.map_err(err)? {
            if bytes.len() + chunk.len() > MAX_INDEX {
                return Err("El índice supera el tamaño permitido.".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        let index: Indice = serde_json::from_slice(&bytes)
            .map_err(|_| "El servicio no entregó un índice de mods válido.")?;
        index.validar()?;
        Ok(index)
    };
    tokio::select! {_=cancel.cancelled()=>Err("Consulta cancelada.".into()), r=work=>r}
}
pub type Publicacion = Arc<RwLock<Option<Indice>>>;
pub async fn servir(
    listener: TcpListener,
    index: Publicacion,
    cancel: CancellationToken,
) -> Result<(), String> {
    servir_archivos(listener, index, cancel, None).await
}
pub async fn servir_archivos(
    listener: TcpListener,
    index: Publicacion,
    cancel: CancellationToken,
    mods: Option<PathBuf>,
) -> Result<(), String> {
    let slots = Arc::new(tokio::sync::Semaphore::new(8));
    loop {
        let (mut socket, _) = tokio::select! {_=cancel.cancelled()=>return Ok(()), r=listener.accept()=>r.map_err(err)?};
        let Ok(permit) = slots.clone().try_acquire_owned() else {
            continue;
        };
        let index = index.clone();
        let mods = mods.clone();
        let c = cancel.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let request = async {
                let mut data = vec![];
                let mut buf = [0u8; 256];
                loop {
                    let n = socket.read(&mut buf).await.map_err(err)?;
                    if n == 0 || data.len() + n > 2048 {
                        return Err("Solicitud inválida".to_string());
                    }
                    data.extend_from_slice(&buf[..n]);
                    if data.windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                let line = std::str::from_utf8(&data)
                    .map_err(err)?
                    .lines()
                    .next()
                    .unwrap_or("");
                let valid = line == format!("GET {RUTA} HTTP/1.1")
                    || line == format!("GET {RUTA} HTTP/1.0");
                let snapshot = index.read().await.clone();
                let file_hash = line.strip_prefix("GET /icaro/v1/files/").and_then(|s| {
                    s.strip_suffix(" HTTP/1.1")
                        .or_else(|| s.strip_suffix(" HTTP/1.0"))
                });
                let (status, body) = if valid {
                    match &snapshot {
                        Some(i) => ("200 OK", serde_json::to_vec(i).map_err(err)?),
                        None => ("503 Service Unavailable", vec![]),
                    }
                } else if let (Some(sha), Some(root), Some(snapshot)) =
                    (file_hash, &mods, &snapshot)
                {
                    if sha.len() != 128 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
                        ("404 Not Found", vec![])
                    } else if let Some(file) = snapshot
                        .archivos
                        .iter()
                        .find(|a| a.sha512.eq_ignore_ascii_case(sha))
                    {
                        let path = root.join(&file.archivo);
                        let bytes = if nombre_valido(&file.archivo)
                            && regular(root, true).is_ok()
                            && regular(&path, false)
                                .is_ok_and(|m| m.len() == file.bytes && m.len() <= MAX_FILE)
                        {
                            let mut bytes = vec![];
                            fs::File::open(&path)
                                .and_then(|f| {
                                    std::io::Read::read_to_end(
                                        &mut std::io::Read::take(f, MAX_FILE + 1),
                                        &mut bytes,
                                    )
                                })
                                .map(|_| bytes)
                                .unwrap_or_default()
                        } else {
                            vec![]
                        };
                        if bytes.len() as u64 == file.bytes
                            && hash(&bytes) == file.sha512.to_lowercase()
                        {
                            ("200 OK", bytes)
                        } else {
                            ("410 Gone", vec![])
                        }
                    } else {
                        ("404 Not Found", vec![])
                    }
                } else {
                    ("404 Not Found", vec![])
                };
                let content = if file_hash.is_some() {
                    "application/octet-stream"
                } else {
                    "application/json"
                };
                let head=format!("HTTP/1.1 {status}\r\nContent-Type: {content}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",body.len());
                socket.write_all(head.as_bytes()).await.map_err(err)?;
                socket.write_all(&body).await.map_err(err)?;
                Ok::<_, String>(())
            };
            tokio::select! { _=c.cancelled()=>{}, _=tokio::time::timeout(Duration::from_secs(120),request)=>{} }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fake {
        versions: BTreeMap<String, Version>,
        hashes: BTreeMap<String, Version>,
        bytes: BTreeMap<String, Vec<u8>>,
        bad: bool,
    }
    impl Fuente for Fake {
        async fn versiones_por_hash(&self, hashes: &[String]) -> Result<BTreeMap<String, Version>, Error> {
            Ok(self.hashes.iter().filter(|(h, _)| hashes.contains(h)).map(|(h,v)| (h.clone(),v.clone())).collect())
        }
        async fn version(&self, id: &str) -> Result<Version, Error> {
            self.versions.get(id).cloned().ok_or(Error::Datos)
        }
        async fn versiones(&self, project: &str, mc: &str, loader: &str) -> Result<Vec<Version>, Error> {
            Ok(self.versions.values().filter(|v| v.project_id == project && compatible(v,mc,loader)).cloned().collect())
        }
        async fn nombre(&self, id: &str) -> Result<String, Error> {
            Ok(id.into())
        }
        async fn archivo(&self, f: &ModFile) -> Result<Vec<u8>, Error> {
            if self.bad {
                Ok(b"corrupto".to_vec())
            } else {
                self.bytes.get(&f.filename).cloned().ok_or(Error::Datos)
            }
        }
    }
    fn fake() -> Fake {
        let mut f = Fake {
            versions: BTreeMap::new(),
            hashes: BTreeMap::new(),
            bytes: BTreeMap::new(),
            bad: false,
        };
        for (id, filename, body) in [
            ("v1", "example-1.jar", b"primero".as_slice()),
            ("v2", "example-2.jar", b"segundo".as_slice()),
        ] {
            f.versions.insert(
                id.into(),
                Version {
                    id: id.into(),
                    project_id: "example".into(),
                    name: "Example".into(),
                    version_number: id.into(),
                    version_type: "release".into(),
                    game_versions: vec!["1.20.1".into()],
                    loaders: vec!["fabric".into()],
                    files: vec![ModFile {
                        filename: filename.into(),
                        url: format!(
                            "https://cdn.modrinth.com/data/example/versions/{id}/{filename}"
                        ),
                        size: body.len() as u64,
                        primary: true,
                        hashes: crate::modrinth::Hashes { sha512: hash(body) },
                    }],
                    dependencies: vec![],
                    changelog: None,
                    date_published: String::new(),
                    downloads: 0,
                },
            );
            f.bytes.insert(filename.into(), body.to_vec());
        }
        f
    }
    struct Temp(PathBuf);
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn store() -> (Temp, Almacen, Instancia) {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).unwrap();
        let temp = Temp(std::env::temp_dir().join(format!("icaro-sync-test-{}", hash(&bytes))));
        let store = Almacen::nuevo(temp.0.join("instances"));
        let i = store
            .crear_con_loader("Amigos", "1.20.1", "fabric", 0)
            .unwrap();
        crate::versiones_instancia::guardar_loader(&store, &i, "0.16.14").unwrap();
        (temp, store, i)
    }
    fn index(f: &Fake, id: &str) -> Indice {
        let v = &f.versions[id];
        Indice {
            esquema: 1,
            minecraft: "1.20.1".into(),
            loader: "fabric".into(),
            loader_version: Some("0.16.14".into()),
            mods: vec![Entrada {
                proyecto: v.project_id.clone(),
                version: v.id.clone(),
                sha512: v.files[0].hashes.sha512.clone(),
            }],
            archivos: vec![],
            avisos: vec![],
        }
    }
    #[tokio::test]
    async fn actualiza_retira_solo_gestionados_y_conserva_archivos_locales() {
        let (_temp, store, i) = store();
        let f = fake();
        super::super::instalar(
            &f,
            &store,
            &i.id,
            vec![f.versions["v1"].clone()],
            &i.minecraft,
            &i.loader,
            |_| {},
        )
        .await
        .unwrap();
        let dir = store.juego(&i.id).unwrap().join("mods");
        fs::write(dir.join("local.jar"), b"personal").unwrap();
        let plan = preparar(&f, &store, &i, index(&f, "v2"), "127.0.0.1:25566".into())
            .await
            .unwrap();
        assert_eq!(plan.cambios.len(), 1);
        assert_eq!(plan.cambios[0].tipo, Tipo::Actualizar);
        assert_eq!(plan.conservados, vec!["local.jar"]);
        aplicar(&f, &store, &i, &plan, CancellationToken::new(), |_, _| {})
            .await
            .unwrap();
        assert_eq!(fs::read(dir.join("example-2.jar")).unwrap(), b"segundo");
        assert!(!dir.join("example-1.jar").exists());
        let mut empty = index(&f, "v2");
        empty.mods.clear();
        let plan = preparar(&f, &store, &i, empty, "127.0.0.1:25566".into())
            .await
            .unwrap();
        assert_eq!(plan.cambios[0].tipo, Tipo::Quitar);
        aplicar(&f, &store, &i, &plan, CancellationToken::new(), |_, _| {})
            .await
            .unwrap();
        assert!(!dir.join("example-2.jar").exists());
        assert_eq!(fs::read(dir.join("local.jar")).unwrap(), b"personal");
    }
    #[tokio::test]
    async fn sincroniza_litemoretica_sin_dependencias_declaradas_desde_cero() {
        for propio in [false, true] {
            let (_temp, store, i) = store();
            let mut f = fake();
            for (project, id, name) in [
                ("OYR5DMCo", "nhi2RkZ9", "litemoretica"),
                ("bEpr0Arc", "litematica1", "litematica"),
                ("GcWjdA9I", "malilib1", "malilib"),
            ] {
                let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
                writer.start_file("fabric.mod.json", zip::write::SimpleFileOptions::default()).unwrap();
                write!(writer, "{{\"id\":\"{name}\",\"version\":\"1\",\"environment\":\"*\"}}").unwrap();
                let bytes = writer.finish().unwrap().into_inner();
                let mut v = f.versions["v1"].clone();
                v.project_id = project.into(); v.id = id.into();
                v.files[0].filename = format!("{name}.jar");
                v.files[0].size = bytes.len() as u64; v.files[0].hashes.sha512 = hash(&bytes);
                if name == "litematica" {
                    v.dependencies = serde_json::from_value(serde_json::json!([
                        {"project_id":"GcWjdA9I","dependency_type":"required"}
                    ])).unwrap();
                }
                f.hashes.insert(hash(&bytes), v.clone());
                f.bytes.insert(v.files[0].filename.clone(), bytes);
                f.versions.insert(id.into(), v);
            }
            let mut indice = index(&f, "nhi2RkZ9");
            let host = store.datos().unwrap().join("host");
            fs::create_dir_all(&host).unwrap();
            if propio {
                let v = &f.versions["nhi2RkZ9"];
                fs::write(host.join("litemoretica.jar"), &f.bytes["litemoretica.jar"]).unwrap();
                indice.esquema = 2; indice.mods.clear();
                indice.archivos.push(ArchivoPropio { id:"icaro-host-litemoretica".into(),
                    nombre:"Litemoretica".into(), version:"1.2.5".into(), archivo:"litemoretica.jar".into(),
                    bytes:v.files[0].size, sha512:v.files[0].hashes.sha512.clone() });
            }
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap().to_string();
            let cancel = CancellationToken::new();
            let servicio = tokio::spawn(servir_archivos(listener, Arc::new(RwLock::new(Some(indice.clone()))), cancel.clone(), Some(host)));
            let dir = store.juego(&i.id).unwrap().join("mods");
            assert!(!dir.exists());
            let plan = preparar(&f, &store, &i, indice.clone(), addr.clone()).await.unwrap();
            for project in ["bEpr0Arc", "GcWjdA9I"] {
                assert!(plan.cambios.iter().any(|c| c.proyecto == project && c.tipo == Tipo::Agregar));
            }
            aplicar(&f, &store, &i, &plan, CancellationToken::new(), |_,_| {}).await.unwrap();
            for name in ["litemoretica.jar", "litematica.jar", "malilib.jar"] {
                assert!(dir.join(name).is_file());
            }
            let plan = preparar(&f, &store, &i, indice, addr).await.unwrap();
            assert!(plan.cambios.is_empty());
            cancel.cancel(); servicio.await.unwrap().unwrap();
        }
        // No extender la excepción a otros proyectos o versiones desconocidas.
        let mut v = fake().versions["v1"].clone();
        assert!(dependencias_cliente(&v).is_empty());
        v.project_id = "OYR5DMCo".into();
        assert!(dependencias_cliente(&v).is_empty());
    }

    #[tokio::test]
    #[ignore = "Descarga Litemoretica y dependencias reales en una instancia temporal; no inicia Minecraft"]
    async fn litemoretica_real_sincroniza_dependencias_cliente() {
        let (_temp, store, _) = store();
        let i = store.crear_con_loader("Cliente real", "1.21.1", "fabric", 0).unwrap();
        crate::versiones_instancia::guardar_loader(&store, &i, "0.19.5").unwrap();
        let source = Catalog::new().unwrap();
        let v = source.version("nhi2RkZ9").await.unwrap();
        assert_eq!(v.project_id, "OYR5DMCo");
        let f = v.files.iter().find(|f| f.primary).unwrap();
        let indice = Indice { esquema:1, minecraft:i.minecraft.clone(), loader:i.loader.clone(),
            loader_version:Some("0.19.5".into()), mods:vec![Entrada {
                proyecto:v.project_id.clone(), version:v.id.clone(), sha512:f.hashes.sha512.clone()
            }], archivos:vec![], avisos:vec![] };
        let plan = preparar(&source, &store, &i, indice.clone(), "127.0.0.1:25566".into()).await.unwrap();
        assert_eq!(plan.cambios.len(), 3);
        aplicar(&source, &store, &i, &plan, CancellationToken::new(), |_,_| {}).await.unwrap();
        let installed = inventario(&store, &i.id).unwrap();
        for project in ["OYR5DMCo", "bEpr0Arc", "GcWjdA9I"] {
            let m = installed.iter().find(|m| m.project_id == project).unwrap();
            let path = store.juego(&i.id).unwrap().join("mods").join(&m.archivo);
            assert_eq!(hash(&fs::read(&path).unwrap()), m.sha512);
            if project == "GcWjdA9I" {
                let mut jar = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
                assert!(jar.by_name("fi/dy/masa/malilib/event/InitializationHandler.class").is_ok());
            }
        }
        assert!(preparar(&source, &store, &i, indice, "127.0.0.1:25566".into()).await.unwrap().cambios.is_empty());
    }

    #[tokio::test]
    async fn indice_antiguo_recupera_bibliotecas_desde_cero_sin_cambiar_pins_del_anfitrion() {
        fn jar(id: &str, entorno: &str) -> Vec<u8> {
            let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
            writer.start_file("fabric.mod.json", zip::write::SimpleFileOptions::default()).unwrap();
            write!(writer, "{{\"id\":\"{id}\",\"version\":\"1\",\"environment\":\"{entorno}\"}}").unwrap();
            writer.finish().unwrap().into_inner()
        }
        let (_temp, store, i) = store();
        let mut f = fake();
        let body = jar("raiz", "*");
        let mut root = f.versions["v1"].clone();
        root.id = "raiz1".into(); root.project_id = "raiz".into();
        root.files[0].filename = "raiz.jar".into();
        root.files[0].hashes.sha512 = hash(&body); root.files[0].size = body.len() as u64;
        root.dependencies = serde_json::from_value(serde_json::json!([
            {"project_id":"biblioteca","dependency_type":"required"},
            {"version_id":"otra1","dependency_type":"required"},
            {"project_id":"example","version_id":"v2","dependency_type":"required"}
        ])).unwrap();
        f.hashes.insert(hash(&body), root);
        for project in ["biblioteca", "otra"] {
            let bytes = jar(project, "*");
            let mut v = f.versions["v1"].clone();
            v.id = format!("{project}1"); v.project_id = project.into();
            v.files[0].filename = format!("{project}.jar");
            v.files[0].size = bytes.len() as u64; v.files[0].hashes.sha512 = hash(&bytes);
            f.bytes.insert(v.files[0].filename.clone(), bytes);
            f.versions.insert(v.id.clone(), v);
        }
        let host = store.datos().unwrap().join("host-mods");
        fs::create_dir_all(&host).unwrap(); fs::write(host.join("raiz.jar"), &body).unwrap();
        let mut indice = index(&f, "v1"); indice.esquema = 2;
        indice.archivos.push(ArchivoPropio { id:"icaro-host-raiz".into(), nombre:"Raíz".into(), version:"1".into(), archivo:"raiz.jar".into(), bytes:body.len() as u64, sha512:hash(&body) });
        let revision = indice.revision().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let cancel = CancellationToken::new();
        let servicio = tokio::spawn(servir_archivos(listener, Arc::new(RwLock::new(Some(indice.clone()))), cancel.clone(), Some(host)));
        let dir = store.juego(&i.id).unwrap().join("mods");
        assert!(!dir.exists());
        let plan = preparar(&f, &store, &i, indice.clone(), addr.to_string()).await.unwrap();
        assert_eq!(plan.revision_origen(), revision);
        assert_eq!(plan.indice.mods.len(), 3);
        assert!(plan.indice.mods.iter().any(|m| m.proyecto == "example" && m.version == "v1"));
        aplicar(&f, &store, &i, &plan, CancellationToken::new(), |_,_| {}).await.unwrap();
        for name in ["raiz.jar", "biblioteca.jar", "otra.jar", "example-1.jar"] { assert!(dir.join(name).is_file()); }
        assert!(!dir.join("example-2.jar").exists());
        let plan = preparar(&f, &store, &i, indice.clone(), addr.to_string()).await.unwrap();
        assert!(plan.cambios.is_empty());
        aplicar(&f, &store, &i, &plan, CancellationToken::new(), |_,_| panic!("No copiar un conjunto ya completo")).await.unwrap();
        let anterior = huella(&store, &i.id).unwrap();
        let server = jar("biblioteca", "server");
        let v = f.versions.get_mut("biblioteca1").unwrap();
        v.files[0].hashes.sha512 = hash(&server); v.files[0].size = server.len() as u64;
        f.bytes.insert("biblioteca.jar".into(), server);
        let plan = preparar(&f, &store, &i, indice, addr.to_string()).await.unwrap();
        assert!(aplicar(&f, &store, &i, &plan, CancellationToken::new(), |_,_| {}).await.unwrap_err().contains("solo admite servidor"));
        assert_eq!(anterior, huella(&store, &i.id).unwrap());
        cancel.cancel(); servicio.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn sincronizar_conserva_mods_personales_de_catalogo_manuales_y_desactivados() {
        let (_temp, store, i) = store();
        let mut f = fake();
        let mut cliente = f.versions["v1"].clone();
        cliente.id = "cliente1".into();
        cliente.project_id = "cliente".into();
        cliente.files[0].filename = "cliente.jar".into();
        cliente.files[0].size = 7;
        cliente.files[0].hashes.sha512 = hash(b"cliente");
        f.bytes.insert("cliente.jar".into(), b"cliente".to_vec());
        f.versions.insert(cliente.id.clone(), cliente.clone());
        super::super::instalar(
            &f, &store, &i.id, vec![f.versions["v1"].clone(), cliente],
            &i.minecraft, &i.loader, |_| {},
        ).await.unwrap();
        let dir = store.juego(&i.id).unwrap().join("mods");
        fs::write(dir.join("manual.jar"), b"manual").unwrap();
        fs::write(dir.join("desactivado.jar.disabled"), b"desactivado").unwrap();
        // Primero registra el conjunto, después lo actualiza y finalmente retira
        // todos los mods del anfitrión. Los personales no pasan a ser gestionados.
        for version in [Some("v1"), Some("v2"), None] {
            let mut indice = index(&f, version.unwrap_or("v2"));
            if version.is_none() { indice.mods.clear(); }
            let plan = preparar(&f, &store, &i, indice, "127.0.0.1:25566".into()).await.unwrap();
            assert!(!plan.cambios.iter().any(|c| c.proyecto == "cliente"));
            aplicar(&f, &store, &i, &plan, CancellationToken::new(), |_, _| {}).await.unwrap();
            assert_eq!(fs::read(dir.join("cliente.jar")).unwrap(), b"cliente");
            assert_eq!(fs::read(dir.join("manual.jar")).unwrap(), b"manual");
            assert_eq!(fs::read(dir.join("desactivado.jar.disabled")).unwrap(), b"desactivado");
            assert!(!dir.join("desactivado.jar").exists());
            assert!(inventario(&store, &i.id).unwrap().iter().any(|m| m.project_id == "cliente"));
            let gestionados: Gestionados = serde_json::from_slice(&fs::read(dir.join(SYNC)).unwrap()).unwrap();
            assert!(!gestionados.proyectos.contains("cliente"));
        }
        assert!(!dir.join("example-1.jar").exists());
        assert!(!dir.join("example-2.jar").exists());
    }

    #[tokio::test]
    async fn conjunto_al_dia_registra_el_origen_una_vez_y_no_vuelve_a_publicar() {
        let (_temp, store, i) = store();
        let f = fake();
        super::super::instalar(
            &f, &store, &i.id, vec![f.versions["v1"].clone()],
            &i.minecraft, &i.loader, |_| {},
        ).await.unwrap();
        let dir = store.juego(&i.id).unwrap().join("mods");
        fs::write(dir.join("personal.jar"), b"personal").unwrap();
        let mut indice = index(&f, "v1");
        indice.esquema = 2;
        indice.archivos.push(ArchivoPropio {
            id: "icaro-host-personal".into(), nombre: "Personal".into(), version: "1".into(),
            archivo: "personal.jar".into(), bytes: 8, sha512: hash(b"personal"),
        });
        let origen = "127.0.0.1:25566";
        let plan = preparar(&f, &store, &i, indice.clone(), origen.into()).await.unwrap();
        assert!(plan.cambios.is_empty());
        assert!(!dir.join(SYNC).exists());
        aplicar(&f, &store, &i, &plan, CancellationToken::new(), |_, _| {}).await.unwrap();
        assert!(dir.join(SYNC).exists());
        let plan = preparar(&f, &store, &i, indice, origen.into()).await.unwrap();
        assert!(plan.cambios.is_empty());
        let registro = fs::metadata(dir.join(SYNC)).unwrap().modified().unwrap();
        aplicar(&f, &store, &i, &plan, CancellationToken::new(), |_, _| {
            panic!("Un conjunto al día no debe volver a preparar archivos");
        }).await.unwrap();
        assert_eq!(registro, fs::metadata(dir.join(SYNC)).unwrap().modified().unwrap());
        fs::write(dir.join("personal.jar"), b"modificado").unwrap();
        assert!(aplicar(&f, &store, &i, &plan, CancellationToken::new(), |_, _| {}).await.is_err());
    }

    #[tokio::test]
    async fn fallo_cancelacion_y_plan_obsoleto_no_sustituyen_el_conjunto() {
        let (_temp, store, i) = store();
        let mut f = fake();
        super::super::instalar(
            &f,
            &store,
            &i.id,
            vec![f.versions["v1"].clone()],
            &i.minecraft,
            &i.loader,
            |_| {},
        )
        .await
        .unwrap();
        let plan = preparar(&f, &store, &i, index(&f, "v2"), "host".into())
            .await
            .unwrap();
        let before = huella(&store, &i.id).unwrap();
        f.bad = true;
        assert!(
            aplicar(&f, &store, &i, &plan, CancellationToken::new(), |_, _| {})
                .await
                .is_err()
        );
        assert_eq!(huella(&store, &i.id).unwrap(), before);
        let cancel = CancellationToken::new();
        cancel.cancel();
        assert!(aplicar(&f, &store, &i, &plan, cancel, |_, _| {})
            .await
            .is_err());
        assert_eq!(huella(&store, &i.id).unwrap(), before);
        let dir = store.juego(&i.id).unwrap().join("mods");
        fs::write(dir.join("nuevo-local.jar"), b"nuevo").unwrap();
        assert!(
            aplicar(&f, &store, &i, &plan, CancellationToken::new(), |_, _| {})
                .await
                .is_err()
        );
        assert_eq!(fs::read(dir.join("example-1.jar")).unwrap(), b"primero");
    }
    #[tokio::test]
    async fn apogee_renombrado_y_duplicado_se_actualiza_por_id_y_hash_sin_perder_mods_personales() {
        fn jar(id: &str, contenido: &str) -> Vec<u8> {
            let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
            writer
                .start_file("fabric.mod.json", zip::write::SimpleFileOptions::default())
                .unwrap();
            write!(
                writer,
                "{{\"id\":\"{id}\",\"version\":\"1.1.1\",\"environment\":\"*\"}}"
            )
            .unwrap();
            writer
                .start_file("registros.txt", zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(contenido.as_bytes()).unwrap();
            writer.finish().unwrap().into_inner()
        }
        for exacto_presente in [false, true] {
            let (_temp, store, i) = store();
            let f = fake();
            let game = store.juego(&i.id).unwrap();
            let dir = game.join("mods");
            fs::create_dir(&dir).unwrap();
            let viejo = jar("apogee", "registros anteriores");
            let nuevo = jar("apogee", "23 registros nuevos del servidor");
            let personal = jar("solo_cliente", "personal");
            fs::write(dir.join("apogee-1.1.1.jar"), &viejo).unwrap();
            fs::write(dir.join("apogee-copia.jar"), &viejo).unwrap();
            fs::write(dir.join("apogee-antiguo.jar.disabled"), &viejo).unwrap();
            fs::write(dir.join("personal.jar"), &personal).unwrap();
            if exacto_presente {
                fs::write(dir.join("apogee-1.2.0.jar"), &nuevo).unwrap();
            }
            let host = game.join("host");
            fs::create_dir(&host).unwrap();
            fs::write(host.join("apogee-1.2.0.jar"), &nuevo).unwrap();
            let mut indice = index(&f, "v1");
            indice.esquema = 2;
            indice.mods.clear();
            indice.archivos.push(ArchivoPropio {
                id: "icaro-host-apogee".into(),
                nombre: "Apogee".into(),
                version: "1.1.1".into(),
                archivo: "apogee-1.2.0.jar".into(),
                bytes: nuevo.len() as u64,
                sha512: hash(&nuevo),
            });
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap().to_string();
            let cancel = CancellationToken::new();
            let servicio = tokio::spawn(servir_archivos(
                listener,
                Arc::new(RwLock::new(Some(indice.clone()))),
                cancel.clone(),
                Some(host),
            ));
            let plan = preparar(&f, &store, &i, indice.clone(), addr.clone())
                .await
                .unwrap();
            assert!(plan.puede_aplicar() && plan.conflictos.is_empty());
            assert_eq!(plan.reemplazos.len(), 2);
            if !exacto_presente {
                assert!(plan.cambios.iter().any(|c| c.tipo == Tipo::Actualizar));
            }
            aplicar(&f, &store, &i, &plan, CancellationToken::new(), |_, _| {})
                .await
                .unwrap();
            assert_eq!(fs::read(dir.join("apogee-1.2.0.jar")).unwrap(), nuevo);
            assert!(!dir.join("apogee-1.1.1.jar").exists() && !dir.join("apogee-copia.jar").exists());
            assert_eq!(fs::read(dir.join("personal.jar")).unwrap(), personal);
            assert!(dir.join("apogee-antiguo.jar.disabled").exists());
            let respaldo = fs::read_dir(&game)
                .unwrap()
                .flatten()
                .find(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .starts_with(".icaro-mods-conflictos-")
                })
                .unwrap()
                .path();
            assert_eq!(fs::read(respaldo.join("apogee-1.1.1.jar")).unwrap(), viejo);
            let plan = preparar(&f, &store, &i, indice, addr).await.unwrap();
            assert!(plan.cambios.is_empty() && plan.reemplazos.is_empty());
            cancel.cancel();
            servicio.await.unwrap().unwrap();
        }
    }

    #[tokio::test]
    async fn conflicto_local_muestra_el_plan_y_solo_reemplaza_con_autorizacion_y_respaldo() {
        let (_temp, store, i) = store();
        let f = fake();
        let game = store.juego(&i.id).unwrap();
        let dir = game.join("mods");
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("survivor.jar"), b"mi mod personal").unwrap();
        fs::write(dir.join("cliente.jar"), b"solo cliente").unwrap();
        let host = game.join("host");
        fs::create_dir(&host).unwrap();
        fs::write(host.join("survivor.jar"), b"mod del anfitrion").unwrap();
        let mut indice = index(&f, "v2");
        indice.esquema = 2;
        indice.archivos.push(ArchivoPropio {
            id: "icaro-host-survivor".into(),
            nombre: "Survivor".into(),
            version: "2".into(),
            archivo: "survivor.jar".into(),
            bytes: 17,
            sha512: hash(b"mod del anfitrion"),
        });
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let cancel = CancellationToken::new();
        let servicio = tokio::spawn(servir_archivos(
            listener,
            Arc::new(RwLock::new(Some(indice.clone()))),
            cancel.clone(),
            Some(host),
        ));
        let mut plan = preparar(&f, &store, &i, indice.clone(), addr.clone())
            .await
            .unwrap();
        assert_eq!(plan.conflictos, ["survivor.jar"]);
        assert_eq!(plan.cambios.len(), 2);
        assert!(!plan.puede_aplicar());
        assert!(
            aplicar(&f, &store, &i, &plan, CancellationToken::new(), |_, _| {})
                .await
                .is_err()
        );
        assert_eq!(
            fs::read(dir.join("survivor.jar")).unwrap(),
            b"mi mod personal"
        );
        plan.autorizar_reemplazos(true);
        aplicar(&f, &store, &i, &plan, CancellationToken::new(), |_, _| {})
            .await
            .unwrap();
        assert_eq!(
            fs::read(dir.join("survivor.jar")).unwrap(),
            b"mod del anfitrion"
        );
        assert_eq!(fs::read(dir.join("cliente.jar")).unwrap(), b"solo cliente");
        let respaldo = fs::read_dir(&game)
            .unwrap()
            .flatten()
            .find(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with(".icaro-mods-conflictos-")
            })
            .unwrap()
            .path();
        assert_eq!(
            fs::read(respaldo.join("survivor.jar")).unwrap(),
            b"mi mod personal"
        );
        let plan = preparar(&f, &store, &i, indice, addr).await.unwrap();
        assert!(plan.conflictos.is_empty() && plan.cambios.is_empty());
        aplicar(&f, &store, &i, &plan, CancellationToken::new(), |_, _| {})
            .await
            .unwrap();
        assert!(respaldo.join("survivor.jar").is_file());
        cancel.cancel();
        servicio.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn no_adivina_versiones_ni_sobrescribe_nombres_locales() {
        let (_temp, store, i) = store();
        let f = fake();
        let mut invalid = index(&f, "v2");
        invalid.mods[0].sha512 = "a".repeat(128);
        assert!(preparar(&f, &store, &i, invalid, "host".into())
            .await
            .is_err());
        let mut invalid = index(&f, "v2");
        invalid.loader_version = Some("0.16.13".into());
        assert!(preparar(&f, &store, &i, invalid, "host".into())
            .await
            .is_err());
        let dir = store.juego(&i.id).unwrap().join("mods");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("example-2.jar"), b"local").unwrap();
        let plan = preparar(&f, &store, &i, index(&f, "v2"), "host".into())
            .await
            .unwrap();
        assert!(
            aplicar(&f, &store, &i, &plan, CancellationToken::new(), |_, _| {})
                .await
                .is_err()
        );
        assert_eq!(fs::read(dir.join("example-2.jar")).unwrap(), b"local");
    }
    #[tokio::test]
    async fn servicio_solo_entrega_indice_y_rechaza_rutas_y_metodos() {
        let f = fake();
        let value = index(&f, "v2");
        let publication = Arc::new(RwLock::new(Some(value.clone())));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let cancel = CancellationToken::new();
        let worker = tokio::spawn(servir(listener, publication.clone(), cancel.clone()));
        assert_eq!(
            consultar(addr, CancellationToken::new())
                .await
                .unwrap()
                .revision()
                .unwrap(),
            value.revision().unwrap()
        );
        for request in [
            "POST /icaro/v1/mods HTTP/1.1\r\nHost: localhost\r\n\r\n",
            "GET /../instance.json HTTP/1.1\r\nHost: localhost\r\n\r\n",
        ] {
            let mut c = tokio::net::TcpStream::connect(addr).await.unwrap();
            c.write_all(request.as_bytes()).await.unwrap();
            let mut response = vec![];
            c.read_to_end(&mut response).await.unwrap();
            assert!(response.starts_with(b"HTTP/1.1 404"));
        }
        *publication.write().await = None;
        assert!(consultar(addr, CancellationToken::new()).await.is_err());
        cancel.cancel();
        worker.await.unwrap().unwrap();
        assert!(endpoint("8.8.8.8").is_err());
        assert!(endpoint("0.0.0.0").is_err());
        assert!(endpoint("10.147.17.1:65535").is_err());
        assert_eq!(endpoint("10.147.17.1").unwrap().port(), 25566);
    }
    #[tokio::test]
    #[ignore = "Descarga dos mods reales de Modrinth en instancias temporales aisladas"]
    async fn modrinth_real_publica_indice_http_y_sincroniza_contenido_exacto() {
        let (_host, host, hi) = store();
        let (_client, client, ci) = store();
        let catalog = Catalog::new().unwrap();
        let api = catalog
            .versions("fabric-api", Some("1.20.1"), Some("fabric"))
            .await
            .unwrap()
            .into_iter()
            .find(|v| v.version_type == "release")
            .unwrap();
        let sodium = catalog
            .versions("sodium", Some("1.20.1"), Some("fabric"))
            .await
            .unwrap()
            .into_iter()
            .find(|v| v.version_type == "release")
            .unwrap();
        super::super::instalar(
            &catalog,
            &host,
            &hi.id,
            vec![api.clone(), sodium.clone()],
            &hi.minecraft,
            &hi.loader,
            |_| {},
        )
        .await
        .unwrap();
        let value = publicar(&catalog, &host, &hi).await.unwrap();
        assert!(value.mods.iter().any(|m| m.proyecto == api.project_id));
        assert!(!value.mods.iter().any(|m| m.proyecto == sodium.project_id));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let token = CancellationToken::new();
        let worker = tokio::spawn(servir(
            listener,
            Arc::new(RwLock::new(Some(value))),
            token.clone(),
        ));
        let received = consultar(addr, CancellationToken::new()).await.unwrap();
        let plan = preparar(&catalog, &client, &ci, received, addr.to_string())
            .await
            .unwrap();
        aplicar(
            &catalog,
            &client,
            &ci,
            &plan,
            CancellationToken::new(),
            |_, _| {},
        )
        .await
        .unwrap();
        let installed = inventario(&client, &ci.id).unwrap();
        assert_eq!(installed.len(), 1);
        assert_eq!(installed[0].version_id, api.id);
        let host_file = host
            .juego(&hi.id)
            .unwrap()
            .join("mods")
            .join(&installed[0].archivo);
        let client_file = client
            .juego(&ci.id)
            .unwrap()
            .join("mods")
            .join(&installed[0].archivo);
        assert_eq!(
            hash_archivo(&host_file).unwrap(),
            hash_archivo(&client_file).unwrap()
        );
        token.cancel();
        worker.await.unwrap().unwrap();
    }
}
