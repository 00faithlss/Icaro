// SPDX-License-Identifier: GPL-3.0-only
//! Medición y mantenimiento acotados a los datos del launcher.
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, Default)]
pub struct Informe {
    pub categorias: Vec<(String, u64)>,
    pub total: u64,
    pub papeleras: Vec<PathBuf>,
}
pub fn regular(path: &Path) -> Result<fs::Metadata, String> {
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return Err("No se recorren enlaces ni uniones de carpetas.".into());
        }
    }
    if meta.file_type().is_symlink() {
        return Err("No se recorren enlaces.".into());
    }
    Ok(meta)
}
fn recorrer(
    path: &Path,
    depth: usize,
    count: &mut usize,
    files: &mut Vec<(PathBuf, u64)>,
    trash: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if depth > 64 || *count > 300_000 {
        return Err("El inventario supera el límite de seguridad.".into());
    }
    let meta = regular(path)?;
    *count += 1;
    if meta.is_file() {
        files.push((path.to_owned(), meta.len()));
    } else if meta.is_dir() {
        if matches!(
            path.file_name().and_then(|n| n.to_str()),
            Some(".papelera" | ".icaro-mods-trash")
        ) {
            trash.push(path.to_owned());
        }
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            recorrer(
                &entry.map_err(|e| e.to_string())?.path(),
                depth + 1,
                count,
                files,
                trash,
            )?;
        }
    }
    Ok(())
}
pub fn medir(root: &Path) -> Result<Informe, String> {
    let mut out = Informe::default();
    if !root.exists() {
        return Ok(out);
    }
    regular(root)?;
    let mut count = 0;
    let mut categories = std::collections::BTreeMap::<String, u64>::new();
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let label = match name {
            "instances" => "Instancias",
            "assets" => "Assets",
            "libraries" => "Bibliotecas",
            "meta" => "Metadatos y clientes",
            "java" => "Java",
            "cache" => "Caché",
            "logs" => "Registros del launcher",
            _ => "Otros",
        };
        let mut files = vec![];
        recorrer(&path, 0, &mut count, &mut files, &mut out.papeleras)?;
        let size = files.iter().map(|(_, s)| s).sum::<u64>();
        out.total += size;
        *categories.entry(label.into()).or_default() += size;
    }
    out.categorias = categories.into_iter().collect();
    Ok(out)
}
pub fn limpiar_cache(root: &Path) -> Result<u64, String> {
    regular(root)?;
    let cache = root.join("cache");
    if !cache.exists() {
        return Ok(0);
    }
    let mut files = vec![];
    recorrer(&cache, 0, &mut 0, &mut files, &mut vec![])?;
    let mut removed = 0;
    for (path, size) in files {
        regular(&path)?;
        fs::remove_file(path).map_err(|e| e.to_string())?;
        removed += size;
    }
    Ok(removed)
}
pub fn trasladar_papeleras(root: &Path) -> Result<usize, String> {
    let report = medir(root)?;
    let mut count = 0;
    for path in report.papeleras.iter().rev() {
        regular(path)?;
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let child = entry.map_err(|e| e.to_string())?.path();
            regular(&child)?;
            crate::papelera::enviar(&child)?;
            count += 1;
        }
        fs::remove_dir(path).map_err(|e| e.to_string())?;
    }
    Ok(count)
}
pub fn escribir_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let parent = path.parent().ok_or("Ruta inválida")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    regular(parent)?;
    if path.exists() {
        regular(path)?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    if fs::read(path).ok().as_deref() == Some(bytes.as_slice()) {
        return Ok(());
    }
    let mut nonce = [0u8; 8];
    getrandom::fill(&mut nonce).map_err(|e| e.to_string())?;
    let temp = parent.join(format!(".icaro-state-{}.tmp", u64::from_le_bytes(nonce)));
    let result = (|| {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| e.to_string())?;
        f.write_all(&bytes)
            .and_then(|_| f.sync_all())
            .map_err(|e| e.to_string())?;
        drop(f);
        fs::rename(&temp, path).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}
pub fn registrar(root: &Path, message: &str) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    regular(root)?;
    let logs = root.join("logs");
    fs::create_dir_all(&logs).map_err(|e| e.to_string())?;
    regular(&logs)?;
    let path = logs.join("launcher.log");
    if path.exists() && regular(&path)?.len() > 1024 * 1024 {
        let oldest = logs.join("launcher.3.log");
        if oldest.exists() {
            regular(&oldest)?;
            fs::remove_file(oldest).map_err(|e| e.to_string())?;
        }
        for n in (1..3).rev() {
            let src = logs.join(format!("launcher.{n}.log"));
            if src.exists() {
                regular(&src)?;
                fs::rename(src, logs.join(format!("launcher.{}.log", n + 1)))
                    .map_err(|e| e.to_string())?;
            }
        }
        fs::rename(&path, logs.join("launcher.1.log")).map_err(|e| e.to_string())?;
    }
    if path.exists() {
        regular(&path)?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    writeln!(
        file,
        "{time} {}",
        message
            .chars()
            .take(2000)
            .collect::<String>()
            .replace(['\r', '\n'], " ")
    )
    .map_err(|e| e.to_string())
}

/// Traslada ajustes existentes a la instancia antes de retirar la copia global.
pub fn migrar_configuraciones(root: &Path) -> Result<(), String> {
    let store = crate::instancias::Almacen::nuevo(root.join("instances"));
    let instances = store.cargar().map_err(|e| e.to_string())?.instancias;
    let path = root.join("estado.json");
    if !path.exists() {
        return Ok(());
    }
    regular(&path)?;
    if fs::metadata(&path).map_err(|e| e.to_string())?.len() > 8 * 1024 * 1024 {
        return Err("Estado demasiado grande".into());
    }
    let mut state: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    if let Some(configs) = state.get("configs").and_then(|v| v.as_array()) {
        let mut unassigned = Vec::new();
        for (n, config) in configs.iter().enumerate() {
            let id = state
                .get("ids")
                .and_then(|v| v.as_array())
                .and_then(|ids| ids.get(n))
                .and_then(|v| v.as_str())
                .or_else(|| instances.get(n).map(|i| i.id.as_str()));
            if let Some(id) = id.filter(|id| instances.iter().any(|i| i.id == *id)) {
                if store
                    .configuracion(id)
                    .map_err(|e| e.to_string())?
                    .is_none()
                {
                    store
                        .guardar_configuracion(id, config)
                        .map_err(|e| e.to_string())?;
                }
            } else {
                unassigned.push(serde_json::json!({"id":id,"configuracion":config}));
            }
        }
        if !unassigned.is_empty() {
            state["configuraciones_legacy"] = unassigned.into();
        }
        state
            .as_object_mut()
            .ok_or("Estado inválido")?
            .remove("configs");
        state["schema"] = 2.into();
        escribir_json(&path, &state)?;
    }
    Ok(())
}

pub fn recuperar_operaciones(root: &Path) -> Result<usize, String> {
    let store = crate::instancias::Almacen::nuevo(root.join("instances"));
    let instances = store.cargar().map_err(|e| e.to_string())?.instancias;
    let mut count = 0;
    for i in instances {
        count += crate::mods::recuperar_operaciones(&store, &i.id).map_err(|e| e.to_string())?;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn temp() -> PathBuf {
        let mut nonce = [0u8; 8];
        getrandom::fill(&mut nonce).unwrap();
        let p = std::env::temp_dir().join(format!("icaro-storage-{}", u64::from_le_bytes(nonce)));
        fs::create_dir(&p).unwrap();
        p
    }
    #[test]
    fn limpieza_cache_y_rotacion_protegen_datos_personales() {
        let root = temp();
        for (folder, bytes) in [
            ("cache", b"cache".as_slice()),
            ("instances/Mundo/minecraft/saves", b"world"),
            ("java", b"runtime"),
            ("assets", b"asset"),
        ] {
            fs::create_dir_all(root.join(folder)).unwrap();
            fs::write(root.join(folder).join("archivo"), bytes).unwrap();
        }
        fs::write(root.join("estado.json"), b"{}").unwrap();
        assert_eq!(medir(&root).unwrap().total, 24);
        assert_eq!(limpiar_cache(&root).unwrap(), 5);
        assert_eq!(
            fs::read(root.join("instances/Mundo/minecraft/saves/archivo")).unwrap(),
            b"world"
        );
        fs::create_dir(root.join("logs")).unwrap();
        fs::write(root.join("logs/launcher.log"), vec![b'x'; 1024 * 1024 + 1]).unwrap();
        registrar(&root, "inicio").unwrap();
        assert!(root.join("logs/launcher.1.log").exists());
        assert!(fs::metadata(root.join("logs/launcher.log")).unwrap().len() < 100);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn migracion_identidad_y_campos_desconocidos_tiene_una_fuente() {
        let root = temp();
        let store = crate::instancias::Almacen::nuevo(root.join("instances"));
        let a = store.crear("Primera", "1.21.1", 0).unwrap();
        let b = store.crear("Segunda", "1.21.1", 0).unwrap();
        let config = serde_json::json!({"java":"C:/Java/java.exe","memoria_max":4096,"campo_futuro":{"x":1}});
        escribir_json(&root.join("estado.json"),&serde_json::json!({"ids":[b.id,a.id],"configs":[config,{"memoria_max":2048}],"global_futuro":true})).unwrap();
        migrar_configuraciones(&root).unwrap();
        assert_eq!(store.configuracion(&b.id).unwrap().unwrap(), config);
        store
            .guardar_configuracion(&b.id, &serde_json::json!({"memoria_max":8192}))
            .unwrap();
        let mut renamed = b.clone();
        renamed.nombre = "Renombrada".into();
        store.guardar(&renamed).unwrap();
        assert_eq!(
            store.configuracion(&b.id).unwrap().unwrap()["campo_futuro"],
            config["campo_futuro"]
        );
        let state: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("estado.json")).unwrap()).unwrap();
        assert!(state.get("configs").is_none());
        assert_eq!(state["global_futuro"], true);
        fs::remove_dir_all(root).unwrap();
    }
}
