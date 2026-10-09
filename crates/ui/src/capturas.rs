// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Galería de capturas de pantalla.

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length, Padding};

use crate::componentes::chip;
use crate::estilo;
use crate::fuentes;
use crate::laminas::Lamina;
use crate::tema::{borde, espacio, texto, Paleta};

/// Miniatura de una captura. En el launcher es la captura real; mientras
/// tanto se usa una lámina de ejemplo.
#[derive(Debug, Clone)]
pub struct Captura {
    pub archivo: String,
    pub fecha: String,
    pub imagen: Lamina,
}

/// Cuadrícula de capturas de tres columnas con filtro por instancia. La
/// elegida lleva contorno de 2 px.
pub fn galeria<'a, M: Clone + 'a>(
    p: Paleta,
    filtros: &[&str],
    filtro: usize,
    al_filtrar: impl Fn(usize) -> M,
    capturas: &[Captura],
    elegida: Option<usize>,
    al_elegir: impl Fn(usize) -> M,
) -> Element<'a, M> {
    let mut chips = row![].spacing(espacio::S2);
    for (i, f) in filtros.iter().enumerate() {
        chips = chips.push(chip(p, f, filtro == i, al_filtrar(i)));
    }
    let mut filas = column![].spacing(espacio::S6);
    for (n, grupo) in capturas.chunks(3).enumerate() {
        let mut fila = row![].spacing(espacio::S6);
        for (k, c) in grupo.iter().enumerate() {
            let i = n * 3 + k;
            let marco = if elegida == Some(i) {
                (borde::MEDIO, p.text)
            } else {
                (borde::FINO, p.border)
            };
            fila = fila.push(
                button(
                    container(
                        column![
                            crate::laminas::grabado(p, c.imagen, Length::Fill, 150),
                            container(
                                column![
                                    text(c.archivo.clone())
                                        .font(fuentes::MONO)
                                        .size(texto::MONO_SM.0)
                                        .color(p.text),
                                    text(c.fecha.clone())
                                        .size(texto::BODY_SM.0)
                                        .color(p.text_muted),
                                ]
                                .spacing(2)
                            )
                            .padding(Padding::from([espacio::S2, espacio::S3])),
                        ]
                    )
                    .style(estilo::tarjeta_marco(p, marco.0, marco.1)),
                )
                .padding(0)
                .width(Length::Fill)
                .on_press(al_elegir(i))
                .style(estilo::sin_estilo(p.text)),
            );
        }
        for _ in grupo.len()..3 {
            fila = fila.push(Space::with_width(Length::Fill));
        }
        filas = filas.push(fila);
    }
    column![chips.align_y(Alignment::Center), filas]
        .spacing(espacio::S6)
        .into()
}
