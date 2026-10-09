// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Láminas de grabado: imágenes de dominio público (CC0) que abren cada
//! sección. Cada una existe en positivo (tema Piedra) y en negativo (Tinta).

use std::sync::OnceLock;

use iced::widget::image::Handle;
use iced::widget::{column, container, image, row, stack, text, Space};
use iced::{Alignment, ContentFit, Element, Length, Padding};

use crate::fuentes;
use crate::tema::{espacio, texto, Modo, Paleta};

macro_rules! laminas {
    ($($variante:ident => $archivo:literal),* $(,)?) => {
        /// Láminas empaquetadas con el programa.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Lamina { $($variante),* }

        impl Lamina {
            /// Imagen para el tema dado. El manejador se crea una sola vez.
            pub fn imagen(self, modo: Modo) -> &'static Handle {
                match (self, modo) {
                    $(
                        (Lamina::$variante, Modo::Piedra) => {
                            static CELDA: OnceLock<Handle> = OnceLock::new();
                            CELDA.get_or_init(|| Handle::from_bytes(
                                &include_bytes!(concat!("../assets/laminas/", $archivo, ".webp"))[..],
                            ))
                        }
                        (Lamina::$variante, Modo::Tinta) => {
                            static CELDA: OnceLock<Handle> = OnceLock::new();
                            CELDA.get_or_init(|| Handle::from_bytes(
                                &include_bytes!(concat!("../assets/laminas/", $archivo, "-negativo.webp"))[..],
                            ))
                        }
                    )*
                }
            }
        }
    };
}

laminas! {
    CaidaCielo => "caida-cielo",
    TivoliCiudad => "tivoli-ciudad",
    HidraCabezas => "hidra-cabezas",
    MercurioFriso => "mercurio-friso",
    FarnesioCielo => "farnesio-cielo",
    CaballeroYelmo => "caballero-yelmo",
    CaballeroDiablo => "caballero-diablo",
    CastilloTorres => "castillo-torres",
    IcaroDedalo => "icaro-g-dedalo",
    MelencoliaReloj => "melencolia-reloj",
    ProdigoAldea => "prodigo-aldea",
    ValleRocas => "valle-rocas",
    DragonCabeza => "dragon-cabeza",
}

/// Alto de la banda de sección.
pub const ALTO_BANDA: f32 = 170.0;

/// Banda: lámina a sangre con el título monumental sobre un bloque de fondo
/// que corta la imagen.
pub fn banda<'a, M: 'a>(
    p: Paleta,
    lamina: Lamina,
    titulo: &str,
    conteo: Option<usize>,
) -> Element<'a, M> {
    banda_con(p, lamina, titulo, conteo, ALTO_BANDA, texto::DISPLAY_XL)
}

/// Banda con alto y tamaño de título propios, como la del editor de
/// instancia (150 px con título `display`).
pub fn banda_con<'a, M: 'a>(
    p: Paleta,
    lamina: Lamina,
    titulo: &str,
    conteo: Option<usize>,
    alto: f32,
    tam_titulo: (f32, f32),
) -> Element<'a, M> {
    let grabado = image(lamina.imagen(p.modo).clone())
        .width(Length::Fill)
        .height(alto)
        .content_fit(ContentFit::Cover);
    let mut rotulo = row![text(titulo.to_uppercase())
        .font(fuentes::DISPLAY)
        .size(tam_titulo.0)
        .line_height(iced::widget::text::LineHeight::Absolute(
            tam_titulo.1.into()
        ))
        .color(p.text)]
    .spacing(espacio::S4)
    .align_y(Alignment::End);
    if let Some(n) = conteo {
        rotulo = rotulo.push(
            text(n.to_string())
                .font(fuentes::MONO)
                .size(texto::MONO.0)
                .color(p.text_muted),
        );
    }
    let bloque = container(rotulo)
        .padding(Padding::from([espacio::S2, espacio::S6]))
        .style(move |_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(p.bg)),
            ..Default::default()
        });
    let capa = column![
        Space::with_height(Length::Fill),
        row![Space::with_width(espacio::S12), bloque]
    ]
    .height(alto);
    stack![grabado, capa].into()
}
