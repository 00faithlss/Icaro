// SPDX-License-Identifier: GPL-3.0-only
//! Skins locales: PNG normalizado, metadatos por cuenta y texturas de loopback.
use crate::{
    almacenamiento,
    auth::Session,
    cuenta::Textura,
    cuentas_locales::{Cuentas, Skin},
};
use sha2::{Digest, Sha256};
use std::{
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

mod servidor;
pub use servidor::{preparar, Preparada};

const MAX_PNG: usize = 1024 * 1024;
#[derive(Debug, Clone)]
pub struct Imagen {
    pub textura: Textura,
    pub antigua: bool,
    pub sugerida_delgada: bool,
}

pub fn decodificar(bytes: &[u8]) -> Result<Imagen, String> {
    if bytes.len() < 24
        || bytes.len() > MAX_PNG
        || &bytes[..8] != b"\x89PNG\r\n\x1a\n"
        || &bytes[12..16] != b"IHDR"
    {
        return Err("Solo .png válidos de hasta 1 MB.".into());
    }
    let ancho = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let alto = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    if ancho != 64 || !matches!(alto, 32 | 64) {
        return Err(format!(
            "La skin debe medir 64 × 64 o 64 × 32 px; esta mide {ancho} × {alto}."
        ));
    }
    let original = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map_err(|_| "No se pudo leer la imagen.")?
        .into_rgba8();
    if original.pixels().all(|p| p[3] == 0) {
        return Err("La imagen está vacía.".into());
    }
    let mut imagen = image::RgbaImage::new(64, 64);
    image::imageops::replace(&mut imagen, &original, 0, 0);
    if alto == 32 {
        // UV del formato antiguo: duplicar y reflejar los miembros izquierdos.
        for (sx, sy, w, h, dx, dy) in [
            (4, 16, 4, 4, 20, 48),
            (8, 16, 4, 4, 24, 48),
            (0, 20, 4, 12, 24, 52),
            (4, 20, 4, 12, 20, 52),
            (8, 20, 4, 12, 16, 52),
            (12, 20, 4, 12, 28, 52),
            (44, 16, 4, 4, 36, 48),
            (48, 16, 4, 4, 40, 48),
            (40, 20, 4, 12, 40, 52),
            (44, 20, 4, 12, 36, 52),
            (48, 20, 4, 12, 32, 52),
            (52, 20, 4, 12, 44, 52),
        ] {
            for y in 0..h {
                for x in 0..w {
                    imagen.put_pixel(dx + x, dy + y, *original.get_pixel(sx + w - 1 - x, sy + y));
                }
            }
        }
    }
    let sugerida_delgada =
        alto == 64 && (20..32).all(|y| (54..56).all(|x| imagen.get_pixel(x, y)[3] == 0));
    // Reencodear retira metadatos y cualquier contenido ajeno al bitmap.
    let mut png = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(imagen.clone())
        .write_to(&mut png, image::ImageOutputFormat::Png)
        .map_err(|_| "No se pudo preparar la skin.")?;
    Ok(Imagen {
        textura: Textura {
            png: png.into_inner().into(),
            rgba: imagen.into_raw().into(),
        },
        antigua: alto == 32,
        sugerida_delgada,
    })
}

pub fn cargar(path: &Path) -> Result<Imagen, String> {
    if !path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("png"))
    {
        return Err(format!(
            "Solo .png. Este archivo es .{}.",
            path.extension().and_then(|e| e.to_str()).unwrap_or("?")
        ));
    }
    almacenamiento::regular(path)?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|_| "No se pudo abrir la skin.")?
        .take((MAX_PNG + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "No se pudo leer la skin.")?;
    decodificar(&bytes)
}

fn ruta(raiz: &Path, hash: &str) -> Result<PathBuf, String> {
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("La referencia de skin no es válida.".into());
    }
    Ok(raiz.join("skins").join(format!("{hash}.png")))
}

pub fn leer(raiz: &Path, nombre: &str) -> Result<Option<(Imagen, bool)>, String> {
    let id = Session::local(nombre)?.profile.id;
    let cuentas = Cuentas::cargar(raiz)?;
    let Some(registro) = cuentas.skins.get(&id) else {
        return Ok(None);
    };
    let path = ruta(raiz, &registro.sha256)?;
    almacenamiento::regular(&raiz.join("skins"))?;
    let imagen = cargar(&path)?;
    if format!("{:x}", Sha256::digest(&imagen.textura.png)) != registro.sha256 {
        return Err("La skin guardada cambió; vuelve a elegir el PNG.".into());
    }
    Ok(Some((imagen, registro.delgada)))
}

