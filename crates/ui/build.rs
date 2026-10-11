// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ColorType, ImageEncoder};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;

#[path = "src/originales.rs"]
mod originales;
mod preparar_imagenes;

mod laminas {
    macro_rules! unica {
        () => {
            false
        };
        ($nombre:ident) => {
            true
        };
    }
    macro_rules! laminas {
        ($($variante:ident => $archivo:literal $(@ $unica:ident)?),* $(,)?) => {
            #[derive(Clone, Copy, PartialEq)]
            pub enum Lamina { $($variante),* }
            pub const TODAS: &[(Lamina, &str, bool)] = &[
                $((Lamina::$variante, $archivo, unica!($($unica)?))),*
            ];
        };
    }
    include!("src/catalogo_laminas.rs");
}

fn preparar(
    lamina: laminas::Lamina,
    nombre: &str,
    unica: bool,
    tinta: bool,
    salida: &Path,
    codigo: SystemTime,
) {
    use laminas::Lamina;
    let (fuente, recorte, invertir) = match lamina {
        Lamina::Faeton => ("assets/originales/faeton.webp".to_owned(), None, tinta),
        Lamina::HermesHilos => (
            "assets/originales/hermes-hilos.webp".to_owned(),
            None,
            false,
        ),
        _ => match originales::recorte(lamina) {
            Some((fuente, recorte)) => (fuente.to_owned(), Some(recorte), tinta),
            None => (
                format!(
                    "assets/laminas/{nombre}{}.webp",
                    if tinta && !unica { "-negativo" } else { "" }
                ),
                // El grabado de Apolo trae un margen de papel irregular.
                (lamina == Lamina::ApoloBelvedere).then_some([0.045, 0.018, 0.965, 0.98]),
                false,
            ),
        },
    };
    let tema = if tinta { "tinta" } else { "piedra" };
    let png = salida.join(format!("{nombre}-{tema}.png"));
    let rgba = salida.join(format!("{nombre}-{tema}.rgba"));
    let cambiado = fs::metadata(&fuente)
        .unwrap()
        .modified()
        .unwrap()
        .max(codigo);
    if [&png, &rgba].iter().all(|p| {
        fs::metadata(p)
            .and_then(|m| m.modified())
            .is_ok_and(|t| t >= cambiado)
    }) {
        return;
    }
    let fuente = fs::read(&fuente).expect("leer original del grabado");
    let imagen = preparar_imagenes::preparar(&fuente, recorte, invertir, lamina == Lamina::Faeton)
        .unwrap_or_else(|e| panic!("Preparar {nombre}: {e}"));
    PngEncoder::new_with_quality(
        File::create(png).unwrap(),
        CompressionType::Fast,
        FilterType::Sub,
    )
    .write_image(
        imagen.as_raw(),
        imagen.width(),
        imagen.height(),
        ColorType::Rgba8,
    )
    .expect("guardar grabado preparado");
    // Píxeles pequeños listos para el primer cuadro: sin decodificar ni esperar
    // al trabajador de alta resolución y sin archivos externos a la app.
    let previa = image::DynamicImage::ImageRgba8(imagen)
        .thumbnail(320, 320)
        .into_rgba8();
    let mut archivo = File::create(rgba).unwrap();
    archivo.write_all(&previa.width().to_le_bytes()).unwrap();
    archivo.write_all(&previa.height().to_le_bytes()).unwrap();
    archivo.write_all(previa.as_raw()).unwrap();
}

fn main() {
    let fuentes = [
        "build.rs",
        "preparar_imagenes.rs",
        "src/catalogo_laminas.rs",
        "src/originales.rs",
    ];
    let mut codigo = SystemTime::UNIX_EPOCH;
    for fuente in fuentes {
        println!("cargo:rerun-if-changed={fuente}");
        codigo = codigo.max(fs::metadata(fuente).unwrap().modified().unwrap());
    }
    println!("cargo:rerun-if-changed=assets/laminas");
    println!("cargo:rerun-if-changed=assets/originales");
    let salida = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("imagenes");
    fs::create_dir_all(&salida).unwrap();
    let siguiente = AtomicUsize::new(0);
    let hilos = std::thread::available_parallelism().map_or(2, |n| n.get().min(4));
    std::thread::scope(|scope| {
        for _ in 0..hilos {
            scope.spawn(|| loop {
                let indice = siguiente.fetch_add(1, Ordering::Relaxed);
                let Some(&(lamina, nombre, unica)) = laminas::TODAS.get(indice) else {
                    break;
                };
                for tinta in [false, true] {
                    preparar(lamina, nombre, unica, tinta, &salida, codigo);
                }
            });
        }
    });
    println!("cargo:rerun-if-changed=assets/icono/icaro.ico");
    println!("cargo:rerun-if-changed=assets/icono/icaro.rc");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let _ = embed_resource::compile_for_examples("assets/icono/icaro.rc", embed_resource::NONE)
            .manifest_optional();
    }
}
