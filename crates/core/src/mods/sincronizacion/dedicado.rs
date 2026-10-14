// SPDX-License-Identifier: GPL-3.0-only
//! Origen de solo lectura: mods de un servidor dedicado, sin Almacen ni instancia.
use super::*;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct ServidorDedicado {
    pub carpeta: PathBuf,
    pub minecraft: String,
    pub loader: String,
    pub loader_version: String,
    pub excluir: BTreeSet<String>,
}
impl ServidorDedicado {
    pub fn validar(&self) -> Result<(), String> {
        if self
            .excluir
            .iter()
            .any(|n| !nombre_valido(n) || !n.to_ascii_lowercase().ends_with(".jar"))
        {
            return Err("Las exclusiones deben ser nombres de JAR, sin rutas.".into());
        }
        if !self.carpeta.is_absolute() {
            return Err("La carpeta del servidor debe ser absoluta.".into());
        }
        self.indice(vec![]).validar()?;
        regular(&self.carpeta, true).map_err(err)?;
        regular(&self.carpeta.join("mods"), true)
            .map_err(|_| "El servidor debe tener una carpeta mods legible.".to_string())?;
        Ok(())
    }
    fn indice(&self, mods: Vec<Entrada>) -> Indice {
        Indice {
            esquema: 1,
            minecraft: self.minecraft.clone(),
            loader: self.loader.clone(),
            loader_version: Some(self.loader_version.clone()),
            mods,
            archivos: vec![],
            avisos: vec![],
        }
    }
    pub fn huella(&self) -> Result<Vec<(String, String)>, String> {
        self.validar()?;
        let archivos = fingerprint(&self.carpeta.join("mods"))?;
        if archivos
            .iter()
            .any(|(n, _)| n.to_ascii_lowercase().ends_with(".jar") && n.contains(['/', '\\']))
        {
            return Err("Hay JAR en subcarpetas de mods. Coloca los mods activos directamente en mods para publicarlos.".into());
        }
        Ok(archivos)
    }
}

trait Catalogo {
    async fn hashes(&self, hashes: &[String]) -> Result<BTreeMap<String, Version>, String>;
    async fn cliente(&self, version: &Version) -> Result<bool, String>;
    async fn version(&self, id: &str) -> Result<Version, String>;
    async fn versiones(
        &self,
        id: &str,
        minecraft: &str,
        loader: &str,
    ) -> Result<Vec<Version>, String>;
}
impl Catalogo for Catalog {
    async fn hashes(&self, hashes: &[String]) -> Result<BTreeMap<String, Version>, String> {
        self.versions_from_hashes(hashes).await.map_err(err)
    }
    async fn cliente(&self, version: &Version) -> Result<bool, String> {
        let project = self.project(&version.project_id).await.map_err(err)?;
        match project.client_side.as_str() {
            "unsupported" => Ok(false),
            "required" | "optional" => Ok(true),
            _ => Err(format!(
                "Modrinth no confirma si {} puede ejecutarse en el cliente.",
                version.project_id
            )),
        }
    }
    async fn version(&self, id: &str) -> Result<Version, String> {
        self.version(id).await.map_err(err)
    }
    async fn versiones(
        &self,
        id: &str,
        minecraft: &str,
        loader: &str,
    ) -> Result<Vec<Version>, String> {
        self.versions(id, Some(minecraft), Some(loader))
            .await
            .map_err(err)
    }
}

