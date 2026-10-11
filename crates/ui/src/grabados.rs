// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Acabados de imprenta para los grabados: el barrido de 3 px, las marcas de
//! registro del Detalle, el acercamiento lento, la Prensa, el friso de
//! meandro y el mosaico del Gabinete.

use std::time::Instant;

use iced::advanced::image::{Image, Renderer as _};
use iced::advanced::layout::{self, Layout, Node};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{overlay, Clipboard, Renderer as _, Shell, Widget};
use iced::widget::image::Handle;
use iced::{event, mouse, window, Color, Element, Event, Length, Point, Rectangle, Size, Vector};

use crate::laminas::Lamina;
use crate::movimiento::{Curva, Preferencia, Transicion};
use crate::tema::{Modo, Paleta};

/// Opacidad de las líneas del barrido: más firme sobre el negativo de Tinta.
fn alfa_barrido(modo: Modo) -> f32 {
    match modo {
        Modo::Piedra => 0.14,
        Modo::Tinta => 0.32,
    }
}

/// Acercamiento lento de una imagen al pasar el puntero.
#[derive(Debug, Clone, Copy)]
pub struct Acercar {
    /// Escala final (1,08 es un 8 %).
    pub escala: f32,
    pub ms: u64,
    pub curva: Curva,
}

/// Imagen de grabado: llena su caja recortando el sobrante (`object-fit:
/// cover`) con el encuadre de `object-position`, y le suma el barrido, las
/// marcas de registro, el acercamiento y la Prensa según se pida.
pub struct Imagen {
    handle: Handle,
    ancho: Length,
    alto: Length,
    foco: (f32, f32),
    barrido: Option<f32>,
    registro: Option<Color>,
    acercar: Option<Acercar>,
    encima: Option<bool>,
    pref: Preferencia,
    prensa: f32,
    tinta_prensa: Color,
    fondo_prensa: Color,
}

impl Imagen {
    pub fn nueva(p: Paleta, lamina: Lamina) -> Self {
        Self {
            handle: lamina.imagen(p.modo).clone(),
            ancho: Length::Fill,
            alto: Length::Fill,
            foco: (0.5, 0.5),
            barrido: None,
            registro: None,
            acercar: None,
            encima: None,
            pref: Preferencia::default(),
            prensa: 1.0,
            tinta_prensa: p.text,
            fondo_prensa: p.bg,
        }
    }

    pub fn tamano(mut self, ancho: impl Into<Length>, alto: impl Into<Length>) -> Self {
        self.ancho = ancho.into();
        self.alto = alto.into();
        self
    }

    /// Encuadre como `object-position`, de 0 a 1 en cada eje.
    pub fn foco(mut self, x: f32, y: f32) -> Self {
        self.foco = (x, y);
        self
    }

    /// Barrido horizontal de 3 px encima, propio del tema.
    pub fn barrido(mut self, p: Paleta) -> Self {
        self.barrido = Some(alfa_barrido(p.modo));
        self
    }

    /// Marcas de registro en las cuatro esquinas, separadas 7 px.
    pub fn registro(mut self, color: Color) -> Self {
        self.registro = Some(color);
        self
    }

    /// Acercamiento lento al pasar el puntero. Con movimiento reducido no hay.
    pub fn acercar(mut self, acercar: Acercar, pref: Preferencia) -> Self {
        self.acercar = Some(acercar);
        self.pref = pref;
        self
    }

    /// Fuerza el estado de "puntero encima" cuando lo decide un contenedor
    /// mayor (la pieza entera del Gabinete, con su leyenda).
    pub fn encima(mut self, encima: bool) -> Self {
        self.encima = Some(encima);
        self
    }

    /// Prensa: avance de 0 a 1 de la impresión de arriba abajo.
    pub fn prensa(mut self, avance: f32) -> Self {
        self.prensa = avance.clamp(0.0, 1.0);
        self
    }
}

struct EstadoImagen {
    zoom: Transicion,
    encima: bool,
}

