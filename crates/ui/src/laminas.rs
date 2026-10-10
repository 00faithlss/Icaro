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
    Icaro => "icaro",
    Hermes => "hermes",
    Faeton => "faeton",
    Rostro => "rostro",
    ApoloBelvedere => "apolo-belvedere",
    Saturno => "saturno",
    Minerva => "minerva",
    Hidra => "hidra",
    HermesHilos => "hermes-hilos",
    CaidaDeFaeton => "caida-de-faeton",
    CascadaMolino => "cascada-molino",
    RuedaRueda => "rueda-rueda",
    DragonAla => "dragon-ala",
    ValleBosque => "valle-bosque",
    PozoCuerdas => "pozo-cuerdas",
    HuidaBosque => "huida-bosque",
    ArcoPuente => "arco-puente",
    CadenasReja => "cadenas-reja",
    FuegoArco => "fuego-arco",
    EscaleraPuentes => "escalera-puentes",
    TorreReja => "torre-reja",
    PuenteTorre => "puente-torre",
    MercadoCarros => "mercado-carros",
    SoldadosAldea => "soldados-aldea",
    TivoliRocas => "tivoli-rocas",
    FortalezasValle => "fortalezas-valle",
    CabanaCasas => "cabana-casas",
    OceanoPez => "oceano-pez",
    TetisDelfin => "tetis-delfin",
    JeronimoCalabaza => "jeronimo-calabaza",
    CastilloPasarela => "castillo-pasarela",
    TivoliCascada => "tivoli-cascada",
    MercadoAldea => "mercado-aldea",
    ParejaAldea => "pareja-aldea",
    CascadaAgua => "cascada-agua",
    SoldadosCastillo => "soldados-castillo",
    IxionNubes => "ixion-nubes",
    DemogorgonCueva => "demogorgon-cueva",
    MelencoliaCometa => "melencolia-cometa",
    HuidaCiudad => "huida-ciudad",
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
impl Lamina {
    /// Punto de la imagen que debe quedar visible al recortarla, de 0 a 1
    /// (como `object-position`): evita cortar rostros y figuras.
    pub const fn foco(self) -> (f32, f32) {
        // Centrado, como antes: no se mueve el encuadre de ninguna lámina.
        (0.5, 0.5)
    }
}

/// Grabado que llena su caja recortando el sobrante, sin deformarlo.
///
/// El `Image` de iced con `Cover` abre una capa de recorte con sus propios
/// límites, que ignora el recorte del `scrollable`, y la imagen se sale por
/// arriba al desplazar. Este widget dibuja la imagen en una capa limitada a
/// la parte visible, centrada en el foco de la lámina.
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
        foco: lamina.foco(),
        zoom: false,
        entero: false,
    })
}

/// Grabado entero y centrado, sin recortar (figuras sobre fondo transparente y
/// medallones). No se amplía más de 1,5 veces para no perder nitidez.
pub fn grabado_entero<'a, M: 'a>(
    p: Paleta,
    lamina: Lamina,
    ancho: impl Into<Length>,
    alto: impl Into<Length>,
) -> Element<'a, M> {
    Element::new(Grabado {
        handle: lamina.imagen(p.modo).clone(),
        ancho: ancho.into(),
        alto: alto.into(),
        foco: (0.5, 0.5),
        zoom: false,
        entero: true,
    })
}

/// Igual que [`grabado`], con el acercamiento lento de 7 % en 1,2 s al pasar
/// el puntero (la única animación de una imagen en el sistema).
pub fn grabado_zoom<'a, M: 'a>(
    p: Paleta,
    lamina: Lamina,
    ancho: impl Into<Length>,
    alto: impl Into<Length>,
) -> Element<'a, M> {
    Element::new(Grabado {
        handle: lamina.imagen(p.modo).clone(),
        ancho: ancho.into(),
        alto: alto.into(),
        foco: lamina.foco(),
        zoom: true,
        entero: false,
    })
}

struct Grabado {
    handle: Handle,
    ancho: Length,
    alto: Length,
    foco: (f32, f32),
    zoom: bool,
    /// Se muestra entera, sin recortar, centrada en la caja.
    entero: bool,
}

