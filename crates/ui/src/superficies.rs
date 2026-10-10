// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Superficies que flotan: diálogo con velo, menú contextual, toast y banner.

use iced::widget::{
    button, center, column, container, opaque, row, stack, text, Space,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Theme};

use crate::componentes::Estado;
use crate::estilo;
use crate::fuentes;
use crate::iconos::{icono, Icono, Tam};
use crate::laminas::Lamina;
use crate::tema::{borde, espacio, medida, texto, Paleta};

fn regla<'a, M: 'a>(alto: f32, color: Color) -> Element<'a, M> {
    container(Space::new(Length::Fill, Length::Fixed(alto)))
        .style(estilo::bloque(color))
        .into()
}

/// Diálogo modal: marco de 2 px, cabeza con título y cierre, cuerpo y pie de
/// acciones a la derecha separado por una regla. Con `detalle`, lleva un
/// grabado en columna a la izquierda. El alto es fijo porque el grabado
/// ocupa toda la columna.
#[allow(clippy::too_many_arguments)]
pub fn dialogo<'a, M: Clone + 'a>(
    p: Paleta,
    titulo: &str,
    cuerpo: Element<'a, M>,
    acciones: Vec<Element<'a, M>>,
    detalle: Option<Lamina>,
    ancho: f32,
    alto: f32,
    cerrar: M,
) -> Element<'a, M> {
    let cabeza = row![
        text(titulo.to_owned())
            .font(fuentes::TITULO)
            .size(texto::HEADING.0 + 2.0)
            .color(p.text),
        Space::with_width(Length::Fill),
        button(icono(Icono::Cerrar, Tam::Base, p.text))
            .padding(espacio::S2)
            .on_press(cerrar)
            .style(estilo::boton_fantasma(p)),
    ]
    .align_y(Alignment::Center);
    let mut pie = row![Space::with_width(Length::Fill)].spacing(espacio::S2);
    for a in acciones {
        pie = pie.push(a);
    }
    let columna = column![
        container(cabeza).padding(Padding::from([espacio::S4, espacio::S6])),
        container(cuerpo)
            .padding(Padding::from([espacio::S2, espacio::S6]))
            .height(Length::Fill),
        regla(borde::FINO, p.border),
        container(pie).padding(Padding::from([espacio::S4, espacio::S6])),
    ]
    .width(Length::Fill);

    let contenido: Element<'a, M> = match detalle {
        Some(l) => row![
            crate::laminas::grabado(p, l, 170, Length::Fill),
            container(Space::new(Length::Fixed(borde::MEDIO), Length::Fill))
                .style(estilo::bloque(p.text)),
            columna,
        ]
        .into(),
        None => columna.into(),
    };
    // El relleno del grosor del marco evita que el grabado tape el contorno.
    container(container(contenido).padding(borde::MEDIO))
        .width(ancho)
        .height(alto)
        .style(estilo::flotante(p))
        .into()
}

/// Pone un diálogo sobre el contenido con el velo `scrim`. Un clic fuera del
/// diálogo emite `al_cerrar`.
pub fn con_velo<'a, M: Clone + 'a>(
    p: Paleta,
    fondo: Element<'a, M>,
    dialogo: Element<'a, M>,
    al_cerrar: M,
) -> Element<'a, M> {
    let velo = iced::widget::mouse_area(
        center(opaque(dialogo)).style(estilo::bloque(p.scrim)),
    )
    .on_press(al_cerrar);
    stack![fondo, opaque(velo)].into()
}

/// Elemento de un menú contextual.
pub enum ElementoMenu<M> {
    Opcion {
        icono: Icono,
        texto: std::borrow::Cow<'static, str>,
        atajo: Option<&'static str>,
        /// Acción destructiva: va al final y en tono de error.
        peligro: bool,
        mensaje: M,
    },
    Separador,
}

