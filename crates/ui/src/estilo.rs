// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Funciones de estilo de iced a partir de la paleta. Todo con radio cero.

use iced::widget::{button, container, text_input};
use iced::{Background, Border, Color, Theme};

use crate::tema::{borde, Paleta};

fn contorno(ancho: f32, color: Color) -> Border {
    Border {
        width: ancho,
        color,
        radius: 0.0.into(),
    }
}

/// Botón primario: bloque de acento; en hover y presionado cambia de relleno.
pub fn boton_primario(p: Paleta) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, estado| {
        let fondo = match estado {
            button::Status::Hovered => p.accent_hover,
            button::Status::Pressed => p.accent_pressed,
            button::Status::Disabled => p.surface_sunken,
            button::Status::Active => p.accent,
        };
        let texto = if estado == button::Status::Disabled {
            p.text_disabled
        } else {
            p.on_accent
        };
        button::Style {
            background: Some(Background::Color(fondo)),
            text_color: texto,
            border: contorno(borde::MEDIO, fondo),
            ..Default::default()
        }
    }
}

/// Botón secundario: contorno de tinta que se invierte al pasar el puntero.
pub fn boton_secundario(p: Paleta) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, estado| {
        let invertido = matches!(estado, button::Status::Hovered | button::Status::Pressed);
        let desactivado = estado == button::Status::Disabled;
        let tinta = if desactivado { p.border } else { p.text };
        button::Style {
            background: invertido.then_some(Background::Color(p.text)),
            text_color: if invertido {
                p.bg
            } else if desactivado {
                p.text_disabled
            } else {
                p.text
            },
            border: contorno(borde::MEDIO, tinta),
            ..Default::default()
        }
    }
}

/// Botón fantasma: sin contorno; el fondo aparece al pasar el puntero.
pub fn boton_fantasma(p: Paleta) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, estado| {
        let fondo = match estado {
            button::Status::Hovered => Some(Background::Color(p.surface_hover)),
            button::Status::Pressed => Some(Background::Color(p.accent_soft)),
            _ => None,
        };
        button::Style {
            background: fondo,
            text_color: if estado == button::Status::Disabled {
                p.text_disabled
            } else {
                p.text
            },
            border: contorno(borde::MEDIO, Color::TRANSPARENT),
            ..Default::default()
        }
    }
}

/// Botón peligroso: contorno cinabrio que se rellena al pasar el puntero.
pub fn boton_peligro(p: Paleta) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, estado| {
        let lleno = matches!(estado, button::Status::Hovered | button::Status::Pressed);
        let desactivado = estado == button::Status::Disabled;
        let tinta = if desactivado { p.border } else { p.error };
        button::Style {
            background: lleno.then_some(Background::Color(p.error)),
            text_color: if lleno {
                p.on_error
            } else if desactivado {
                p.text_disabled
            } else {
                p.error
            },
            border: contorno(borde::MEDIO, tinta),
            ..Default::default()
        }
    }
}

/// Tarjeta en reposo (regla fina) o seleccionada (regla media de tinta).
pub fn tarjeta(p: Paleta, seleccionada: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.surface)),
        border: if seleccionada {
            contorno(borde::MEDIO, p.text)
        } else {
            contorno(borde::FINO, p.border)
        },
        ..Default::default()
    }
}

/// Superficie que flota: diálogos, menús y avisos, con contorno de tinta.
pub fn flotante(p: Paleta) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.surface_raised)),
        border: contorno(borde::MEDIO, p.text),
        ..Default::default()
    }
}

/// Fondo liso de la ventana.
pub fn fondo(p: Paleta) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.bg)),
        text_color: Some(p.text),
        ..Default::default()
    }
}

/// Campo de texto: pozo hundido con contorno fuerte; el foco engrosa la regla.
pub fn campo(p: Paleta) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_, estado| {
        let (ancho, color) = match estado {
            text_input::Status::Focused => (borde::MEDIO, p.focus),
            text_input::Status::Hovered => (borde::MEDIO, p.text),
            text_input::Status::Disabled => (borde::FINO, p.border),
            text_input::Status::Active => (borde::MEDIO, p.border_strong),
        };
        text_input::Style {
            background: Background::Color(p.surface_sunken),
            border: contorno(ancho, color),
            icon: p.text_muted,
            placeholder: p.text_muted,
            value: if estado == text_input::Status::Disabled {
                p.text_disabled
            } else {
                p.text
            },
            selection: p.selection,
        }
    }
}

/// Insignia de estado: relleno suave y regla fina del color del estado.
pub fn insignia(fondo: Color, tinta: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(fondo)),
        text_color: Some(tinta),
        border: contorno(borde::FINO, tinta),
        ..Default::default()
    }
}
