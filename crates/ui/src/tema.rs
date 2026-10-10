// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Tokens del sistema de diseño de Ícaro: paletas, espaciado, medidas,
//! escala tipográfica y duraciones. Los nombres siguen `tokens.json`.

use iced::Color;

/// Color opaco a partir de `0xRRGGBB`.
pub const fn hex(v: u32) -> Color {
    Color::from_rgb(
        ((v >> 16) & 0xff) as f32 / 255.0,
        ((v >> 8) & 0xff) as f32 / 255.0,
        (v & 0xff) as f32 / 255.0,
    )
}

/// Color con transparencia a partir de `0xRRGGBB` y un alfa de 0 a 1.
pub const fn hex_a(v: u32, alfa: f32) -> Color {
    Color::from_rgba(
        ((v >> 16) & 0xff) as f32 / 255.0,
        ((v >> 8) & 0xff) as f32 / 255.0,
        (v & 0xff) as f32 / 255.0,
        alfa,
    )
}

/// Tema de la interfaz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Modo {
    /// Oscuro.
    #[default]
    Tinta,
    /// Claro.
    Piedra,
}

impl Modo {
    pub const fn nombre(self) -> &'static str {
        match self {
            Modo::Tinta => "Tinta",
            Modo::Piedra => "Piedra",
        }
    }

    pub const fn paleta(self) -> Paleta {
        match self {
            Modo::Tinta => TINTA,
            Modo::Piedra => PIEDRA,
        }
    }
}

/// Paleta completa de un tema.
#[derive(Debug, Clone, Copy)]
pub struct Paleta {
    pub modo: Modo,
    pub bg: Color,
    pub surface: Color,
    pub surface_raised: Color,
    pub surface_sunken: Color,
    pub surface_hover: Color,
    pub border: Color,
    pub border_strong: Color,
    pub text: Color,
    pub text_muted: Color,
    pub text_disabled: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,
    pub on_accent: Color,
    pub accent_soft: Color,
    pub accent_text: Color,
    pub fulgor: Color,
    pub on_fulgor: Color,
    pub success: Color,
    pub success_soft: Color,
    pub warning: Color,
    pub warning_soft: Color,
    pub error: Color,
    pub error_soft: Color,
    pub on_error: Color,
    pub info: Color,
    pub info_soft: Color,
    pub focus: Color,
    pub scrim: Color,
    pub selection: Color,
}

/// Tema oscuro.
pub const TINTA: Paleta = Paleta {
    modo: Modo::Tinta,
    bg: hex(0x0a0a0a),
    surface: hex(0x111111),
    surface_raised: hex(0x191919),
    surface_sunken: hex(0x050505),
    surface_hover: hex(0x222222),
    border: hex(0x3a3a37),
    border_strong: hex(0x8a8a85),
    text: hex(0xecece8),
    text_muted: hex(0xc4c4be),
    text_disabled: hex(0x8a8a85),
    accent: hex(0xf4f4f0),
    accent_hover: hex(0xffffff),
    accent_pressed: hex(0xcfcfca),
    on_accent: hex(0x0a0a0a),
    accent_soft: hex(0x262624),
    accent_text: hex(0xffffff),
    fulgor: hex(0xffffff),
    on_fulgor: hex(0x0a0a0a),
    success: hex(0x6cc9b1),
    success_soft: hex(0x10261f),
    warning: hex(0xe3b04b),
    warning_soft: hex(0x2a2010),
    error: hex(0xff7b6b),
    error_soft: hex(0x331512),
    on_error: hex(0x0a0a0a),
    info: hex(0x9db0ff),
    info_soft: hex(0x161a33),
    focus: hex(0xffffff),
    scrim: hex_a(0x000000, 0.72),
    selection: hex(0x3d3d3a),
};

