// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Exportación ZIP del contenido elegido, sin credenciales ni cachés de Ícaro.
use crate::instancias::{Almacen, Instancia};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, Default)]
pub struct Medidas {
    pub bytes: [u64; 5],
    pub mundos: usize,
    pub capturas: usize,
}
type Resultado<T> = Result<T, String>;

/// Inventario leído en un trabajador; la vista nunca recorre el disco.
#[derive(Debug, Clone, Default)]
pub struct Arbol {
    pub nodos: Vec<Nodo>,
}
#[derive(Debug, Clone)]
pub struct Nodo {
    pub ruta: String,
    pub carpeta: bool,
    pub bytes: u64,
    pub etiqueta: Option<&'static str>,
    pub seleccionado: bool,
    pub abierto: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seleccion {
    Recomendada,
    Todo,
    Compartir,
    Mundos,
    Personalizada,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Formato {
    Icaro,
    Prism,
}
impl Arbol {
    pub fn seleccionar(&mut self, seleccion: Seleccion) {
        for nodo in &mut self.nodos {
            let top = nodo.ruta.split('/').next().unwrap_or("");
            nodo.seleccionado = match seleccion {
                Seleccion::Recomendada => !matches!(
                    nodo.etiqueta,
                    Some("Caché" | "Registro" | "Respaldo" | "Personal")
                ),
                Seleccion::Todo => true,
                Seleccion::Compartir => matches!(
                    top,
                    "mods"
                        | "config"
                        | "resourcepacks"
                        | "shaderpacks"
                        | "options.txt"
                        | "icon.png"
                        | "servers.dat"
                ),
                Seleccion::Mundos => top == "saves",
                Seleccion::Personalizada => nodo.seleccionado,
            };
        }
    }
    pub fn marca(&self, ruta: &str) -> (bool, bool) {
        let prefijo = format!("{ruta}/");
        let mut yes = 0;
        let mut no = 0;
        for n in self.nodos.iter().filter(|n| {
            (ruta.is_empty() || n.ruta == ruta || n.ruta.starts_with(&prefijo)) && !n.carpeta
        }) {
            if n.seleccionado {
                yes += 1
            } else {
                no += 1
            }
        }
        if yes + no == 0 {
            return (
                self.nodos
                    .iter()
                    .find(|n| n.ruta == ruta)
                    .is_some_and(|n| n.seleccionado),
                false,
            );
        }
        (yes > 0 && no == 0, yes > 0 && no > 0)
    }
    pub fn alternar(&mut self, ruta: &str) {
        let marcado = self.marca(ruta).0;
        let prefijo = format!("{ruta}/");
        for n in &mut self.nodos {
            if ruta.is_empty() || n.ruta == ruta || n.ruta.starts_with(&prefijo) {
                n.seleccionado = !marcado;
            }
        }
    }
    pub fn resumen(&self) -> (usize, u64) {
        let padres: std::collections::BTreeSet<_> = self
            .nodos
            .iter()
            .filter_map(|n| n.ruta.rsplit_once('/').map(|(padre, _)| padre))
            .collect();
        self.nodos
            .iter()
            .filter(|n| n.seleccionado && (!n.carpeta || !padres.contains(n.ruta.as_str())))
            .fold((0, 0), |(c, b), n| {
                (c + 1, b + if n.carpeta { 0 } else { n.bytes })
            })
    }
    pub fn visible(&self, nodo: &Nodo) -> bool {
        nodo.ruta.rsplit_once('/').is_none_or(|(parent, _)| {
            self.nodos
                .iter()
                .filter(|n| {
                    n.carpeta && (n.ruta == parent || parent.starts_with(&format!("{}/", n.ruta)))
                })
                .all(|n| n.abierto)
        })
    }
    /// Computes all checkboxes together; drawing each row must not scan the complete inventory.
    pub fn marcas(&self) -> std::collections::HashMap<&str, (bool, bool)> {
        let mut counts = std::collections::HashMap::<&str, (usize, usize)>::new();
        for nodo in self.nodos.iter().filter(|n| !n.carpeta) {
            let mut ruta = nodo.ruta.as_str();
            loop {
                let count = counts.entry(ruta).or_default();
                if nodo.seleccionado {
                    count.0 += 1
                } else {
                    count.1 += 1
                }
                if ruta.is_empty() {
                    break;
                }
                ruta = ruta.rsplit_once('/').map_or("", |(parent, _)| parent);
            }
        }
        let mut marks: std::collections::HashMap<_, _> = self
            .nodos
            .iter()
            .map(|n| (n.ruta.as_str(), (n.seleccionado, false)))
            .collect();
        marks.entry("").or_insert((false, false));
        for (ruta, (yes, no)) in counts {
            marks.insert(ruta, (yes > 0 && no == 0, yes > 0 && no > 0));
        }
        marks
    }
    /// Each ancestor is looked up once instead of searching all folders for each file.
    pub fn visibles(&self) -> impl Iterator<Item = &Nodo> {
        let cerradas: std::collections::HashSet<_> = self
            .nodos
            .iter()
            .filter(|n| n.carpeta && !n.abierto)
            .map(|n| n.ruta.as_str())
            .collect();
        self.nodos.iter().filter(move |n| {
            let mut parent = n.ruta.rsplit_once('/').map(|(p, _)| p);
            while let Some(ruta) = parent {
                if cerradas.contains(ruta) {
                    return false;
                }
                parent = ruta.rsplit_once('/').map(|(p, _)| p);
            }
            true
        })
    }
}
fn etiqueta_ruta(ruta: &str) -> Option<&'static str> {
    let top = ruta.split('/').next().unwrap_or("").to_ascii_lowercase();
    if matches!(
        top.as_str(),
        "cache" | ".bobby" | "downloads" | "webcache" | "assets" | "libraries" | "versions"
    ) {
        Some("Caché")
    } else if matches!(
        top.as_str(),
        "logs" | "crash-reports" | "debug" | ".mixin.out"
    ) || top.starts_with("hs_err_pid")
        || top.ends_with(".log")
    {
        Some("Registro")
    } else if matches!(top.as_str(), "backups" | "backup")
        || ruta.ends_with("_old")
        || ruta.ends_with(".bak")
    {
        Some("Respaldo")
    } else if matches!(
        top.as_str(),
        "command_history.txt" | "usercache.json" | "usernamecache.json"
    ) {
        Some("Personal")
    } else if matches!(
        top.as_str(),
        "journeymap" | "xaero" | "xaeroworldmap" | "xaerominimap"
    ) {
        Some("Mapas")
    } else if top.starts_with("distant_horizons") || top == "dh_data" {
        Some("Datos de mod")
    } else {
        None
    }
}
fn inventariar(path: &Path, root: &Path, nodos: &mut Vec<Nodo>, depth: usize) -> Resultado<u64> {
    if depth > 32 || nodos.len() >= 100_000 {
        return Err("La instancia supera el límite de archivos de una exportación.".into());
    }
    let m = metadata(path)?;
    let ruta = path
        .strip_prefix(root)
        .map_err(|_| "Ruta inválida.")?
        .to_str()
        .ok_or("El nombre de un archivo no es válido.")?
        .replace('\\', "/");
    // Bloqueos y temporales nunca forman parte de una instancia portable.
    if ruta.split('/').any(|p| {
        p.starts_with(".icaro-")
            && !matches!(
                p,
                ".icaro-mods.json" | ".icaro-resourcepacks.json" | ".icaro-shaderpacks.json"
            )
    }) {
        return Ok(0);
    }
    let pos = nodos.len();
    nodos.push(Nodo {
        ruta: ruta.clone(),
        carpeta: m.is_dir(),
        bytes: m.len(),
        etiqueta: etiqueta_ruta(&ruta),
        seleccionado: true,
        abierto: false,
    });
    let bytes = if m.is_dir() {
        let mut hijos = fs::read_dir(path)
            .map_err(|_| "No se pudo leer una carpeta.")?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "No se pudo leer un archivo.")?;
        hijos.sort_by_key(|e| e.file_name().to_string_lossy().to_lowercase());
        let mut total = 0u64;
        for e in hijos {
            total = total
                .checked_add(inventariar(&e.path(), root, nodos, depth + 1)?)
                .ok_or("El tamaño total es demasiado grande.")?;
        }
        total
    } else if m.is_file() {
        m.len()
    } else {
        return Err("La instancia contiene un archivo que no puede exportarse.".into());
    };
    nodos[pos].bytes = bytes;
    Ok(bytes)
}
pub fn arbol(store: &Almacen, id: &str) -> Resultado<Arbol> {
    let game = store.juego(id).map_err(|e| e.to_string())?;
    metadata(&game)?;
    let mut tree = Arbol::default();
    let mut entries = fs::read_dir(&game)
        .map_err(|_| "No se pudo abrir la instancia.")?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "No se pudo leer la instancia.")?;
    entries.sort_by_key(|e| e.file_name().to_string_lossy().to_lowercase());
    for e in entries {
        inventariar(&e.path(), &game, &mut tree.nodos, 0)?;
    }
    tree.seleccionar(Seleccion::Recomendada);
    Ok(tree)
}
pub fn nombre_sugerido(i: &Instancia) -> String {
    let nombre: String = i
        .nombre
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, ' ' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!(
        "{}-{}.zip",
        nombre.trim().trim_end_matches('.'),
        i.minecraft
    )
}

