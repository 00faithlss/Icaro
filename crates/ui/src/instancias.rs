// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Tarjeta de instancia: portada, nombre, versión, estado y acciones.

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Color, Element, Length, Padding};

use crate::componentes::{insignia, progreso_bloques, Estado};
use crate::estilo;
use crate::fuentes;
use crate::iconos::{icono, Icono, Tam};
use crate::laminas::Lamina;
use crate::tema::{borde, espacio, texto, Paleta};

/// Alto de la portada dentro de la tarjeta.
pub const ALTO_PORTADA: f32 = 131.0;

/// Alto de la tarjeta; fijo para que toda la cuadrícula quede pareja.
pub const ALTO_TARJETA: f32 = 340.0;
/// Ancho fijo de la tarjeta: no crece al maximizar la ventana.
pub const ANCHO_TARJETA: f32 = 300.0;

/// Estado visible de una instancia.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EstadoInstancia {
    Lista,
    Desactualizada,
    /// Avance de 0 a 1.
    Instalando(f32),
    Jugando,
    Error,
}

/// Datos que muestra una tarjeta de instancia.
#[derive(Debug, Clone)]
pub struct DatosInstancia {
    pub nombre: String,
    /// Versión y cargador, por ejemplo `1.21.4 · FABRIC`.
    pub version: String,
    pub ultima_vez: String,
    pub mods: String,
    pub portada: Lamina,
    pub estado: EstadoInstancia,
    pub seleccionada: bool,
    /// Texto del estado: causa del error, paso de la instalación o tiempo
    /// de sesión.
    pub detalle: String,
}

/// Mensajes que emite la tarjeta.
#[derive(Debug, Clone)]
pub struct AccionesTarjeta<M> {
    pub seleccionar: M,
    pub principal: M,
    pub mas: M,
}

/// Botón del pie: ghost, sin contorno; el fondo aparece al pasar el puntero.
fn pie_fantasma<'a, M: Clone + 'a>(
    p: Paleta,
    contenido: Option<(Icono, String)>,
    tinta: Color,
    mensaje: Option<M>,
    invertido: bool,
) -> iced::widget::Button<'a, M> {
    let mut fila = row![].spacing(espacio::S2).align_y(Alignment::Center);
    match contenido {
        Some((g, e)) => {
            fila = fila.push(icono(g, Tam::Base, tinta)).push(
                text(e.to_uppercase()).font(fuentes::ETIQUETA).size(texto::LABEL.0).color(tinta),
            );
        }
        None => fila = fila.push(icono(Icono::Acciones, Tam::Base, tinta)),
    }
    let activo = mensaje.is_some();
    button(container(fila).center(Length::Fill))
        .height(Length::Fill)
        .padding(0)
        .on_press_maybe(mensaje)
        .style(move |_, estado| {
            let encima = activo && matches!(estado, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: encima.then_some(iced::Background::Color(if invertido {
                    Color { a: 0.18, ..p.bg }
                } else {
                    p.surface_hover
                })),
                text_color: tinta,
                ..Default::default()
            }
        })
}

/// Icono procedural de 48 px: una trama de 8 × 8 píxeles simétrica sacada del
/// nombre, estampada sobre el borde de la portada.
fn sello<'a, M: 'a>(p: Paleta, nombre: &str) -> Element<'a, M> {
    let mut h: u32 = 2_166_136_261;
    for b in nombre.bytes() {
        h = (h ^ u32::from(b)).wrapping_mul(16_777_619);
    }
    let mut estado = h;
    let mut siguiente = move || {
        estado ^= estado << 13;
        estado ^= estado >> 17;
        estado ^= estado << 5;
        estado
    };
    let mut col = column![];
    for _ in 0..8 {
        let mut mitad = [0u32; 4];
        for m in mitad.iter_mut() {
            *m = siguiente() % 3;
        }
        let mut fila = row![];
        for i in 0..8 {
            let v = mitad[if i < 4 { i } else { 7 - i }];
            let color = match v {
                0 => p.bg,
                1 => p.text_muted,
                _ => p.text,
            };
            fila = fila.push(container(Space::new(6.0, 6.0)).style(estilo::bloque(color)));
        }
        col = col.push(fila);
    }
    container(col)
        .padding(2)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(p.bg)),
            border: iced::Border { color: p.text, width: 2.0, radius: 0.0.into() },
            ..Default::default()
        })
        .into()
}

