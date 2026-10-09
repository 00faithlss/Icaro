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

/// Chip filtrable: contorno fino; activo invierte a bloque de tinta.
pub fn chip(p: Paleta, activo: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, estado| {
        let sobre = matches!(estado, button::Status::Hovered | button::Status::Pressed);
        let (fondo, tinta) = if activo {
            (Some(p.text), p.bg)
        } else if sobre {
            (Some(p.surface_hover), p.text)
        } else {
            (None, p.text)
        };
        button::Style {
            background: fondo.map(Background::Color),
            text_color: tinta,
            border: contorno(borde::FINO, p.border_strong),
            ..Default::default()
        }
    }
}

/// Opción de un grupo segmentado: sin contorno propio, la activa es un bloque.
pub fn segmento(p: Paleta, activo: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, estado| {
        let sobre = matches!(estado, button::Status::Hovered | button::Status::Pressed);
        let (fondo, tinta) = if activo {
            (Some(p.text), p.bg)
        } else if sobre {
            (Some(p.surface_hover), p.text)
        } else {
            (None, p.text)
        };
        button::Style {
            background: fondo.map(Background::Color),
            text_color: tinta,
            border: Border::default(),
            ..Default::default()
        }
    }
}

/// Marco del grupo segmentado.
pub fn marco(p: Paleta) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        border: contorno(borde::MEDIO, p.text),
        ..Default::default()
    }
}

/// Casilla cuadrada: pozo apagado, bloque de acento encendido.
pub fn casilla(
    p: Paleta,
) -> impl Fn(&Theme, iced::widget::checkbox::Status) -> iced::widget::checkbox::Style {
    use iced::widget::checkbox::Status;
    move |_, estado| {
        let (marcada, sobre, desactivada) = match estado {
            Status::Active { is_checked } => (is_checked, false, false),
            Status::Hovered { is_checked } => (is_checked, true, false),
            Status::Disabled { is_checked } => (is_checked, false, true),
        };
        let color = if desactivada {
            p.border
        } else if sobre {
            p.text
        } else {
            p.border_strong
        };
        iced::widget::checkbox::Style {
            background: Background::Color(if marcada { p.accent } else { p.surface_sunken }),
            icon_color: p.on_accent,
            border: contorno(borde::MEDIO, if marcada { p.accent } else { color }),
            text_color: Some(if desactivada { p.text_disabled } else { p.text }),
        }
    }
}

/// Selector desplegable: disparador como un campo.
pub fn selector(
    p: Paleta,
) -> impl Fn(&Theme, iced::widget::pick_list::Status) -> iced::widget::pick_list::Style {
    use iced::widget::pick_list::Status;
    move |_, estado| {
        let color = match estado {
            Status::Active => p.border_strong,
            Status::Hovered => p.text_muted,
            Status::Opened => p.text,
        };
        iced::widget::pick_list::Style {
            text_color: p.text,
            placeholder_color: p.text_muted,
            handle_color: p.text,
            background: Background::Color(p.surface_sunken),
            border: contorno(borde::MEDIO, color),
        }
    }
}

/// Lista flotante del selector; la opción elegida invierte la tinta.
pub fn menu_selector(p: Paleta) -> impl Fn(&Theme) -> iced::overlay::menu::Style {
    move |_| iced::overlay::menu::Style {
        background: Background::Color(p.surface_raised),
        border: contorno(borde::MEDIO, p.text),
        text_color: p.text,
        selected_text_color: p.bg,
        selected_background: Background::Color(p.text),
    }
}

/// Barra de progreso continua, sin esquinas.
pub fn progreso(p: Paleta) -> impl Fn(&Theme) -> iced::widget::progress_bar::Style {
    move |_| iced::widget::progress_bar::Style {
        background: Background::Color(p.surface_sunken),
        bar: Background::Color(p.text),
        border: contorno(borde::FINO, p.border_strong),
    }
}

/// Bloque de la pista del interruptor.
pub fn pista(p: Paleta, encendido: bool, sobre: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(if encendido {
            p.accent
        } else {
            p.surface_sunken
        })),
        border: contorno(
            borde::MEDIO,
            if encendido {
                p.accent
            } else if sobre {
                p.text
            } else {
                p.border_strong
            },
        ),
        ..Default::default()
    }
}

/// Pieza cuadrada que se desplaza dentro de la pista.
pub fn perilla(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(color)),
        ..Default::default()
    }
}

/// Botón sin relleno para envolver controles propios (interruptor).
pub fn sin_estilo(texto: Color) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, _| button::Style {
        text_color: texto,
        ..Default::default()
    }
}

/// Tarjeta con contorno explícito: el grosor y el tono dicen el estado
/// (2 px seleccionada, 3 px en ejecución, tono de error).
pub fn tarjeta_marco(p: Paleta, ancho: f32, color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.surface)),
        border: contorno(ancho, color),
        ..Default::default()
    }
}

/// Bloque de color liso, sin contorno.
pub fn bloque(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(color)),
        ..Default::default()
    }
}