impl<M> Widget<M, iced::Theme, iced::Renderer> for Imagen {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<EstadoImagen>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(EstadoImagen {
            zoom: Transicion::fija(1.0, Instant::now()),
            encima: false,
        })
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.ancho, self.alto)
    }

    fn layout(
        &self,
        _arbol: &mut Tree,
        _renderer: &iced::Renderer,
        limites: &layout::Limits,
    ) -> Node {
        layout::atomic(limites, self.ancho, self.alto)
    }

    fn on_event(
        &mut self,
        arbol: &mut Tree,
        evento: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _portapapeles: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        _visible: &Rectangle,
    ) -> event::Status {
        let Some(acercar) = self.acercar else {
            return event::Status::Ignored;
        };
        let estado = arbol.state.downcast_mut::<EstadoImagen>();
        let ahora = Instant::now();
        let encima = self
            .encima
            .unwrap_or_else(|| cursor.is_over(layout.bounds()));
        if encima != estado.encima {
            estado.encima = encima;
            let meta = if encima { acercar.escala } else { 1.0 };
            estado
                .zoom
                .ir_a(meta, ahora, acercar.ms, acercar.curva, self.pref);
            shell.request_redraw(window::RedrawRequest::NextFrame);
        }
        if matches!(evento, Event::Window(window::Event::RedrawRequested(_)))
            && estado.zoom.animando(ahora)
        {
            shell.request_redraw(window::RedrawRequest::NextFrame);
        }
        event::Status::Ignored
    }

    fn draw(
        &self,
        arbol: &Tree,
        renderer: &mut iced::Renderer,
        _tema: &iced::Theme,
        _estilo: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        visible: &Rectangle,
    ) {
        let caja = layout.bounds();
        let tam = renderer.measure_image(&self.handle);
        if tam.width == 0 || tam.height == 0 || caja.width <= 0.0 || caja.height <= 0.0 {
            return;
        }
        let escala_zoom = if self.pref.reducir {
            1.0
        } else {
            arbol
                .state
                .downcast_ref::<EstadoImagen>()
                .zoom
                .valor(Instant::now())
        };
        let (iw, ih) = (tam.width as f32, tam.height as f32);
        let cubre = (caja.width / iw).max(caja.height / ih);
        let (w, h) = (iw * cubre, ih * cubre);
        let x = caja.x + (caja.width - w) * self.foco.0;
        let y = caja.y + (caja.height - h) * self.foco.1;
        // El acercamiento parte del centro de la caja, como `transform: scale`.
        let (cx, cy) = (caja.center_x(), caja.center_y());
        let dibujo = Rectangle {
            x: cx + (x - cx) * escala_zoom,
            y: cy + (y - cy) * escala_zoom,
            width: w * escala_zoom,
            height: h * escala_zoom,
        };

        // La Prensa descubre la imagen de arriba abajo.
        let impresa = Rectangle { height: caja.height * self.prensa, ..caja };
        if let Some(recorte) = impresa.intersection(visible) {
            renderer.with_layer(recorte, |renderer| {
                renderer.draw_image(
                    Image {
                        handle: self.handle.clone(),
                        filter_method: iced::advanced::image::FilterMethod::Linear,
                        rotation: iced::Radians(0.0),
                        opacity: 1.0,
                        snap: true,
                    },
                    dibujo,
                );
                if let Some(alfa) = self.barrido {
                    let linea = Color { a: alfa, ..Color::BLACK };
                    let mut yl = caja.y + 2.0;
                    while yl < caja.y + caja.height {
                        renderer.fill_quad(
                            Quad {
                                bounds: Rectangle { x: caja.x, y: yl, width: caja.width, height: 1.0 },
                                ..Default::default()
                            },
                            linea,
                        );
                        yl += 3.0;
                    }
                }
            });
        }
        // Barra de tinta de la Prensa, con un filete del color de fondo.
        if self.prensa < 1.0 {
            let barra = Rectangle {
                x: caja.x,
                y: caja.y + (caja.height - 3.0).max(0.0) * self.prensa,
                width: caja.width,
                height: 3.0,
            };
            let filete = Rectangle {
                x: barra.x - 1.0,
                y: barra.y - 1.0,
                width: barra.width + 2.0,
                height: barra.height + 2.0,
            };
            renderer.fill_quad(Quad { bounds: filete, ..Default::default() }, self.fondo_prensa);
            renderer.fill_quad(Quad { bounds: barra, ..Default::default() }, self.tinta_prensa);
        }
        if let Some(color) = self.registro {
            marcas_de_registro(renderer, caja, color);
        }
    }
}