/// Tarjeta de instancia con portada de grabado y pie de acciones.
pub fn tarjeta_instancia<'a, M: Clone + 'a>(
    p: Paleta,
    datos: &DatosInstancia,
    acciones: AccionesTarjeta<M>,
) -> Element<'a, M> {
    let portada = crate::laminas::grabado_zoom(p, datos.portada, Length::Fill, ALTO_PORTADA);

    let mut cuerpo = column![
        text(datos.nombre.clone())
            .font(fuentes::TITULO)
            .size(texto::HEADING.0)
            .color(p.text),
        text(datos.version.to_uppercase())
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted),
    ]
    .spacing(espacio::S1);

    cuerpo = cuerpo.push(Space::with_height(espacio::S1));
    cuerpo = match datos.estado {
        EstadoInstancia::Lista => cuerpo.push(insignia(p, Estado::Exito, "Lista")),
        EstadoInstancia::Desactualizada => {
            cuerpo.push(insignia(p, Estado::Aviso, "Desactualizada"))
        }
        EstadoInstancia::Instalando(avance) => cuerpo
            .push(progreso_bloques(p, 10, (avance * 10.0).round() as usize))
            .push(
                text(datos.detalle.clone())
                    .size(texto::BODY_SM.0)
                    .color(p.text_muted),
            ),
        EstadoInstancia::Jugando => cuerpo.push(
            row![
                container(Space::new(8, 8)).style(estilo::bloque(p.success)),
                text(datos.detalle.clone())
                    .size(texto::BODY_SM.0)
                    .color(p.text),
            ]
            .spacing(espacio::S2)
            .align_y(Alignment::Center),
        ),
        EstadoInstancia::Error => cuerpo.push(insignia(p, Estado::Error, &datos.detalle)),
    };
    cuerpo = cuerpo.push(
        row![
            text(datos.ultima_vez.clone())
                .size(texto::BODY_SM.0)
                .color(p.text_muted),
            text(datos.mods.clone())
                .size(texto::BODY_SM.0)
                .color(p.text_muted),
        ]
        .spacing(espacio::S4),
    );

    let invertido = datos.estado == EstadoInstancia::Jugando;
    let tinta = if invertido { p.bg } else { p.text };
    let (glifo, etiqueta, color, activo) = match datos.estado {
        EstadoInstancia::Lista => (Icono::Jugar, "Jugar".to_owned(), tinta, true),
        EstadoInstancia::Desactualizada => {
            (Icono::Actualizar, "Actualizar".to_owned(), tinta, true)
        }
        EstadoInstancia::Instalando(avance) => (
            Icono::Espera,
            format!("Instalando {} %", (avance * 100.0).round()),
            p.text_muted,
            false,
        ),
        EstadoInstancia::Jugando => (Icono::Detener, "Detener".to_owned(), tinta, true),
        EstadoInstancia::Error => (Icono::Instancias, "Reparar".to_owned(), p.error, true),
    };
    let regla_pie = if invertido { Color { a: 0.3, ..p.bg } } else { p.border };
    let pie = row![
        pie_fantasma(p, Some((glifo, etiqueta)), color, activo.then(|| acciones.principal.clone()), invertido)
            .width(Length::Fill),
        container(Space::new(1.0, Length::Fill)).style(estilo::bloque(regla_pie)),
        pie_fantasma(p, None, tinta, Some(acciones.mas.clone()), invertido).width(44),
    ]
    .height(44);
    let pie = column![
        container(Space::new(Length::Fill, 1.0)).style(estilo::bloque(
            if datos.estado == EstadoInstancia::Error { p.error } else { regla_pie }
        )),
        container(pie).style(estilo::bloque(if invertido { p.text } else { Color::TRANSPARENT })),
    ];

    let interior = column![
        portada,
        container(cuerpo)
            .padding(Padding { top: 36.0, right: 20.0, bottom: 16.0, left: 20.0 })
            .height(Length::Fill),
        pie,
    ];
    let interior = iced::widget::stack![
        interior,
        container(sello(p, &datos.nombre)).padding(Padding { top: ALTO_PORTADA - 24.0, left: 20.0, ..Padding::ZERO }),
    ];

    // Contorno: 1 px en reposo, 2 px seleccionada, 3 px en ejecución, tono de error.
    let (ancho, color_marco) = match datos.estado {
        EstadoInstancia::Jugando => (borde::GRUESO, p.text),
        EstadoInstancia::Error => (borde::MEDIO, p.error),
        _ if datos.seleccionada => (borde::MEDIO, p.text),
        _ => (borde::FINO, p.border),
    };
    button(
        container(interior)
            .width(Length::Fill)
            .height(ALTO_TARJETA)
            .style(estilo::tarjeta_marco(p, ancho, color_marco)),
    )
    .padding(0)
    .width(ANCHO_TARJETA)
    .height(ALTO_TARJETA)
    .on_press(acciones.seleccionar)
    .style(estilo::sin_estilo(p.text))
    .into()
}

/// Tile de creación al final de la cuadrícula: rayado de grabado y signo más.
pub fn tarjeta_nueva<'a, M: Clone + 'a>(p: Paleta, al_pulsar: M) -> Element<'a, M> {
    let contenido = column![
        icono(Icono::Mas, Tam::Md, p.text),
        text("NUEVA INSTANCIA")
            .font(fuentes::ETIQUETA)
            .size(texto::LABEL.0)
            .color(p.text),
    ]
    .spacing(espacio::S3)
    .align_x(Alignment::Center);
    button(
        container(contenido)
            .center(Length::Fill)
            .style(estilo::tarjeta(p, false)),
    )
    .padding(0)
    .width(ANCHO_TARJETA)
    .height(ALTO_TARJETA)
    .on_press(al_pulsar)
    .style(estilo::sin_estilo(p.text))
    .into()
}
