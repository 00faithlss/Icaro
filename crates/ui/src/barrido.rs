// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Barrido: líneas horizontales de 1 px cada 3 px sobre una lámina, como el
//! rayado de una plancha de grabado. En Piedra multiplica a 14 %; en Tinta
//! traza líneas negras a 32 %.

use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::Tree;
use iced::advanced::{Layout, Renderer as _, Widget};
use iced::{mouse, Color, Element, Length, Rectangle, Size, Theme};

use crate::tema::{Modo, Paleta};

/// Capa de barrido para apilar sobre una lámina del mismo tamaño.
pub fn barrido<'a, M: 'a>(
    p: Paleta,
    ancho: impl Into<Length>,
    alto: impl Into<Length>,
) -> Element<'a, M> {
    Element::new(Barrido {
        ancho: ancho.into(),
        alto: alto.into(),
        opacidad: if p.modo == Modo::Piedra { 0.14 } else { 0.32 },
    })
}

struct Barrido {
    ancho: Length,
    alto: Length,
    opacidad: f32,
}

impl<M> Widget<M, Theme, iced::Renderer> for Barrido {
    fn size(&self) -> Size<Length> {
        Size::new(self.ancho, self.alto)
    }

    fn layout(&self, _: &mut Tree, _: &iced::Renderer, limites: &Limits) -> Node {
        iced::advanced::layout::atomic(limites, self.ancho, self.alto)
    }

    fn draw(
        &self,
        _: &Tree,
        renderer: &mut iced::Renderer,
        _: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
        visible: &Rectangle,
    ) {
        let caja = layout.bounds();
        let Some(recorte) = caja.intersection(visible) else {
            return;
        };
        let color = Color { a: self.opacidad, ..Color::BLACK };
        renderer.with_layer(recorte, |renderer| {
            // Una línea de 1 px al final de cada celda de 3 px.
            let mut y = caja.y + 2.0;
            while y < recorte.y + recorte.height {
                if y + 1.0 > recorte.y {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle { x: caja.x, y, width: caja.width, height: 1.0 },
                            ..Default::default()
                        },
                        color,
                    );
                }
                y += 3.0;
            }
        });
    }
}
