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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seleccion_y_botones_invierten_sin_grises_ni_perder_contornos() {
        for p in [crate::tema::TINTA, crate::tema::PIEDRA] {
            for estado in [button::Status::Hovered, button::Status::Pressed] {
                for activo in [false, true] {
                    for s in [chip(p, activo)(&Theme::Dark, estado), segmento(p, activo)(&Theme::Dark, estado)] {
                        assert_eq!(s.background, Some(Background::Color(if activo { p.bg } else { p.text })));
                        assert_eq!(s.text_color, if activo { p.text } else { p.bg });
                        assert_eq!(s.border.color, p.text);
                        assert!(s.border.width > 0.);
                    }
                }
                let s = boton_primario(p)(&Theme::Dark, estado);
                assert_eq!(s.background, Some(Background::Color(p.bg)));
                assert_eq!(s.text_color, p.accent);
                assert_eq!(s.border.color, p.accent);
                let s = boton_fantasma(p)(&Theme::Dark, estado);
                assert_eq!(s.background, Some(Background::Color(p.text)));
                assert_eq!(s.text_color, p.bg);
                assert_eq!(s.border.color, p.text);
            }
            for s in [chip(p, true)(&Theme::Dark, button::Status::Disabled), segmento(p, false)(&Theme::Dark, button::Status::Disabled)] {
                assert_eq!(s.text_color, p.text_disabled);
                assert_eq!(s.border.color, p.border);
            }
        }
    }

    fn luminancia(c: Color) -> f32 {
        let lineal = |v: f32| if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
        0.2126 * lineal(c.r) + 0.7152 * lineal(c.g) + 0.0722 * lineal(c.b)
    }

    #[test]
    fn todos_los_botones_conservan_contraste_en_reposo_hover_y_presionado() {
        for p in [crate::tema::TINTA, crate::tema::PIEDRA] {
            for estado in [button::Status::Active, button::Status::Hovered, button::Status::Pressed] {
                for estilo in [boton_primario(p)(&Theme::Dark, estado), boton_secundario(p)(&Theme::Dark, estado),
                    boton_fantasma(p)(&Theme::Dark, estado), boton_peligro(p)(&Theme::Dark, estado)] {
                    let fondo = match estilo.background {
                        Some(Background::Color(c)) => c,
                        None => p.bg,
                        _ => panic!("fondo inesperado"),
                    };
                    let a = luminancia(estilo.text_color);
                    let b = luminancia(fondo);
                    let contraste = (a.max(b) + 0.05) / (a.min(b) + 0.05);
                    assert!(contraste >= 4.5, "{:?} {estado:?}: {contraste}", p.modo);
                }
            }
        }
    }
}

/// Botón primario: bloque de acento; el puntero invierte tinta y papel sin borrar la regla.
pub fn boton_primario(p: Paleta) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, estado| {
        let invertido = matches!(estado, button::Status::Hovered | button::Status::Pressed);
        let desactivado = estado == button::Status::Disabled;
        let fondo = if desactivado { p.surface_sunken } else if invertido { p.bg } else { p.accent };
        let texto = if desactivado { p.text_disabled } else if invertido { p.accent } else { p.on_accent };
        button::Style {
            background: Some(Background::Color(fondo)),
            text_color: texto,
            border: contorno(borde::MEDIO, if desactivado { p.border } else { p.accent }),
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
        let invertido = matches!(estado, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: invertido.then_some(Background::Color(p.text)),
            text_color: if estado == button::Status::Disabled { p.text_disabled } else if invertido { p.bg } else { p.text },
            border: contorno(borde::MEDIO, if invertido { p.text } else { Color::TRANSPARENT }),
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

/// Chip filtrable: la selección y el puntero invierten el bloque sin perder el contorno.
pub fn chip(p: Paleta, activo: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, estado| opcion_monocroma(p, activo, estado)
}

/// Opción segmentada: conserva la regla y usa tinta/papel en todos sus estados.
pub fn segmento(p: Paleta, activo: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, estado| opcion_monocroma(p, activo, estado)
}

fn opcion_monocroma(p: Paleta, activo: bool, estado: button::Status) -> button::Style {
    let desactivado = estado == button::Status::Disabled;
    let sobre = matches!(estado, button::Status::Hovered | button::Status::Pressed);
    let lleno = activo != sobre;
    button::Style {
        background: (!desactivado).then_some(Background::Color(if lleno { p.text } else { p.bg })),
        text_color: if desactivado { p.text_disabled } else if lleno { p.bg } else { p.text },
        border: contorno(borde::MEDIO, if desactivado { p.border } else { p.text }),
        ..Default::default()
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

/// Botón del pie de una tarjeta: contorno de tinta, con relleno opcional
/// (la versión invertida que se muestra al pasar el puntero).
pub fn boton_pie(
    tinta: Color,
    relleno: Option<Color>,
    activo: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, _| button::Style {
        background: relleno.map(Background::Color),
        text_color: tinta,
        border: contorno(
            if activo { borde::MEDIO } else { 0.0 },
            relleno.unwrap_or(tinta),
        ),
        ..Default::default()
    }
}
