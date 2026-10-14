// SPDX-License-Identifier: GPL-3.0-only
//! Preferencias del jugador compartidas entre instancias, con mezcla por opción.
use crate::{instancias::Almacen, minecraft::Instalado};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

const MAX: u64 = 4 * 1024 * 1024;
static LOCK: Mutex<()> = Mutex::new(());
const OPCIONES: &[&str] = &[
    "mouseSensitivity",
    "invertYMouse",
    "rawMouseInput",
    "mouseWheelSensitivity",
    "discrete_mouse_scroll",
    "touchscreen",
    "autoJump",
    "toggleSprint",
    "toggleCrouch",
    "fov",
    "renderDistance",
    "simulationDistance",
    "maxFps",
    "enableVsync",
    "fancyGraphics",
    "graphicsMode",
    "ao",
    "bobView",
    "gamma",
    "guiScale",
    "clouds",
    "renderClouds",
    "particles",
    "mipmapLevels",
    "entityShadows",
    "entityDistanceScaling",
    "biomeBlendRadius",
    "screenEffectScale",
    "fovEffectScale",
    "darknessEffectScale",
];
const INICIALES: &[&str] = &[
    "mouseSensitivity",
    "invertYMouse",
    "fov",
    "renderDistance",
    "maxFps",
    "enableVsync",
    "bobView",
    "gamma",
    "guiScale",
    "particles",
    "mipmapLevels",
    "entityShadows",
];
const TECLAS_VANILLA: &[&str] = &[
    "attack",
    "use",
    "forward",
    "back",
    "left",
    "right",
    "jump",
    "sneak",
    "sprint",
    "drop",
    "inventory",
    "chat",
    "playerlist",
    "pickItem",
    "command",
    "screenshot",
    "togglePerspective",
    "smoothCamera",
    "fullscreen",
    "spectatorOutlines",
    "advancements",
    "swapOffhand",
    "swapHands",
    "saveToolbarActivator",
    "loadToolbarActivator",
    "hotbar.1",
    "hotbar.2",
    "hotbar.3",
    "hotbar.4",
    "hotbar.5",
    "hotbar.6",
    "hotbar.7",
    "hotbar.8",
    "hotbar.9",
];