impl<'a, M: 'a> From<Imagen> for Element<'a, M> {
    fn from(imagen: Imagen) -> Self {
        Element::new(imagen)
    }
}

/// Esquinas en L de 14 px y 2 px de grosor, centradas en cada vértice de la
/// imagen (7 px hacia afuera).
fn marcas_de_registro(renderer: &mut iced::Renderer, caja: Rectangle, color: Color) {
    const LARGO: f32 = 14.0;
    const GROSOR: f32 = 2.0;
    const SEPARACION: f32 = 7.0;
    let izq = caja.x - SEPARACION;
    let der = caja.x + caja.width + SEPARACION - LARGO;
    let arriba = caja.y - SEPARACION;
    let abajo = caja.y + caja.height + SEPARACION - LARGO;
    for (x, y, h_abajo, v_der) in [
        (izq, arriba, false, false),
        (der, arriba, false, true),
        (izq, abajo, true, false),
        (der, abajo, true, true),
    ] {
        let y_h = if h_abajo { y + LARGO - GROSOR } else { y };
        let x_v = if v_der { x + LARGO - GROSOR } else { x };
        renderer.fill_quad(
            Quad {
                bounds: Rectangle { x, y: y_h, width: LARGO, height: GROSOR },
                ..Default::default()
            },
            color,
        );
        renderer.fill_quad(
            Quad {
                bounds: Rectangle { x: x_v, y, width: GROSOR, height: LARGO },
                ..Default::default()
            },
            color,
        );
    }
}

// ---------------------------------------------------------------- friso

/// Rectángulos del meandro en una celda de 8 × 9 píxeles de arte.
const MEANDRO: [(f32, f32, f32, f32); 14] = [
    (0.0, 0.0, 8.0, 1.0),
    (0.0, 2.0, 7.0, 1.0),
    (0.0, 3.0, 1.0, 1.0),
    (6.0, 3.0, 1.0, 1.0),
    (0.0, 4.0, 1.0, 1.0),
    (2.0, 4.0, 3.0, 1.0),
    (6.0, 4.0, 1.0, 1.0),
    (0.0, 5.0, 1.0, 1.0),
    (2.0, 5.0, 1.0, 1.0),
    (6.0, 5.0, 1.0, 1.0),
    (0.0, 6.0, 1.0, 1.0),
    (2.0, 6.0, 5.0, 1.0),
    (0.0, 7.0, 1.0, 1.0),
    (0.0, 8.0, 8.0, 1.0),
];

/// Alto del friso y ancho de cada celda repetida (píxel de arte de 2 px).
pub const ALTO_FRISO: f32 = 18.0;
const CELDA_FRISO: f32 = 16.0;

struct Friso {
    ancho: f32,
    color: Color,
}

impl<M> Widget<M, iced::Theme, iced::Renderer> for Friso {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.ancho), Length::Fixed(ALTO_FRISO))
    }

    fn layout(
        &self,
        _arbol: &mut Tree,
        _renderer: &iced::Renderer,
        limites: &layout::Limits,
    ) -> Node {
        layout::atomic(limites, Length::Fixed(self.ancho), Length::Fixed(ALTO_FRISO))
    }

    fn draw(
        &self,
        _arbol: &Tree,
        renderer: &mut iced::Renderer,
        _tema: &iced::Theme,
        _estilo: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        visible: &Rectangle,
    ) {
        let caja = layout.bounds();
        let Some(recorte) = caja.intersection(visible) else {
            return;
        };
        let k = CELDA_FRISO / 8.0;
        renderer.with_layer(recorte, |renderer| {
            let mut x0 = caja.x;
            while x0 < caja.x + caja.width {
                for (x, y, w, h) in MEANDRO {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle {
                                x: x0 + x * k,
                                y: caja.y + y * k,
                                width: w * k,
                                height: h * k,
                            },
                            ..Default::default()
                        },
                        self.color,
                    );
                }
                x0 += CELDA_FRISO;
            }
        });
    }
}

