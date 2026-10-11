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
