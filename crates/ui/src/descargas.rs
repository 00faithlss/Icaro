// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Cola de descargas, historial y límites.


use iced::widget::{button, column, container, progress_bar, row, text, tooltip, Space};
use iced::{Alignment, Element, Length, Padding};

use crate::componentes::{self, boton_compacto, boton_con_icono, contador, Variante};
use crate::estilo;
use crate::fuentes;
use crate::iconos::{icono, Icono, Tam};
use crate::tema::{borde, espacio, medida, texto, Paleta};

/// Estado de un elemento de la cola.
#[derive(Debug, Clone, PartialEq)]
pub enum EstadoDescarga {
    /// Avance de 0 a 1.
    Activa(f32),
    Pausada(f32),
    /// Posición en la cola.
    EnCola(usize),
    /// Motivo del fallo.
    Fallo(String),
    /// Terminó: la fila se queda 5 s y pasa al historial.
    Completada,
    /// Cancelar borraría lo ya descargado (más de 100 MB): pide confirmación.
    /// Lleva el tamaño que se perdería, por ejemplo `412,8 MB`.
    Confirmando(String),
}

/// Elemento de la cola de descargas.
#[derive(Debug, Clone)]
pub struct Descarga {
    pub nombre: String,
    pub destino: String,
    pub estado: EstadoDescarga,
    /// Línea de datos: tamaño, velocidad y tiempo restante.
    pub detalle: String,
}

/// Fila del historial.
#[derive(Debug, Clone)]
pub struct EntradaHistorial {
    pub nombre: String,
    pub destino: String,
    pub tamano: String,
    pub fecha: String,
}

/// Mensajes que emite la cola.
pub struct AccionesCola<M> {
    pub pausar: fn(usize) -> M,
    pub cancelar: fn(usize) -> M,
    pub reintentar: fn(usize) -> M,
    /// Desiste de cancelar y sigue descargando.
    pub seguir: fn(usize) -> M,
}

/// Botón de solo icono de 28 px, sin nombre visible.
pub fn boton_icono<'a, M: Clone + 'a>(p: Paleta, glifo: Icono, m: M) -> Element<'a, M> {
    button(container(crate::iconos::icono_texto(glifo, Tam::Base)).center(Length::Fill))
        .width(medida::CONTROL_SM)
        .height(medida::CONTROL_SM)
        .padding(0)
        .on_press(m)
        .style(estilo::boton_fantasma(p))
        .into()
}

/// Botón de solo icono con su nombre en un tooltip invertido (`text` sobre
/// `bg`).
pub fn boton_icono_con_ayuda<'a, M: Clone + 'a>(
    p: Paleta,
    glifo: Icono,
    ayuda: &str,
    m: M,
) -> Element<'a, M> {
    let globo = container(text(ayuda.to_owned()).size(12).color(p.bg))
        .padding(Padding::from([espacio::S2 - 2.0, espacio::S3 - 2.0]))
        .style(estilo::bloque(p.text));
    tooltip(boton_icono(p, glifo, m), globo, tooltip::Position::Bottom)
        .gap(6)
        .into()
}

pub fn regla<'a, M: 'a>(alto: f32, color: iced::Color) -> Element<'a, M> {
    container(Space::new(Length::Fill, Length::Fixed(alto)))
        .style(estilo::bloque(color))
        .into()
}

/// Barra continua de 6 px: pista hundida con regla fina y relleno de tinta.
fn barra<'a, M: 'a>(p: Paleta, avance: f32, tinta: iced::Color) -> Element<'a, M> {
    progress_bar(0.0..=1.0, avance.clamp(0.0, 1.0))
        .height(6.0)
        .style(move |_: &iced::Theme| progress_bar::Style {
            background: iced::Background::Color(p.surface_sunken),
            bar: iced::Background::Color(tinta),
            border: iced::Border { width: 1.0, color: p.border, radius: 0.0.into() },
        })
        .into()
}

fn mono_sm<'a, M: 'a>(p: Paleta, contenido: String) -> Element<'a, M> {
    text(contenido)
        .font(fuentes::MONO)
        .size(texto::MONO_SM.0)
        .color(p.text_muted)
        .into()
}

