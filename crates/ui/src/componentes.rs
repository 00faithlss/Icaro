// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Componentes básicos construidos sobre los estilos del sistema.

use iced::widget::{
    button, checkbox, column, container, pick_list, progress_bar, row, text, text_input, Space,
};
use iced::{Element, Length, Padding};

use crate::estilo;
use crate::fuentes;
use crate::tema::{espacio, medida, texto, Paleta};

/// Variante visual de un botón.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variante {
    Primario,
    Secundario,
    Fantasma,
    Peligro,
}

/// Botón de texto de alto `control`. Sin mensaje queda deshabilitado.
pub fn boton<'a, M: Clone + 'a>(
    p: Paleta,
    etiqueta: &str,
    variante: Variante,
    al_pulsar: Option<M>,
) -> Element<'a, M> {
    let contenido = text(etiqueta.to_uppercase())
        .font(fuentes::ETIQUETA)
        .size(texto::LABEL.0);
    let b = button(
        container(contenido)
            .height(Length::Fill)
            .align_y(iced::alignment::Vertical::Center),
    )
    .height(medida::CONTROL)
    .padding(Padding::from([0.0, espacio::S4]))
    .on_press_maybe(al_pulsar);
    match variante {
        Variante::Primario => b.style(estilo::boton_primario(p)),
        Variante::Secundario => b.style(estilo::boton_secundario(p)),
        Variante::Fantasma => b.style(estilo::boton_fantasma(p)),
        Variante::Peligro => b.style(estilo::boton_peligro(p)),
    }
    .into()
}

/// Campo de texto de alto `control`.
pub fn campo<'a, M: Clone + 'a>(
    p: Paleta,
    marcador: &str,
    valor: &str,
    al_cambiar: impl Fn(String) -> M + 'a,
) -> Element<'a, M> {
    text_input(marcador, valor)
        .on_input(al_cambiar)
        .font(fuentes::CUERPO)
        .size(texto::BODY.0)
        .padding(Padding::from([8.0, espacio::S3]))
        .style(estilo::campo(p))
        .into()
}

/// Etiqueta de control en mayúsculas (`label`).
pub fn etiqueta<'a, M: 'a>(p: Paleta, contenido: &str) -> Element<'a, M> {
    text(contenido.to_uppercase())
        .font(fuentes::ETIQUETA)
        .size(texto::LABEL.0)
        .color(p.text_muted)
        .into()
}

/// Tipo de estado de una insignia.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Estado {
    Exito,
    Aviso,
    Error,
    Info,
}

/// Insignia con palabra de estado; el color nunca va solo.
pub fn insignia<'a, M: 'a>(p: Paleta, estado: Estado, palabra: &str) -> Element<'a, M> {
    let (fondo, tinta) = match estado {
        Estado::Exito => (p.success_soft, p.success),
        Estado::Aviso => (p.warning_soft, p.warning),
        Estado::Error => (p.error_soft, p.error),
        Estado::Info => (p.info_soft, p.info),
    };
    container(
        text(palabra.to_uppercase())
            .font(fuentes::ETIQUETA)
            .size(texto::LABEL.0),
    )
    .padding(Padding::from([espacio::S1, espacio::S2]))
    .style(estilo::insignia(fondo, tinta))
    .into()
}

/// Tarjeta con relleno `space-6`.
pub fn tarjeta<'a, M: 'a>(
    p: Paleta,
    seleccionada: bool,
    contenido: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    container(contenido)
        .padding(espacio::S6)
        .style(estilo::tarjeta(p, seleccionada))
        .into()
}

/// Chip filtrable.
pub fn chip<'a, M: Clone + 'a>(
    p: Paleta,
    etiqueta: &str,
    activo: bool,
    al_pulsar: M,
) -> Element<'a, M> {
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
    .on_press(al_pulsar)
    .style(estilo::chip(p, activo))
    .into()
}

