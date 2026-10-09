// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Láminas de grabado: imágenes de dominio público (CC0) que abren cada
//! sección. Cada una existe en positivo (tema Piedra) y en negativo (Tinta).

use std::sync::OnceLock;

use iced::widget::image::Handle;
use iced::widget::{column, container, row, stack, text, Space};
use iced::{Alignment, Element, Length, Padding};

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
    CaballeroCastillo => "caballero-castillo",
    FaetonMano => "faeton-mano",
    PiranesiRueda => "piranesi-rueda",
    FaetonPaisaje => "faeton-paisaje",
}

/// Alto de la banda de sección.
pub const ALTO_BANDA: f32 = 200.0;

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
    let grabado = self::grabado(p, lamina, Length::Fill, alto);
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


/// Grabado que llena su caja recortando el sobrante, centrado y sin
/// deformarlo (como `ContentFit::Cover`).
///
/// El `Image` de iced con `Cover` abre una capa de recorte con sus propios
/// límites, que ignora el recorte del `scrollable`, y la imagen se sale por
/// arriba al desplazar. Este widget dibuja la imagen en una capa limitada a
/// la parte visible.
pub fn grabado<'a, M: 'a>(
    p: Paleta,
    lamina: Lamina,
    ancho: impl Into<Length>,
    alto: impl Into<Length>,
) -> Element<'a, M> {
    Element::new(Grabado {
        handle: lamina.imagen(p.modo).clone(),
        ancho: ancho.into(),
        alto: alto.into(),
    })
}

struct Grabado {
    handle: Handle,
    ancho: Length,
    alto: Length,
}

impl<M> iced::advanced::Widget<M, iced::Theme, iced::Renderer> for Grabado {
    fn size(&self) -> iced::Size<Length> {
        iced::Size::new(self.ancho, self.alto)
    }

    fn layout(
        &self,
        _arbol: &mut iced::advanced::widget::Tree,
        _renderer: &iced::Renderer,
        limites: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        iced::advanced::layout::atomic(limites, self.ancho, self.alto)
    }

    fn draw(
        &self,
        _arbol: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        _tema: &iced::Theme,
        _estilo: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        visible: &iced::Rectangle,
    ) {
        use iced::advanced::image::Renderer as _;
        use iced::advanced::Renderer as _;

        let caja = layout.bounds();
        let Some(recorte) = caja.intersection(visible) else {
            return;
        };
        let tam = renderer.measure_image(&self.handle);
        if tam.width == 0 || tam.height == 0 || caja.width <= 0.0 || caja.height <= 0.0 {
            return;
        }
        let (iw, ih) = (tam.width as f32, tam.height as f32);
        let escala = (caja.width / iw).max(caja.height / ih);
        let dibujo = iced::Rectangle {
            x: caja.center_x() - iw * escala / 2.0,
            y: caja.center_y() - ih * escala / 2.0,
            width: iw * escala,
            height: ih * escala,
        };
        renderer.with_layer(recorte, |renderer| {
            renderer.draw_image(
                iced::advanced::image::Image {
                    handle: self.handle.clone(),
                    filter_method: iced::advanced::image::FilterMethod::Linear,
                    rotation: iced::Radians(0.0),
                    opacity: 1.0,
                    snap: true,
                },
                dibujo,
            );
        });
    }
}
