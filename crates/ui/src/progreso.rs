// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Progreso del sistema de diseño. Cada widget solicita cuadros solamente
//! mientras está visible y animado; reducir movimiento lo deja estático.

use std::time::Instant;
use iced::advanced::{layout, renderer, widget::{tree, Tree}, Clipboard, Layout, Shell, Widget};
use iced::advanced::Renderer as _;
use iced::{mouse, Color, Element, Event, Length, Point, Rectangle, Size};
use crate::tema::Paleta;

#[derive(Clone, Copy)]
enum Tipo { Continuo(f32), Bloques(usize, usize), Indeterminado, Spinner, Esqueleto }

pub fn continuo<'a, M: 'a>(p: Paleta, avance: f32, reducir: bool) -> Element<'a, M> {
    crear(p, Tipo::Continuo(avance.clamp(0., 1.)), Length::Fill, Length::Fixed(6.), reducir)
}
pub fn bloques<'a, M: 'a>(p: Paleta, total: usize, hechos: usize, reducir: bool) -> Element<'a, M> {
    crear(p, Tipo::Bloques(total, hechos.min(total)), Length::Fill, Length::Fixed(14.), reducir)
}
pub fn indeterminado<'a, M: 'a>(p: Paleta, reducir: bool) -> Element<'a, M> {
    crear(p, Tipo::Indeterminado, Length::Fill, Length::Fixed(6.), reducir)
}
pub fn spinner<'a, M: 'a>(p: Paleta, reducir: bool) -> Element<'a, M> {
    crear(p, Tipo::Spinner, Length::Fixed(18.), Length::Fixed(18.), reducir)
}
pub fn esqueleto<'a, M: 'a>(p: Paleta, ancho: impl Into<Length>, alto: impl Into<Length>, reducir: bool) -> Element<'a, M> {
    crear(p, Tipo::Esqueleto, ancho.into(), alto.into(), reducir)
}
fn crear<'a, M: 'a>(p: Paleta, tipo: Tipo, ancho: Length, alto: Length, reducir: bool) -> Element<'a, M> {
    Element::new(Progreso { p, tipo, ancho, alto, reducir })
}
struct Progreso { p: Paleta, tipo: Tipo, ancho: Length, alto: Length, reducir: bool }
struct Tiempo { inicio: Instant, cambio: Instant, de: f32, a: f32 }
impl Default for Tiempo {
    fn default() -> Self { let ahora = Instant::now(); Self { inicio: ahora, cambio: ahora, de: 0., a: 0. } }
}
fn salida(t: f32) -> f32 {
    if t <= 0. { 0. } else if t >= 1. { 1. } else { crate::movimiento::Curva::Salida.aplicar(t) }
}
impl Tiempo {
    fn valor(&self, ahora: Instant) -> f32 {
        self.de + (self.a - self.de) * salida(ahora.saturating_duration_since(self.cambio).as_secs_f32() / 0.2)
    }
}
impl<M> Widget<M, iced::Theme, iced::Renderer> for Progreso {
    fn tag(&self) -> tree::Tag { tree::Tag::of::<Tiempo>() }
    fn state(&self) -> tree::State {
        let mut t = Tiempo::default();
        if let Tipo::Continuo(v) = self.tipo { t.de = v; t.a = v; }
        tree::State::new(t)
    }
    fn diff(&self, arbol: &mut Tree) {
        if let Tipo::Continuo(v) = self.tipo {
            let t = arbol.state.downcast_mut::<Tiempo>();
            if v != t.a {
                let ahora = Instant::now();
                t.de = t.valor(ahora); t.a = v; t.cambio = ahora;
            }
        }
    }
    fn size(&self) -> Size<Length> { Size::new(self.ancho, self.alto) }
    fn layout(&self, _: &mut Tree, _: &iced::Renderer, limites: &layout::Limits) -> layout::Node {
        layout::atomic(limites, self.ancho, self.alto)
    }
    fn on_event(&mut self, arbol: &mut Tree, evento: Event, lugar: Layout<'_>, _: mouse::Cursor,
        _: &iced::Renderer, _: &mut dyn Clipboard, shell: &mut Shell<'_, M>, visible: &Rectangle) -> iced::event::Status {
        if self.reducir || lugar.bounds().intersection(visible).is_none() { return iced::event::Status::Ignored; }
        if matches!(evento, Event::Window(iced::window::Event::RedrawRequested(_))) {
            let t = arbol.state.downcast_ref::<Tiempo>();
            let activa = match self.tipo {
                Tipo::Continuo(_) => t.cambio.elapsed().as_millis() < 200,
                Tipo::Bloques(total, hechos) => total > 0 && hechos < total,
                _ => true,
            };
            if activa { shell.request_redraw(iced::window::RedrawRequest::NextFrame); }
        }
        iced::event::Status::Ignored
    }
    fn draw(&self, arbol: &Tree, renderer: &mut iced::Renderer, _: &iced::Theme,
        _: &renderer::Style, lugar: Layout<'_>, _: mouse::Cursor, visible: &Rectangle) {
        let r = lugar.bounds(); let Some(recorte) = r.intersection(visible) else { return; };
        let t = arbol.state.downcast_ref::<Tiempo>();
        let ms = if self.reducir { 0 } else { t.inicio.elapsed().as_millis() as u64 };
        let p = self.p;
        renderer.with_layer(recorte, |renderer| {
            let mut rect = |r: Rectangle, color: Color, borde: bool| {
                renderer.fill_quad(renderer::Quad { bounds: r, border: iced::Border {
                    width: if borde { 1. } else { 0. }, color: p.border, ..Default::default() },
                    ..Default::default() }, color);
            };
            match self.tipo {
                Tipo::Continuo(v) => {
                    rect(r, p.surface_sunken, true);
                    let v = if self.reducir { v } else { t.valor(Instant::now()) };
                    rect(Rectangle { width: r.width * v, ..r }, p.text, false);
                }
                Tipo::Indeterminado => {
                    rect(r, p.surface_sunken, true);
                    let x = if self.reducir { 0.35 } else { (ms % 1100) as f32 / 1100. * 1.32 - 0.3 };
                    // Recortar la franja dentro de la pista, también al entrar/salir.
                    let franja = Rectangle { x: r.x + r.width * x, width: r.width * 0.3, ..r };
                    if let Some(f) = franja.intersection(&r) { rect(f, p.text, false); }
                }
                Tipo::Bloques(total, hechos) => {
                    let n = total.clamp(10, 24);
                    let llenos = if total == 0 { 0 } else { hechos.saturating_mul(n) / total };
                    let w = ((r.width - 2. * (n - 1) as f32) / n as f32).max(0.);
                    for i in 0..n {
                        let lleno = i < llenos;
                        let actual = total > 0 && hechos < total && i == llenos;
                        let color = if lleno { p.text } else if actual {
                            Color { a: if self.reducir || ms % 600 < 300 { 1. } else { 0.35 }, ..p.text }
                        } else { p.surface_sunken };
                        rect(Rectangle { x: r.x + i as f32 * (w + 2.), width: w, ..r }, color, !lleno && !actual);
                    }
                }
                Tipo::Spinner => {
                    let activa = (ms / 200 % 4) as usize;
                    for (i, (x, y)) in [(0., 0.), (10., 0.), (10., 10.), (0., 10.)].into_iter().enumerate() {
                        rect(Rectangle::new(Point::new(r.x + x, r.y + y), Size::new(8., 8.)),
                            Color { a: if self.reducir || i == activa { 1. } else { 0.2 }, ..p.text }, false);
                    }
                }
                Tipo::Esqueleto => {
                    let paso = (ms % 1400) / 350;
                    let alfa = if self.reducir { 1. } else { [1., 0.775, 0.55, 0.775][paso as usize] };
                    rect(r, p.surface_sunken, false);
                    let mut y = r.y;
                    while y < r.y + r.height {
                        rect(Rectangle { y, height: 2f32.min(r.y + r.height - y), ..r }, Color { a: alfa, ..p.border }, false);
                        y += 4.;
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn continuo_anima_desde_valor_visible_sin_saltar_ante_un_nuevo_destino() {
        let inicio = Instant::now();
        let mut t = Tiempo { inicio, cambio: inicio, de: 0., a: 1. };
        let mitad = inicio + std::time::Duration::from_millis(100);
        let visible = t.valor(mitad);
        assert!(visible > 0.5 && visible < 1.);
        t.de = visible; t.a = 0.5; t.cambio = mitad;
        assert_eq!(t.valor(mitad), visible);
        assert_eq!(t.valor(mitad + std::time::Duration::from_millis(200)), 0.5);
    }
}
