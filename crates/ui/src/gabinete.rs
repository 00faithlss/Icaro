// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Gabinete de grabados: un mosaico de recortes a distintos tamaños, como una
//! pared de museo. Vive en Acerca de y sirve de selector de portada para las
//! instancias.

use iced::widget::{column, container, mouse_area, row, text, Space};
use iced::{Alignment, Element, Length, Padding};

use crate::botones::{boton, TamBoton};
use crate::componentes::Variante;
use crate::estilo;
use crate::fuentes;
use crate::grabados::{Imagen, Mosaico};
use crate::laminas::Lamina;
use crate::tema::{espacio, texto, Paleta};
use crate::visor::ItemVisor;

/// Columnas del mosaico, alto de fila y hueco, en px.
pub const COLUMNAS: usize = 12;
pub const FILA: f32 = 72.0;
pub const HUECO: f32 = 16.0;

/// Una pieza del gabinete: un recorte con su leyenda.
#[derive(Debug, Clone, Copy)]
pub struct Pieza {
    pub lamina: Lamina,
    pub titulo: &'static str,
    pub autor: &'static str,
    pub columnas: usize,
    pub filas: usize,
    /// Encuadre como `object-position`, de 0 a 1.
    pub foco: (f32, f32),
}

const fn pieza(
    lamina: Lamina,
    titulo: &'static str,
    autor: &'static str,
    columnas: usize,
    filas: usize,
    foco: (f32, f32),
) -> Pieza {
    Pieza { lamina, titulo, autor, columnas, filas, foco }
}

/// Las diez piezas de la pared, en el orden de la preview del sistema.
pub const PIEZAS: [Pieza; 10] = [
    pieza(Lamina::FaetonCuerpo, "Faetón · cuerpo", "Goltzius, 1588", 5, 5, (0.45, 0.40)),
    pieza(Lamina::SaturnoRostro, "Saturno · rostro", "Goltzius, 1592", 4, 4, (0.50, 0.40)),
    pieza(Lamina::MercurioCaduceo, "Caduceo", "Goltzius", 3, 7, (0.50, 0.30)),
    pieza(Lamina::CaidaCielo, "Cielo de Faetón", "ca. 1588", 4, 3, (0.50, 0.40)),
    pieza(Lamina::MinervaRostro, "Minerva · rostro", "Muller, ca. 1600", 3, 4, (0.50, 0.60)),
    pieza(Lamina::ApoloMano, "Apolo · mano", "Goltzius, 1588", 2, 4, (0.50, 0.40)),
    pieza(Lamina::HidraCabezas, "La Hidra", "Aldegrever, 1550", 4, 5, (0.50, 0.35)),
    pieza(Lamina::BelvedereRostro, "Belvedere · perfil", "Raimondi, ca. 1520", 3, 4, (0.50, 0.40)),
    pieza(Lamina::FarnesioEspalda, "Hércules · espalda", "Goltzius, ca. 1592", 5, 4, (0.50, 0.30)),
    pieza(Lamina::MercurioFriso, "Friso", "Goltzius", 7, 2, (0.50, 0.50)),
];

/// Piezas para el visor, con la leyenda como título y detalle.
pub fn items_visor() -> Vec<ItemVisor> {
    PIEZAS
        .iter()
        .map(|z| ItemVisor {
            titulo: z.titulo.to_owned(),
            detalle: z.autor.to_owned(),
            lamina: z.lamina,
            aspecto: None,
        })
        .collect()
}

/// Mensajes del gabinete.
pub struct MensajesGabinete<M> {
    /// El puntero entra o sale de una pieza.
    pub pasar: fn(Option<usize>) -> M,
    /// Clic en una pieza.
    pub abrir: fn(usize) -> M,
    /// "Usar como portada", solo como selector.
    pub usar: Option<fn(usize) -> M>,
}

/// Encabezado de la pared: título monumental y la procedencia a la derecha.
pub fn encabezado<'a, M: 'a>(p: Paleta) -> Element<'a, M> {
    row![
        text("GABINETE")
            .font(fuentes::DISPLAY)
            .size(texto::DISPLAY_XL.0)
            .line_height(iced::widget::text::LineHeight::Absolute(texto::DISPLAY_XL.1.into()))
            .color(p.text),
        Space::with_width(Length::Fill),
        text("Grabados de Goltzius, Raimondi, Muller y Aldegrever, siglos XVI y XVII. Dominio público, The Met y National Gallery of Art.")
            .size(texto::BODY_SM.0)
            .color(p.text_muted)
            .width(Length::Fixed(330.0))
            .align_x(iced::alignment::Horizontal::Right),
    ]
    .align_y(Alignment::End)
    .into()
}

/// Pared de piezas. Con `elegida` funciona como selector de portada: la
/// pieza elegida invierte su leyenda y ofrece "Usar como portada".
pub fn gabinete<'a, M: Clone + 'a>(
    p: Paleta,
    _encima: Option<usize>,
    elegida: Option<usize>,
    _reducir: bool,
    m: &MensajesGabinete<M>,
) -> Element<'a, M> {
    let mut pared = Mosaico::nuevo(COLUMNAS, FILA, HUECO);
    for (i, z) in PIEZAS.iter().enumerate() {
        let imagen = Imagen::nueva(p, z.lamina)
            .barrido(p)
            .registro(p.text)
            .foco(z.foco.0, z.foco.1)
            .tamano(Length::Fill, Length::Fill);
        let es_elegida = elegida == Some(i);
        let tinta_titulo = if es_elegida { p.bg } else { p.text };
        let tinta_autor = if es_elegida { p.bg } else { p.text_muted };
        let titulo = text(z.titulo.to_uppercase())
            .font(fuentes::ETIQUETA)
            .size(texto::LABEL.0)
            .color(tinta_titulo);
        let autor = text(z.autor)
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(tinta_autor);
        // En piezas angostas la leyenda se apila para que no se corte.
        let texto_leyenda: Element<M> = if z.columnas <= 3 {
            column![titulo, autor].spacing(espacio::S1).into()
        } else {
            row![titulo, Space::with_width(Length::Fill), autor]
                .spacing(espacio::S4)
                .align_y(Alignment::Center)
                .into()
        };
        let leyenda: Element<M> = container(texto_leyenda)
            .padding(Padding::from([espacio::S1, if es_elegida { espacio::S2 } else { 0.0 }]))
            .width(Length::Fill)
            .style(estilo::bloque(if es_elegida { p.text } else { iced::Color::TRANSPARENT }))
            .into();
        let mut pieza = column![Element::from(imagen), leyenda]
            .spacing(espacio::S2)
            .height(Length::Fill);
        if es_elegida {
            if let Some(usar) = m.usar {
                pieza = pieza.push(boton(p, None, "Usar como portada", Variante::Primario, TamBoton::Sm, Some(usar(i))));
            }
        }
        let area = mouse_area(pieza)
            .on_enter((m.pasar)(Some(i)))
            .on_exit((m.pasar)(None))
            .on_press((m.abrir)(i));
        pared = pared.push(z.columnas, z.filas, area);
    }
    pared.into()
}