/// Friso de meandro en píxeles. Solo en estados vacíos, primera ejecución y
/// Acerca de, a 160-200 px de ancho.
pub fn friso<'a, M: 'a>(p: Paleta, ancho: f32) -> Element<'a, M> {
    Element::new(Friso { ancho, color: Color { a: 0.9, ..p.text } })
}

// -------------------------------------------------------------- mosaico

/// Reparte piezas de `(columnas, filas)` en una cuadrícula de `columnas`
/// con el mismo orden que el auto-emplazamiento de CSS Grid: el cursor solo
/// avanza, así que una pieza nunca rellena un hueco que quedó atrás.
/// Devuelve `(columna, fila)` de cada pieza.
pub fn empaquetar(piezas: &[(usize, usize)], columnas: usize) -> Vec<(usize, usize)> {
    let mut ocupado: Vec<Vec<bool>> = Vec::new();
    let libre = |ocupado: &Vec<Vec<bool>>, c: usize, f: usize, ancho: usize, alto: usize| {
        (f..f + alto).all(|fila| {
            ocupado
                .get(fila)
                .is_none_or(|celdas| (c..c + ancho).all(|col| !celdas[col]))
        })
    };
    let (mut col, mut fila) = (0, 0);
    let mut puestas = Vec::with_capacity(piezas.len());
    for &(ancho, alto) in piezas {
        let ancho = ancho.min(columnas);
        loop {
            if col + ancho > columnas {
                col = 0;
                fila += 1;
                continue;
            }
            if libre(&ocupado, col, fila, ancho, alto) {
                break;
            }
            col += 1;
        }
        while ocupado.len() < fila + alto {
            ocupado.push(vec![false; columnas]);
        }
        for celdas in &mut ocupado[fila..fila + alto] {
            celdas[col..col + ancho].fill(true);
        }
        puestas.push((col, fila));
    }
    puestas
}

/// Cuadrícula de 12 columnas con filas fijas: cada hija ocupa entre una y
/// varias columnas y filas.
pub struct Mosaico<'a, M> {
    piezas: Vec<(usize, usize)>,
    hijas: Vec<Element<'a, M>>,
    columnas: usize,
    fila: f32,
    hueco: f32,
}

impl<'a, M> Mosaico<'a, M> {
    pub fn nuevo(columnas: usize, fila: f32, hueco: f32) -> Self {
        Self { piezas: Vec::new(), hijas: Vec::new(), columnas, fila, hueco }
    }

    /// Agrega una pieza de `columnas` × `filas`.
    pub fn push(mut self, columnas: usize, filas: usize, pieza: impl Into<Element<'a, M>>) -> Self {
        self.piezas.push((columnas, filas));
        self.hijas.push(pieza.into());
        self
    }
}

impl<'a, M> Widget<M, iced::Theme, iced::Renderer> for Mosaico<'a, M> {
    fn children(&self) -> Vec<Tree> {
        self.hijas.iter().map(Tree::new).collect()
    }