pub fn guardar(
    raiz: &Path,
    nombre: &str,
    imagen: &Imagen,
    delgada: bool,
) -> Result<Cuentas, String> {
    let id = Session::local(nombre)?.profile.id;
    let mut cuentas = Cuentas::cargar(raiz)?;
    if !cuentas.nombres.iter().any(|n| n == nombre) {
        return Err("La cuenta ya no está en el launcher.".into());
    }
    let imagen = decodificar(&imagen.textura.png)?;
    let sha256 = format!("{:x}", Sha256::digest(&imagen.textura.png));
    let path = ruta(raiz, &sha256)?;
    let dir = path.parent().unwrap();
    std::fs::create_dir_all(dir).map_err(|_| "No se pudo crear la carpeta de skins.")?;
    almacenamiento::regular(dir)?;
    if path.exists() {
        let previa = cargar(&path)?;
        if previa.textura.png != imagen.textura.png {
            return Err("La copia guardada de esta skin fue modificada.".into());
        }
    } else {
        use std::io::Write;
        let mut nonce = [0; 8];
        getrandom::fill(&mut nonce).map_err(|_| "No se pudo preparar el guardado.")?;
        let temp = dir.join(format!(".skin-{:x}.tmp", u64::from_le_bytes(nonce)));
        let resultado = (|| {
            let mut f = std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temp)
                .map_err(|_| "No se pudo guardar la skin.")?;
            f.write_all(&imagen.textura.png)
                .and_then(|_| f.sync_all())
                .map_err(|_| "No se pudo guardar la skin.")?;
            drop(f);
            std::fs::rename(&temp, &path).map_err(|_| "No se pudo publicar la skin.")
        })();
        if resultado.is_err() {
            let _ = std::fs::remove_file(&temp);
        }
        resultado?;
    }
    cuentas.skins.insert(id, Skin { sha256, delgada });
    cuentas.guardar(raiz)?;
    Ok(cuentas)
}

pub fn quitar(raiz: &Path, nombre: &str) -> Result<Cuentas, String> {
    let id = Session::local(nombre)?.profile.id;
    let mut cuentas = Cuentas::cargar(raiz)?;
    cuentas.skins.remove(&id);
    cuentas.guardar(raiz)?;
    Ok(cuentas)
}

#[cfg(test)]
mod tests {
    use super::*;
    pub(super) fn ejemplo(alto: u32) -> Vec<u8> {
        let mut imagen = image::RgbaImage::from_pixel(64, alto, image::Rgba([100, 130, 170, 255]));
        imagen.put_pixel(4, 20, image::Rgba([255, 0, 0, 255]));
        let mut b = Cursor::new(vec![]);
        image::DynamicImage::ImageRgba8(imagen)
            .write_to(&mut b, image::ImageOutputFormat::Png)
            .unwrap();
        b.into_inner()
    }
    #[test]
    fn dimensiones_legacy_transparencia_y_persistencia_verificada() {
        let antigua = decodificar(&ejemplo(32)).unwrap();
        assert!(antigua.antigua && !antigua.sugerida_delgada);
        assert_eq!(
            &antigua.textura.rgba[(52 * 64 + 23) * 4..(52 * 64 + 23) * 4 + 4],
            &[255, 0, 0, 255]
        );
        assert!(decodificar(b"no png").is_err());
        let mut vacia = Cursor::new(vec![]);
        image::DynamicImage::ImageRgba8(image::RgbaImage::new(64, 64))
            .write_to(&mut vacia, image::ImageOutputFormat::Png)
            .unwrap();
        assert_eq!(
            decodificar(&vacia.into_inner()).unwrap_err(),
            "La imagen está vacía."
        );
        let raiz = std::env::temp_dir().join(format!(
            "icaro-skin-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        Cuentas::activar(&raiz, "Jugador_1").unwrap();
        let cuentas = guardar(&raiz, "Jugador_1", &antigua, false).unwrap();
        assert!(leer(&raiz, "Jugador_1").unwrap().is_some());
        quitar(&raiz, "Jugador_1").unwrap();
        assert!(leer(&raiz, "Jugador_1").unwrap().is_none());
        for skin in cuentas.skins.values() {
            std::fs::remove_file(ruta(&raiz, &skin.sha256).unwrap()).unwrap();
        }
        std::fs::remove_file(raiz.join("accounts/locales.json")).unwrap();
        std::fs::remove_dir(raiz.join("accounts")).unwrap();
        std::fs::remove_dir(raiz.join("skins")).unwrap();
        std::fs::remove_dir(raiz).unwrap();
    }
}
