// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Fuentes del sistema: Archivo para la interfaz y Martian Mono para datos.
//! Son instancias estáticas de las fuentes variables originales (licencia OFL).

use iced::font::{Family, Stretch, Style, Weight};
use iced::Font;

const fn nombre(familia: &'static str) -> Font {
    Font {
        family: Family::Name(familia),
        weight: Weight::Normal,
        stretch: Stretch::Normal,
        style: Style::Normal,
    }
}

/// Texto corrido.
pub const CUERPO: Font = nombre("Archivo");
/// Texto con énfasis (`heading`).
pub const CUERPO_NEGRITA: Font = nombre("Archivo SemiBold");
/// `label` y botones.
pub const ETIQUETA: Font = nombre("Archivo SemiExpanded");
/// `title`.
pub const TITULO: Font = nombre("Archivo Expanded Bold");
/// `display` y `display-xl`.
pub const DISPLAY: Font = nombre("Archivo Expanded");
/// Datos, consola y versiones.
pub const MONO: Font = nombre("Martian Mono");
/// Cifras protagonistas.
pub const MONO_MEDIO: Font = nombre("Martian Mono Medium");

/// Bytes de las fuentes, para registrarlas al iniciar la aplicación.
pub const BYTES: [&[u8]; 7] = [
    include_bytes!("../assets/fonts/Archivo-Regular.ttf"),
    include_bytes!("../assets/fonts/Archivo-SemiBold.ttf"),
    include_bytes!("../assets/fonts/ArchivoSemiExpanded-SemiBold.ttf"),
    include_bytes!("../assets/fonts/ArchivoExpanded-Bold.ttf"),
    include_bytes!("../assets/fonts/ArchivoExpanded-ExtraBold.ttf"),
    include_bytes!("../assets/fonts/MartianMono-Regular.ttf"),
    include_bytes!("../assets/fonts/MartianMono-Medium.ttf"),
];