    fn diff(&self, arbol: &mut Tree) {
        arbol.diff_children(&self.hijas);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &self,
        arbol: &mut Tree,
        renderer: &iced::Renderer,
        limites: &layout::Limits,
    ) -> Node {
        let ancho = limites.max().width;
        let cols = self.columnas as f32;
        let col_w = (ancho - self.hueco * (cols - 1.0)) / cols;
        let sitios = empaquetar(&self.piezas, self.columnas);
        let mut filas_total = 0;
        let mut nodos = Vec::with_capacity(self.hijas.len());
        for (i, hija) in self.hijas.iter().enumerate() {
            let (c, f) = sitios[i];
            let (span_c, span_f) = self.piezas[i];
            filas_total = filas_total.max(f + span_f);
            let w = span_c as f32 * col_w + (span_c as f32 - 1.0) * self.hueco;
            let h = span_f as f32 * self.fila + (span_f as f32 - 1.0) * self.hueco;
            let fijo = Size::new(w, h);
            let nodo = hija
                .as_widget()
                .layout(&mut arbol.children[i], renderer, &layout::Limits::new(fijo, fijo))
                .move_to(Point::new(
                    c as f32 * (col_w + self.hueco),
                    f as f32 * (self.fila + self.hueco),
                ));
            nodos.push(nodo);
        }
        let alto = if filas_total == 0 {
            0.0
        } else {
            filas_total as f32 * self.fila + (filas_total as f32 - 1.0) * self.hueco
        };
        Node::with_children(Size::new(ancho, alto), nodos)
    }

    fn operate(
        &self,
        arbol: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operacion: &mut dyn iced::advanced::widget::Operation,
    ) {
        operacion.container(None, layout.bounds(), &mut |operacion| {
            self.hijas
                .iter()
                .zip(&mut arbol.children)
                .zip(layout.children())
                .for_each(|((hija, estado), layout)| {
                    hija.as_widget().operate(estado, layout, renderer, operacion);
                });
        });
    }

    fn on_event(
        &mut self,
        arbol: &mut Tree,
        evento: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        portapapeles: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        visible: &Rectangle,
    ) -> event::Status {
        self.hijas
            .iter_mut()
            .zip(&mut arbol.children)
            .zip(layout.children())
            .map(|((hija, estado), layout)| {
                hija.as_widget_mut().on_event(
                    estado,
                    evento.clone(),
                    layout,
                    cursor,
                    renderer,
                    portapapeles,
                    shell,
                    visible,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        arbol: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        visible: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.hijas
            .iter()
            .zip(&arbol.children)
            .zip(layout.children())
            .map(|((hija, estado), layout)| {
                hija.as_widget()
                    .mouse_interaction(estado, layout, cursor, visible, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        arbol: &Tree,
        renderer: &mut iced::Renderer,
        tema: &iced::Theme,
        estilo: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        visible: &Rectangle,
    ) {
        for ((hija, estado), layout) in self
            .hijas
            .iter()
            .zip(&arbol.children)
            .zip(layout.children())
        {
            hija.as_widget()
                .draw(estado, renderer, tema, estilo, layout, cursor, visible);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        arbol: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        traslado: Vector,
    ) -> Option<overlay::Element<'b, M, iced::Theme, iced::Renderer>> {
        overlay::from_children(&mut self.hijas, arbol, layout, renderer, traslado)
    }
}

impl<'a, M: 'a> From<Mosaico<'a, M>> for Element<'a, M> {
    fn from(mosaico: Mosaico<'a, M>) -> Self {
        Element::new(mosaico)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Las diez piezas del Gabinete de la preview del sistema.
    const GABINETE: [(usize, usize); 10] =
        [(5, 5), (4, 4), (3, 7), (4, 3), (3, 4), (2, 4), (4, 5), (3, 4), (5, 4), (7, 2)];

    #[test]
    fn el_gabinete_empieza_como_la_preview() {
        let sitios = empaquetar(&GABINETE, 12);
        assert_eq!(sitios[0], (0, 0));
        assert_eq!(sitios[1], (5, 0));
        assert_eq!(sitios[2], (9, 0));
        // La cuarta cabe bajo la segunda, en el hueco de la primera fila libre.
        assert_eq!(sitios[3], (5, 4));
    }

    #[test]
    fn ninguna_pieza_se_superpone_ni_se_sale() {
        let sitios = empaquetar(&GABINETE, 12);
        let mut ocupado = std::collections::HashSet::new();
        for (&(ancho, alto), &(c, f)) in GABINETE.iter().zip(&sitios) {
            assert!(c + ancho <= 12);
            for fila in f..f + alto {
                for col in c..c + ancho {
                    assert!(ocupado.insert((col, fila)), "celda repetida {col},{fila}");
                }
            }
        }
    }
}
