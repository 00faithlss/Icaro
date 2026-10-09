// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Cola de descargas, historial y límites.

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length, Padding};

use crate::componentes::{self, contador, progreso};
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
}

pub fn boton_icono<'a, M: Clone + 'a>(p: Paleta, glifo: Icono, m: M) -> Element<'a, M> {
    button(container(icono(glifo, Tam::Base, p.text)).center(Length::Fill))
        .width(medida::CONTROL_SM)
        .height(medida::CONTROL_SM)
        .padding(0)
        .on_press(m)
        .style(estilo::boton_fantasma(p))
        .into()
}

pub fn regla<'a, M: 'a>(alto: f32, color: iced::Color) -> Element<'a, M> {
    container(Space::new(Length::Fill, Length::Fixed(alto)))
        .style(estilo::bloque(color))
        .into()
}

fn fila_descarga<'a, M: Clone + 'a>(
    p: Paleta,
    i: usize,
    d: &Descarga,
    a: &AccionesCola<M>,
) -> Element<'a, M> {
    let titulo = row![
        text(d.nombre.clone())
            .font(fuentes::TITULO)
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
                    boton_icono(
                        p,
                        if pausada { Icono::Jugar } else { Icono::Pausa },
                        (a.pausar)(i)
                    ),
                    boton_icono(p, Icono::Cerrar, (a.cancelar)(i)),
                ]
                .into(),
                column![
                    progreso(p, *av),
                    text(d.detalle.clone())
                        .font(fuentes::MONO)
                        .size(texto::MONO_SM.0)
                        .color(p.text_muted),
                ]
                .spacing(espacio::S2)
                .into(),
            )
        }
        EstadoDescarga::EnCola(n) => (
            boton_icono(p, Icono::Cerrar, (a.cancelar)(i)),
            text(format!("En cola · {n} · {}", d.detalle))
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted)
                .into(),
        ),
        EstadoDescarga::Fallo(motivo) => (
            componentes::boton(p, "Reintentar", componentes::Variante::Secundario, Some((a.reintentar)(i))),
            row![
                icono(Icono::Alerta, Tam::Sm, p.error),
                text(motivo.clone())
                    .size(texto::BODY_SM.0)
                    .color(p.error),
            ]
            .spacing(espacio::S2)
            .align_y(Alignment::Center)
            .into(),
        ),
    };
    column![
        container(
            column![
                row![titulo, Space::with_width(Length::Fill), acciones]
                    .align_y(Alignment::Center),
                cuerpo,
            ]
            .spacing(espacio::S2)
        )
        .padding(Padding::from([espacio::S4, 0.0])),
        regla(borde::FINO, p.border),
    ]
    .into()
}

/// Cola de descargas: velocidad total arriba y una fila por elemento.
pub fn cola_descargas<'a, M: Clone + 'a>(
    p: Paleta,
    velocidad: &str,
    resumen: &str,
    descargas: &[Descarga],
    acciones: &AccionesCola<M>,
) -> Element<'a, M> {
    let mut filas = column![regla(borde::MEDIO, p.text)];
    for (i, d) in descargas.iter().enumerate() {
        filas = filas.push(fila_descarga(p, i, d, acciones));
    }
    column![
        column![
            text(velocidad.to_owned())
                .font(fuentes::DISPLAY)
                .size(texto::DISPLAY.0)
                .color(p.text),
            text(resumen.to_owned())
                .size(texto::BODY_SM.0)
                .color(p.text_muted),
        ],
        filas,
    ]
    .spacing(espacio::S4)
    .into()
}

/// Historial de descargas como tabla de cuatro columnas.
pub fn historial<'a, M: Clone + 'a>(
    p: Paleta,
    entradas: &[EntradaHistorial],
    limpiar: M,
) -> Element<'a, M> {
    let mut tabla = column![regla(borde::FINO, p.border)];
    for e in entradas {
        tabla = tabla.push(
            container(
                row![
                    text(e.nombre.clone())
                        .size(texto::BODY.0)
                        .color(p.text)
                        .width(Length::FillPortion(3)),
                    text(e.destino.clone())
                        .size(texto::BODY_SM.0)
                        .color(p.text_muted)
                        .width(Length::FillPortion(2)),
                    text(e.tamano.clone())
                        .font(fuentes::MONO)
                        .size(texto::MONO_SM.0)
                        .color(p.text)
                        .width(Length::FillPortion(1)),
                    text(e.fecha.clone())
                        .font(fuentes::MONO)
                        .size(texto::MONO_SM.0)
                        .color(p.text)
                        .width(Length::FillPortion(1)),
                ]
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([espacio::S3, 0.0])),
        );
        tabla = tabla.push(regla(borde::FINO, p.border));
    }
    column![
        row![
            text("HISTORIAL")
                .font(fuentes::TITULO)
                .size(texto::HEADING.0)
                .color(p.text),
            Space::with_width(Length::Fill),
            componentes::boton(p, "Limpiar historial", componentes::Variante::Fantasma, Some(limpiar)),
        ]
        .align_y(Alignment::Center),
        tabla,
    ]
    .spacing(espacio::S2)
    .into()
}

/// Pie con los límites: descargas simultáneas y velocidad máxima.
pub fn limites<'a, M: Clone + 'a>(
    p: Paleta,
    simultaneas: i32,
    restar: M,
    sumar: M,
    velocidad: &str,
) -> Element<'a, M> {
    row![
        componentes::etiqueta(p, "Simultáneas"),
        contador(p, simultaneas, restar, sumar),
        Space::with_width(espacio::S8),
        componentes::etiqueta(p, "Velocidad máxima"),
        text(velocidad.to_owned())
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text),
    ]
    .spacing(espacio::S3)
    .align_y(Alignment::Center)
    .into()
}