/// Tema claro.
pub const PIEDRA: Paleta = Paleta {
    modo: Modo::Piedra,
    bg: hex(0xe6e6e3),
    surface: hex(0xf4f4f2),
    surface_raised: hex(0xffffff),
    surface_sunken: hex(0xd8d8d4),
    surface_hover: hex(0xececea),
    border: hex(0xbdbdb8),
    border_strong: hex(0x6e6e69),
    text: hex(0x0d0d0d),
    text_muted: hex(0x2f2f2d),
    text_disabled: hex(0x6e6e69),
    accent: hex(0x0d0d0d),
    accent_hover: hex(0x2a2a28),
    accent_pressed: hex(0x000000),
    on_accent: hex(0xf4f4f2),
    accent_soft: hex(0xdcdcd8),
    accent_text: hex(0x0d0d0d),
    fulgor: hex(0xffffff),
    on_fulgor: hex(0x0a0a0a),
    success: hex(0x1d6b5c),
    success_soft: hex(0xd5ebe5),
    warning: hex(0x855400),
    warning_soft: hex(0xf2e3c4),
    error: hex(0xb02a1c),
    error_soft: hex(0xf5d9d4),
    on_error: hex(0xffffff),
    info: hex(0x2d46a0),
    info_soft: hex(0xdbe0f5),
    focus: hex(0x0d0d0d),
    scrim: hex_a(0x0a0a0a, 0.55),
    selection: hex(0xc8c8c3),
};

/// Escala de espaciado, en píxeles.
pub mod espacio {
    pub const S1: f32 = 4.0;
    pub const S2: f32 = 8.0;
    pub const S3: f32 = 12.0;
    pub const S4: f32 = 16.0;
    pub const S5: f32 = 20.0;
    pub const S6: f32 = 24.0;
    pub const S8: f32 = 32.0;
    pub const S12: f32 = 48.0;
    pub const S16: f32 = 64.0;
    pub const S24: f32 = 96.0;
}

/// Ícaro no redondea esquinas.
pub mod radio {
    pub const CERO: f32 = 0.0;
}

/// Grosores de regla.
pub mod borde {
    pub const FINO: f32 = 1.0;
    pub const MEDIO: f32 = 2.0;
    pub const GRUESO: f32 = 3.0;
}

/// Medidas fijas de la ventana y los controles.
pub mod medida {
    pub const CONTROL_SM: f32 = 28.0;
    pub const CONTROL: f32 = 36.0;
    pub const CONTROL_LG: f32 = 44.0;
    pub const CONTROL_JUGAR: f32 = 56.0;
    pub const BARRA_TITULO: f32 = 36.0;
    pub const BARRA_LATERAL: f32 = 232.0;
    pub const BARRA_LATERAL_COLAPSADA: f32 = 64.0;
    pub const PANEL_LATERAL: f32 = 640.0;
    pub const VENTANA_MIN: f32 = 940.0;
    pub const ICONO_SM: f32 = 12.0;
    pub const ICONO: f32 = 24.0;
    pub const ICONO_LG: f32 = 48.0;
}

/// Escala tipográfica: tamaño y alto de línea, en píxeles.
pub mod texto {
    pub const DISPLAY_XL: (f32, f32) = (64.0, 60.0);
    pub const DISPLAY: (f32, f32) = (40.0, 40.0);
    pub const TITLE: (f32, f32) = (24.0, 28.0);
    pub const HEADING: (f32, f32) = (16.0, 22.0);
    pub const BODY: (f32, f32) = (14.0, 21.0);
    pub const BODY_SM: (f32, f32) = (13.0, 19.0);
    pub const LABEL: (f32, f32) = (11.0, 14.0);
    pub const LEYENDA: (f32, f32) = (13.0, 18.0);
    pub const MONO: (f32, f32) = (13.0, 20.0);
    pub const MONO_SM: (f32, f32) = (11.0, 16.0);
    pub const CIFRA: (f32, f32) = (32.0, 36.0);
    pub const CODE_XL: (f32, f32) = (28.0, 32.0);
}

/// Duraciones de movimiento, en milisegundos.
pub mod duracion {
    pub const INSTANT: u64 = 60;
    pub const FAST: u64 = 120;
    pub const BASE: u64 = 200;
    pub const SLOW: u64 = 320;
    pub const LOSA: u64 = 280;
    pub const RASTRILLO: u64 = 320;
    pub const MURO: u64 = 240;
    pub const PRENSA: u64 = 420;
    pub const PICAR: u64 = 1200;
    pub const UMBRAL: u64 = 1400;
}