fn fila_descarga<'a, M: Clone + 'a>(
    p: Paleta,
    i: usize,
    d: &Descarga,
    a: &AccionesCola<M>,
    animacion: Option<(bool, bool, Option<(usize, usize)>)>,
) -> Element<'a, M> {
    let titulo = row![
        text(d.nombre.clone())
            .font(fuentes::CUERPO_NEGRITA)
            .size(texto::HEADING.0)
            .color(p.text),
        text(d.destino.clone())
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted),
    ]
    .spacing(espacio::S3)
    .align_y(Alignment::Center);

    let (acciones, cuerpo): (Element<'a, M>, Element<'a, M>) = match &d.estado {
        EstadoDescarga::Activa(av) | EstadoDescarga::Pausada(av) => {
            let pausada = matches!(d.estado, EstadoDescarga::Pausada(_));
            (
                row![
                    boton_icono_con_ayuda(
                        p,
                        if pausada { Icono::Jugar } else { Icono::Pausa },
                        if pausada { "Reanudar" } else { "Pausar" },
                        (a.pausar)(i)
                    ),
                    boton_icono_con_ayuda(p, Icono::Cerrar, "Cancelar", (a.cancelar)(i)),
                ]
                .spacing(espacio::S1)
                .into(),
                column![
                    match animacion {
                        Some((reducir, _, Some((total, hechos)))) if !pausada => crate::progreso::bloques(p, total, hechos, reducir),
                        Some((reducir, true, _)) if !pausada => crate::progreso::indeterminado(p, reducir),
                        Some((reducir, false, _)) if !pausada => crate::progreso::continuo(p, *av, reducir),
                        _ => barra(p, *av, if pausada { p.text_muted } else { p.text }),
                    },
                    mono_sm(p, d.detalle.clone()),
                ]
                .spacing(espacio::S2)
                .into(),
            )
        }
        EstadoDescarga::EnCola(n) => (
            boton_icono_con_ayuda(p, Icono::Cerrar, "Quitar de la cola", (a.cancelar)(i)),
            mono_sm(p, format!("En cola · {n} · {}", d.detalle)),
        ),
        EstadoDescarga::Fallo(motivo) => (
            boton_con_icono(
                p,
                Icono::Actualizar,
                "Reintentar",
                Variante::Secundario,
                Some((a.reintentar)(i)),
                true,
            ),
            row![
                icono(Icono::Alerta, Tam::Sm, p.error),
                text(motivo.clone()).size(texto::BODY_SM.0).color(p.error),
            ]
            .spacing(espacio::S2)
            .align_y(Alignment::Center)
            .into(),
        ),
        EstadoDescarga::Confirmando(perdido) => (
            row![
                boton_compacto(p, "Seguir descargando", Variante::Secundario, Some((a.seguir)(i))),
                boton_compacto(p, "Cancelar y borrar", Variante::Peligro, Some((a.cancelar)(i))),
            ]
            .spacing(espacio::S2)
            .into(),
            row![
                icono(Icono::Aviso, Tam::Sm, p.warning),
                text(format!("Cancelar borra los {perdido} que ya se descargaron."))
                    .size(texto::BODY_SM.0)
                    .color(p.text),
            ]
            .spacing(espacio::S2)
            .align_y(Alignment::Center)
            .into(),
        ),
        EstadoDescarga::Completada => (
            // Mismo ancho que Pausar y Cancelar, para que las barras alineen.
            Space::with_width(60).into(),
            column![
                barra(p, 1.0, p.text),
                mono_sm(p, format!("Completada · {}", d.detalle)),
            ]
            .spacing(espacio::S2)
            .into(),
        ),
    };
    column![
        container(
            row![
                column![titulo, cuerpo].spacing(espacio::S2).width(Length::Fill),
                acciones,
            ]
            .spacing(espacio::S6)
            .align_y(Alignment::Center)
        )
        .padding(Padding::from([espacio::S4, espacio::S5])),
        regla(borde::FINO, p.border),
    ]
    .into()
}

/// Cola de descargas: una fila por elemento bajo una regla de 2 px. La
/// velocidad total va en la banda de la sección, no aquí.
pub fn cola_descargas<'a, M: Clone + 'a>(
    p: Paleta,
    descargas: &[Descarga],
    acciones: &AccionesCola<M>,
) -> Element<'a, M> {
    let mut filas = column![regla(borde::MEDIO, p.text)];
    for (i, d) in descargas.iter().enumerate() {
        filas = filas.push(fila_descarga(p, i, d, acciones, None));
    }
    filas.into()
}