// Un Fabric exclusivo de servidor puede ser privado y no estar en Modrinth.
// Solo se excluye si su propio descriptor lo declara explícitamente.
fn descriptor_fabric(
    path: &Path,
    avisos: &mut Vec<String>,
) -> Result<Option<serde_json::Value>, String> {
    let nombre = path.file_name().unwrap_or_default().to_string_lossy();
    let aviso = |detalle: &str, avisos: &mut Vec<String>| {
        let mensaje = format!("{nombre}: {detalle}");
        if !avisos.contains(&mensaje) {
            avisos.push(mensaje);
        }
    };
    let file =
        fs::File::open(path).map_err(|e| format!("{nombre}: no se pudo leer el JAR: {e}"))?;
    let Ok(mut jar) = zip::ZipArchive::new(file) else {
        return Ok(None);
    };
    let Ok(mut descriptor) = jar.by_name("fabric.mod.json") else {
        return Ok(None);
    };
    if descriptor.size() > 128 * 1024 {
        aviso(
            "fabric.mod.json supera el límite; no se pudo confirmar su entorno o identidad.",
            avisos,
        );
        return Ok(None);
    }
    let mut bytes = vec![];
    if descriptor
        .by_ref()
        .take(128 * 1024 + 1)
        .read_to_end(&mut bytes)
        .is_err()
        || bytes.len() > 128 * 1024
    {
        aviso("fabric.mod.json no se pudo leer dentro del límite; revisa si el mod es necesario en el cliente.",avisos);
        return Ok(None);
    }
    let strict = serde_json::from_slice::<serde_json::Value>(&bytes);
    let data = match strict {
        Ok(data) => data,
        Err(_) => {
            // Solo la copia en memoria se normaliza. Los escapes válidos y el JAR quedan intactos.
            let normalizada: Vec<u8> = bytes
                .iter()
                .map(|b| if *b < 0x20 { b' ' } else { *b })
                .collect();
            match serde_json::from_slice::<serde_json::Value>(&normalizada) {
                Ok(data) => {
                    aviso("se toleraron caracteres de control sin escapar en fabric.mod.json; el JAR original se conserva.",avisos);
                    data
                }
                Err(_) => {
                    aviso("fabric.mod.json ilegible; no se pudo confirmar su entorno o identidad. Revisa si el mod es necesario en el cliente.",avisos);
                    return Ok(None);
                }
            }
        }
    };
    if !data.is_object() {
        aviso(
            "fabric.mod.json no contiene un objeto; no se pudo confirmar su entorno o identidad.",
            avisos,
        );
        return Ok(None);
    }
    Ok(Some(data))
}
fn exclusivo_servidor(path: &Path, loader: &str, avisos: &mut Vec<String>) -> Result<bool, String> {
    if !matches!(loader, "fabric" | "quilt") {
        return Ok(false);
    }
    Ok(descriptor_fabric(path, avisos)?
        .is_some_and(|data| data.get("environment").and_then(|v| v.as_str()) == Some("server")))
}