/// ZIP portable con publicación atómica, sin sobrescribir silenciosamente.
pub fn archivo(
    store: &Almacen,
    i: &Instancia,
    tree: &Arbol,
    formato: Formato,
    destination: &Path,
    mut progreso: impl FnMut(u64, u64),
) -> Resultado<PathBuf> {
    let game = store.juego(&i.id).map_err(|e| e.to_string())?;
    let _lock = crate::mods::guardia(&game).map_err(|e| e.to_string())?;
    let parent = destination
        .parent()
        .ok_or("Elige un archivo de destino válido.")?
        .canonicalize()
        .map_err(|_| "Elige una carpeta existente.")?;
    metadata(&parent)?;
    if parent.starts_with(
        game.canonicalize()
            .map_err(|_| "No se pudo abrir la instancia.")?,
    ) {
        return Err("El destino debe estar fuera de la carpeta de la instancia.".into());
    }
    let name = destination
        .file_name()
        .ok_or("Elige un nombre de archivo.")?;
    let final_path = parent.join(name);
    if final_path.exists() {
        return Err("Ese archivo ya existe; elige otro nombre para conservarlo.".into());
    }
    let loader = crate::versiones_instancia::leer_loader(store, i);
    if formato == Formato::Prism && i.loader != "vanilla" && loader.is_none() {
        return Err(
            "Selecciona una versión real del loader antes de exportar para Prism o MultiMC.".into(),
        );
    }
    let mut random = [0u8; 8];
    getrandom::fill(&mut random).map_err(|_| "No se pudo preparar la exportación.")?;
    let temp = parent.join(format!(".icaro-export-{}.tmp", u64::from_le_bytes(random)));
    let result = (|| {
        let out = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|_| "No se pudo crear el ZIP.")?;
        let mut zip = zip::ZipWriter::new(out);
        let opt = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .large_file(true);
        if formato == Formato::Icaro {
            zip.start_file("instance.json", opt)
                .map_err(|_| "No se pudo escribir el ZIP.")?;
            zip.write_all(&serde_json::to_vec_pretty(i).map_err(|_| "Metadatos inválidos.")?)
                .map_err(|_| "No se pudo guardar la configuración.")?;
            if let Some(pin) = &loader {
                zip.start_file("loader.json", opt)
                    .map_err(|_| "No se pudo escribir el ZIP.")?;
                zip.write_all(
                    serde_json::json!({"minecraft":i.minecraft,"loader":i.loader,"version":pin})
                        .to_string()
                        .as_bytes(),
                )
                .map_err(|_| "No se pudo guardar el loader.")?;
            }
        } else {
            zip.start_file("instance.cfg", opt)
                .map_err(|_| "No se pudo escribir el ZIP.")?;
            zip.write_all(
                format!(
                    "[General]\nname={}\nInstanceType=OneSix\n",
                    i.nombre.replace(['\r', '\n'], " ")
                )
                .as_bytes(),
            )
            .map_err(|_| "No se pudo guardar la configuración.")?;
            let mut components = vec![
                serde_json::json!({"uid":"net.minecraft","version":i.minecraft,"important":true}),
            ];
            if let Some(pin) = loader {
                let uid = match i.loader.as_str() {
                    "fabric" => "net.fabricmc.fabric-loader",
                    "quilt" => "org.quiltmc.quilt-loader",
                    "forge" => "net.minecraftforge",
                    "neoforge" => "net.neoforged",
                    _ => return Err("Loader no compatible con este formato.".into()),
                };
                components.push(serde_json::json!({"uid":uid,"version":pin}));
            }
            zip.start_file("mmc-pack.json", opt)
                .map_err(|_| "No se pudo escribir el ZIP.")?;
            zip.write_all(
                serde_json::json!({"formatVersion":1,"components":components})
                    .to_string()
                    .as_bytes(),
            )
            .map_err(|_| "No se pudo guardar el perfil.")?;
        }
        // La identidad y el icono pertenecen a la instancia, fuera de minecraft.
        let instance_root = store.carpeta(&i.id).map_err(|e| e.to_string())?;
        let metadata_names: &[&str] = if formato == Formato::Icaro {
            &["instance.cfg", "icon.png", "icono.json"]
        } else {
            &["icon.png"]
        };
        for name in metadata_names {
            let path = instance_root.join(name);
            if !path.exists() {
                continue;
            }
            let m = metadata(&path)?;
            if !m.is_file() || m.len() > 10 * 1024 * 1024 {
                return Err("Los metadatos de la instancia no son válidos.".into());
            }
            zip.start_file(*name, opt)
                .map_err(|_| "No se pudo guardar los metadatos.")?;
            use std::io::Read;
            let n = std::io::copy(
                &mut fs::File::open(path)
                    .map_err(|_| "No se pudo leer los metadatos.")?
                    .take(m.len() + 1),
                &mut zip,
            )
            .map_err(|_| "No se pudo guardar los metadatos.")?;
            if n != m.len() {
                return Err("Los metadatos cambiaron durante la exportación.".into());
            }
        }
        let (count, total) = tree.resumen();
        if count == 0 && !tree.nodos.is_empty() {
            return Err("Selecciona al menos un archivo para exportar.".into());
        }
        let mut copied = 0u64;
        progreso(0, total);
        let prefix = if formato == Formato::Icaro {
            "minecraft"
        } else {
            ".minecraft"
        };
        for node in tree.nodos.iter().filter(|n| n.seleccionado) {
            let relative = Path::new(&node.ruta);
            if relative.is_absolute()
                || relative
                    .components()
                    .any(|c| !matches!(c, std::path::Component::Normal(_)))
                || node.ruta.contains('\\')
            {
                return Err("Ruta de exportación inválida.".into());
            }
            // Revalida todos los antecesores antes de abrir: rechaza junctions.
            let mut full = game.clone();
            for part in relative.components() {
                full.push(part.as_os_str());
                metadata(&full)?;
            }
            let m = metadata(&full)?;
            if node.carpeta {
                if !m.is_dir() {
                    return Err(
                        "La instancia cambió durante la exportación; vuelve a abrir el diálogo."
                            .into(),
                    );
                }
                zip.add_directory(format!("{prefix}/{}/", node.ruta), opt)
                    .map_err(|_| "No se pudo crear una carpeta en el ZIP.")?;
                continue;
            }
            if !m.is_file() || m.len() != node.bytes {
                return Err(
                    "La instancia cambió durante la exportación; vuelve a abrir el diálogo.".into(),
                );
            }
            zip.start_file(format!("{prefix}/{}", node.ruta), opt)
                .map_err(|_| "No se pudo escribir el ZIP.")?;
            use std::io::Read;
            let mut input = fs::File::open(&full)
                .map_err(|_| "No se pudo leer un archivo.")?
                .take(node.bytes + 1);
            let mut buf = [0u8; 64 * 1024];
            let mut file_copied = 0u64;
            loop {
                let n = input
                    .read(&mut buf)
                    .map_err(|_| "No se pudo leer un archivo.")?;
                if n == 0 {
                    break;
                }
                zip.write_all(&buf[..n])
                    .map_err(|_| "No se pudo guardar el ZIP.")?;
                copied += n as u64;
                file_copied += n as u64;
                progreso(copied, total);
            }
            if file_copied != node.bytes || metadata(&full)?.len() != node.bytes {
                return Err(
                    "Un archivo cambió durante la exportación; vuelve a abrir el diálogo.".into(),
                );
            }
        }
        zip.finish()
            .map_err(|_| "No se pudo terminar el ZIP.")?
            .sync_all()
            .map_err(|_| "No se pudo guardar el ZIP.")?;
        fs::hard_link(&temp, &final_path)
            .map_err(|_| "No se pudo publicar el ZIP; elige otro nombre o destino.")?;
        Ok(final_path)
    })();
    let _ = fs::remove_file(temp);
    result
}
fn metadata(path: &Path) -> Resultado<fs::Metadata> {
    let m =
        fs::symlink_metadata(path).map_err(|_| "No se pudo leer un archivo de la instancia.")?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err("La exportación no sigue enlaces a otras carpetas.".into());
        }
    }
    if m.file_type().is_symlink() {
        return Err("La exportación no sigue enlaces a otras carpetas.".into());
    }
    Ok(m)
}
fn visitar(
    path: &Path,
    root: &Path,
    list: &mut Vec<(PathBuf, String, u64)>,
    depth: usize,
) -> Resultado<()> {
    if !path.exists() {
        return Ok(());
    }
    if depth > 32 || list.len() > 100_000 {
        return Err("La instancia supera el límite de archivos de una exportación.".into());
    }
    let m = metadata(path)?;
    if m.is_dir() {
        for e in fs::read_dir(path).map_err(|_| "No se pudo leer una carpeta de la instancia.")? {
            visitar(
                &e.map_err(|_| "No se pudo leer un archivo.")?.path(),
                root,
                list,
                depth + 1,
            )?;
        }
    } else if m.is_file() {
        let relative = path
            .strip_prefix(root)
            .map_err(|_| "Ruta de exportación inválida.")?
            .to_string_lossy()
            .replace('\\', "/");
        if relative
            .split('/')
            .any(|p| p.starts_with(".icaro-") && p != ".icaro-mods.json")
        {
            return Ok(());
        }
        list.push((path.into(), relative, m.len()));
    } else {
        return Err("La instancia contiene un archivo que no puede exportarse.".into());
    }
    Ok(())
}
fn grupo(game: &Path, index: usize) -> Resultado<Vec<(PathBuf, String, u64)>> {
    let mut list = vec![];
    let names: &[&str] = match index {
        0 => &[
            "mods",
            "config",
            "resourcepacks",
            "shaderpacks",
            "datapacks",
        ],
        1 => &["saves"],
        2 => &["options.txt", "optionsof.txt", "servers.dat"],
        3 => &["screenshots"],
        _ => &["logs", "crash-reports"],
    };
    for n in names {
        visitar(&game.join(n), game, &mut list, 0)?;
    }
    Ok(list)
}
pub fn medir(store: &Almacen, id: &str) -> Resultado<Medidas> {
    let game = store
        .juego(id)
        .map_err(|_| "No se pudo abrir la instancia.")?;
    metadata(&game)?;
    let mut sizes = Medidas::default();
    for n in 0..5 {
        let files = grupo(&game, n)?;
        sizes.bytes[n] = files.iter().map(|(_, _, len)| *len).sum();
        if n == 3 {
            sizes.capturas = files.len();
        }
    }
    let saves = game.join("saves");
    if saves.exists() {
        sizes.mundos = fs::read_dir(saves)
            .map_err(|_| "No se pudo leer los mundos.")?
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_ok_and(|m| m.is_dir()))
            .count();
    }
    Ok(sizes)
}
pub fn zip(
    store: &Almacen,
    i: &Instancia,
    options: [bool; 4],
    destination: &Path,
) -> Resultado<PathBuf> {
    let game = store
        .juego(&i.id)
        .map_err(|_| "No se pudo abrir la instancia.")?;
    metadata(&game)?;
    let _lock = crate::mods::guardia(&game).map_err(|e| e.to_string())?;
    let target = destination
        .canonicalize()
        .map_err(|_| "Elige una carpeta de destino que exista.")?;
    metadata(&target)?;
    if !target.is_dir()
        || target.starts_with(
            game.canonicalize()
                .map_err(|_| "No se pudo abrir la instancia.")?,
        )
    {
        return Err("El destino debe estar fuera de la carpeta de la instancia.".into());
    }
    let mut list = grupo(&game, 0)?;
    for (n, include) in options.iter().enumerate() {
        if *include {
            list.extend(grupo(&game, n + 1)?);
        }
    }
    let name: String = i
        .nombre
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '-' | '_'))
        .take(60)
        .collect();
    let mut random = [0u8; 8];
    getrandom::fill(&mut random).map_err(|_| "No se pudo preparar la exportación.")?;
    let suffix: String = random.iter().map(|b| format!("{b:02x}")).collect();
    let final_path = target.join(format!("icaro-{name}-{suffix}.zip"));
    let temp = target.join(format!(".icaro-export-{suffix}.tmp"));
    let mut created = false;
    let result = (|| {
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|_| "No se pudo crear el archivo de exportación.")?;
        created = true;
        let mut zip = zip::ZipWriter::new(file);
        let opt = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .large_file(true);
        zip.start_file("instance.json", opt)
            .map_err(|_| "No se pudo escribir el ZIP.")?;
        zip.write_all(
            &serde_json::to_vec_pretty(i).map_err(|_| "No se pudo guardar la configuración.")?,
        )
        .map_err(|_| "No se pudo escribir el ZIP.")?;
        for (path, name, _) in list {
            metadata(&path)?;
            zip.start_file(format!("minecraft/{name}"), opt)
                .map_err(|_| "No se pudo escribir el ZIP.")?;
            std::io::copy(
                &mut fs::File::open(path)
                    .map_err(|_| "No se pudo leer un archivo de la instancia.")?,
                &mut zip,
            )
            .map_err(|_| "No se pudo copiar un archivo al ZIP.")?;
        }
        zip.finish()
            .map_err(|_| "No se pudo terminar el ZIP.")?
            .sync_all()
            .map_err(|_| "No se pudo guardar el ZIP.")?;
        fs::hard_link(&temp, &final_path)
            .map_err(|_| "No se pudo guardar el archivo final; elige otra carpeta de destino.")?;
        Ok(final_path)
    })();
    if created {
        let _ = fs::remove_file(&temp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (PathBuf, Almacen, Instancia) {
        let mut random = [0u8; 8];
        getrandom::fill(&mut random).unwrap();
        let root =
            std::env::temp_dir().join(format!("icaro-export-tree-{}", u64::from_le_bytes(random)));
        let store = Almacen::nuevo(root.join("instances"));
        let i = store
            .crear_con_loader("Mi mundo", "1.21.1", "vanilla", 0)
            .unwrap();
        let game = store.juego(&i.id).unwrap();
        for (name, bytes) in [
            ("mods/a.jar", b"jar".as_slice()),
            ("config/a.json", b"{}"),
            ("saves/Mundo/level.dat", b"mundo"),
            ("logs/latest.log", b"registro"),
            ("command_history.txt", b"personal"),
            ("options.txt", b"opciones"),
            ("screenshots/a.png", b"foto"),
        ] {
            let path = game.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }
        fs::create_dir(root.join("out")).unwrap();
        (root, store, i)
    }
    #[test]
    fn arbol_presets_casilla_mixta_y_expansion_son_coherentes() {
        let (root, store, i) = fixture();
        let mut a = arbol(&store, &i.id).unwrap();
        assert!(a.marca("").1);
        assert!(
            !a.nodos
                .iter()
                .find(|n| n.ruta == "logs/latest.log")
                .unwrap()
                .seleccionado
        );
        assert!(!a.visible(
            a.nodos
                .iter()
                .find(|n| n.ruta == "saves/Mundo/level.dat")
                .unwrap()
        ));
        a.seleccionar(Seleccion::Todo);
        assert_eq!(a.marca(""), (true, false));
        a.alternar("saves");
        assert_eq!(a.marca("saves"), (false, false));
        assert_eq!(a.marca(""), (false, true));
        a.seleccionar(Seleccion::Mundos);
        assert_eq!(a.resumen(), (1, 5));
        a.seleccionar(Seleccion::Compartir);
        assert_eq!(a.resumen(), (3, 13));
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn inventario_grande_oculto_conserva_marcas_y_expansion_de_carpetas() {
        let nodo = |ruta: String, carpeta, seleccionado, abierto| Nodo {
            ruta,
            carpeta,
            seleccionado,
            abierto,
            bytes: 1,
            etiqueta: None,
        };
        let mut a = Arbol {
            nodos: vec![
                nodo("saves".into(), true, true, false),
                nodo("saves/Mundo".into(), true, true, true),
                nodo("carpeta-vacia".into(), true, false, false),
                nodo("options.txt".into(), false, true, false),
            ],
        };
        for n in 0..100_000 {
            a.nodos.push(nodo(
                format!("saves/Mundo/r.{n}.mca"),
                false,
                n % 2 == 0,
                false,
            ));
        }
        let marcas = a.marcas();
        assert_eq!(marcas[""], (false, true));
        assert_eq!(marcas["saves"], (false, true));
        assert_eq!(marcas["saves/Mundo"], (false, true));
        assert_eq!(marcas["saves/Mundo/r.0.mca"], (true, false));
        assert_eq!(marcas["saves/Mundo/r.1.mca"], (false, false));
        assert_eq!(marcas["carpeta-vacia"], (false, false));
        assert_eq!(a.visibles().count(), 3);
        a.nodos[0].abierto = true;
        assert_eq!(a.visibles().count(), 100_004);
        a.nodos[1].abierto = false;
        assert_eq!(a.visibles().count(), 4);
    }
    #[test]
    fn zip_arbol_exporta_ambos_formatos_y_conserva_destinos_existentes() {
        use std::io::Read;
        let (root, store, i) = fixture();
        let mut a = arbol(&store, &i.id).unwrap();
        a.seleccionar(Seleccion::Compartir);
        let target = root.join("out/icaro.zip");
        let mut avances = vec![];
        archivo(&store, &i, &a, Formato::Icaro, &target, |n, t| {
            avances.push((n, t))
        })
        .unwrap();
        assert_eq!(avances.last(), Some(&(13, 13)));
        let mut archive = zip::ZipArchive::new(fs::File::open(&target).unwrap()).unwrap();
        assert!(archive.by_name("instance.json").is_ok());
        assert!(archive.by_name("minecraft/mods/a.jar").is_ok());
        assert!(archive.by_name("minecraft/saves/Mundo/level.dat").is_err());
        drop(archive);
        let saved = fs::read(&target).unwrap();
        assert!(archivo(&store, &i, &a, Formato::Icaro, &target, |_, _| {}).is_err());
        assert_eq!(fs::read(&target).unwrap(), saved);
        let prism = root.join("out/prism.zip");
        archivo(&store, &i, &a, Formato::Prism, &prism, |_, _| {}).unwrap();
        let mut archive = zip::ZipArchive::new(fs::File::open(&prism).unwrap()).unwrap();
        let mut index = String::new();
        archive
            .by_name("mmc-pack.json")
            .unwrap()
            .read_to_string(&mut index)
            .unwrap();
        let index: serde_json::Value = serde_json::from_str(&index).unwrap();
        assert_eq!(index["components"][0]["version"], "1.21.1");
        assert!(archive.by_name("instance.cfg").is_ok());
        assert!(archive.by_name(".minecraft/options.txt").is_ok());
        drop(archive);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn cambios_de_archivos_rutas_inseguras_y_loader_sin_pin_no_publican_zip() {
        let (root, store, mut i) = fixture();
        let a = arbol(&store, &i.id).unwrap();
        let target = root.join("out/instancia.zip");
        i.loader = "fabric".into();
        assert!(archivo(&store, &i, &a, Formato::Prism, &target, |_, _| {})
            .unwrap_err()
            .contains("versión real"));
        assert!(!target.exists());
        crate::versiones_instancia::guardar_loader(&store, &i, "0.16.10").unwrap();
        archivo(&store, &i, &a, Formato::Prism, &target, |_, _| {}).unwrap();
        let mut archive = zip::ZipArchive::new(fs::File::open(&target).unwrap()).unwrap();
        let mut body = String::new();
        use std::io::Read;
        archive
            .by_name("mmc-pack.json")
            .unwrap()
            .read_to_string(&mut body)
            .unwrap();
        assert!(body.contains("0.16.10"));
        drop(archive);
        let unsafe_target = root.join("out/unsafe.zip");
        let mut b = a.clone();
        b.nodos[0].ruta = "../outside".into();
        b.nodos[0].seleccionado = true;
        assert!(archivo(&store, &i, &b, Formato::Icaro, &unsafe_target, |_, _| {}).is_err());
        assert!(!unsafe_target.exists());
        fs::write(store.juego(&i.id).unwrap().join("mods/a.jar"), b"cambio").unwrap();
        let changed = root.join("out/changed.zip");
        assert!(archivo(&store, &i, &a, Formato::Icaro, &changed, |_, _| {}).is_err());
        assert!(!changed.exists());
        assert!(!fs::read_dir(root.join("out")).unwrap().any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn exporta_zip_real_solo_con_el_contenido_elegido_y_sin_sobrescribir() {
        let mut random = [0u8; 8];
        getrandom::fill(&mut random).unwrap();
        let root = std::env::temp_dir().join(format!(
            "icaro-export-test-{:x}",
            u64::from_le_bytes(random)
        ));
        let store = Almacen::nuevo(root.join("instances"));
        let i = store
            .crear_con_loader("Prueba", "1.21.1", "fabric", 0)
            .unwrap();
        let game = store.juego(&i.id).unwrap();
        for dir in ["mods", "config", "saves/Mundo", "screenshots", "logs"] {
            fs::create_dir_all(game.join(dir)).unwrap();
        }
        for (name, bytes) in [
            ("mods/a.jar", b"jar".as_slice()),
            ("mods/.icaro-mods.json", b"[]"),
            ("config/a.json", b"{}"),
            ("saves/Mundo/level.dat", b"mundo"),
            ("screenshots/a.png", b"imagen"),
            ("logs/latest.log", b"log"),
            ("options.txt", b"opciones"),
        ] {
            fs::write(game.join(name), bytes).unwrap();
        }
        let target = root.join("out");
        fs::create_dir(&target).unwrap();
        let measures = medir(&store, &i.id).unwrap();
        assert_eq!(measures.bytes, [7, 5, 8, 6, 3]);
        assert_eq!(measures.mundos, 1);
        assert_eq!(measures.capturas, 1);
        let path = zip(&store, &i, [true, true, false, false], &target).unwrap();
        let archive = zip::ZipArchive::new(fs::File::open(&path).unwrap()).unwrap();
        let names: Vec<_> = archive
            .file_names()
            .map(|n| n.unwrap().into_owned())
            .collect();
        assert!(
            names.contains(&"instance.json".to_owned())
                && names.contains(&"minecraft/mods/a.jar".to_owned())
                && names.contains(&"minecraft/mods/.icaro-mods.json".to_owned())
                && names.contains(&"minecraft/saves/Mundo/level.dat".to_owned())
                && names.contains(&"minecraft/options.txt".to_owned())
        );
        assert!(!names
            .iter()
            .any(|n| n.contains("screenshots") || n.contains("logs")));
        assert!(zip(&store, &i, [true; 4], &game).is_err());
        assert!(path.exists());
        drop(archive);
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        fs::remove_dir_all(root).unwrap();
    }
}