#[derive(Default)]
struct EstadoZoom {
    /// Escala actual, de 1,0 a 1,07.
    escala: f32,
    encima: bool,
    ultimo: Option<std::time::Instant>,
}

const ZOOM_MAX: f32 = 1.07;
const ZOOM_MS: f32 = 1200.0;

impl<M> iced::advanced::Widget<M, iced::Theme, iced::Renderer> for Grabado {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<EstadoZoom>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(EstadoZoom { escala: 1.0, ..Default::default() })
    }

    fn size(&self) -> iced::Size<Length> {
        iced::Size::new(self.ancho, self.alto)
    }

    fn layout(
        &self,
        _arbol: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limites: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        if self.entero {
            // La caja toma la proporción de la imagen, para que el marco la
            // abrace en vez de dejar bandas vacías.
            use iced::advanced::image::Renderer as _;
            let tam = renderer.measure_image(&self.handle);
            let max = limites.max();
            if tam.width > 0 && tam.height > 0 && max.width.is_finite() {
                let proporcion = tam.height as f32 / tam.width as f32;
                let mut w = max.width;
                let mut h = w * proporcion;
                if max.height.is_finite() && h > max.height {
                    h = max.height;
                    w = h / proporcion;
                }
                return iced::advanced::layout::Node::new(iced::Size::new(w, h));
            }
        }
        iced::advanced::layout::atomic(limites, self.ancho, self.alto)
    }

    fn on_event(
        &mut self,
        arbol: &mut iced::advanced::widget::Tree,
        evento: iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &iced::Renderer,
        _portapapeles: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        _visible: &iced::Rectangle,
    ) -> iced::event::Status {
        if !self.zoom {
            return iced::event::Status::Ignored;
        }
        let estado = arbol.state.downcast_mut::<EstadoZoom>();
        match evento {
            iced::Event::Mouse(_) => {
                let encima = cursor.is_over(layout.bounds());
                if encima != estado.encima {
                    estado.encima = encima;
                    estado.ultimo = None;
                    shell.request_redraw(iced::window::RedrawRequest::NextFrame);
                }
            }
            iced::Event::Window(iced::window::Event::RedrawRequested(ahora)) => {
                let meta = if estado.encima { ZOOM_MAX } else { 1.0 };
                if (estado.escala - meta).abs() > 0.0005 {
                    let dt = estado.ultimo.map_or(0.016, |u| ahora.duration_since(u).as_secs_f32() * 1000.0 / 1000.0);
                    estado.ultimo = Some(ahora);
                    let paso = (ZOOM_MAX - 1.0) * dt * 1000.0 / ZOOM_MS;
                    estado.escala = if estado.escala < meta {
                        (estado.escala + paso).min(meta)
                    } else {
                        (estado.escala - paso * 2.0).max(meta)
                    };
                    shell.request_redraw(iced::window::RedrawRequest::NextFrame);
                }
            }
            _ => {}
        }
        iced::event::Status::Ignored
    }

    fn draw(
        &self,
        arbol: &iced::advanced::widget::Tree,
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
        let escala_zoom = arbol.state.downcast_ref::<EstadoZoom>().escala.max(1.0);
        let (iw, ih) = (tam.width as f32, tam.height as f32);
        let escala = if self.entero {
            (caja.width / iw).min(caja.height / ih)
        } else {
            (caja.width / iw).max(caja.height / ih) * escala_zoom
        };
        let (w, h) = (iw * escala, ih * escala);
        // El foco de la imagen queda lo más cerca posible del centro de la caja.
        let (x, y) = if self.entero {
            (caja.center_x() - w / 2.0, caja.center_y() - h / 2.0)
        } else {
            (
                (caja.center_x() - self.foco.0 * w).clamp(caja.x + caja.width - w, caja.x),
                (caja.center_y() - self.foco.1 * h).clamp(caja.y + caja.height - h, caja.y),
            )
        };
        let dibujo = iced::Rectangle { x, y, width: w, height: h };
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
