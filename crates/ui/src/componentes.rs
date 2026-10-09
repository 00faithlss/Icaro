// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Componentes básicos construidos sobre los estilos del sistema.

use iced::widget::{
    button, checkbox, container, pick_list, progress_bar, row, text, text_input, Space,
};
use iced::{Element, Length, Padding};

use crate::estilo;
use crate::fuentes;
use crate::tema::{espacio, medida, texto, Paleta};

/// Variante visual de un botón.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variante {
    Primario,
    Secundario,
    Fantasma,
    Peligro,
}

/// Botón de texto de alto `control`. Sin mensaje queda deshabilitado.
pub fn boton<'a, M: Clone + 'a>(
    p: Paleta,
    etiqueta: &str,
    variante: Variante,
    al_pulsar: Option<M>,
) -> Element<'a, M> {
    let contenido = text(etiqueta.to_uppercase())
        .font(fuentes::ETIQUETA)
        .size(texto::LABEL.0);
    let b = button(
        container(contenido)
            .height(Length::Fill)
            .align_y(iced::alignment::Vertical::Center),
    )
    .height(medida::CONTROL)
    .padding(Padding::from([0.0, espacio::S4]))
    .on_press_maybe(al_pulsar);
    match variante {
        Variante::Primario => b.style(estilo::boton_primario(p)),
        Variante::Secundario => b.style(estilo::boton_secundario(p)),
        Variante::Fantasma => b.style(estilo::boton_fantasma(p)),
        Variante::Peligro => b.style(estilo::boton_peligro(p)),
    }
    .into()
}

/// Campo de texto de alto `control`.
pub fn campo<'a, M: Clone + 'a>(
    p: Paleta,
    marcador: &str,
    valor: &str,
    al_cambiar: impl Fn(String) -> M + 'a,
) -> Element<'a, M> {
    text_input(marcador, valor)
        .on_input(al_cambiar)
        .font(fuentes::CUERPO)
        .size(texto::BODY.0)
        .padding(Padding::from([8.0, espacio::S3]))
        .style(estilo::campo(p))
        .into()
}

/// Etiqueta de control en mayúsculas (`label`).
pub fn etiqueta<'a, M: 'a>(p: Paleta, contenido: &str) -> Element<'a, M> {
    text(contenido.to_uppercase())
        .font(fuentes::ETIQUETA)
        .size(texto::LABEL.0)
        .color(p.text_muted)
        .into()
}

/// Tipo de estado de una insignia.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Estado {
    Exito,
    Aviso,
    Error,
    Info,
}

/// Insignia con palabra de estado; el color nunca va solo.
pub fn insignia<'a, M: 'a>(p: Paleta, estado: Estado, palabra: &str) -> Element<'a, M> {
    let (fondo, tinta) = match estado {
        Estado::Exito => (p.success_soft, p.success),
        Estado::Aviso => (p.warning_soft, p.warning),
        Estado::Error => (p.error_soft, p.error),
        Estado::Info => (p.info_soft, p.info),
    };
    container(
        text(palabra.to_uppercase())
            .font(fuentes::ETIQUETA)
            .size(texto::LABEL.0),
    )
    .padding(Padding::from([espacio::S1, espacio::S2]))
    .style(estilo::insignia(fondo, tinta))
    .into()
}

/// Tarjeta con relleno `space-6`.
pub fn tarjeta<'a, M: 'a>(
    p: Paleta,
    seleccionada: bool,
    contenido: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    container(contenido)
        .padding(espacio::S6)
        .style(estilo::tarjeta(p, seleccionada))
        .into()
}

/// Chip filtrable.
pub fn chip<'a, M: Clone + 'a>(
    p: Paleta,
    etiqueta: &str,
    activo: bool,
    al_pulsar: M,
) -> Element<'a, M> {
    button(
        container(
            text(etiqueta.to_uppercase())
                .font(fuentes::ETIQUETA)
                .size(texto::LABEL.0),
        )
        .center_y(Length::Fill),
    )
    .height(medida::CONTROL_SM)
    .padding(Padding::from([0.0, espacio::S3]))
    .on_press(al_pulsar)
    .style(estilo::chip(p, activo))
    .into()
}

