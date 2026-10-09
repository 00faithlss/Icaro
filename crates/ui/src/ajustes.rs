// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Filas y encabezados de la pantalla Ajustes.

use iced::widget::{button, column, container, image, row, text, Space};
use iced::{Alignment, ContentFit, Element, Length, Padding};

use crate::estilo;
use crate::fuentes;
use crate::laminas::Lamina;
use crate::tema::{borde, espacio, texto, Paleta};

/// Fila de ajuste: nombre y explicación a la izquierda (máximo 420 px de
/// texto), control a la derecha y una regla fina debajo.
pub fn fila_ajuste<'a, M: Clone + 'a>(
    p: Paleta,
    titulo: &str,
    explicacion: &str,
    control: Element<'a, M>,
) -> Element<'a, M> {
    column![
        container(
            row![
                column![
                    text(titulo.to_owned())
                        .font(fuentes::TITULO)
                        .size(texto::HEADING.0)
                        .color(p.text),
                    text(explicacion.to_owned())
                        .size(texto::BODY_SM.0)
                        .color(p.text_muted),
                ]
                .spacing(espacio::S1)
                .width(Length::Fill)
                .max_width(420),
                Space::with_width(Length::Fill),
                control,
            ]
            .spacing(espacio::S8)
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([espacio::S5, 0.0])),
        container(Space::new(Length::Fill, Length::Fixed(borde::FINO)))
            .style(estilo::bloque(p.border)),
    ]
    .into()
}

/// Encabezado de sección de ajustes: título en `display` y, opcionalmente,
/// la lámina que la preside a la derecha.
pub fn encabezado_ajustes<'a, M: 'a>(p: Paleta, titulo: &str, lamina: Option<Lamina>) -> Element<'a, M> {
    let mut fila = row![
        text(titulo.to_uppercase())
            .font(fuentes::DISPLAY)
            .size(texto::DISPLAY.0)
            .color(p.text)
            .width(Length::Fill),
    ]
    .align_y(Alignment::End)
    .spacing(espacio::S8);
    if let Some(l) = lamina {
        fila = fila.push(
            container(
                image(l.imagen(p.modo).clone())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(ContentFit::Cover),
            )
            .width(120)
            .height(150)
            .style(estilo::marco(p)),
        );
    }
    column![
        fila,
        container(Space::new(Length::Fill, Length::Fixed(borde::MEDIO)))
            .style(estilo::bloque(p.text)),
    ]
    .spacing(espacio::S4)
    .into()
}

/// Índice de secciones de ajustes: la visible lleva una regla izquierda de
/// 3 px y texto pleno.
pub fn indice_ajustes<'a, M: Clone + 'a>(
    p: Paleta,
    secciones: &[&str],
    activa: usize,
    al_elegir: impl Fn(usize) -> M,
) -> Element<'a, M> {
    let mut col = column![].spacing(2);
    for (i, s) in secciones.iter().enumerate() {
        let actual = i == activa;
        let color = if actual { p.text } else { p.text_muted };
        let marca = container(Space::new(Length::Fixed(borde::GRUESO), Length::Fixed(34.0)))
            .style(estilo::bloque(if actual {
                p.text
            } else {
                iced::Color::TRANSPARENT
            }));
        col = col.push(
            button(
                row![
                    marca,
                    container(
                        text((*s).to_owned())
                            .font(if actual {
                                fuentes::CUERPO_NEGRITA
                            } else {
                                fuentes::CUERPO
                            })
                            .size(texto::BODY_SM.0)
                            .color(color),
                    )
                    .padding(Padding::from([0.0, espacio::S3]))
                    .center_y(34),
                ]
                .align_y(Alignment::Center),
            )
            .padding(0)
            .width(Length::Fill)
            .on_press(al_elegir(i))
            .style(estilo::sin_estilo(color)),
        );
    }
    col.width(200).into()
}