pub async fn publicar(catalog: &Catalog, servidor: &ServidorDedicado) -> Result<Indice, String> {
    publicar_con(catalog, servidor).await
}
async fn publicar_con(
    catalog: &impl Catalogo,
    servidor: &ServidorDedicado,
) -> Result<Indice, String> {
    let before = servidor.huella()?;
    let mut active = vec![];
    let mut avisos = vec![];
    for (name, sha) in &before {
        if name.to_ascii_lowercase().ends_with(".jar")
            && !servidor.excluir.contains(name)
            && !exclusivo_servidor(
                &servidor.carpeta.join("mods").join(name),
                &servidor.loader,
                &mut avisos,
            )?
        {
            active.push((name, sha));
        }
    }
    if active.len() > 512 {
        return Err("El servidor supera 512 mods.".into());
    }
    let hashes: Vec<_> = active.iter().map(|(_, h)| (*h).clone()).collect();
    let by_hash = if hashes.is_empty() {
        BTreeMap::new()
    } else {
        catalog.hashes(&hashes).await?
    };
    let mut encontrados = BTreeMap::new();
    let mut jars = BTreeMap::new();
    let mut propios_modrinth = BTreeSet::new();
    let mut archivos = vec![];
    let mut elegidos: BTreeMap<String, (Version, Entrada)> = BTreeMap::new();
    for (name, sha) in active {
        let Some(v) = by_hash.get(sha) else {
            archivos.push(archivo_propio(
                &servidor.carpeta.join("mods").join(name),
                name,
                sha,
                &mut avisos,
            )?);
            avisos.push(format!("{name}: no está en Modrinth; se descargará directamente del anfitrión. Revisa que sea necesario en el cliente."));
            continue;
        };
        if !v
            .files
            .iter()
            .any(|f| f.hashes.sha512.eq_ignore_ascii_case(sha))
        {
            return Err(format!(
                "Modrinth no confirma el archivo por SHA-512 de {name}."
            ));
        }
        if encontrados
            .insert(v.project_id.clone(), v.clone())
            .is_some()
        {
            return Err(format!(
                "Hay dos archivos del proyecto {} en mods.",
                v.project_id
            ));
        }
        if catalog.cliente(v).await? {
            jars.insert(v.project_id.clone(), (name.clone(), sha.clone()));
            if !compatible(v, &servidor.minecraft, &servidor.loader) {
                archivos.push(archivo_propio(
                    &servidor.carpeta.join("mods").join(name),
                    name,
                    sha,
                    &mut avisos,
                )?);
                avisos.push(format!("{name}: Modrinth reconoce el hash, pero no declara Minecraft {} y loader {}. Se publicará el JAR instalado como archivo propio; revisa su compatibilidad en el cliente.",servidor.minecraft,servidor.loader));
                propios_modrinth.insert(v.project_id.clone());
                continue;
            }
            elegidos.insert(
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
    }
    // Un proyecto que depende de una entrada propia tampoco puede resolverse estrictamente
    // en Modrinth. Conservar su JAR instalado y propagar esa elección evita otro bloqueo.
    loop {
        let afectados: Vec<_> = elegidos
            .iter()
            .filter(|(_, (v, _))| {
                v.dependencies
                    .iter()
                    .filter(|d| d.dependency_type == "required")
                    .any(|d| {
                        d.project_id
                            .as_ref()
                            .is_some_and(|id| propios_modrinth.contains(id))
                            || d.version_id.as_ref().is_some_and(|id| {
                                encontrados.values().any(|v| {
                                    &v.id == id && propios_modrinth.contains(&v.project_id)
                                })
                            })
                    })
            })
            .map(|(id, _)| id.clone())
            .collect();
        if afectados.is_empty() {
            break;
        }
        for id in afectados {
            let (name, sha) = &jars[&id];
            archivos.push(archivo_propio(
                &servidor.carpeta.join("mods").join(name),
                name,
                sha,
                &mut avisos,
            )?);
            avisos.push(format!("{name}: depende de un JAR con metadatos incompletos; se publicará el archivo instalado del anfitrión."));
            elegidos.remove(&id);
            propios_modrinth.insert(id);
        }
    }
    let mut revisados = BTreeSet::new();
    loop {
        let Some(v) = elegidos
            .values()
            .map(|(v, _)| v)
            .find(|v| !revisados.contains(&v.id))
            .cloned()
        else {
            break;
        };
        revisados.insert(v.id.clone());
        for dep in v
            .dependencies
            .iter()
            .filter(|d| d.dependency_type == "required")
        {
            let existente = dep
                .project_id
                .as_ref()
                .and_then(|p| encontrados.get(p))
                .or_else(|| {
                    dep.version_id
                        .as_ref()
                        .and_then(|id| encontrados.values().find(|v| &v.id == id))
                });
            let candidate = if let Some(v) = existente {
                v.clone()
            } else if let Some(id) = &dep.version_id {
                catalog.version(id).await?
            } else if let Some(id) = &dep.project_id {
                let mut versions = catalog
                    .versiones(id, &servidor.minecraft, &servidor.loader)
                    .await?;
                versions.sort_by(|a, b| {
                    b.date_published
                        .cmp(&a.date_published)
                        .then(a.id.cmp(&b.id))
                });
                versions
                    .into_iter()
                    .find(|v| {
                        v.project_id == *id
                            && v.version_type == "release"
                            && compatible(v, &servidor.minecraft, &servidor.loader)
                    })
                    .ok_or_else(|| {
                        format!("No hay una dependencia estable compatible para {id}.")
                    })?
            } else {
                return Err(
                    "Una dependencia obligatoria no tiene proyecto ni versión de Modrinth.".into(),
                );
            };
            if dep
                .version_id
                .as_ref()
                .is_some_and(|id| id != &candidate.id)
                || dep
                    .project_id
                    .as_ref()
                    .is_some_and(|id| id != &candidate.project_id)
            {
                return Err(format!("Conflicto en una dependencia de {}.", v.project_id));
            }
            if !catalog.cliente(&candidate).await? {
                continue;
            }
            if !compatible(&candidate, &servidor.minecraft, &servidor.loader) {
                return Err(format!("Una dependencia de {} no confirma Minecraft o loader y no hay un JAR instalado que pueda publicarse como propio.",v.project_id));
            }
            if let Some((prev, _)) = elegidos.get(&candidate.project_id) {
                if prev.id != candidate.id {
                    return Err(format!(
                        "Dependencias incompatibles para {}.",
                        candidate.project_id
                    ));
                }
                continue;
            }
            let file = candidate
                .files
                .iter()
                .find(|f| f.primary && f.filename.ends_with(".jar"))
                .or_else(|| {
                    candidate
                        .files
                        .iter()
                        .find(|f| f.filename.ends_with(".jar"))
                })
                .ok_or("La dependencia no tiene un JAR de cliente.")?;
            if !nombre_valido(&file.filename) || file.size > MAX_FILE {
                return Err("Archivo de dependencia inválido.".into());
            }
            let entry = Entrada {
                proyecto: candidate.project_id.clone(),
                version: candidate.id.clone(),
                sha512: file.hashes.sha512.clone(),
            };
            elegidos.insert(candidate.project_id.clone(), (candidate, entry));
            if elegidos.len() > 512 {
                return Err("Las dependencias superan 512 mods.".into());
            }
        }
    }
    let mut index = servidor.indice(elegidos.into_values().map(|(_, entry)| entry).collect());
    index.archivos = archivos;
    index.avisos = avisos;
    if !index.archivos.is_empty() {
        index.esquema = 2;
    }
    index.validar()?;
    if before != servidor.huella()? {
        return Err(
            "Los mods del servidor cambiaron durante la publicación. Se volverán a consultar."
                .into(),
        );
    }
    Ok(index)
}

fn archivo_propio(
    path: &Path,
    name: &str,
    sha: &str,
    avisos: &mut Vec<String>,
) -> Result<ArchivoPropio, String> {
    let bytes = regular(path, false).map_err(err)?.len();
    let _jar = zip::ZipArchive::new(fs::File::open(path).map_err(err)?)
        .map_err(|_| format!("{name} no es un JAR válido."))?;
    let mut id = hash(name.to_lowercase().as_bytes())[..24].to_string();
    let mut nombre = name.to_string();
    let mut version = "Archivo del anfitrión".to_string();
    if let Some(data) = descriptor_fabric(path, avisos)? {
        if let Some(value) = data.get("id").and_then(|v| v.as_str()).filter(|v| {
            !v.is_empty()
                && v.len() <= 64
                && v.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        }) {
            id = value.to_string();
        }
        if let Some(value) = data.get("name").and_then(|v| v.as_str()) {
            nombre = value.to_string();
        }
        if let Some(value) = data.get("version").and_then(|v| v.as_str()) {
            version = value.to_string();
        }
    }
    Ok(ArchivoPropio {
        id: format!("icaro-host-{id}"),
        nombre,
        version,
        archivo: name.into(),
        bytes,
        sha512: sha.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    fn jar_descriptor(descriptor: &[u8]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        writer
            .start_file("fabric.mod.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(descriptor).unwrap();
        writer.finish().unwrap().into_inner()
    }
    #[tokio::test]
    async fn descripciones_multilinea_y_descriptor_roto_no_bloquean_el_indice_ni_alteran_jars() {
        let (_temp, s) = servidor();
        let mut catalog = Fake::default();
        let debugify=jar_descriptor(b"{\"id\":\"debugify\",\"version\":\"1\",\"description\":\"linea uno\nlinea dos\tcon tab\",\"environment\":\"*\"}");
        let weapon=jar_descriptor(b"{\"id\":\"weaponmaster\",\"version\":\"4.4.2\",\"description\":\"uno\r\ndos\",\"environment\":\"*\"}");
        let roto = jar_descriptor(b"{\"id\":\"roto\",descripcion truncada");
        let server = jar_descriptor(
            b"{\"id\":\"backend\",\"description\":\"uno\ndos\",\"environment\":\"server\"}",
        );
        for (name, body) in [
            ("Debugify-1.21.1+1.0.jar", &debugify),
            ("weaponmaster_ydm-fabric-1.21.1-4.4.2.jar", &weapon),
            ("roto.jar", &roto),
            ("backend.jar", &server),
        ] {
            fs::write(s.carpeta.join("mods").join(name), body).unwrap();
        }
        // Un mod identificado y otro propio recorren la misma lectura tolerante.
        catalog.hashes.insert(
            hash(&debugify),
            version("debugify", "dbg1", &debugify, serde_json::json!([])),
        );
        let before = s.huella().unwrap();
        let index = publicar_con(&catalog, &s).await.unwrap();
        assert_eq!(index.mods.len(), 1);
        assert_eq!(index.archivos.len(), 2);
        let weapon_entry = index
            .archivos
            .iter()
            .find(|a| a.archivo.starts_with("weaponmaster"))
            .unwrap();
        assert_eq!(weapon_entry.id, "icaro-host-weaponmaster");
        assert_eq!(weapon_entry.sha512, hash(&weapon));
        assert!(index
            .avisos
            .iter()
            .any(|a| a.contains("Debugify-1.21.1+1.0.jar") && a.contains("se toleraron")));
        assert!(index
            .avisos
            .iter()
            .any(|a| a.starts_with("roto.jar:") && a.contains("ilegible")));
        assert_eq!(
            index
                .avisos
                .iter()
                .filter(|a| a.starts_with("roto.jar:") && a.contains("ilegible"))
                .count(),
            1
        );
        assert_eq!(before, s.huella().unwrap());
    }
    #[test]
    fn descriptor_valido_con_escapes_no_se_normaliza_y_tamano_excesivo_es_aviso() {
        let (_temp, s) = servidor();
        let path = s.carpeta.join("mods/prueba.jar");
        let mut avisos = vec![];
        fs::write(&path,jar_descriptor(br#"{"id":"test","description":"linea\nsegunda\t\"citada\"","environment":"server"}"#)).unwrap();
        let data = descriptor_fabric(&path, &mut avisos).unwrap().unwrap();
        assert_eq!(
            data["description"].as_str(),
            Some("linea\nsegunda\t\"citada\"")
        );
        assert!(avisos.is_empty());
        fs::write(&path, jar_descriptor(&vec![b' '; 128 * 1024 + 1])).unwrap();
        assert!(descriptor_fabric(&path, &mut avisos).unwrap().is_none());
        assert!(avisos[0].starts_with("prueba.jar:"));
    }
    #[tokio::test]
    #[ignore = "Descarga Fabric API real de Modrinth en un servidor temporal aislado"]
    async fn modrinth_real_publica_desde_carpeta_dedicada_y_el_cliente_verifica_el_indice() {
        let (_temp, s) = servidor();
        let catalog = Catalog::new().unwrap();
        let version = catalog
            .versions("fabric-api", Some(&s.minecraft), Some(&s.loader))
            .await
            .unwrap()
            .into_iter()
            .find(|v| v.version_type == "release")
            .unwrap();
        let file = version.files.iter().find(|f| f.primary).unwrap();
        let bytes = <Catalog as Fuente>::archivo(&catalog, file).await.unwrap();
        assert_eq!(hash(&bytes), file.hashes.sha512);
        fs::write(s.carpeta.join("mods").join(&file.filename), bytes).unwrap();
        let index = publicar(&catalog, &s).await.unwrap();
        assert!(index
            .mods
            .iter()
            .any(|e| e.version == version.id && e.sha512 == file.hashes.sha512));
        let store = Almacen::nuevo(s.carpeta.join("cliente/instances"));
        let instance = store
            .crear_con_loader("Cliente", &s.minecraft, &s.loader, 0)
            .unwrap();
        crate::versiones_instancia::guardar_loader(&store, &instance, &s.loader_version).unwrap();
        let plan =
            super::super::preparar(&catalog, &store, &instance, index, "127.0.0.1:25566".into())
                .await
                .unwrap();
        assert!(plan
            .cambios
            .iter()
            .any(|c| c.proyecto == version.project_id && c.tipo == Tipo::Agregar));
        super::super::aplicar(
            &catalog,
            &store,
            &instance,
            &plan,
            CancellationToken::new(),
            |_, _| {},
        )
        .await
        .unwrap();
        assert_eq!(
            hash_archivo(
                &store
                    .juego(&instance.id)
                    .unwrap()
                    .join("mods")
                    .join(&file.filename)
            )
            .unwrap(),
            file.hashes.sha512
        );
    }
    struct Temp(PathBuf);
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn servidor() -> (Temp, ServidorDedicado) {
        let mut random = [0; 16];
        getrandom::fill(&mut random).unwrap();
        let dir = std::env::temp_dir().join(format!("icaro-dedicado-{}", hash(&random)));
        fs::create_dir_all(dir.join("mods")).unwrap();
        let s = ServidorDedicado {
            carpeta: dir.clone(),
            minecraft: "1.21.1".into(),
            loader: "fabric".into(),
            loader_version: "0.16.14".into(),
            excluir: BTreeSet::new(),
        };
        (Temp(dir), s)
    }
    fn jar(environment: &str) -> Vec<u8> {
        jar_id(environment, "test")
    }
    fn jar_id(environment: &str, id: &str) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        writer
            .start_file("fabric.mod.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        write!(writer, "{{\"schemaVersion\":1,\"id\":\"{id}\",\"version\":\"1\",\"environment\":\"{environment}\"}}").unwrap();
        writer.finish().unwrap().into_inner()
    }
    fn version(project: &str, id: &str, bytes: &[u8], dependencies: serde_json::Value) -> Version {
        serde_json::from_value(serde_json::json!({
            "id":id,"project_id":project,"name":project,"version_number":"1","version_type":"release",
            "game_versions":["1.21.1"],"loaders":["fabric"],"date_published":"2026-10-14T00:00:00Z",
            "files":[{"filename":format!("{project}.jar"),"url":format!("https://cdn.modrinth.com/data/{project}/versions/{id}/{project}.jar"),"size":bytes.len(),"primary":true,"hashes":{"sha512":hash(bytes)}}],
            "dependencies":dependencies
        })).unwrap()
    }
    #[tokio::test]
    #[ignore = "Descarga Ksyxis 1.4.4 real de Modrinth en una carpeta temporal"]
    async fn ksyxis_real_no_bloquea_el_indice_con_minecraft_1211() {
        let (_temp, mut s) = servidor();
        s.loader_version = "0.19.5".into();
        let catalog = Catalog::new().unwrap();
        let v = catalog
            .versions("ksyxis", None, None)
            .await
            .unwrap()
            .into_iter()
            .find(|v| v.version_number == "1.4.4")
            .expect("Ksyxis 1.4.4 publicado");
        let file = v.files.iter().find(|f| f.primary).unwrap();
        let bytes = <Catalog as Fuente>::archivo(&catalog, file).await.unwrap();
        assert_eq!(hash(&bytes), file.hashes.sha512);
        fs::write(s.carpeta.join("mods").join(&file.filename), &bytes).unwrap();
        let before = s.huella().unwrap();
        let index = publicar(&catalog, &s).await.unwrap();
        if Catalogo::cliente(&catalog, &v).await.unwrap() {
            if !compatible(&v, &s.minecraft, &s.loader) {
                assert!(index
                    .archivos
                    .iter()
                    .any(|a| a.sha512 == file.hashes.sha512));
                assert!(index.avisos.iter().any(|a| a.contains("no declara")));
            } else {
                assert!(index.mods.iter().any(|m| m.sha512 == file.hashes.sha512));
            }
        } else {
            assert!(index.mods.is_empty() && index.archivos.is_empty());
        }
        assert_eq!(before, s.huella().unwrap());
    }
    #[tokio::test]
    async fn universales_con_versiones_extremas_o_loader_no_listado_son_propios_con_aviso() {
        let (_temp, s) = servidor();
        let mut catalog = Fake::default();
        for (name, project, versions, loaders, server_only) in [
            (
                "Ksyxis-1.4.4.jar",
                "ksyxis",
                vec!["1.8", "26.3"],
                vec!["fabric", "forge"],
                false,
            ),
            (
                "multi-loader.jar",
                "multiloader",
                vec!["1.21.1"],
                vec!["forge"],
                false,
            ),
            (
                "server-only.jar",
                "backend",
                vec!["1.8", "26.3"],
                vec!["forge"],
                true,
            ),
        ] {
            let body = jar_id("*", project);
            fs::write(s.carpeta.join("mods").join(name), &body).unwrap();
            let mut v = version(project, project, &body, serde_json::json!([]));
            v.game_versions = versions.into_iter().map(String::from).collect();
            v.loaders = loaders.into_iter().map(String::from).collect();
            catalog.hashes.insert(hash(&body), v);
            if server_only {
                catalog.server_only.insert(project.into());
            }
        }
        for (project, dependency) in [("dependiente", "ksyxis"), ("dependiente2", "dependiente")] {
            let body = jar_id("*", project);
            fs::write(s.carpeta.join("mods").join(format!("{project}.jar")), &body).unwrap();
            catalog.hashes.insert(hash(&body),version(project,project,&body,serde_json::json!([{"project_id":dependency,"version_id":dependency,"dependency_type":"required"}])));
        }
        let before = s.huella().unwrap();
        let index = publicar_con(&catalog, &s).await.unwrap();
        assert!(index.mods.is_empty());
        assert_eq!(index.archivos.len(), 4);
        assert_eq!(index.avisos.len(), 4);
        assert!(index
            .avisos
            .iter()
            .any(|a| a.starts_with("Ksyxis-1.4.4.jar:") && a.contains("no declara")));
        assert!(index
            .archivos
            .iter()
            .all(|a| a.archivo != "server-only.jar"));
        assert_eq!(before, s.huella().unwrap());
        // Si el catálogo devuelve otro hash, eso sí invalida la publicación.
        catalog
            .hashes
            .values_mut()
            .for_each(|v| v.files[0].hashes.sha512 = "0".repeat(128));
        assert!(publicar_con(&catalog, &s)
            .await
            .unwrap_err()
            .contains("SHA-512"));
    }
    #[derive(Default)]
    struct Fake {
        hashes: BTreeMap<String, Version>,
        versions: BTreeMap<String, Version>,
        server_only: BTreeSet<String>,
        modificar: Option<PathBuf>,
    }
    impl Catalogo for Fake {
        async fn hashes(&self, _: &[String]) -> Result<BTreeMap<String, Version>, String> {
            if let Some(path) = &self.modificar {
                fs::write(path, b"mod nuevo").unwrap();
            }
            Ok(self.hashes.clone())
        }
        async fn cliente(&self, version: &Version) -> Result<bool, String> {
            Ok(!self.server_only.contains(&version.project_id))
        }
        async fn version(&self, id: &str) -> Result<Version, String> {
            self.versions
                .get(id)
                .cloned()
                .ok_or("Versión ausente".into())
        }
        async fn versiones(&self, id: &str, _: &str, _: &str) -> Result<Vec<Version>, String> {
            Ok(self
                .versions
                .values()
                .filter(|v| v.project_id == id)
                .cloned()
                .collect())
        }
    }
    #[tokio::test]
    async fn carpeta_dedicada_publica_hashes_excluye_servidor_y_agrega_dependencia_cliente_sin_escribir(
    ) {
        let (_temp, s) = servidor();
        let mut catalog = Fake::default();
        let common = jar("*");
        let private = jar("server");
        let dedicated = b"servidor conocido";
        fs::write(s.carpeta.join("mods/common.jar"), &common).unwrap();
        fs::write(s.carpeta.join("mods/private.jar"), private).unwrap();
        fs::write(s.carpeta.join("mods/backend.jar"), dedicated).unwrap();
        fs::write(s.carpeta.join("server.properties"), b"server-port=25565").unwrap();
        fs::create_dir(s.carpeta.join("world")).unwrap();
        fs::write(s.carpeta.join("world/level.dat"), b"mundo").unwrap();
        catalog.hashes.insert(hash(&common), version("common", "v1", &common, serde_json::json!([{"version_id":"dep1","project_id":"clientdep","dependency_type":"required"}])));
        catalog.hashes.insert(
            hash(dedicated),
            version("backend", "back1", dedicated, serde_json::json!([])),
        );
        catalog.server_only.insert("backend".into());
        catalog.versions.insert(
            "dep1".into(),
            version("clientdep", "dep1", b"cliente", serde_json::json!([])),
        );
        let before = s.huella().unwrap();
        let index = publicar_con(&catalog, &s).await.unwrap();
        assert_eq!(
            index
                .mods
                .iter()
                .map(|e| e.proyecto.as_str())
                .collect::<Vec<_>>(),
            vec!["clientdep", "common"]
        );
        assert_eq!(index.mods[1].sha512, hash(&common));
        assert_eq!(before, s.huella().unwrap());
        assert_eq!(
            fs::read(s.carpeta.join("world/level.dat")).unwrap(),
            b"mundo"
        );
        assert_eq!(
            fs::read(s.carpeta.join("server.properties")).unwrap(),
            b"server-port=25565"
        );
        // El mismo contrato HTTP que consulta el launcher sigue siendo compatible.
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let published = Arc::new(RwLock::new(Some(index)));
        let cancel = CancellationToken::new();
        let service = tokio::spawn(super::super::servir(
            listener,
            published.clone(),
            cancel.clone(),
        ));
        let fetched = super::super::consultar(addr, CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(fetched.mods.len(), 2);
        fs::remove_file(s.carpeta.join("mods/common.jar")).unwrap();
        let next = publicar_con(&catalog, &s).await.unwrap();
        assert!(next.mods.is_empty());
        *published.write().await = Some(next);
        assert!(super::super::consultar(addr, CancellationToken::new())
            .await
            .unwrap()
            .mods
            .is_empty());
        cancel.cancel();
        service.await.unwrap().unwrap();
    }
    #[tokio::test]
    async fn archivo_no_identificado_es_aviso_y_cambio_durante_consulta_impide_publicar() {
        let (_temp, s) = servidor();
        let body = jar("*");
        fs::write(s.carpeta.join("mods/common.jar"), &body).unwrap();
        let mut catalog = Fake::default();
        let index = publicar_con(&catalog, &s).await.unwrap();
        assert_eq!(index.archivos.len(), 1);
        assert_eq!(index.avisos.len(), 1);
        catalog.hashes.insert(
            hash(&body),
            version("common", "v1", &body, serde_json::json!([])),
        );
        catalog.modificar = Some(s.carpeta.join("mods/new.jar"));
        assert!(publicar_con(&catalog, &s)
            .await
            .unwrap_err()
            .contains("cambiaron"));
    }
    #[tokio::test]
    async fn ciento_sesenta_y_tres_mods_con_diecinueve_propios_publican_lista_completa() {
        let (_temp, mut s) = servidor();
        s.loader_version = "0.19.5".into();
        let mut catalog = Fake::default();
        for n in 0..163 {
            let project = format!("mod{n}");
            let body = jar_id("*", &project);
            fs::write(s.carpeta.join("mods").join(format!("{project}.jar")), &body).unwrap();
            if n < 144 {
                catalog.hashes.insert(
                    hash(&body),
                    version(&project, &format!("ver{n}"), &body, serde_json::json!([])),
                );
            }
        }
        let index = publicar_con(&catalog, &s).await.unwrap();
        assert_eq!(index.mods.len(), 144);
        assert_eq!(index.archivos.len(), 19);
        assert_eq!(index.avisos.len(), 19);
        assert_eq!(index.loader_version.as_deref(), Some("0.19.5"));
        s.excluir.insert("mod162.jar".into());
        assert_eq!(publicar_con(&catalog, &s).await.unwrap().archivos.len(), 18);
    }
    #[tokio::test]
    async fn privados_http_se_agregan_actualizan_y_retiran_sin_tocar_locales_y_rechazan_cambios() {
        let (_temp, mut s) = servidor();
        s.loader_version = "0.19.5".into();
        let catalog = Fake::default();
        let source = Catalog::new().unwrap();
        let file = s.carpeta.join("mods/casinocraft.jar");
        let original = jar("*");
        fs::write(&file, &original).unwrap();
        let index = publicar_con(&catalog, &s).await.unwrap();
        let store = Almacen::nuevo(s.carpeta.join("clientes/instances"));
        let instance = store
            .crear_con_loader("Cliente", "1.21.1", "fabric", 0)
            .unwrap();
        crate::versiones_instancia::guardar_loader(&store, &instance, "0.19.5").unwrap();
        let dir = store.juego(&instance.id).unwrap().join("mods");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("manual.jar"), b"mod personal").unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let published = Arc::new(RwLock::new(Some(index.clone())));
        let cancel = CancellationToken::new();
        let service = tokio::spawn(super::super::servir_archivos(
            listener,
            published.clone(),
            cancel.clone(),
            Some(s.carpeta.join("mods")),
        ));
        let downloaded = super::super::consultar(addr, CancellationToken::new())
            .await
            .unwrap();
        let plan = super::super::preparar(&source, &store, &instance, downloaded, addr.to_string())
            .await
            .unwrap();
        assert_eq!(plan.cambios.len(), 1);
        // El archivo cambia después de preparar: no se entregará el contenido nuevo bajo un hash antiguo.
        fs::write(&file, jar("client")).unwrap();
        assert!(super::super::aplicar(
            &source,
            &store,
            &instance,
            &plan,
            CancellationToken::new(),
            |_, _| {}
        )
        .await
        .is_err());
        assert!(!dir.join("casinocraft.jar").exists());
        fs::write(&file, &original).unwrap();
        super::super::aplicar(
            &source,
            &store,
            &instance,
            &plan,
            CancellationToken::new(),
            |_, _| {},
        )
        .await
        .unwrap();
        assert_eq!(fs::read(dir.join("casinocraft.jar")).unwrap(), original);
        let updated = jar("client");
        fs::write(&file, &updated).unwrap();
        let next = publicar_con(&catalog, &s).await.unwrap();
        *published.write().await = Some(next.clone());
        let plan = super::super::preparar(&source, &store, &instance, next, addr.to_string())
            .await
            .unwrap();
        assert_eq!(plan.cambios[0].tipo, Tipo::Actualizar);
        super::super::aplicar(
            &source,
            &store,
            &instance,
            &plan,
            CancellationToken::new(),
            |_, _| {},
        )
        .await
        .unwrap();
        assert_eq!(fs::read(dir.join("casinocraft.jar")).unwrap(), updated);
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        for path in [
            "/icaro/v1/files/../server.properties",
            "/icaro/v1/files/nohash",
        ] {
            assert_eq!(
                client
                    .get(format!("http://{addr}{path}"))
                    .send()
                    .await
                    .unwrap()
                    .status(),
                reqwest::StatusCode::NOT_FOUND
            );
        }
        fs::remove_file(&file).unwrap();
        let next = publicar_con(&catalog, &s).await.unwrap();
        let plan =
            super::super::preparar(&source, &store, &instance, next.clone(), addr.to_string())
                .await
                .unwrap();
        assert_eq!(plan.cambios[0].tipo, Tipo::Quitar);
        fs::write(dir.join("casinocraft.jar"), b"modificado por el jugador").unwrap();
        assert!(
            super::super::preparar(&source, &store, &instance, next, addr.to_string())
                .await
                .unwrap_err()
                .contains("modificado")
        );
        fs::write(dir.join("casinocraft.jar"), &updated).unwrap();
        super::super::aplicar(
            &source,
            &store,
            &instance,
            &plan,
            CancellationToken::new(),
            |_, _| {},
        )
        .await
        .unwrap();
        assert!(!dir.join("casinocraft.jar").exists());
        assert_eq!(fs::read(dir.join("manual.jar")).unwrap(), b"mod personal");
        cancel.cancel();
        service.await.unwrap().unwrap();
    }
    #[tokio::test]
    async fn dependencia_fijada_en_conflicto_no_se_sustituye_por_la_ultima() {
        let (_temp, s) = servidor();
        let mut catalog = Fake::default();
        for (project, id, bytes, deps) in [
            (
                "common",
                "v1",
                b"comun".as_slice(),
                serde_json::json!([{"version_id":"dep2","project_id":"dep","dependency_type":"required"}]),
            ),
            (
                "dep",
                "dep1",
                b"dep instalada".as_slice(),
                serde_json::json!([]),
            ),
        ] {
            fs::write(s.carpeta.join("mods").join(format!("{project}.jar")), bytes).unwrap();
            catalog
                .hashes
                .insert(hash(bytes), version(project, id, bytes, deps));
        }
        assert!(publicar_con(&catalog, &s)
            .await
            .unwrap_err()
            .contains("Conflicto"));
    }
    #[test]
    fn origen_invalido_o_jars_anidados_no_generan_un_indice_vacio() {
        let (_temp, mut s) = servidor();
        s.loader_version.clear();
        assert!(s.validar().is_err());
        s.loader_version = "0.16.14".into();
        fs::create_dir(s.carpeta.join("mods/otra")).unwrap();
        fs::write(s.carpeta.join("mods/otra/active.jar"), b"mod").unwrap();
        assert!(s.huella().is_err());
        fs::remove_dir_all(s.carpeta.join("mods")).unwrap();
        assert!(s.huella().is_err());
    }
}
