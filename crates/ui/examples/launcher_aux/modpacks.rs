// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Catálogo de ejemplo de modpacks. Cada uno se instala como una instancia
//! nueva; todavía no hay conexión con un servicio real de descargas.

use icaro_ui::laminas::Lamina;

pub const CATEGORIAS: [&str; 5] = ["Todos", "Aventura", "Técnico", "Rendimiento", "Vanilla+"];

pub struct Modpack {
    pub nombre: &'static str,
    pub autor: &'static str,
    pub descripcion: &'static str,
    pub minecraft: &'static str,
    pub cargador: &'static str,
    pub mods: u32,
    pub tamano: &'static str,
    pub descargas: &'static str,
    /// Una de `CATEGORIAS` (sin "Todos").
    pub categoria: &'static str,
    pub portada: Lamina,
}

pub const CATALOGO: &[Modpack] = &[
    Modpack {
        nombre: "Fabulously Optimized",
        autor: "Fabulously Team",
        descripcion: "Rendimiento máximo con Sodium, Lithium e Iris, sin cambiar la jugabilidad.",
        minecraft: "1.21.4",
        cargador: "Fabric",
        mods: 62,
        tamano: "48 MB",
        descargas: "9,2 M",
        categoria: "Rendimiento",
        portada: Lamina::CascadaMolino,
    },
    Modpack {
        nombre: "Create: Above and Beyond",
        autor: "simibubi",
        descripcion: "Progresión guiada alrededor de máquinas, engranajes y automatización.",
        minecraft: "1.20.1",
        cargador: "Forge",
        mods: 198,
        tamano: "412 MB",
        descargas: "3,4 M",
        categoria: "Técnico",
        portada: Lamina::RuedaRueda,
    },
    Modpack {
        nombre: "Aventura Épica",
        autor: "Comunidad",
        descripcion: "Mazmorras, jefes nuevos y estructuras que cambian la exploración.",
        minecraft: "1.20.1",
        cargador: "NeoForge",
        mods: 241,
        tamano: "620 MB",
        descargas: "5,1 M",
        categoria: "Aventura",
        portada: Lamina::DragonAla,
    },
    Modpack {
        nombre: "Vanilla Plus",
        autor: "Mojang Fans",
        descripcion: "Pequeñas mejoras de calidad de vida que respetan el juego original.",
        minecraft: "1.21.4",
        cargador: "Fabric",
        mods: 34,
        tamano: "26 MB",
        descargas: "7,8 M",
        categoria: "Vanilla+",
        portada: Lamina::ValleBosque,
    },
    Modpack {
        nombre: "Industria Moderna",
        autor: "Taller Redstone",
        descripcion: "Energía, tuberías y fábricas a gran escala con herramientas de apoyo.",
        minecraft: "1.20.1",
        cargador: "Forge",
        mods: 167,
        tamano: "355 MB",
        descargas: "1,9 M",
        categoria: "Técnico",
        portada: Lamina::PozoCuerdas,
    },
    Modpack {
        nombre: "Exploración Salvaje",
        autor: "Rutas Abiertas",
        descripcion: "Biomas nuevos, fauna y mapas para recorrer el mundo sin prisa.",
        minecraft: "1.21.1",
        cargador: "Fabric",
        mods: 88,
        tamano: "140 MB",
        descargas: "2,6 M",
        categoria: "Aventura",
        portada: Lamina::HuidaBosque,
    },
];