/// Grupo segmentado de opciones exclusivas.
pub fn segmentado<'a, M: Clone + 'a>(
    p: Paleta,
    opciones: &[&str],
    activa: usize,
    al_elegir: impl Fn(usize) -> M,
) -> Element<'a, M> {
    let mut fila = row![];
    for (i, o) in opciones.iter().enumerate() {
        if i > 0 {
            fila = fila.push(
                container(Space::new(
                    Length::Fixed(1.0),
                    Length::Fixed(medida::CONTROL - 4.0),
                ))
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(p.text)),
                    ..Default::default()
                }),
            );
        }
        fila = fila.push(
            button(
                container(
                    text(o.to_uppercase())
                        .font(fuentes::ETIQUETA)
                        .size(texto::LABEL.0),
                )
                .center_y(Length::Fill),
            )
            .height(medida::CONTROL - 2.0 * 2.0)
            .padding(Padding::from([0.0, espacio::S3]))
            .on_press(al_elegir(i))
            .style(estilo::segmento(p, i == activa)),
        );
    }
    container(fila).style(estilo::marco(p)).into()
}

/// Casilla con etiqueta.
pub fn casilla<'a, M: 'a>(
    p: Paleta,
    etiqueta: &str,
    marcada: bool,
    al_cambiar: impl Fn(bool) -> M + 'a,
) -> Element<'a, M> {
    checkbox(etiqueta, marcada)
        .on_toggle(al_cambiar)
        .font(fuentes::CUERPO)
        .size(20.0)
        .text_size(texto::BODY.0)
        .spacing(espacio::S2)
        .style(estilo::casilla(p))
        .into()
}

/// Interruptor cuadrado: la perilla pasa de un lado al otro de la pista.
pub fn interruptor<'a, M: Clone + 'a>(p: Paleta, encendido: bool, al_cambiar: M) -> Element<'a, M> {
    const ANCHO: f32 = 44.0;
    const ALTO: f32 = 24.0;
    const PERILLA: f32 = 12.0;
    let color = if encendido { p.on_accent } else { p.text };
    let perilla = container(Space::new(Length::Fixed(PERILLA), Length::Fixed(PERILLA)))
        .style(estilo::perilla(color));
    let interior = if encendido {
        row![Space::with_width(Length::Fill), perilla]
    } else {
        row![perilla, Space::with_width(Length::Fill)]
    };
    let pista = container(interior.align_y(iced::Alignment::Center))
        .width(ANCHO)
        .height(ALTO)
        .padding(Padding::from([0.0, espacio::S2 - 2.0]))
        .center_y(ALTO)
        .style(estilo::pista(p, encendido, false));
    button(pista)
        .padding(0)
        .on_press(al_cambiar)
        .style(estilo::sin_estilo(p.text))
        .into()
}

/// Barra de progreso continua con texto de estado a su lado.
pub fn progreso<'a, M: 'a>(p: Paleta, avance: f32) -> Element<'a, M> {
    progress_bar(0.0..=1.0, avance.clamp(0.0, 1.0))
        .height(8.0)
        .style(estilo::progreso(p))
        .into()
}

/// Progreso en bloques, para tareas con partes contables.
pub fn progreso_bloques<'a, M: 'a>(p: Paleta, total: usize, hechos: usize) -> Element<'a, M> {
    let mut fila = row![].spacing(2.0);
    for i in 0..total {
        let lleno = i < hechos;
        fila = fila.push(
            container(Space::new(Length::Fill, Length::Fixed(12.0))).style(
                move |_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(if lleno {
                        p.text
                    } else {
                        p.surface_sunken
                    })),
                    border: iced::Border {
                        width: 1.0,
                        color: p.border_strong,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                },
            ),
        );
    }
    fila.into()
}

/// Selector desplegable de opciones de texto.
pub fn selector<'a, T, M>(
    p: Paleta,
    opciones: Vec<T>,
    elegida: Option<T>,
    al_elegir: impl Fn(T) -> M + 'a,
) -> Element<'a, M>
where
    T: ToString + PartialEq + Clone + 'a,
    M: Clone + 'a,
{
    pick_list(opciones, elegida, al_elegir)
        .placeholder("Elegir")
        .font(fuentes::CUERPO)
        .text_size(texto::BODY.0)
        .padding(Padding::from([8.0, espacio::S3]))
        .style(estilo::selector(p))
        .menu_style(estilo::menu_selector(p))
        .into()
}

