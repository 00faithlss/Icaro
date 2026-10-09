// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Estructura de la ventana: barra de título propia, barra lateral con las
//! siete secciones, contenido y barra de juego.

use iced::widget::{button, column, container, mouse_area, progress_bar, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Theme};

use crate::fuentes;
use crate::iconos::{icono, Icono, Tam};
use crate::tema::{borde, espacio, medida, texto, Paleta};

/// Secciones de la barra lateral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Seccion {
    Instancias,
    Servidores,
    Descargas,
    Consola,
    Mods,
    Capturas,
    Ajustes,
}

impl Seccion {
    pub const TODAS: [Seccion; 7] = [
        Seccion::Instancias,
        Seccion::Servidores,
        Seccion::Descargas,
        Seccion::Consola,
        Seccion::Mods,
        Seccion::Capturas,
        Seccion::Ajustes,
    ];

    pub const fn nombre(self) -> &'static str {
        match self {
            Seccion::Instancias => "Instancias",
            Seccion::Servidores => "Servidores",
            Seccion::Descargas => "Descargas",
            Seccion::Consola => "Consola",
            Seccion::Mods => "Mods",
            Seccion::Capturas => "Capturas",
            Seccion::Ajustes => "Ajustes",
        }
    }

    pub const fn icono(self) -> Icono {
        match self {
            Seccion::Instancias => Icono::Instancias,
            Seccion::Servidores => Icono::Servidores,
            Seccion::Descargas => Icono::Descargas,
            Seccion::Consola => Icono::Consola,
            Seccion::Mods => Icono::Mods,
            Seccion::Capturas => Icono::Capturas,
            Seccion::Ajustes => Icono::Ajustes,
        }
    }

    /// Número del atajo `Ctrl` + número.
    pub const fn atajo(self) -> u8 {
        self as u8 + 1
    }
}

/// Cuenta que se muestra al pie de la barra lateral.
pub struct Cuenta<'a> {
    pub nombre: &'a str,
    pub proveedor: &'a str,
}

/// Resumen de las descargas activas; la barra lateral lo muestra solo si existe.
pub struct DescargasActivas {
    pub cantidad: usize,
    /// Velocidad ya formateada, por ejemplo `6,1 MB/s`.
    pub velocidad: String,
    /// Avance de 0 a 1.
    pub avance: f32,
}

/// Mensajes que emite la estructura.
pub struct MensajesShell<'a, M> {
    pub ir_a: Box<dyn Fn(Seccion) -> M + 'a>,
    pub arrastrar: M,
    pub buscar: M,
    pub minimizar: M,
    pub maximizar: M,
    pub cerrar: M,
}

fn regla<'a, M: 'a>(color: Color, grosor: f32) -> Element<'a, M> {
    container(Space::new(Length::Fill, Length::Fixed(grosor)))
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(color)),
            ..Default::default()
        })
        .into()
}

