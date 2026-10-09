// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Consola del juego: registro en vivo con filtros por nivel.

use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Element, Length, Padding};

use crate::componentes::{boton, campo, chip, Variante};
use crate::estilo;
use crate::fuentes;
use crate::tema::{espacio, Paleta};

/// Nivel de una línea del registro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nivel {
    Info,
    Warn,
    Error,
}

impl Nivel {
    const fn palabra(self) -> &'static str {
        match self {
            Nivel::Info => "INFO ",
            Nivel::Warn => "WARN ",
            Nivel::Error => "ERROR",
        }
    }
}

/// Línea del registro.
#[derive(Debug, Clone)]
pub struct Linea {
    pub hora: String,
    pub nivel: Nivel,
    pub mensaje: String,
}

/// Mensajes de la consola.
pub struct MensajesConsola<M> {
    pub filtro: fn(usize) -> M,
    pub buscar: fn(String) -> M,
    pub copiar: M,
    pub limpiar: M,
}

/// Consola: filtros, búsqueda y pozo hundido con marco de 2 px. El nivel se
/// escribe además de colorearse; las filas de error llevan fondo suave.
pub fn consola<'a, M: Clone + 'a>(
    p: Paleta,
    lineas: &[Linea],
    filtro: usize,
    busqueda: &str,
    m: MensajesConsola<M>,
) -> Element<'a, M> {
    let mut chips = row![].spacing(espacio::S2);
    for (i, n) in ["Todo", "Info", "Warn", "Error"].iter().enumerate() {
        chips = chips.push(chip(p, n, filtro == i, (m.filtro)(i)));
    }
    let barra = row![
        chips,
        container(campo(p, "Buscar en el registro", busqueda, m.buscar)).width(260),
        Space::with_width(Length::Fill),
        boton(p, "Copiar", Variante::Secundario, Some(m.copiar)),
        boton(p, "Limpiar", Variante::Fantasma, Some(m.limpiar)),
    ]
    .spacing(espacio::S3)
    .align_y(Alignment::Center);

    let mut filas = column![];
    for l in lineas.iter().filter(|l| match filtro {
        1 => l.nivel == Nivel::Info,
        2 => l.nivel == Nivel::Warn,
        3 => l.nivel == Nivel::Error,
        _ => true,
    }) {
        let (tinta, fondo) = match l.nivel {
            Nivel::Info => (p.text_muted, iced::Color::TRANSPARENT),
            Nivel::Warn => (p.warning, iced::Color::TRANSPARENT),
            Nivel::Error => (p.error, p.error_soft),
        };
        filas = filas.push(
            container(
                row![
                    text(l.hora.clone())
                        .font(fuentes::MONO)
                        .size(12)
                        .color(p.text_muted),
                    text(l.nivel.palabra())
                        .font(fuentes::MONO_MEDIO)
                        .size(12)
                        .color(tinta),
                    text(l.mensaje.clone())
                        .font(fuentes::MONO)
                        .size(12)
                        .color(if l.nivel == Nivel::Info { p.text } else { tinta })
                        .width(Length::Fill),
                ]
                .spacing(espacio::S4),
            )
            .width(Length::Fill)
            .padding(Padding::from([2.0, espacio::S3]))
            .style(estilo::bloque(fondo)),
        );
    }
    column![
        barra,
        container(crate::scroll::desplazable(filas))
            .padding(2)
            .height(Length::Fill)
            .style(move |_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(p.surface_sunken)),
                border: iced::Border {
                    color: p.text,
                    width: 2.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }),
    ]
    .spacing(espacio::S3)
    .height(Length::Fill)
    .into()
}