/// Pestañas con conteo opcional. La activa lleva texto pleno y una regla
/// inferior gruesa; la fila entera descansa sobre una regla fina.
pub fn pestanas<'a, M: Clone + 'a>(
    p: Paleta,
    pestanas: &[(&str, Option<usize>)],
    activa: usize,
    al_elegir: impl Fn(usize) -> M,
) -> Element<'a, M> {
    let regla = |alto: f32, color: iced::Color| {
        container(Space::new(Length::Fill, Length::Fixed(alto))).style(
            move |_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(color)),
                ..Default::default()
            },
        )
    };
    let mut fila = row![].spacing(espacio::S6);
    for (i, (nombre, conteo)) in pestanas.iter().enumerate() {
        let color = if i == activa { p.text } else { p.text_muted };
        let mut contenido = row![text(nombre.to_uppercase())
            .font(fuentes::ETIQUETA)
            .size(texto::LABEL.0)
            .color(color)]
        .spacing(espacio::S2)
        .align_y(iced::Alignment::Center);
        if let Some(n) = conteo {
            contenido = contenido.push(
                container(
                    text(n.to_string())
                        .font(fuentes::MONO)
                        .size(texto::MONO_SM.0)
                        .color(color),
                )
                .padding(Padding::from([0.0, espacio::S1]))
                .style(move |_: &iced::Theme| container::Style {
                    border: iced::Border {
                        color,
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }),
            );
        }
        let marca = if i == activa {
            regla(crate::tema::borde::GRUESO, p.text)
        } else {
            regla(crate::tema::borde::GRUESO, iced::Color::TRANSPARENT)
        };
        fila = fila.push(
            button(
                column![
                    container(contenido)
                        .height(medida::CONTROL)
                        .align_y(iced::alignment::Vertical::Center),
                    marca
                ]
                .width(Length::Shrink),
            )
            .padding(0)
            .on_press(al_elegir(i))
            .style(estilo::sin_estilo(color)),
        );
    }
    column![fila, regla(crate::tema::borde::FINO, p.border)].into()
}

/// Migas de pan. Todas menos la última son enlaces.
pub fn migas<'a, M: Clone + 'a>(
    p: Paleta,
    tramos: &[&str],
    al_elegir: impl Fn(usize) -> M,
) -> Element<'a, M> {
    let mut fila = row![].spacing(espacio::S2).align_y(iced::Alignment::Center);
    for (i, t) in tramos.iter().enumerate() {
        let ultimo = i + 1 == tramos.len();
        if i > 0 {
            fila = fila.push(
                text("/")
                    .font(fuentes::MONO)
                    .size(texto::MONO_SM.0)
                    .color(p.text_disabled),
            );
        }
        let etiqueta = text(t.to_uppercase())
            .font(fuentes::ETIQUETA)
            .size(texto::LABEL.0);
        fila = fila.push(if ultimo {
            Element::from(etiqueta.color(p.text))
        } else {
            button(etiqueta.color(p.text_muted))
                .padding(0)
                .on_press(al_elegir(i))
                .style(estilo::sin_estilo(p.text_muted))
                .into()
        });
    }
    fila.into()
}

fn colores_estado(p: Paleta, estado: Estado) -> (iced::Color, iced::Color) {
    match estado {
        Estado::Exito => (p.success_soft, p.success),
        Estado::Aviso => (p.warning_soft, p.warning),
        Estado::Error => (p.error_soft, p.error),
        Estado::Info => (p.info_soft, p.info),
    }
}

/// Aviso en línea: bloque suave con contorno del color del estado, icono y
/// mensaje. El estado nunca va solo en color: lleva icono y texto.
pub fn aviso<'a, M: 'a>(p: Paleta, estado: Estado, mensaje: &str) -> Element<'a, M> {
    let (fondo, tinta) = colores_estado(p, estado);
    let glifo = match estado {
        Estado::Exito => crate::iconos::Icono::Check,
        Estado::Aviso | Estado::Error => crate::iconos::Icono::Aviso,
        Estado::Info => crate::iconos::Icono::Info,
    };
    container(
        row![
            crate::iconos::icono(glifo, crate::iconos::Tam::Base, tinta),
            text(mensaje.to_owned())
                .size(texto::BODY.0)
                .color(p.text),
        ]
        .spacing(espacio::S3)
        .align_y(iced::Alignment::Center),
    )
    .padding(Padding::from([espacio::S3, espacio::S4]))
    .width(Length::Fill)
    .style(move |_: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(fondo)),
        border: iced::Border {
            color: tinta,
            width: crate::tema::borde::FINO,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

/// Punto de estado cuadrado: lleno de tono si está activo, gris si no.
pub fn punto<'a, M: 'a>(p: Paleta, activo: bool) -> Element<'a, M> {
    let color = if activo { p.success } else { p.text_disabled };
    container(Space::new(10, 10))
        .style(estilo::bloque(color))
        .into()
}

