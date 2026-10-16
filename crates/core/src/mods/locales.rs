// SPDX-License-Identifier: GPL-3.0-only
//! Importación atómica de JAR personales, sin registrarlos como mods del servidor.
use super::*;

/// Copia los JAR seleccionados; un fallo conserva íntegro el inventario anterior.
/// No descarga dependencias ni certifica la compatibilidad de un archivo local.
pub fn importar(store: &Almacen, id: &str, archivos: &[PathBuf]) -> Result<usize, Error> {
    if archivos.is_empty() || archivos.len() > 512 {
        return Err(Error::Datos);
    }
    let instancia = store
        .cargar()
        .map_err(|_| Error::Disco)?
        .instancias
        .into_iter()
        .find(|i| i.id == id)
        .ok_or(Error::Datos)?;
    if instancia.loader == "vanilla" {
        return Err(Error::Compatible(
            "mods locales: la instancia usa Vanilla".into(),
        ));
    }
    let game = store.juego(id).map_err(|_| Error::Disco)?;
    let _lock = guardia(&game)?;
    recuperar(&game)?;
    let dir = game.join("mods");
    let anterior = sincronizacion::fingerprint(&dir).map_err(Error::Papelera)?;
    let mut ocupados = anterior
        .iter()
        .map(|(nombre, _)| {
            nombre
                .strip_suffix(".disabled")
                .unwrap_or(nombre)
                .to_lowercase()
        })
        .collect::<BTreeSet<_>>();
    let mut aleatorio = [0; 16];
    getrandom::fill(&mut aleatorio).map_err(|_| Error::Disco)?;
    let stage = game.join(format!(".icaro-mods-next-{}", hash(&aleatorio)));
    fs::create_dir(&stage).map_err(|_| Error::Disco)?;
    let resultado = (|| {
        if dir.exists() {
            copiar(&dir, &stage, 0, &mut 0, &mut 0)?;
        }
        let mut total = 0u64;
        for fuente in archivos {
            let nombre = fuente
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or(Error::Datos)?;
            if !nombre_valido(nombre) || !nombre.ends_with(".jar") {
                return Err(Error::Datos);
            }
            if !ocupados.insert(nombre.to_lowercase()) {
                return Err(Error::Archivo(nombre.into()));
            }
            let longitud = regular(fuente, false)?.len();
            total = total.saturating_add(longitud);
            if longitud == 0 || longitud > MAX_FILE || total > 2 * 1024 * 1024 * 1024 {
                return Err(Error::Datos);
            }
            let destino = stage.join(nombre);
            let mut entrada = fs::File::open(fuente).map_err(|_| Error::Disco)?;
            let mut salida = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&destino)
                .map_err(|_| Error::Disco)?;
            let copiados = std::io::copy(
                &mut Read::by_ref(&mut entrada).take(MAX_FILE + 1),
                &mut salida,
            )
            .map_err(|_| Error::Disco)?;
            salida.sync_all().map_err(|_| Error::Disco)?;
            if copiados != longitud {
                return Err(Error::Datos);
            }
            let jar = zip::ZipArchive::new(fs::File::open(&destino).map_err(|_| Error::Disco)?)
                .map_err(|_| Error::Datos)?;
            if jar.is_empty() {
                return Err(Error::Datos);
            }
        }
        if sincronizacion::fingerprint(&dir).map_err(Error::Papelera)? != anterior {
            return Err(Error::Ocupada);
        }
        let backup = game.join(".icaro-mods-backup");
        if backup.exists() {
            borrar_preparacion(&game, &backup)?;
        }
        if dir.exists() {
            fs::rename(&dir, &backup).map_err(|_| Error::Disco)?;
        }
        if fs::rename(&stage, &dir).is_err() {
            if backup.exists() {
                let _ = fs::rename(&backup, &dir);
            }
            return Err(Error::Disco);
        }
        if backup.exists() {
            let _ = borrar_preparacion(&game, &backup);
        }
        Ok(archivos.len())
    })();
    if stage.exists() {
        let _ = borrar_preparacion(&game, &stage);
    }
    resultado
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn importa_varios_jar_sin_gestionarlos_y_revierte_un_lote_invalido() {
        let prueba = super::super::tests::Prueba::nueva();
        let (store, i) = prueba.instancia();
        let origen = prueba.0.join("origen");
        fs::create_dir(&origen).unwrap();
        for nombre in ["personal.jar", "otro.jar"] {
            let mut zip = zip::ZipWriter::new(fs::File::create(origen.join(nombre)).unwrap());
            zip.start_file("fabric.mod.json", zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(br#"{"id":"personal","version":"1"}"#)
                .unwrap();
            zip.finish().unwrap();
        }
        let seleccion = [origen.join("personal.jar"), origen.join("otro.jar")];
        assert_eq!(importar(&store, &i.id, &seleccion).unwrap(), 2);
        let mods = inventario(&store, &i.id).unwrap();
        assert_eq!(mods.len(), 2);
        assert!(mods.iter().all(|m| m.project_id.is_empty()));
        let dir = store.juego(&i.id).unwrap().join("mods");
        fs::rename(dir.join("personal.jar"), dir.join("personal.jar.disabled")).unwrap();
        let anterior = sincronizacion::huella(&store, &i.id).unwrap();
        assert!(importar(&store, &i.id, &seleccion).is_err());
        fs::copy(&seleccion[0], origen.join("nuevo.jar")).unwrap();
        fs::write(origen.join("invalido.jar"), b"no es un JAR").unwrap();
        assert!(importar(
            &store,
            &i.id,
            &[origen.join("nuevo.jar"), origen.join("invalido.jar")]
        )
        .is_err());
        assert_eq!(sincronizacion::huella(&store, &i.id).unwrap(), anterior);
        assert!(!store.juego(&i.id).unwrap().join("mods/nuevo.jar").exists());
    }
}
