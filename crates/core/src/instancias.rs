// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Instancias locales: identidad estable, carpeta aislada y metadatos persistentes.

use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_METADATA: u64 = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Instancia {
    pub id: String,
    pub nombre: String,
    pub minecraft: String,
    #[serde(default = "vanilla")]
    pub loader: String,
    pub portada: u8,
    pub creada: u64,
}
fn vanilla() -> String {
    "vanilla".into()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No se pudo leer o guardar la instancia en disco.")]
    Disco,
    #[error("Los datos de la instancia no son válidos.")]
    Datos,
    #[error("El nombre debe contener entre 1 y 80 caracteres, sin caracteres de control.")]
    Nombre,
    #[error("Ya existe una instancia con ese nombre.")]
    Repetida,
}

#[derive(Debug, Clone)]
pub struct Almacen {
    raiz: PathBuf,
}
pub struct Informe {
    pub instancias: Vec<Instancia>,
    pub omitidas: usize,
}

fn id_valido(id: &str) -> bool {
    id.len() == 32
        && id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn id_nuevo() -> Result<String, Error> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| Error::Disco)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
fn validar(i: &Instancia) -> Result<(), Error> {
    if i.nombre.trim().is_empty()
        || i.nombre.chars().count() > 80
        || i.nombre.chars().any(char::is_control)
    {
        return Err(Error::Nombre);
    }
    if !matches!(
        i.loader.as_str(),
        "vanilla" | "fabric" | "forge" | "neoforge" | "quilt"
    ) || !id_valido(&i.id)
        || i.portada > 3
        || i.minecraft.is_empty()
        || i.minecraft.len() > 128
        || !i
            .minecraft
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
    {
        return Err(Error::Datos);
    }
    Ok(())
}
fn enlace(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

impl Almacen {
    pub fn nuevo(raiz: impl Into<PathBuf>) -> Self {
        Self { raiz: raiz.into() }
    }
    pub fn carpeta(&self, id: &str) -> Result<PathBuf, Error> {
        if !id_valido(id) {
            return Err(Error::Datos);
        }
        let path = self.raiz.join(id);
        let metadata = fs::symlink_metadata(&path).map_err(|_| Error::Disco)?;
        if !metadata.is_dir() || enlace(&metadata) {
            return Err(Error::Datos);
        }
        Ok(path)
    }
    pub fn juego(&self, id: &str) -> Result<PathBuf, Error> {
        let path = self.carpeta(id)?.join("minecraft");
        let metadata = fs::symlink_metadata(&path).map_err(|_| Error::Disco)?;
        if !metadata.is_dir() || enlace(&metadata) {
            return Err(Error::Datos);
        }
        Ok(path)
    }
    fn leer(&self, id: &str) -> Result<Instancia, Error> {
        let path = self.carpeta(id)?.join("instance.json");
        let metadata = fs::symlink_metadata(&path).map_err(|_| Error::Disco)?;
        if !metadata.is_file() || enlace(&metadata) || metadata.len() > MAX_METADATA {
            return Err(Error::Datos);
        }
        let mut bytes = Vec::new();
        fs::File::open(path)
            .map_err(|_| Error::Disco)?
            .take(MAX_METADATA + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| Error::Disco)?;
        if bytes.len() > MAX_METADATA as usize {
            return Err(Error::Datos);
        }
        let i: Instancia = serde_json::from_slice(&bytes).map_err(|_| Error::Datos)?;
        validar(&i)?;
        if i.id != id {
            return Err(Error::Datos);
        }
        self.juego(id)?;
        Ok(i)
    }
    pub fn cargar(&self) -> Result<Informe, Error> {
        let entries = match fs::read_dir(&self.raiz) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Informe {
                    instancias: vec![],
                    omitidas: 0,
                })
            }
            Err(_) => return Err(Error::Disco),
        };
        let mut informe = Informe {
            instancias: vec![],
            omitidas: 0,
        };
        for entry in entries {
            let entry = entry.map_err(|_| Error::Disco)?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if !id_valido(&name) {
                continue;
            }
            match self.leer(&name) {
                Ok(i) => informe.instancias.push(i),
                Err(_) => informe.omitidas += 1,
            }
        }
        informe
            .instancias
            .sort_by(|a, b| a.creada.cmp(&b.creada).then_with(|| a.id.cmp(&b.id)));
        Ok(informe)
    }
    pub fn crear(&self, nombre: &str, minecraft: &str, portada: u8) -> Result<Instancia, Error> {
        self.crear_con_loader(nombre, minecraft, "vanilla", portada)
    }
    pub fn crear_con_loader(
        &self,
        nombre: &str,
        minecraft: &str,
        loader: &str,
        portada: u8,
    ) -> Result<Instancia, Error> {
        let i = Instancia {
            id: id_nuevo()?,
            nombre: nombre.trim().into(),
            minecraft: minecraft.into(),
            loader: loader.to_lowercase(),
            portada,
            creada: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| Error::Datos)?
                .as_nanos()
                .try_into()
                .map_err(|_| Error::Datos)?,
        };
        validar(&i)?;
        if self
            .cargar()?
            .instancias
            .iter()
            .any(|r| r.nombre.to_lowercase() == i.nombre.to_lowercase())
        {
            return Err(Error::Repetida);
        }
        fs::create_dir_all(&self.raiz).map_err(|_| Error::Disco)?;
        let path = self.raiz.join(&i.id);
        fs::create_dir(&path).map_err(|_| Error::Disco)?;
        fs::create_dir(path.join("minecraft")).map_err(|_| Error::Disco)?;
        self.guardar(&i)?;
        Ok(i)
    }
    pub fn guardar(&self, i: &Instancia) -> Result<(), Error> {
        validar(i)?;
        if self
            .cargar()?
            .instancias
            .iter()
            .any(|r| r.id != i.id && r.nombre.to_lowercase() == i.nombre.to_lowercase())
        {
            return Err(Error::Repetida);
        }
        let path = self.carpeta(&i.id)?;
        let destino = path.join("instance.json");
        if let Ok(m) = fs::symlink_metadata(&destino) {
            if !m.is_file() || enlace(&m) {
                return Err(Error::Datos);
            }
        }
        let temporal = path.join(format!("{}.tmp", id_nuevo()?));
        let bytes = serde_json::to_vec_pretty(i).map_err(|_| Error::Datos)?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporal)
            .map_err(|_| Error::Disco)?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|_| Error::Disco)?;
        drop(file);
        fs::rename(&temporal, &destino).map_err(|_| Error::Disco)?;
        Ok(())
    }

    fn papelera(&self) -> Result<PathBuf, Error> {
        let root = fs::canonicalize(&self.raiz).map_err(|_| Error::Disco)?;
        let path = root.join(".papelera");
        match fs::create_dir(&path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_) => return Err(Error::Disco),
        }
        let metadata = fs::symlink_metadata(&path).map_err(|_| Error::Disco)?;
        if !metadata.is_dir() || enlace(&metadata) {
            return Err(Error::Datos);
        }
        let resolved = fs::canonicalize(&path).map_err(|_| Error::Disco)?;
        if resolved.parent() != Some(root.as_path()) {
            return Err(Error::Datos);
        }
        Ok(resolved)
    }

    /// Retira una instancia activa sin borrar sus mundos; el destino permanece dentro del almacén.
    pub fn eliminar(&self, id: &str) -> Result<(), Error> {
        self.leer(id)?;
        let root = fs::canonicalize(&self.raiz).map_err(|_| Error::Disco)?;
        let source = fs::canonicalize(self.carpeta(id)?).map_err(|_| Error::Disco)?;
        if source.parent() != Some(root.as_path())
            || source.file_name() != Some(std::ffi::OsStr::new(id))
        {
            return Err(Error::Datos);
        }
        let trash = self.papelera()?;
        let target = trash.join(id);
        if fs::symlink_metadata(&target).is_ok() {
            return Err(Error::Repetida);
        }
        fs::rename(source, target).map_err(|_| Error::Disco)
    }

    pub fn restaurar(&self, id: &str) -> Result<Instancia, Error> {
        if !id_valido(id) {
            return Err(Error::Datos);
        }
        let root = fs::canonicalize(&self.raiz).map_err(|_| Error::Disco)?;
        let trash = self.papelera()?;
        let record = Almacen::nuevo(&trash).leer(id)?;
        if self
            .cargar()?
            .instancias
            .iter()
            .any(|i| i.nombre.to_lowercase() == record.nombre.to_lowercase())
        {
            return Err(Error::Repetida);
        }
        let source = fs::canonicalize(trash.join(id)).map_err(|_| Error::Disco)?;
        if source.parent() != Some(trash.as_path()) {
            return Err(Error::Datos);
        }
        let target = root.join(id);
        if fs::symlink_metadata(&target).is_ok() {
            return Err(Error::Repetida);
        }
        fs::rename(source, target).map_err(|_| Error::Disco)?;
        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Prueba {
        raiz: PathBuf,
    }
    impl Prueba {
        fn nueva() -> Self {
            Self {
                raiz: std::env::temp_dir()
                    .join(format!("icaro-instancias-test-{}", id_nuevo().unwrap())),
            }
        }
        fn almacen(&self) -> Almacen {
            Almacen::nuevo(&self.raiz)
        }
    }
    impl Drop for Prueba {
        fn drop(&mut self) {
            if self.raiz.parent() == Some(std::env::temp_dir().as_path()) {
                let _ = fs::remove_dir_all(&self.raiz);
            }
        }
    }
    #[test]
    fn loader_persists_and_older_metadata_defaults_to_vanilla() {
        let p = Prueba::nueva();
        let store = p.almacen();
        let fabric = store
            .crear_con_loader("Fabric", "1.21.1", "fabric", 0)
            .unwrap();
        let vanilla = store.crear("Vanilla", "1.21.1", 1).unwrap();
        let path = store.carpeta(&vanilla.id).unwrap().join("instance.json");
        let mut value = serde_json::to_value(&vanilla).unwrap();
        value.as_object_mut().unwrap().remove("loader");
        fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
        let loaded = store.cargar().unwrap().instancias;
        assert_eq!(
            loaded.iter().find(|i| i.id == fabric.id).unwrap().loader,
            "fabric"
        );
        assert_eq!(
            loaded.iter().find(|i| i.id == vanilla.id).unwrap().loader,
            "vanilla"
        );
        assert!(store
            .crear_con_loader("Bad", "1.21.1", "../outside", 0)
            .is_err());
    }
    #[test]
    fn create_reopen_rename_and_game_folders_are_independent() {
        let p = Prueba::nueva();
        let store = p.almacen();
        let mut a = store.crear("Mi mundo", "1.21.4", 2).unwrap();
        let b = store.crear("Otro", "1.20.1", 0).unwrap();
        fs::write(
            store.juego(&a.id).unwrap().join("options.txt"),
            b"my options",
        )
        .unwrap();
        assert!(!store.juego(&b.id).unwrap().join("options.txt").exists());
        assert_eq!(p.almacen().cargar().unwrap().instancias, vec![a.clone(), b]);
        assert!(matches!(
            store.crear("MI MUNDO", "1.21.4", 0),
            Err(Error::Repetida)
        ));
        let game = store.juego(&a.id).unwrap();
        a.nombre = "Nombre nuevo".into();
        store.guardar(&a).unwrap();
        assert_eq!(store.juego(&a.id).unwrap(), game);
        assert_eq!(store.cargar().unwrap().instancias[0], a);
    }
    #[test]
    fn invalid_paths_and_corrupt_metadata_do_not_hide_valid_instances() {
        let p = Prueba::nueva();
        let store = p.almacen();
        assert!(matches!(store.crear("", "1.21.4", 0), Err(Error::Nombre)));
        assert!(store.crear("Mundo", "../../outside", 0).is_err());
        assert!(!p.raiz.exists());
        let good = store.crear("Mundo", "1.21.4", 0).unwrap();
        let bad = store.crear("Otra", "1.21.4", 0).unwrap();
        fs::write(
            store.carpeta(&bad.id).unwrap().join("instance.json"),
            b"broken",
        )
        .unwrap();
        let report = store.cargar().unwrap();
        assert_eq!(report.instancias, vec![good]);
        assert_eq!(report.omitidas, 1);
        assert!(store.carpeta("../outside").is_err());
    }

    #[test]
    fn delete_and_restore_keep_files_and_do_not_touch_other_instances() {
        let p = Prueba::nueva();
        let store = p.almacen();
        let a = store.crear("Mundo", "1.21.4", 0).unwrap();
        let b = store.crear("Otra", "1.20.1", 1).unwrap();
        fs::write(
            store.juego(&a.id).unwrap().join("options.txt"),
            b"saved options",
        )
        .unwrap();
        assert!(store.eliminar("../outside").is_err());
        store.eliminar(&a.id).unwrap();
        assert_eq!(store.cargar().unwrap().instancias, vec![b.clone()]);
        assert!(!p.raiz.join(&a.id).exists());
        assert_eq!(
            fs::read(
                p.raiz
                    .join(".papelera")
                    .join(&a.id)
                    .join("minecraft/options.txt")
            )
            .unwrap(),
            b"saved options"
        );
        assert_eq!(store.restaurar(&a.id).unwrap(), a);
        assert_eq!(store.cargar().unwrap().instancias, vec![a.clone(), b]);
        assert_eq!(
            fs::read(store.juego(&a.id).unwrap().join("options.txt")).unwrap(),
            b"saved options"
        );
        store.eliminar(&a.id).unwrap();
        let replacement = store.crear("MUNDO", "1.21.4", 2).unwrap();
        assert!(matches!(store.restaurar(&a.id), Err(Error::Repetida)));
        assert!(store.juego(&replacement.id).unwrap().exists());
        assert!(p.raiz.join(".papelera").join(&a.id).exists());
    }

    #[test]
    fn invalid_trash_does_not_remove_the_active_instance() {
        let p = Prueba::nueva();
        let store = p.almacen();
        let a = store.crear("Mundo", "1.21.4", 0).unwrap();
        fs::write(p.raiz.join(".papelera"), b"not a directory").unwrap();
        assert!(matches!(store.eliminar(&a.id), Err(Error::Datos)));
        assert_eq!(store.cargar().unwrap().instancias, vec![a]);
        assert!(store.restaurar("../outside").is_err());
    }
}