#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Tecla {
    moderna: Option<String>,
    antigua: Option<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Compartidas {
    esquema: u32,
    opciones: BTreeMap<String, String>,
    teclas: BTreeMap<String, Tecla>,
}
impl Default for Compartidas {
    fn default() -> Self {
        Self {
            esquema: 1,
            opciones: BTreeMap::new(),
            teclas: BTreeMap::new(),
        }
    }
}
pub struct Sesion {
    archivo: PathBuf,
    juego: PathBuf,
    inicial: BTreeMap<String, String>,
}
pub fn ruta(store: &Almacen) -> Result<PathBuf, String> {
    Ok(store
        .datos()
        .map_err(|e| e.to_string())?
        .join("opciones-minecraft.json"))
}
fn regular(path: &Path, directorio: bool) -> Result<(), String> {
    let m =
        fs::symlink_metadata(path).map_err(|_| "No se pudieron leer las opciones de Minecraft.")?;
    #[cfg(windows)]
    let enlace = {
        use std::os::windows::fs::MetadataExt;
        m.file_attributes() & 0x400 != 0
    };
    #[cfg(not(windows))]
    let enlace = m.file_type().is_symlink();
    if enlace || (directorio && !m.is_dir()) || (!directorio && (!m.is_file() || m.len() > MAX)) {
        return Err("La ruta o el tamaño de las opciones de Minecraft no son válidos.".into());
    }
    Ok(())
}
fn leer(path: &Path) -> Result<Option<String>, String> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("No se pudieron leer las opciones de Minecraft.".into()),
        Ok(_) => regular(path, false)?,
    }
    let mut bytes = vec![];
    fs::File::open(path)
        .map_err(|_| "No se pudieron abrir las opciones de Minecraft.")?
        .take(MAX + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "No se pudieron leer las opciones de Minecraft.")?;
    if bytes.len() as u64 > MAX {
        return Err("Las opciones de Minecraft superan el tamaño permitido.".into());
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| "Las opciones de Minecraft no son texto válido.".into())
}
fn escribir(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("Ruta de opciones inválida.")?;
    regular(parent, true)?;
    if path.exists() {
        regular(path, false)?;
    }
    if bytes.len() as u64 > MAX {
        return Err("Las opciones de Minecraft superan el tamaño permitido.".into());
    }
    if fs::read(path).ok().as_deref() == Some(bytes) {
        return Ok(());
    }
    let mut nonce = [0u8; 8];
    getrandom::fill(&mut nonce).map_err(|_| "No se pudieron preparar las opciones.")?;
    let temp = parent.join(format!(
        ".icaro-options-{:016x}.tmp",
        u64::from_le_bytes(nonce)
    ));
    let result = (|| {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|_| "No se pudieron preparar las opciones.")?;
        f.write_all(bytes)
            .and_then(|_| f.sync_all())
            .map_err(|_| "No se pudieron guardar las opciones.")?;
        drop(f);
        fs::rename(&temp, path)
            .map_err(|_| "No se pudieron aplicar las opciones de Minecraft.".to_owned())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}
fn moderna(valor: &str) -> Option<bool> {
    if valor.starts_with("key.keyboard.")
        || valor.starts_with("key.mouse.")
        || valor == "key.unknown"
    {
        valor
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
            .then_some(true)
    } else {
        valor
            .parse::<i32>()
            .ok()
            .filter(|n| (-100..=255).contains(n))
            .map(|_| false)
    }
}
fn seleccionada(clave: &str, valor: &str) -> bool {
    if clave.len() > 256
        || valor.len() > 1024
        || clave.chars().any(char::is_control)
        || valor.chars().any(char::is_control)
    {
        return false;
    }
    if clave.starts_with("key_") {
        return moderna(valor).is_some();
    }
    OPCIONES.contains(&clave)
        && serde_json::from_str::<serde_json::Value>(valor).is_ok_and(|v| {
            matches!(
                v,
                serde_json::Value::Bool(_)
                    | serde_json::Value::Number(_)
                    | serde_json::Value::String(_)
            )
        })
}
fn opciones(texto: &str) -> Result<BTreeMap<String, String>, String> {
    let mut values = BTreeMap::new();
    for linea in texto.lines() {
        if let Some((k, v)) = linea.split_once(':') {
            if seleccionada(k, v) && values.insert(k.to_owned(), v.to_owned()).is_some() {
                return Err("Las opciones de Minecraft contienen un ajuste repetido.".into());
            }
        }
    }
    if values.len() > 2048 {
        return Err("Hay demasiadas opciones compartidas.".into());
    }
    Ok(values)
}
fn cargar(path: &Path) -> Result<Compartidas, String> {
    let Some(texto) = leer(path)? else {
        return Ok(Compartidas::default());
    };
    let c: Compartidas = serde_json::from_str(&texto)
        .map_err(|_| "Las opciones compartidas no son válidas; se conservó el archivo.")?;
    if c.esquema != 1
        || c.opciones.len() + c.teclas.len() > 2048
        || c.opciones
            .iter()
            .any(|(k, v)| k.starts_with("key_") || !seleccionada(k, v))
        || c.teclas.iter().any(|(k, v)| {
            !k.starts_with("key_")
                || k.len() > 256
                || k.chars().any(char::is_control)
                || v.moderna.as_ref().is_some_and(|s| moderna(s) != Some(true))
                || v.antigua
                    .as_ref()
                    .is_some_and(|s| moderna(s) != Some(false))
        })
    {
        return Err("Las opciones compartidas tienen un esquema o valores no admitidos; se conservó el archivo.".into());
    }
    Ok(c)
}
fn mismo_tipo(a: &str, b: &str) -> bool {
    let tipo = |v: &str| match serde_json::from_str::<serde_json::Value>(v) {
        Ok(serde_json::Value::Bool(_)) => 1,
        Ok(serde_json::Value::Number(_)) => 2,
        Ok(serde_json::Value::String(_)) => 3,
        _ => 0,
    };
    tipo(a) != 0 && tipo(a) == tipo(b)
}
fn mezclar(texto: &str, cambios: &BTreeMap<String, String>) -> String {
    let mut pendientes = cambios.clone();
    let mut result = String::new();
    for linea in texto.split_inclusive('\n') {
        let contenido = linea.trim_end_matches(['\r', '\n']);
        if let Some((k, _)) = contenido.split_once(':') {
            if let Some(v) = pendientes.remove(k) {
                result.push_str(&format!("{k}:{v}"));
                result.push_str(&linea[contenido.len()..]);
                continue;
            }
        }
        result.push_str(linea);
    }
    for (k, v) in pendientes {
        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(&format!("{k}:{v}\n"));
    }
    result
}
fn version_datos(instalado: &Instalado) -> Option<u64> {
    for path in instalado.classpath.iter().rev() {
        let Ok(file) = fs::File::open(path) else {
            continue;
        };
        let Ok(mut jar) = zip::ZipArchive::new(file) else {
            continue;
        };
        let Ok(mut entry) = jar.by_name("version.json") else {
            continue;
        };
        if entry.size() > 64 * 1024 {
            continue;
        }
        let mut text = String::new();
        if entry.read_to_string(&mut text).is_err() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            if v["id"].as_str() != Some(instalado.version.as_str()) {
                continue;
            }
            if let Some(n) = v["world_version"]
                .as_u64()
                .filter(|n| *n <= i32::MAX as u64)
            {
                return Some(n);
            }
        }
    }
    None
}
/// Mezcla únicamente ajustes compatibles y registra la base de esta sesión.
pub fn preparar(archivo: &Path, juego: &Path, instalado: &Instalado) -> Result<Sesion, String> {
    let _lock = LOCK
        .lock()
        .map_err(|_| "No se pudo acceder a las opciones compartidas.")?;
    regular(juego, true)?;
    let c = cargar(archivo)?;
    let path = juego.join("options.txt");
    let existente = leer(&path)?;
    let mut texto = existente.clone().unwrap_or_default();
    let actuales = opciones(&texto)?;
    // Una instalación nueva usa su propia versión de datos, nunca la de otra instancia.
    let version = if existente.is_none() {
        version_datos(instalado)
    } else {
        None
    };
    if let Some(v) = version {
        texto = format!("version:{v}\n");
    }
    let mut cambios = BTreeMap::new();
    for (k, v) in &c.opciones {
        if actuales.get(k).is_some_and(|actual| mismo_tipo(actual, v))
            || (version.is_some() && INICIALES.contains(&k.as_str()))
        {
            cambios.insert(k.clone(), v.clone());
        }
    }
    for (k, v) in &c.teclas {
        let actual = actuales.get(k).and_then(|v| moderna(v));
        let vanilla = k
            .strip_prefix("key_key.")
            .is_some_and(|k| TECLAS_VANILLA.contains(&k));
        let formato = actual.or_else(|| (version.is_some() && vanilla).then_some(true));
        let valor = match formato {
            Some(true) => &v.moderna,
            Some(false) => &v.antigua,
            None => continue,
        };
        if let Some(valor) = valor {
            cambios.insert(k.clone(), valor.clone());
        }
    }
    texto = mezclar(&texto, &cambios);
    if !cambios.is_empty() {
        escribir(&path, texto.as_bytes())?;
    }
    Ok(Sesion {
        archivo: archivo.to_owned(),
        juego: juego.to_owned(),
        inicial: opciones(&texto)?,
    })
}
/// Guarda solo los cambios de esta sesión para no deshacer otra sesión simultánea.
pub fn capturar(sesion: &Sesion) -> Result<(), String> {
    let _lock = LOCK
        .lock()
        .map_err(|_| "No se pudo acceder a las opciones compartidas.")?;
    let Some(texto) = leer(&sesion.juego.join("options.txt"))? else {
        return Ok(());
    };
    let actuales = opciones(&texto)?;
    let mut c = cargar(&sesion.archivo)?;
    for (k, v) in actuales {
        if let Some(formato) = moderna(&v).filter(|_| k.starts_with("key_")) {
            let tecla = c.teclas.entry(k.clone()).or_default();
            let destino = if formato {
                &mut tecla.moderna
            } else {
                &mut tecla.antigua
            };
            if destino.is_none() || sesion.inicial.get(&k) != Some(&v) {
                *destino = Some(v);
            }
        } else if !c.opciones.contains_key(&k) || sesion.inicial.get(&k) != Some(&v) {
            c.opciones.insert(k, v);
        }
    }
    let bytes = serde_json::to_vec_pretty(&c)
        .map_err(|_| "No se pudieron preparar las opciones compartidas.")?;
    escribir(&sesion.archivo, &bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Datos(PathBuf);
    impl Datos {
        fn nuevos() -> Self {
            let mut nonce = [0u8; 8];
            getrandom::fill(&mut nonce).unwrap();
            let path = std::env::temp_dir()
                .join(format!("icaro-opciones-{:016x}", u64::from_le_bytes(nonce)));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn global(&self) -> PathBuf {
            self.0.join("opciones-minecraft.json")
        }
        fn juego(&self, nombre: &str, texto: &str) -> PathBuf {
            let path = self.0.join(nombre);
            fs::create_dir(&path).unwrap();
            fs::write(path.join("options.txt"), texto).unwrap();
            path
        }
    }
    impl Drop for Datos {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn perfil() -> Instalado {
        Instalado {
            version: "1.21.1".into(),
            perfil_id: None,
            quick_play_multiplayer: None,
            main_class: "net.minecraft.client.main.Main".into(),
            classpath: vec![],
            natives: PathBuf::new(),
            assets: PathBuf::new(),
            asset_index: "17".into(),
            java_major: 21,
            java_component: "java-runtime-delta".into(),
            java_perfil: None,
            game_arguments: vec![],
            jvm_arguments: vec![],
        }
    }
    fn semilla(d: &Datos) {
        let game = d.juego("origen", "version:4903\nmouseSensitivity:0.25\nrenderDistance:12\nkey_key.forward:key.keyboard.z\nkey_mod.presente:key.keyboard.r\nkey_mod.ausente:key.keyboard.g\nresourcePacks:[\"file/Origen.zip\"]\n");
        let s = preparar(&d.global(), &game, &perfil()).unwrap();
        capturar(&s).unwrap();
    }
    #[test]
    fn comparte_controles_y_graficos_y_filtra_las_acciones_de_mods_del_destino() {
        let d = Datos::nuevos();
        semilla(&d);
        let game = d.juego("destino", "version:3955\r\nmouseSensitivity:0.5\r\nrenderDistance:8\r\nkey_key.forward:key.keyboard.w\r\nkey_mod.presente:key.keyboard.q\r\nresourcePacks:[\"file/Propio.zip\"]\r\nlang:es_cl\r\n");
        preparar(&d.global(), &game, &perfil()).unwrap();
        let texto = fs::read_to_string(game.join("options.txt")).unwrap();
        assert!(texto.contains("mouseSensitivity:0.25\r\n"));
        assert!(texto.contains("renderDistance:12\r\n"));
        assert!(texto.contains("key_key.forward:key.keyboard.z\r\n"));
        assert!(texto.contains("key_mod.presente:key.keyboard.r\r\n"));
        assert!(!texto.contains("key_mod.ausente"));
        assert!(
            texto.contains("version:3955\r\nresourcePacks:")
                || texto.starts_with("version:3955\r\n")
        );
        assert!(texto.contains("resourcePacks:[\"file/Propio.zip\"]\r\nlang:es_cl\r\n"));
        assert!(!fs::read_to_string(d.global())
            .unwrap()
            .contains("resourcePacks"));
    }
    #[test]
    fn cerrar_una_sesion_antigua_no_revierte_cambios_de_otra_instancia() {
        let d = Datos::nuevos();
        semilla(&d);
        let a = d.juego("a", "mouseSensitivity:0.5\nrenderDistance:8\n");
        let b = d.juego("b", "mouseSensitivity:0.5\nrenderDistance:8\n");
        let sa = preparar(&d.global(), &a, &perfil()).unwrap();
        let sb = preparar(&d.global(), &b, &perfil()).unwrap();
        fs::write(
            a.join("options.txt"),
            "mouseSensitivity:0.3\nrenderDistance:12\n",
        )
        .unwrap();
        capturar(&sa).unwrap();
        fs::write(
            b.join("options.txt"),
            "mouseSensitivity:0.25\nrenderDistance:16\n",
        )
        .unwrap();
        capturar(&sb).unwrap();
        let c = cargar(&d.global()).unwrap();
        assert_eq!(c.opciones["mouseSensitivity"], "0.3");
        assert_eq!(c.opciones["renderDistance"], "16");
    }
    #[test]
    fn nueva_instancia_usa_su_version_de_datos_y_solo_teclas_vanilla() {
        let d = Datos::nuevos();
        semilla(&d);
        let jar = d.0.join("cliente.jar");
        let mut zip = zip::ZipWriter::new(fs::File::create(&jar).unwrap());
        zip.start_file("version.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(br#"{"id":"1.21.1","world_version":3955}"#)
            .unwrap();
        zip.finish().unwrap();
        let game = d.0.join("nueva");
        fs::create_dir(&game).unwrap();
        let mut p = perfil();
        let otra = d.0.join("otra-version.jar");
        let mut zip = zip::ZipWriter::new(fs::File::create(&otra).unwrap());
        zip.start_file("version.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(br#"{"id":"26.2","world_version":4903}"#)
            .unwrap();
        zip.finish().unwrap();
        p.classpath = vec![jar, otra];
        preparar(&d.global(), &game, &p).unwrap();
        let texto = fs::read_to_string(game.join("options.txt")).unwrap();
        assert!(texto.starts_with("version:3955\n"));
        assert!(texto.contains("mouseSensitivity:0.25\n"));
        assert!(texto.contains("key_key.forward:key.keyboard.z\n"));
        assert!(!texto.contains("key_mod."));
    }
    #[test]
    fn archivo_global_danado_se_conserva_y_no_modifica_la_instancia() {
        let d = Datos::nuevos();
        fs::write(d.global(), b"roto").unwrap();
        let game = d.juego("destino", "mouseSensitivity:0.5\n");
        assert!(preparar(&d.global(), &game, &perfil()).is_err());
        assert_eq!(fs::read(d.global()).unwrap(), b"roto");
        assert_eq!(
            fs::read_to_string(game.join("options.txt")).unwrap(),
            "mouseSensitivity:0.5\n"
        );
    }
    #[test]
    fn teclas_antiguas_y_modernas_conservan_su_formato() {
        let d = Datos::nuevos();
        let vieja = d.juego("vieja", "key_key.jump:57\n");
        let s = preparar(&d.global(), &vieja, &perfil()).unwrap();
        capturar(&s).unwrap();
        let nueva = d.juego("nueva", "key_key.jump:key.keyboard.space\n");
        let s = preparar(&d.global(), &nueva, &perfil()).unwrap();
        capturar(&s).unwrap();
        fs::write(nueva.join("options.txt"), "key_key.jump:key.keyboard.e\n").unwrap();
        capturar(&s).unwrap();
        preparar(&d.global(), &vieja, &perfil()).unwrap();
        assert_eq!(
            fs::read_to_string(vieja.join("options.txt")).unwrap(),
            "key_key.jump:57\n"
        );
        assert_eq!(
            cargar(&d.global()).unwrap().teclas["key_key.jump"]
                .moderna
                .as_deref(),
            Some("key.keyboard.e")
        );
    }
}