/// Señal de latencia: cuatro barras crecientes, `nivel` de 0 a 4.
pub fn senal<'a, M: 'a>(p: Paleta, nivel: usize) -> Element<'a, M> {
    let mut fila = row![]
        .spacing(2.0)
        .align_y(iced::alignment::Vertical::Bottom);
    for i in 0..4 {
        let color = if i < nivel { p.text } else { p.border_strong };
        fila = fila.push(
            container(Space::new(Length::Fixed(4.0), Length::Fixed(4.0 + 4.0 * i as f32)))
                .style(estilo::bloque(color)),
        );
    }
    fila.into()
}

/// Contador entero con botones de restar y sumar, de alto `control-sm`.
pub fn contador<'a, M: Clone + 'a>(p: Paleta, valor: i32, restar: M, sumar: M) -> Element<'a, M> {
    let paso = |glifo: crate::iconos::Icono, m: M| {
        button(
            container(crate::iconos::icono(glifo, crate::iconos::Tam::Sm, p.text))
                .center(Length::Fill),
        )
        .width(medida::CONTROL_SM)
        .height(medida::CONTROL_SM - 4.0)
        .padding(0)
        .on_press(m)
        .style(estilo::boton_fantasma(p))
    };
    container(
        row![
            paso(crate::iconos::Icono::Menos, restar),
            container(
                text(valor.to_string())
                    .font(fuentes::MONO)
                    .size(texto::MONO.0)
                    .color(p.text)
            )
            .center_x(48),
            paso(crate::iconos::Icono::Mas, sumar),
        ]
        .align_y(iced::Alignment::Center),
    )
    .style(estilo::marco(p))
    .into()
}

/// Entero con separador de miles en formato chileno: `6.144`.
pub fn miles(n: u32) -> String {
    let digitos = n.to_string();
    let mut salida = String::new();
    for (i, c) in digitos.chars().enumerate() {
        if i > 0 && (digitos.len() - i).is_multiple_of(3) {
            salida.push('.');
        }
        salida.push(c);
    }
    salida
}

/// Botón Jugar grande (`control-jugar`) con un dato a la derecha, por
/// ejemplo la memoria asignada.
pub fn boton_jugar<'a, M: Clone + 'a>(
    p: Paleta,
    etiqueta: &str,
    meta: Option<&str>,
    al_pulsar: M,
) -> Element<'a, M> {
    let mut fila = row![
        crate::iconos::icono(crate::iconos::Icono::Jugar, crate::iconos::Tam::Base, p.on_accent),
        text(etiqueta.to_uppercase())
            .font(fuentes::TITULO)
            .size(texto::HEADING.0)
            .color(p.on_accent),
    ]
    .spacing(espacio::S3)
    .align_y(iced::Alignment::Center);
    if let Some(m) = meta {
        fila = fila.push(Space::with_width(Length::Fill)).push(
            text(m.to_owned())
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.on_accent),
        );
    }
    button(container(fila).center_y(Length::Fill))
        .width(260)
        .height(medida::CONTROL_JUGAR)
        .padding(Padding::from([0.0, espacio::S5]))
        .on_press(al_pulsar)
        .style(estilo::boton_primario(p))
        .into()
}

