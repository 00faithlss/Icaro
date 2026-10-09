// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Desplazamiento suave de la rueda, al estilo de Lenis, sin barra visible.
//!
//! La rueda mueve un objetivo y la posición real lo persigue con una
//! interpolación independiente de la tasa de cuadros. Todo el estado vive en
//! el propio widget: la aplicación solo envuelve su contenido con
//! [`desplazable`].

use std::time::Instant;

use iced::advanced::layout::{Layout, Limits, Node};
use iced::advanced::widget::{tree, Operation, Tree};
use iced::advanced::{mouse, overlay, renderer, Clipboard, Shell, Widget};
use iced::widget::scrollable::{self, Direction, Scrollbar};
use iced::window::RedrawRequest;
use iced::{event, Element, Event, Length, Rectangle, Renderer, Size, Theme, Vector};

/// Píxeles que avanza una muesca de la rueda.
const PASO_MUESCA: f32 = 110.0;
/// Rapidez con que la posición alcanza al objetivo (por segundo).
const RAPIDEZ: f32 = 9.0;
/// Distancia bajo la cual se da por alcanzado el objetivo.
const UMBRAL: f32 = 0.3;

/// Lista desplazable sin barra y con rueda suave.
pub fn desplazable<'a, M: 'a>(contenido: impl Into<Element<'a, M>>) -> Element<'a, M> {
    let lista = scrollable::Scrollable::with_direction(
        contenido,
        Direction::Vertical(Scrollbar::new().width(0).scroller_width(0).margin(0)),
    )
    .height(Length::Fill);
    Element::new(Suave {
        contenido: lista.into(),
    })
}

#[derive(Debug, Default)]
struct Estado {
    objetivo: f32,
    actual: f32,
    ultimo: Option<Instant>,
}

/// Envoltura que convierte la rueda en un movimiento interpolado: captura la
/// rueda, mueve el objetivo y, en cada cuadro, entrega al `scrollable` el
/// tramo que corresponde como un desplazamiento en píxeles.
struct Suave<'a, M> {
    contenido: Element<'a, M>,
}

impl<'a, M> Widget<M, Theme, Renderer> for Suave<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Estado>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Estado::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.contenido)]
    }

    fn diff(&self, arbol: &mut Tree) {
        arbol.diff_children(std::slice::from_ref(&self.contenido));
    }

    fn size(&self) -> Size<Length> {
        self.contenido.as_widget().size()
    }

    fn layout(&self, arbol: &mut Tree, renderer: &Renderer, limites: &Limits) -> Node {
        let hijo = self
            .contenido
            .as_widget()
            .layout(&mut arbol.children[0], renderer, limites);
        Node::with_children(hijo.size(), vec![hijo])
    }

    fn draw(
        &self,
        arbol: &Tree,
        renderer: &mut Renderer,
        tema: &Theme,
        estilo: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        visible: &Rectangle,
    ) {
        if let Some(hijo) = layout.children().next() {
            self.contenido
                .as_widget()
                .draw(&arbol.children[0], renderer, tema, estilo, hijo, cursor, visible);
        }
    }

    fn operate(
        &self,
        arbol: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operacion: &mut dyn Operation,
    ) {
        if let Some(hijo) = layout.children().next() {
            self.contenido
                .as_widget()
                .operate(&mut arbol.children[0], hijo, renderer, operacion);
        }
    }

    fn on_event(
        &mut self,
        arbol: &mut Tree,
        evento: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        portapapeles: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        visible: &Rectangle,
    ) -> event::Status {
        let Some(hijo) = layout.children().next() else {
            return event::Status::Ignored;
        };
        let caja = hijo.bounds();

        match &evento {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if cursor.is_over(caja) => {
                // Alto del contenido dentro del scrollable (su primer hijo).
                let alto = hijo.children().next().map_or(caja.height, |c| c.bounds().height);
                let maximo = (alto - caja.height).max(0.0);
                let dy = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y * PASO_MUESCA,
                    mouse::ScrollDelta::Pixels { y, .. } => *y,
                };
                let estado = arbol.state.downcast_mut::<Estado>();
                estado.objetivo = (estado.objetivo - dy).clamp(0.0, maximo);
                shell.request_redraw(RedrawRequest::NextFrame);
                return event::Status::Captured;
            }
            Event::Window(iced::window::Event::RedrawRequested(ahora)) => {
                let estado = arbol.state.downcast_mut::<Estado>();
                if (estado.objetivo - estado.actual).abs() > UMBRAL {
                    let dt = estado
                        .ultimo
                        .map_or(1.0 / 60.0, |u| ahora.duration_since(u).as_secs_f32())
                        .min(0.1);
                    estado.ultimo = Some(*ahora);
                    let k = 1.0 - (-RAPIDEZ * dt).exp();
                    let mut paso = (estado.objetivo - estado.actual) * k;
                    if (estado.objetivo - estado.actual - paso).abs() <= UMBRAL {
                        paso = estado.objetivo - estado.actual;
                    }
                    estado.actual += paso;
                    let pendiente = (estado.objetivo - estado.actual).abs() > UMBRAL;
                    if !pendiente {
                        estado.ultimo = None;
                    }
                    // Un desplazamiento en píxeles hacia abajo es negativo.
                    let sintetico = Event::Mouse(mouse::Event::WheelScrolled {
                        delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -paso },
                    });
                    let sobre = mouse::Cursor::Available(caja.center());
                    self.contenido.as_widget_mut().on_event(
                        &mut arbol.children[0],
                        sintetico,
                        hijo,
                        sobre,
                        renderer,
                        portapapeles,
                        shell,
                        visible,
                    );
                    if pendiente {
                        shell.request_redraw(RedrawRequest::NextFrame);
                    }
                }
            }
            _ => {}
        }

        self.contenido.as_widget_mut().on_event(
            &mut arbol.children[0],
            evento,
            hijo,
            cursor,
            renderer,
            portapapeles,
            shell,
            visible,
        )
    }

    fn mouse_interaction(
        &self,
        arbol: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        visible: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        layout
            .children()
            .next()
            .map_or(mouse::Interaction::None, |hijo| {
                self.contenido.as_widget().mouse_interaction(
                    &arbol.children[0],
                    hijo,
                    cursor,
                    visible,
                    renderer,
                )
            })
    }

    fn overlay<'b>(
        &'b mut self,
        arbol: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        traslado: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        let hijo = layout.children().next()?;
        self.contenido
            .as_widget_mut()
            .overlay(&mut arbol.children[0], hijo, renderer, traslado)
    }
}
