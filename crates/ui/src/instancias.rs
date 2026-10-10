// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Tarjeta de instancia: portada, nombre, versión, estado y acciones.

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Color, Element, Length, Padding};

use crate::componentes::{insignia, progreso_bloques, Estado};
use crate::estilo;
use crate::fuentes;
use crate::iconos::{icono, Icono, Tam};
use crate::laminas::Lamina;
use crate::tema::{borde, espacio, medida, texto, Paleta};

/// Alto de la portada dentro de la tarjeta.
pub const ALTO_PORTADA: f32 = 170.0;

/// Alto de la tarjeta; fijo para que toda la cuadrícula quede pareja.
pub const ALTO_TARJETA: f32 = 350.0;
/// Ancho fijo de la tarjeta: no crece al maximizar la ventana.
pub const ANCHO_TARJETA: f32 = 300.0;

/// Estado visible de una instancia.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EstadoInstancia {
    Lista,
    Desactualizada,
    /// Avance de 0 a 1.
    Instalando(f32),
    Jugando,
    Error,
}

/// Datos que muestra una tarjeta de instancia.
#[derive(Debug, Clone)]
pub struct DatosInstancia {
    pub nombre: String,
    /// Versión y cargador, por ejemplo `1.21.4 · FABRIC`.
    pub version: String,
    pub ultima_vez: String,
    pub mods: String,
    pub portada: Lamina,
    pub estado: EstadoInstancia,
    pub seleccionada: bool,
    /// Texto del estado: causa del error, paso de la instalación o tiempo
    /// de sesión.
    pub detalle: String,
}

/// Mensajes que emite la tarjeta.
#[derive(Debug, Clone)]
pub struct AccionesTarjeta<M> {
    pub seleccionar: M,
    pub principal: M,
    pub mas: M,
}

fn boton_pie<'a, M: Clone + 'a>(
    p: Paleta,
    glifo: Icono,
    etiqueta: Option<String>,
    color: Color,
    mensaje: Option<M>,
) -> Element<'a, M> {
    // Contenido y botón en un color dado; el invertido se muestra al pasar el
    // puntero y cambia a la vez el relleno, el icono y la etiqueta.
    let construir = |tinta: Color, relleno: Option<Color>| -> Element<'a, M> {
        let mut contenido = row![icono(glifo, Tam::Base, tinta)]
            .spacing(espacio::S2)
            .align_y(Alignment::Center);
        if let Some(e) = etiqueta.clone() {
            contenido = contenido.push(
                text(e.to_uppercase())
                    .font(fuentes::ETIQUETA)
                    .size(texto::LABEL.0)
                    .color(tinta),
            );
        }
        button(container(contenido).center_y(Length::Fill))
            .height(medida::CONTROL)
            .padding(Padding::from([0.0, espacio::S3]))
            .on_press_maybe(mensaje.clone())
            .style(estilo::boton_pie(tinta, relleno, mensaje.is_some()))
            .into()
    };
    // El pie invertido y las acciones de error o deshabilitadas no se invierten.
    let normal = construir(color, None);
    if mensaje.is_some() && (color == p.text || color == p.bg) {
        let (relleno, tinta) = if color == p.text { (p.text, p.bg) } else { (p.bg, p.text) };
        iced::widget::hover(normal, construir(tinta, Some(relleno)))
    } else {
        normal
    }
}