/// Menú contextual: marco de 2 px, ítems de 34 px que invierten la tinta.
pub fn menu_contextual<'a, M: Clone + 'a>(p: Paleta, elementos: Vec<ElementoMenu<M>>) -> Element<'a, M> {
    let mut col = column![];
    for e in elementos {
        match e {
            ElementoMenu::Separador => col = col.push(regla(borde::FINO, p.border)),
            ElementoMenu::Opcion {
                icono: glifo,
                texto: etiqueta,
                atajo,
                peligro,
                mensaje,
            } => {
                let tinta = if peligro { p.error } else { p.text };
                // La fila normal y su versión invertida (relleno de tinta,
                // texto y atajo en el color de fondo) al pasar el puntero.
                let construir = |tinta: Color, atajo_c: Color, relleno: Option<Color>| -> Element<'a, M> {
                    let mut fila = row![
                        icono(glifo, Tam::Base, tinta),
                        text(etiqueta.clone())
                            .size(texto::BODY.0)
                            .color(tinta)
                            .width(Length::Fill),
                    ]
                    .spacing(espacio::S3)
                    .align_y(Alignment::Center);
                    if let Some(a) = atajo {
                        fila = fila.push(
                            text(a)
                                .font(fuentes::MONO)
                                .size(texto::MONO_SM.0)
                                .color(atajo_c),
                        );
                    }
                    button(container(fila).center_y(Length::Fill))
                        .width(Length::Fill)
                        .height(40)
                        .padding(Padding::from([0.0, espacio::S4]))
                        .on_press(mensaje.clone())
                        .style(move |_: &Theme, _| button::Style {
                            background: relleno.map(Background::Color),
                            text_color: tinta,
                            ..Default::default()
                        })
                        .into()
                };
                let (relleno, sobre) = if peligro { (p.error, p.on_error) } else { (p.text, p.bg) };
                col = col.push(iced::widget::hover(
                    construir(tinta, p.text_muted, None),
                    construir(sobre, sobre, Some(relleno)),
                ));
            }
        }
    }
    container(col)
        .padding(borde::MEDIO)
        .width(260)
        .style(estilo::flotante(p))
        .into()
}

/// Toast: una línea invertida en tinta plena, con acción opcional. El de
/// error usa el fondo `error`.
pub fn toast<'a, M: Clone + 'a>(
    p: Paleta,
    glifo: Icono,
    mensaje: &str,
    accion: Option<(&str, M)>,
    cerrar: Option<M>,
    error: bool,
) -> Element<'a, M> {
    let (fondo, tinta) = if error { (p.error, p.on_error) } else { (p.text, p.bg) };
    let mut fila = row![
        icono(glifo, Tam::Base, tinta),
        text(mensaje.to_owned())
            .size(texto::BODY.0)
            .color(tinta)
            .width(Length::Fill),
    ]
    .spacing(espacio::S3)
    .align_y(Alignment::Center);
    let contorno = move |_: &Theme, _| button::Style {
        text_color: tinta,
        border: Border {
            color: tinta,
            width: borde::MEDIO,
            radius: 0.0.into(),
        },
        ..Default::default()
    };
    if let Some((etiqueta, m)) = accion {
        fila = fila.push(
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
            .on_press(m)
            .style(contorno),
        );
    }
    if let Some(m) = cerrar {
        fila = fila.push(
            button(icono(Icono::Cerrar, Tam::Base, tinta))
                .padding(espacio::S1)
                .on_press(m)
                .style(estilo::sin_estilo(tinta)),
        );
    }
    container(fila)
        .width(460)
        .padding(Padding::from([espacio::S3, espacio::S4]))
        .style(estilo::bloque(fondo))
        .into()
}

/// Banner persistente a todo el ancho: regla inferior de 2 px del tono sobre
/// su fondo suave, con una acción a la derecha.
pub fn banner<'a, M: Clone + 'a>(
    p: Paleta,
    estado: Estado,
    glifo: Icono,
    titulo: &str,
    mensaje: &str,
    accion: Option<(&str, M)>,
) -> Element<'a, M> {
    let (fondo, tono) = match estado {
        Estado::Exito => (p.success_soft, p.success),
        Estado::Aviso => (p.warning_soft, p.warning),
        Estado::Error => (p.error_soft, p.error),
        Estado::Info => (p.info_soft, p.info),
    };
    let mut fila = row![
        icono(glifo, Tam::Base, tono),
        text(format!("{titulo} "))
            .font(fuentes::CUERPO_NEGRITA)
            .size(texto::BODY.0)
            .color(p.text),
        text(mensaje.to_owned())
            .size(texto::BODY.0)
            .color(p.text)
            .width(Length::Fill),
    ]
    .spacing(espacio::S2)
    .align_y(Alignment::Center);
    if let Some((etiqueta, m)) = accion {
        fila = fila.push(
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
            .on_press(m)
            .style(estilo::boton_secundario(p)),
        );
    }
    column![
        container(fila)
            .width(Length::Fill)
            .padding(Padding::from([espacio::S3, espacio::S12]))
            .style(estilo::bloque(fondo)),
        regla(borde::MEDIO, tono),
    ]
    .into()
}

/// Como `con_velo`, con el diálogo cayendo desde arriba (Losa): `desfase` son
/// los píxeles por encima de su sitio; negativo es el impacto final.
pub fn con_velo_losa<'a, M: Clone + 'a>(
    p: Paleta,
    fondo: Element<'a, M>,
    dialogo: Element<'a, M>,
    al_cerrar: M,
    desfase: f32,
) -> Element<'a, M> {
    let relleno = if desfase >= 0.0 {
        Padding { bottom: desfase * 2.0, ..Padding::ZERO }
    } else {
        Padding { top: -desfase * 2.0, ..Padding::ZERO }
    };
    con_velo(p, fondo, container(dialogo).padding(relleno).into(), al_cerrar)
}