/// Tecla de atajo en `mono-sm` con regla fina.
pub fn kbd<'a, M: 'a>(p: Paleta, contenido: &str) -> Element<'a, M> {
    container(
        text(contenido.to_string())
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted),
    )
    .padding(Padding::from([0.0, espacio::S1]))
    .style(move |_: &Theme| container::Style {
        border: Border {
            width: borde::FINO,
            color: p.border,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn estilo_control(p: Paleta, cierre: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, estado| {
        let (fondo, tinta) = match estado {
            button::Status::Hovered | button::Status::Pressed => {
                if cierre {
                    (Some(p.error), p.on_error)
                } else {
                    (Some(p.text), p.bg)
                }
            }
            _ => (None, p.text),
        };
        button::Style {
            background: fondo.map(Background::Color),
            text_color: tinta,
            border: Border::default(),
            ..Default::default()
        }
    }
}

fn boton_ventana<'a, M: Clone + 'a>(
    p: Paleta,
    contenido: Element<'a, M>,
    cierre: bool,
    mensaje: M,
) -> Element<'a, M> {
    button(
        container(contenido)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .width(medida::BARRA_TITULO + 8.0)
    .height(medida::BARRA_TITULO)
    .padding(0)
    .on_press(mensaje)
    .style(estilo_control(p, cierre))
    .into()
}

fn barra_titulo<'a, M: Clone + 'a>(p: Paleta, m: &MensajesShell<'a, M>) -> Element<'a, M> {
    let marca = text("ÍCARO")
        .font(fuentes::DISPLAY)
        .size(13.0)
        .color(p.text);
    let buscar = button(
        container(
            row![
                icono(Icono::Buscar, Tam::Sm, p.text_muted),
                text("Buscar instancias, mods, servidores")
                    .font(fuentes::CUERPO)
                    .size(texto::BODY_SM.0)
                    .color(p.text_muted),
                kbd(p, "Ctrl K"),
            ]
            .spacing(espacio::S2)
            .align_y(Alignment::Center),
        )
        .center_y(Length::Fill),
    )
    .height(medida::CONTROL_SM)
    .padding(Padding::from([0.0, espacio::S3]))
    .on_press(m.buscar.clone())
    .style(move |_, estado| {
        let activo = matches!(estado, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(Background::Color(p.surface_sunken)),
            text_color: p.text,
            border: Border {
                width: borde::MEDIO,
                color: if activo { p.text } else { p.border_strong },
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    });
    let cuadrado =
        container(Space::new(Length::Fixed(12.0), Length::Fixed(12.0))).style(move |_: &Theme| {
            container::Style {
                border: Border {
                    width: borde::MEDIO,
                    color: p.text,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        });
    let controles = row![
        boton_ventana(
            p,
            icono(Icono::Menos, Tam::Sm, p.text).into(),
            false,
            m.minimizar.clone()
        ),
        boton_ventana(p, cuadrado.into(), false, m.maximizar.clone()),
        boton_ventana(
            p,
            icono(Icono::Cerrar, Tam::Sm, p.text).into(),
            true,
            m.cerrar.clone()
        ),
    ];
    let fila = row![
        container(marca).padding(Padding::from([0.0, espacio::S4])),
        Space::with_width(Length::Fill),
        buscar,
        Space::with_width(Length::Fill),
        controles,
    ]
    .height(medida::BARRA_TITULO)
    .align_y(Alignment::Center);
    column![
        mouse_area(
            container(fila)
                .width(Length::Fill)
                .style(move |_: &Theme| container::Style {
                    background: Some(Background::Color(p.surface)),
                    ..Default::default()
                })
        )
        .on_press(m.arrastrar.clone()),
        regla(p.text, borde::MEDIO),
    ]
    .into()
}

fn item_nav<'a, M: Clone + 'a>(
    p: Paleta,
    seccion: Seccion,
    activa: bool,
    colapsada: bool,
    mensaje: M,
) -> Element<'a, M> {
    let tinta = if activa { p.on_accent } else { p.text };
    let mut fila = row![icono(seccion.icono(), Tam::Base, tinta)]
        .spacing(espacio::S3)
        .align_y(Alignment::Center);
    if !colapsada {
        fila = fila
            .push(
                text(seccion.nombre())
                    .font(fuentes::CUERPO_NEGRITA)
                    .size(texto::BODY.0),
            )
            .push(Space::with_width(Length::Fill))
            .push(
                text(seccion.atajo().to_string())
                    .font(fuentes::MONO)
                    .size(texto::MONO_SM.0),
            );
    }
    let ancho = if colapsada {
        Length::Fixed(medida::BARRA_LATERAL_COLAPSADA - 2.0 * espacio::S2)
    } else {
        Length::Fill
    };
    button(container(fila).center_y(Length::Fill))
        .width(ancho)
        .height(medida::CONTROL_LG)
        .padding(Padding::from([0.0, espacio::S3]))
        .on_press(mensaje)
        .style(move |_, estado| {
            let fondo = if activa {
                Some(p.accent)
            } else if matches!(estado, button::Status::Hovered | button::Status::Pressed) {
                Some(p.surface_hover)
            } else {
                None
            };
            button::Style {
                background: fondo.map(Background::Color),
                text_color: tinta,
                border: Border::default(),
                ..Default::default()
            }
        })
        .into()
}

fn indicador_descargas<'a, M: 'a>(p: Paleta, d: &DescargasActivas) -> Element<'a, M> {
    let titulo = if d.cantidad == 1 {
        "1 descarga".to_string()
    } else {
        format!("{} descargas", d.cantidad)
    };
    column![
        row![
            text(titulo.to_uppercase())
                .font(fuentes::ETIQUETA)
                .size(texto::LABEL.0),
            Space::with_width(Length::Fill),
            text(d.velocidad.clone())
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted),
        ],
        progress_bar(0.0..=1.0, d.avance.clamp(0.0, 1.0))
            .height(8.0)
            .style(move |_: &Theme| progress_bar::Style {
                background: Background::Color(p.surface_sunken),
                bar: Background::Color(p.text),
                border: Border {
                    width: borde::FINO,
                    color: p.border_strong,
                    radius: 0.0.into(),
                },
            }),
    ]
    .spacing(espacio::S2)
    .padding(Padding::from([0.0, espacio::S2]))
    .into()
}

fn bloque_cuenta<'a, M: 'a>(p: Paleta, c: &Cuenta<'a>) -> Element<'a, M> {
    container(
        row![
            column![
                text(c.nombre.to_string())
                    .font(fuentes::CUERPO_NEGRITA)
                    .size(texto::BODY.0),
                text(c.proveedor.to_string())
                    .font(fuentes::MONO)
                    .size(texto::MONO_SM.0)
                    .color(p.text_muted),
            ]
            .spacing(0),
            Space::with_width(Length::Fill),
            icono(Icono::Flecha, Tam::Sm, p.text),
        ]
        .align_y(Alignment::Center),
    )
    .padding(espacio::S3)
    .width(Length::Fill)
    .style(move |_: &Theme| container::Style {
        border: Border {
            width: borde::MEDIO,
            color: p.border_strong,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn barra_lateral<'a, M: Clone + 'a>(
    p: Paleta,
    activa: Seccion,
    colapsada: bool,
    cuenta: &Cuenta<'a>,
    descargas: Option<&DescargasActivas>,
    m: &MensajesShell<'a, M>,
) -> Element<'a, M> {
    let mut items = column![].spacing(espacio::S1);
    for s in Seccion::TODAS {
        items = items.push(item_nav(p, s, s == activa, colapsada, (m.ir_a)(s)));
    }
    let mut pie = column![].spacing(espacio::S3);
    if !colapsada {
        if let Some(d) = descargas {
            pie = pie.push(indicador_descargas(p, d));
        }
        pie = pie.push(bloque_cuenta(p, cuenta));
    }
    let ancho = if colapsada {
        medida::BARRA_LATERAL_COLAPSADA
    } else {
        medida::BARRA_LATERAL
    };
    let cuerpo = column![items, Space::with_height(Length::Fill), pie];
    row![
        container(cuerpo)
            .padding(espacio::S2)
            .width(ancho - borde::MEDIO)
            .height(Length::Fill)
            .style(move |_: &Theme| container::Style {
                background: Some(Background::Color(p.surface)),
                ..Default::default()
            }),
        container(Space::new(Length::Fixed(borde::MEDIO), Length::Fill)).style(move |_: &Theme| {
            container::Style {
                background: Some(Background::Color(p.text)),
                ..Default::default()
            }
        }),
    ]
    .into()
}

/// Ventana completa. `pie` es la barra de juego; si no hay, no se dibuja.
#[allow(clippy::too_many_arguments)]
pub fn app_shell<'a, M: Clone + 'a>(
    p: Paleta,
    activa: Seccion,
    colapsada: bool,
    cuenta: Cuenta<'a>,
    descargas: Option<DescargasActivas>,
    contenido: Element<'a, M>,
    pie: Option<Element<'a, M>>,
    m: MensajesShell<'a, M>,
) -> Element<'a, M> {
    let lateral = barra_lateral(p, activa, colapsada, &cuenta, descargas.as_ref(), &m);
    let mut derecha = column![container(contenido)
        .width(Length::Fill)
        .height(Length::Fill)];
    if let Some(pie) = pie {
        derecha = derecha.push(regla(p.text, borde::MEDIO)).push(pie);
    }
    let cuerpo = row![lateral, derecha.width(Length::Fill)];
    container(column![barra_titulo(p, &m), cuerpo])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(p.bg)),
            text_color: Some(p.text),
            ..Default::default()
        })
        .into()
}