/// Memoria con el contexto del equipo: barra de RAM dividida en sistema,
/// Minecraft y libre, y un deslizador con la zona recomendada marcada.
/// Por encima del 75 % de la RAM el valor pasa a tono de aviso.
#[allow(clippy::too_many_arguments)]
pub fn deslizador_memoria<'a, M: Clone + 'a>(
    p: Paleta,
    valor: u32,
    ram_total: u32,
    sistema: u32,
    zona: (u32, u32),
    al_cambiar: impl Fn(u32) -> M + 'a,
) -> Element<'a, M> {
    use iced::widget::slider;
    let maximo = ram_total.saturating_sub(2048).max(1024);
    let alto_riesgo = valor * 4 > ram_total * 3;
    let tinta = if alto_riesgo { p.warning } else { p.text };
    let libre = ram_total.saturating_sub(sistema + valor);
    let segmento = |partes: u32, color: iced::Color| {
        container(Space::new(Length::FillPortion(partes.max(1) as u16), 10))
            .style(estilo::bloque(color))
    };
    let barra = container(
        row![
            segmento(sistema, p.text_muted),
            segmento(valor, tinta),
            segmento(libre, p.surface_sunken),
        ]
        .spacing(1),
    )
    .style(estilo::marco(p))
    .padding(2);
    let zona_marcada = row![
        Space::with_width(Length::FillPortion(zona.0.saturating_sub(1024).max(1) as u16)),
        container(Space::new(Length::Fill, 4))
            .width(Length::FillPortion((zona.1 - zona.0).max(1) as u16))
            .style(estilo::bloque(p.success)),
        Space::with_width(Length::FillPortion(maximo.saturating_sub(zona.1).max(1) as u16)),
    ];
    let control = slider(1024..=maximo, valor.clamp(1024, maximo), al_cambiar)
        .step(256u32)
        .height(24)
        .style(move |_, _| slider::Style {
            rail: slider::Rail {
                backgrounds: (
                    iced::Background::Color(tinta),
                    iced::Background::Color(p.surface_sunken),
                ),
                width: 6.0,
                border: iced::Border {
                    color: p.border_strong,
                    width: 1.0,
                    radius: 0.0.into(),
                },
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Rectangle {
                    width: 14,
                    border_radius: 0.0.into(),
                },
                background: iced::Background::Color(tinta),
                border_width: 2.0,
                border_color: p.bg,
            },
        });
    let mut aviso_alto = column![];
    if alto_riesgo {
        aviso_alto = aviso_alto.push(
            text("Por encima del 75 % de la RAM el sistema puede quedarse corto.")
                .size(texto::BODY_SM.0)
                .color(p.warning),
        );
    }
    column![
        row![
            text(format!("RAM {} MB", miles(ram_total)))
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted),
            Space::with_width(Length::Fill),
            text(format!("{} MB", miles(valor)))
                .font(fuentes::MONO)
                .size(texto::MONO.0)
                .color(tinta),
        ],
        barra,
        control,
        zona_marcada,
        text(format!(
            "Recomendado {} a {} GB",
            zona.0 / 1024,
            zona.1 / 1024
        ))
        .font(fuentes::MONO)
        .size(texto::MONO_SM.0)
        .color(p.text_muted),
        aviso_alto,
    ]
    .spacing(espacio::S2)
    .width(360)
    .into()
}

/// Estado vacío: una lámina a la izquierda, un título que dice qué falta,
/// una explicación y las acciones que lo resuelven.
pub fn estado_vacio<'a, M: 'a>(
    p: Paleta,
    lamina: crate::laminas::Lamina,
    titulo: &str,
    explicacion: &str,
    acciones: Vec<Element<'a, M>>,
) -> Element<'a, M> {
    let mut fila = row![].spacing(espacio::S2);
    for a in acciones {
        fila = fila.push(a);
    }
    container(
        row![
            container(
                iced::widget::image(lamina.imagen(p.modo).clone())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(iced::ContentFit::Cover),
            )
            .width(Length::FillPortion(1))
            .height(300)
            .style(estilo::marco(p)),
            column![
                text(titulo.to_uppercase())
                    .font(fuentes::DISPLAY)
                    .size(texto::DISPLAY.0)
                    .color(p.text),
                text(explicacion.to_owned())
                    .size(texto::BODY.0)
                    .color(p.text_muted),
                fila,
            ]
            .spacing(espacio::S4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(espacio::S12)
        .align_y(iced::Alignment::Center),
    )
    .padding(espacio::S6)
    .width(Length::Fill)
    .into()
}
