// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Tarjeta de servidor y bloque de la red privada.

use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Element, Length, Padding};

use crate::componentes::{self, boton, insignia, punto, senal, Estado, Variante};
use crate::estilo;
use crate::fuentes;
use crate::iconos::{icono, Icono, Tam};
use crate::tema::{espacio, texto, Paleta};

/// Estado de un servidor guardado.
#[derive(Debug, Clone, PartialEq)]
pub enum EstadoServidor {
    EnLinea {
        jugadores: u32,
        maximo: u32,
        /// Latencia en milisegundos.
        ms: u32,
    },
    /// Texto de cuándo dejó de responder, por ejemplo `desde las 03:12`.
    FueraDeLinea(String),
}

/// Servidor guardado.
#[derive(Debug, Clone)]
pub struct Servidor {
    pub nombre: String,
    pub direccion: String,
    /// Mensaje del día.
    pub motd: Option<String>,
    pub estado: EstadoServidor,
    /// Instancia vinculada y si sincroniza mods al iniciar.
    pub vinculo: Option<(String, bool)>,
    /// Versión del servidor cuando difiere de la de la instancia.
    pub version_distinta: Option<String>,
}

fn nivel_senal(ms: u32) -> usize {
    match ms {
        0..=60 => 4,
        61..=120 => 3,
        121..=250 => 2,
        _ => 1,
    }
}

/// Tarjeta de servidor con datos de ping y Jugar directo.
pub fn tarjeta_servidor<'a, M: Clone + 'a>(p: Paleta, s: &Servidor, jugar: M) -> Element<'a, M> {
    let en_linea = matches!(s.estado, EstadoServidor::EnLinea { .. });
    let mut titulo = row![
        punto(p, en_linea),
        text(s.nombre.clone())
            .font(fuentes::TITULO)
            .size(texto::HEADING.0)
            .color(p.text),
    ]
    .spacing(espacio::S3)
    .align_y(Alignment::Center);
    if let Some((instancia, sincroniza)) = &s.vinculo {
        let palabra = if *sincroniza {
            format!("{instancia} · sincroniza")
        } else {
            format!("Vinculado a {instancia}")
        };
        titulo = titulo.push(insignia(p, Estado::Info, &palabra));
    }
    if let Some(v) = &s.version_distinta {
        titulo = titulo.push(insignia(p, Estado::Aviso, &format!("Servidor {v}")));
    }
    let mut datos = column![
        titulo,
        text(s.direccion.clone())
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted),
    ]
    .spacing(espacio::S1)
    .width(Length::Fill);
    if let Some(m) = &s.motd {
        datos = datos.push(text(m.clone()).size(texto::BODY_SM.0).color(p.text_muted));
    }
    if let EstadoServidor::FueraDeLinea(cuando) = &s.estado {
        datos = datos.push(
            text(format!("Fuera de línea {cuando}"))
                .size(texto::BODY_SM.0)
                .color(p.text_muted),
        );
    }

    let lado: Element<'a, M> = match &s.estado {
        EstadoServidor::EnLinea { jugadores, maximo, ms } => row![
            column![
                text(format!("{jugadores} / {maximo}"))
                    .font(fuentes::MONO)
                    .size(18)
                    .color(p.text),
                text("jugadores")
                    .font(fuentes::MONO)
                    .size(texto::MONO_SM.0)
                    .color(p.text_muted),
            ]
            .align_x(Alignment::End),
            row![
                senal(p, nivel_senal(*ms)),
                text(format!("{ms} ms"))
                    .font(fuentes::MONO)
                    .size(texto::MONO_SM.0)
                    .color(p.text),
            ]
            .spacing(espacio::S2)
            .align_y(Alignment::Center),
            boton(p, "Jugar", Variante::Primario, Some(jugar)),
        ]
        .spacing(espacio::S6)
        .align_y(Alignment::Center)
        .into(),
        EstadoServidor::FueraDeLinea(_) => row![
            text("Sin respuesta")
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted),
            boton(p, "Jugar", Variante::Secundario, None),
        ]
        .spacing(espacio::S6)
        .align_y(Alignment::Center)
        .into(),
    };
    container(row![datos, lado].spacing(espacio::S6).align_y(Alignment::Center))
        .padding(espacio::S6)
        .width(Length::Fill)
        .style(estilo::tarjeta(p, false))
        .into()
}

/// Bloque de la red privada (ZeroTier) que conecta a los amigos.
pub fn red_privada<'a, M: Clone + 'a>(
    p: Paleta,
    conectada: bool,
    red: &str,
    ip: &str,
    en_linea: (u32, u32),
    gestionar: M,
) -> Element<'a, M> {
    let titulo = if conectada {
        "Red privada conectada"
    } else {
        "Red privada desconectada"
    };
    let detalle = row![
        text(format!("{red} · tu IP "))
            .size(texto::BODY_SM.0)
            .color(p.text_muted),
        text(ip.to_owned())
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text),
        text(format!(" · {} de {} en línea", en_linea.0, en_linea.1))
            .size(texto::BODY_SM.0)
            .color(p.text_muted),
    ]
    .align_y(Alignment::Center);
    container(
        row![
            icono(Icono::Servidores, Tam::Md, p.text),
            column![
                row![
                    punto(p, conectada),
                    text(titulo)
                        .font(fuentes::TITULO)
                        .size(texto::HEADING.0)
                        .color(p.text),
                ]
                .spacing(espacio::S3)
                .align_y(Alignment::Center),
                detalle,
            ]
            .spacing(espacio::S1)
            .width(Length::Fill),
            Space::with_width(espacio::S4),
            componentes::boton(p, "Gestionar red", Variante::Secundario, Some(gestionar)),
        ]
        .spacing(espacio::S6)
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([espacio::S4, espacio::S6]))
    .width(Length::Fill)
    .style(estilo::tarjeta(p, false))
    .into()
}
