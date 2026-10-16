// SPDX-License-Identifier: GPL-3.0-only
//! ZonaSoltar del sistema de diseño: reposo rayado y doble marco durante arrastre.
use crate::{
    fuentes,
    iconos::{icono, Icono, Tam},
    tema::Paleta,
};
use iced::{
    widget::{button, canvas, column, container, stack, text},
    Alignment, Element, Length,
};

pub fn zona_soltar<'a, M: Clone + 'a>(
    p: Paleta,
    arrastrando: bool,
    detalle: Option<String>,
    elegir: Option<M>,
) -> Element<'a, M> {
    zona_soltar_estado(p, arrastrando, detalle, None, elegir)
}

/// Estado rechazado del mismo componente; mantiene el formato permitido como recuperación.
pub fn zona_soltar_estado<'a, M: Clone + 'a>(
    p: Paleta,
    arrastrando: bool,
    detalle: Option<String>,
    rechazo: Option<String>,
    elegir: Option<M>,
) -> Element<'a, M> {
    let rechazado = rechazo.is_some();
    let detalle = rechazo.or(detalle);
    let detalle =
        detalle.unwrap_or_else(|| ".mrpack o .zip completo · haz clic para elegir".into());
    let contenido = column![
        icono(
            if rechazado {
                Icono::Alerta
            } else if arrastrando {
                Icono::Descargas
            } else {
                Icono::Subir
            },
            Tam::Md,
            if rechazado { p.error } else { p.text }
        ),
        text(if rechazado {
            "Este archivo no se puede importar"
        } else if arrastrando {
            "Suelta para importar"
        } else {
            "Arrastra un modpack aquí"
        })
        .font(fuentes::CUERPO_NEGRITA)
        .size(16)
        .color(p.text),
        text(detalle)
            .font(if arrastrando {
                fuentes::MONO
            } else {
                fuentes::CUERPO
            })
            .size(13)
            .color(p.text_muted),
    ]
    .spacing(12)
    .align_x(Alignment::Center);
    let centro = container(contenido)
        .padding(20)
        .center_x(Length::Fill)
        .center_y(180);
    let interior: Element<'a, M> = if arrastrando {
        container(centro)
            .style(move |_| container::Style {
                border: iced::Border {
                    color: p.text,
                    width: 2.,
                    radius: 0.into(),
                },
                ..Default::default()
            })
            .into()
    } else {
        stack![
            canvas::Canvas::new(Rayado(p))
                .width(Length::Fill)
                .height(180),
            centro
        ]
        .into()
    };
    button(container(interior).padding(if arrastrando { 4 } else { 0 }))
        .width(Length::Fill)
        .padding(0)
        .on_press_maybe(elegir)
        .style(move |_, estado| button::Style {
            background: Some(iced::Background::Color(if arrastrando {
                p.surface_hover
            } else {
                p.surface_sunken
            })),
            border: iced::Border {
                color: if rechazado {
                    p.error
                } else if arrastrando || matches!(estado, button::Status::Hovered) {
                    p.text
                } else {
                    p.border_strong
                },
                width: 2.,
                radius: 0.into(),
            },
            text_color: p.text,
            ..Default::default()
        })
        .into()
}
/// Variante compacta para el panel de skin, con la misma trama e iconografía.
pub fn zona_soltar_skin<'a, M: Clone + 'a>(
    p: Paleta,
    arrastrando: bool,
    titulo: String,
    detalle: String,
    rechazado: bool,
    elegir: Option<M>,
) -> Element<'a, M> {
    let centro = container(
        column![
            icono(
                Icono::Subir,
                Tam::Lg,
                if rechazado { p.error } else { p.text }
            ),
            text(titulo)
                .font(fuentes::CUERPO_NEGRITA)
                .size(14)
                .color(p.text),
            text(detalle)
                .size(13)
                .color(if rechazado { p.error } else { p.text_muted })
        ]
        .spacing(12)
        .align_x(Alignment::Center),
    )
    .padding([20, 16])
    .center_x(Length::Fill)
    .center_y(180);
    let interior: Element<'a, M> = if arrastrando {
        container(centro)
            .padding(4)
            .style(move |_| container::Style {
                border: iced::Border {
                    color: p.text,
                    width: 2.,
                    radius: 0.into(),
                },
                ..Default::default()
            })
            .into()
    } else {
        stack![
            canvas::Canvas::new(Rayado(p))
                .width(Length::Fill)
                .height(180),
            centro
        ]
        .into()
    };
    button(interior)
        .on_press_maybe(elegir)
        .padding(0)
        .width(Length::Fill)
        .style(move |_, estado| button::Style {
            background: Some(
                if arrastrando {
                    p.surface_hover
                } else {
                    p.surface_sunken
                }
                .into(),
            ),
            border: iced::Border {
                color: if rechazado {
                    p.error
                } else if arrastrando || matches!(estado, button::Status::Hovered) {
                    p.text
                } else {
                    p.border_strong
                },
                width: 2.,
                radius: 0.into(),
            },
            text_color: p.text,
            ..Default::default()
        })
        .into()
}
struct Rayado(Paleta);
impl<M> canvas::Program<M> for Rayado {
    type State = ();
    fn draw(
        &self,
        _: &(),
        renderer: &iced::Renderer,
        _: &iced::Theme,
        bounds: iced::Rectangle,
        _: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let mut tinta = self.0.border;
        tinta.a *= 0.6;
        let mut y = 8.;
        while y < bounds.height {
            frame.fill_rectangle(
                iced::Point::new(0., y),
                iced::Size::new(bounds.width, 1.),
                tinta,
            );
            y += 8.;
        }
        vec![frame.into_geometry()]
    }
}
