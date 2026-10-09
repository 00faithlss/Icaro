// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Componentes básicos construidos sobre los estilos del sistema.

use iced::widget::{button, container, text, text_input};
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
