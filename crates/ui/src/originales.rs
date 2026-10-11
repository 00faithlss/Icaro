// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Encuadres del Design System sobre los originales suministrados.
//! Se incluyen únicamente fuentes con una ganancia real de resolución.

use crate::laminas::Lamina;

pub(crate) fn recorte(lamina: Lamina) -> Option<(&'static str, [f32; 4])> {
    Some(match lamina {
        Lamina::FortalezasValle => (
            "assets/originales/fortalezas.webp",
            [0.55, 0.33, 0.97, 0.86],
        ),
        Lamina::ProdigoAldea => (
            "assets/originales/hijo-prodigo.webp",
            [0.0, 0.02, 1.0, 0.48],
        ),
        Lamina::HuidaCiudad => (
            "assets/originales/huida-egipto.webp",
            [0.0, 0.3, 0.42, 0.72],
        ),
        Lamina::HuidaBosque => (
            "assets/originales/huida-egipto.webp",
            [0.38, 0.06, 0.76, 0.66],
        ),
        Lamina::MelencoliaReloj => (
            "assets/originales/melencolia.webp",
            [0.46, 0.035, 0.95, 0.31],
        ),
        Lamina::MelencoliaCometa => ("assets/originales/melencolia.webp", [0.03, 0.02, 0.5, 0.25]),
        Lamina::MercadoAldea => (
            "assets/originales/mercado-rural.webp",
            [0.35, 0.17, 1.0, 0.63],
        ),
        Lamina::MercadoCarros => (
            "assets/originales/mercado-rural.webp",
            [0.38, 0.64, 0.9, 0.96],
        ),
        Lamina::ArcoPuente => (
            "assets/originales/piranesi-arco.webp",
            [0.34, 0.44, 0.76, 0.96],
        ),
        Lamina::EscaleraPuentes => (
            "assets/originales/piranesi-escalera.webp",
            [0.34, 0.04, 0.96, 0.46],
        ),
        Lamina::FuegoArco => (
            "assets/originales/piranesi-fuego.webp",
            [0.3, 0.57, 0.96, 0.99],
        ),
        Lamina::PozoCuerdas => (
            "assets/originales/piranesi-pozo.webp",
            [0.07, 0.04, 0.41, 0.93],
        ),
        Lamina::PuenteTorre => (
            "assets/originales/piranesi-puente.webp",
            [0.59, 0.09, 0.99, 0.63],
        ),
        Lamina::RuedaRueda => (
            "assets/originales/piranesi-rueda.webp",
            [0.0, 0.0, 1.0, 0.56],
        ),
        Lamina::TorreReja => (
            "assets/originales/piranesi-torre.webp",
            [0.28, 0.28, 0.64, 0.74],
        ),
        Lamina::TivoliCiudad => ("assets/originales/tivoli.webp", [0.0, 0.14, 0.46, 0.51]),
        Lamina::TivoliRocas => ("assets/originales/tivoli.webp", [0.0, 0.44, 1.0, 0.93]),
        Lamina::ValleRocas => (
            "assets/originales/valle-alpino.webp",
            [0.49, 0.19, 0.86, 0.56],
        ),
        _ => return None,
    })
}
