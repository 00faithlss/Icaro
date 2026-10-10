// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Iconos Pixelarticons (MIT). Se dibujan en una grilla de 24 con píxeles de
//! 2 unidades, así que solo se usan en múltiplos de 12 px.

use iced::widget::svg::{self, Handle, Svg};
use iced::{Color, Length, Theme};

macro_rules! iconos {
    ($($variante:ident => $archivo:literal),* $(,)?) => {
        /// Iconos disponibles.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Icono { $($variante),* }

        impl Icono {
            const fn bytes(self) -> &'static [u8] {
                match self {
                    $(Icono::$variante => include_bytes!(concat!("../assets/iconos/", $archivo, ".svg"))),*
                }
            }
        }
    };
}

iconos! {
    Instancias => "blocks",
    Servidores => "server",
    Descargas => "download",
    Consola => "terminal",
    Mods => "tool-case",
    Capturas => "camera",
    Ajustes => "sliders",
    Jugar => "play",
    Buscar => "search",
    Mas => "plus",
    Menos => "minus",
    Cerrar => "close",
    Flecha => "chevron-up",
    ChevronAbajo => "chevron-down",
    ChevronDerecha => "chevron-right",
    Carpeta => "folder",
    Editar => "pencil",
    Subir => "upload",
    Acciones => "more-vertical",
    Actualizar => "reload",
    Aviso => "warning-diamond",
    Check => "check",
    Detener => "stop",
    Espera => "hourglass",
    Arrastrar => "drag-and-drop",
    Alerta => "square-alert",
    Papelera => "trash",
    Copiar => "copy",
    Externo => "external-link",
    Pausa => "pause",
    Candado => "lock",
    Info => "circle-info",
}

/// Tamaños permitidos, múltiplos del píxel de arte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tam {
    /// 12 px.
    Sm,
    /// 24 px.
    Base,
    /// 36 px.
    Md,
    /// 48 px.
    Lg,
}

impl Tam {
    pub const fn px(self) -> f32 {
        match self {
            Tam::Sm => 12.0,
            Tam::Base => 24.0,
            Tam::Md => 36.0,
            Tam::Lg => 48.0,
        }
    }
}

/// Icono de una sola tinta.
pub fn icono<'a>(icono: Icono, tam: Tam, color: Color) -> Svg<'a, Theme> {
    svg(Handle::from_memory(icono.bytes()))
        .width(Length::Fixed(tam.px()))
        .height(Length::Fixed(tam.px()))
        .style(move |_: &Theme, _| svg::Style { color: Some(color) })
}

fn svg<'a>(handle: Handle) -> Svg<'a, Theme> {
    svg::Svg::new(handle)
}