/// Grupo segmentado de opciones exclusivas.
pub fn segmentado<'a, M: Clone + 'a>(
    p: Paleta,
    opciones: &[&str],
    activa: usize,
    al_elegir: impl Fn(usize) -> M,
) -> Element<'a, M> {
    let mut fila = row![];
    for (i, o) in opciones.iter().enumerate() {
        if i > 0 {
            fila = fila.push(
                container(Space::new(
                    Length::Fixed(1.0),
                    Length::Fixed(medida::CONTROL - 4.0),
                ))
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(p.text)),
                    ..Default::default()
                }),
            );
        }
        fila = fila.push(
            button(
                container(
                    text(o.to_uppercase())
                        .font(fuentes::ETIQUETA)
                        .size(texto::LABEL.0),
                )
                .center_y(Length::Fill),
            )
            .height(medida::CONTROL - 2.0 * 2.0)
            .padding(Padding::from([0.0, espacio::S3]))
            .on_press(al_elegir(i))
            .style(estilo::segmento(p, i == activa)),
        );
    }
    container(fila).style(estilo::marco(p)).into()
}

/// Casilla con etiqueta.
pub fn casilla<'a, M: 'a>(
    p: Paleta,
    etiqueta: &str,
    marcada: bool,
    al_cambiar: impl Fn(bool) -> M + 'a,
) -> Element<'a, M> {
    checkbox(etiqueta, marcada)
        .on_toggle(al_cambiar)
        .font(fuentes::CUERPO)
        .size(20.0)
        .text_size(texto::BODY.0)
        .spacing(espacio::S2)
        .style(estilo::casilla(p))
        .into()
}

/// Interruptor cuadrado: la perilla pasa de un lado al otro de la pista.
pub fn interruptor<'a, M: Clone + 'a>(p: Paleta, encendido: bool, al_cambiar: M) -> Element<'a, M> {
    const ANCHO: f32 = 44.0;
    const ALTO: f32 = 24.0;
    const PERILLA: f32 = 12.0;
    let color = if encendido { p.on_accent } else { p.text };
    let perilla = container(Space::new(Length::Fixed(PERILLA), Length::Fixed(PERILLA)))
        .style(estilo::perilla(color));
    let interior = if encendido {
        row![Space::with_width(Length::Fill), perilla]
    } else {
        row![perilla, Space::with_width(Length::Fill)]
    };
    let pista = container(interior.align_y(iced::Alignment::Center))
        .width(ANCHO)
        .height(ALTO)
        .padding(Padding::from([0.0, espacio::S2 - 2.0]))
        .center_y(ALTO)
        .style(estilo::pista(p, encendido, false));
    button(pista)
        .padding(0)
        .on_press(al_cambiar)
        .style(estilo::sin_estilo(p.text))
        .into()
}

/// Barra de progreso continua con texto de estado a su lado.
pub fn progreso<'a, M: 'a>(p: Paleta, avance: f32) -> Element<'a, M> {
    progress_bar(0.0..=1.0, avance.clamp(0.0, 1.0))
        .height(8.0)
        .style(estilo::progreso(p))
        .into()
}

/// Progreso en bloques, para tareas con partes contables.
pub fn progreso_bloques<'a, M: 'a>(p: Paleta, total: usize, hechos: usize) -> Element<'a, M> {
    let mut fila = row![].spacing(2.0);
    for i in 0..total {
        let lleno = i < hechos;
        fila = fila.push(
            container(Space::new(Length::Fill, Length::Fixed(12.0))).style(
                move |_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(if lleno {
                        p.text
                    } else {
                        p.surface_sunken
                    })),
                    border: iced::Border {
                        width: 1.0,
                        color: p.border_strong,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                },
            ),
        );
    }
    fila.into()
}

/// Selector desplegable de opciones de texto.
pub fn selector<'a, T, M>(
    p: Paleta,
    opciones: Vec<T>,
    elegida: Option<T>,
    al_elegir: impl Fn(T) -> M + 'a,
) -> Element<'a, M>
where
    T: ToString + PartialEq + Clone + 'a,
    M: Clone + 'a,
{
    pick_list(opciones, elegida, al_elegir)
        .placeholder("Elegir")
        .font(fuentes::CUERPO)
        .text_size(texto::BODY.0)
        .padding(Padding::from([8.0, espacio::S3]))
        .style(estilo::selector(p))
        .menu_style(estilo::menu_selector(p))
        .into()
}