/// Tarjeta de instancia con portada de grabado y pie de acciones.
pub fn tarjeta_instancia<'a, M: Clone + 'a>(
    p: Paleta,
    datos: &DatosInstancia,
    acciones: AccionesTarjeta<M>,
) -> Element<'a, M> {
    let portada = crate::laminas::grabado(p, datos.portada, Length::Fill, ALTO_PORTADA);

    let mut cuerpo = column![
        text(datos.nombre.clone())
            .font(fuentes::TITULO)
            .size(texto::HEADING.0)
            .color(p.text),
        text(datos.version.to_uppercase())
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted),
    ]
    .spacing(espacio::S1);

    cuerpo = cuerpo.push(Space::with_height(espacio::S1));
    cuerpo = match datos.estado {
        EstadoInstancia::Lista => cuerpo.push(insignia(p, Estado::Exito, "Lista")),
        EstadoInstancia::Desactualizada => {
            cuerpo.push(insignia(p, Estado::Aviso, "Desactualizada"))
        }
        EstadoInstancia::Instalando(avance) => cuerpo
            .push(progreso_bloques(p, 10, (avance * 10.0).round() as usize))
            .push(
                text(datos.detalle.clone())
                    .size(texto::BODY_SM.0)
                    .color(p.text_muted),
            ),
        EstadoInstancia::Jugando => cuerpo.push(
            row![
                container(Space::new(8, 8)).style(estilo::bloque(p.success)),
                text(datos.detalle.clone())
                    .size(texto::BODY_SM.0)
                    .color(p.text),
            ]
            .spacing(espacio::S2)
            .align_y(Alignment::Center),
        ),
        EstadoInstancia::Error => cuerpo.push(insignia(p, Estado::Error, &datos.detalle)),
    };
    cuerpo = cuerpo.push(
        row![
            text(datos.ultima_vez.clone())
                .size(texto::BODY_SM.0)
                .color(p.text_muted),
            text(datos.mods.clone())
                .size(texto::BODY_SM.0)
                .color(p.text_muted),
        ]
        .spacing(espacio::S4),
    );

    let invertido = datos.estado == EstadoInstancia::Jugando;
    let tinta = if invertido { p.bg } else { p.text };
    let (glifo, etiqueta, color, activo) = match datos.estado {
        EstadoInstancia::Lista => (Icono::Jugar, "Jugar".to_owned(), tinta, true),
        EstadoInstancia::Desactualizada => {
            (Icono::Actualizar, "Actualizar".to_owned(), tinta, true)
        }
        EstadoInstancia::Instalando(avance) => (
            Icono::Espera,
            format!("Instalando {} %", (avance * 100.0).round()),
            p.text_muted,
            false,
        ),
        EstadoInstancia::Jugando => (Icono::Detener, "Detener".to_owned(), tinta, true),
        EstadoInstancia::Error => (Icono::Instancias, "Reparar".to_owned(), p.error, true),
    };
    let pie = row![
        boton_pie(
            p,
            glifo,
            Some(etiqueta),
            color,
            activo.then(|| acciones.principal.clone())
        ),
        Space::with_width(Length::Fill),
        boton_pie(p, Icono::Acciones, None, tinta, Some(acciones.mas.clone())),
    ]
    .align_y(Alignment::Center);
    let pie = container(pie)
        .padding(Padding::from([espacio::S2, espacio::S2]))
        .width(Length::Fill)
        .style(estilo::bloque(if invertido {
            p.text
        } else {
            Color::TRANSPARENT
        }));

    let interior = column![
        portada,
        container(cuerpo)
            .padding(Padding::from([espacio::S4, espacio::S4]))
            .height(Length::Fill),
        pie,
    ];

    // Contorno: 1 px en reposo, 2 px seleccionada, 3 px en ejecución, tono de error.
    let (ancho, color_marco) = match datos.estado {
        EstadoInstancia::Jugando => (borde::GRUESO, p.text),
        EstadoInstancia::Error => (borde::MEDIO, p.error),
        _ if datos.seleccionada => (borde::MEDIO, p.text),
        _ => (borde::FINO, p.border),
    };
    button(
        container(interior)
            .width(Length::Fill)
            .height(ALTO_TARJETA)
            .style(estilo::tarjeta_marco(p, ancho, color_marco)),
    )
    .padding(0)
    .width(ANCHO_TARJETA)
    .height(ALTO_TARJETA)
    .on_press(acciones.seleccionar)
    .style(estilo::sin_estilo(p.text))
    .into()
}

/// Tile de creación al final de la cuadrícula: rayado de grabado y signo más.
pub fn tarjeta_nueva<'a, M: Clone + 'a>(p: Paleta, al_pulsar: M) -> Element<'a, M> {
    let contenido = column![
        icono(Icono::Mas, Tam::Md, p.text),
        text("NUEVA INSTANCIA")
            .font(fuentes::ETIQUETA)
            .size(texto::LABEL.0)
            .color(p.text),
    ]
    .spacing(espacio::S3)
    .align_x(Alignment::Center);
    button(
        container(contenido)
            .center(Length::Fill)
            .style(estilo::tarjeta(p, false)),
    )
    .padding(0)
    .width(ANCHO_TARJETA)
    .height(ALTO_TARJETA)
    .on_press(al_pulsar)
    .style(estilo::sin_estilo(p.text))
    .into()
}