/// Misma cola del System Design con la animación de Progreso. Las filas
/// indeterminadas no publican porcentajes ni cantidades que no se conocen.
pub fn cola_descargas_animada<'a, M: Clone + 'a>(
    p: Paleta, descargas: &[Descarga], acciones: &AccionesCola<M>,
    reducir: bool, indeterminadas: &[usize],
    contables: &[(usize, usize, usize)],
) -> Element<'a, M> {
    let mut filas = column![regla(borde::MEDIO, p.text)];
    for (i, d) in descargas.iter().enumerate() {
        let partes = contables.iter().find(|(fila, _, _)| *fila == i).map(|(_, total, hechos)| (*total, *hechos));
        filas = filas.push(fila_descarga(p, i, d, acciones, Some((reducir, indeterminadas.contains(&i), partes))));
    }
    filas.into()
}

/// Historial de descargas plegable, como tabla de cuatro columnas. `abierto`
/// muestra las filas; `alternar` pliega o despliega la lista.
pub fn historial<'a, M: Clone + 'a>(
    p: Paleta,
    entradas: &[EntradaHistorial],
    abierto: bool,
    alternar: M,
    limpiar: M,
) -> Element<'a, M> {
    let titulo = button(
        row![
            icono(
                if abierto { Icono::ChevronAbajo } else { Icono::ChevronDerecha },
                Tam::Base,
                p.text,
            ),
            text("Historial")
                .font(fuentes::TITULO)
                .size(18)
                .line_height(iced::widget::text::LineHeight::Absolute(24.0.into()))
                .color(p.text),
        ]
        .spacing(espacio::S2)
        .align_y(Alignment::Center),
    )
    .padding(0)
    .on_press(alternar)
    .style(estilo::sin_estilo(p.text));
    let cabecera = row![
        titulo,
        Space::with_width(Length::Fill),
        boton_compacto(
            p,
            "Limpiar historial",
            Variante::Fantasma,
            (!entradas.is_empty()).then_some(limpiar),
        ),
    ]
    .align_y(Alignment::Center);
    let mut tabla = column![];
    if abierto {
        tabla = tabla.push(regla(borde::FINO, p.border));
        for e in entradas {
            tabla = tabla.push(
                container(
                    row![
                        text(e.nombre.clone())
                            .size(texto::BODY.0)
                            .color(p.text)
                            .width(Length::FillPortion(33)),
                        text(e.destino.clone())
                            .size(texto::BODY_SM.0)
                            .color(p.text_muted)
                            .width(Length::FillPortion(33)),
                        text(e.tamano.clone())
                            .font(fuentes::MONO)
                            .size(12)
                            .color(p.text)
                            .align_x(iced::alignment::Horizontal::Right)
                            .width(Length::FillPortion(8)),
                        text(e.fecha.clone())
                            .font(fuentes::MONO)
                            .size(12)
                            .color(p.text)
                            .align_x(iced::alignment::Horizontal::Right)
                            .width(Length::FillPortion(26)),
                    ]
                    .align_y(Alignment::Center),
                )
                .padding(Padding::from([0.0, espacio::S3]))
                .height(44)
                .center_y(Length::Fixed(44.0)),
            );
            tabla = tabla.push(regla(borde::FINO, p.border));
        }
        if entradas.is_empty() {
            tabla = tabla.push(
                container(
                    text("Aún no hay descargas terminadas.")
                        .size(texto::BODY_SM.0)
                        .color(p.text_muted),
                )
                .padding(Padding::from([espacio::S4, espacio::S3])),
            );
        }
    }
    column![cabecera, tabla].spacing(espacio::S2).into()
}

/// Pie con los límites: descargas simultáneas y velocidad máxima. Va fijo
/// bajo el contenido, con regla fina arriba.
pub fn limites<'a, M: Clone + 'a>(
    p: Paleta,
    simultaneas: i32,
    restar: M,
    sumar: M,
    velocidad: &str,
) -> Element<'a, M> {
    column![
        regla(borde::FINO, p.border),
        container(
            row![
                row![
                    componentes::etiqueta(p, "Simultáneas"),
                    contador(p, simultaneas, restar, sumar),
                ]
                .spacing(espacio::S3)
                .align_y(Alignment::Center),
                row![
                    componentes::etiqueta(p, "Velocidad máxima"),
                    text(velocidad.to_owned())
                        .font(fuentes::MONO)
                        .size(texto::MONO_SM.0)
                        .color(p.text),
                ]
                .spacing(espacio::S3)
                .align_y(Alignment::Center),
            ]
            .spacing(espacio::S8)
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([espacio::S3, espacio::S12]))
        .width(Length::Fill)
        .style(estilo::bloque(p.surface)),
    ]
    .into()
}
