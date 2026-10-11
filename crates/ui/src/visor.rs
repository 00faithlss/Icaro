// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Visor a pantalla completa de capturas y de piezas del Gabinete: velo casi
//! opaco, la imagen centrada sin marco, barra superior con nombre y acciones,
//! flechas para recorrer y zoom en pasos de 25 %. Apertura y cierre inmediatos.

use std::time::Instant;

use iced::advanced::image::{Image, Renderer as _};
use iced::advanced::layout::{self, Layout, Node};
use iced::advanced::renderer;
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{Clipboard, Renderer as _, Shell, Widget};
use iced::widget::image::Handle;
use iced::widget::{button, column, container, opaque, row, text, Space};
use iced::{
    event, mouse, window, Alignment, Background, Border, Color, Element, Event, Length, Padding,
    Point, Rectangle, Size, Vector,
};

use crate::botones::{boton_estilo, boton_icono, TamBoton};
use crate::estilo;
use crate::fuentes;
use crate::iconos::Icono;
use crate::laminas::Lamina;
use crate::tema::{borde, espacio, texto, Modo, Paleta, TINTA};

/// Zoom mínimo, máximo y paso, en porcentaje.
pub const ZOOM_MIN: u32 = 25;
pub const ZOOM_MAX: u32 = 400;
pub const ZOOM_PASO: u32 = 25;

/// Velo del visor: casi opaco, igual en ambos temas.
const VELO: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };

/// Estado del visor abierto.
#[derive(Debug, Clone, Copy)]
pub struct EstadoVisor {
    pub indice: usize,
    /// Porcentaje de zoom; 100 es la imagen ajustada a la ventana.
    pub zoom: u32,
}

impl EstadoVisor {
    pub fn nuevo(indice: usize, _ahora: Instant) -> Self {
        Self { indice, zoom: 100 }
    }

    pub fn acercar(&mut self) {
        self.zoom = (self.zoom + ZOOM_PASO).min(ZOOM_MAX);
    }

    pub fn alejar(&mut self) {
        self.zoom = self.zoom.saturating_sub(ZOOM_PASO).max(ZOOM_MIN);
    }

    /// Cambia de imagen y vuelve al ajuste de ventana.
    pub fn ir_a(&mut self, indice: usize) {
        self.indice = indice;
        self.zoom = 100;
    }

}

/// Datos de la imagen que se muestra.
pub struct ItemVisor {
    pub titulo: String,
    pub detalle: String,
    pub lamina: Lamina,
    /// Marco 16:9 para capturas; `None` conserva la proporción del grabado.
    pub aspecto: Option<f32>,
}

/// Acciones de la barra superior; las que faltan no se dibujan.
pub struct MensajesVisor<M> {
    pub cerrar: M,
    pub anterior: M,
    pub siguiente: M,
    pub acercar: M,
    pub alejar: M,
    pub copiar: Option<M>,
    pub carpeta: Option<M>,
    pub eliminar: Option<M>,
}

/// Visor completo, para apilar encima de la ventana.
pub fn visor<'a, M: Clone + 'a>(
    modo: Modo,
    estado: &EstadoVisor,
    _ahora: Instant,
    _reducir: bool,
    item: &ItemVisor,
    total: usize,
    m: MensajesVisor<M>,
) -> Element<'a, M> {
    // Sobre el velo oscuro todo va en tinta clara, también en el tema Piedra.
    let p = TINTA;
    let mut acciones = row![].spacing(espacio::S1).align_y(Alignment::Center);
    if let Some(copiar) = m.copiar {
        acciones = acciones.push(boton_estilo(Some(Icono::Copiar), "Copiar", TamBoton::Sm, estilo::boton_fantasma(p), Some(copiar)));
    }
    if let Some(carpeta) = m.carpeta {
        acciones = acciones.push(boton_estilo(Some(Icono::Carpeta), "Abrir carpeta", TamBoton::Sm, estilo::boton_fantasma(p), Some(carpeta)));
    }
    if let Some(eliminar) = m.eliminar {
        acciones = acciones.push(boton_estilo(Some(Icono::Papelera), "Eliminar", TamBoton::Sm, fantasma_error(p), Some(eliminar)));
    }
    acciones = acciones.push(boton_icono(p, Icono::Cerrar, "Cerrar", m.cerrar.clone()));

    let barra = container(
        row![
            text(item.titulo.clone())
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text),
            text(item.detalle.clone())
                .size(texto::BODY_SM.0)
                .color(p.text_muted),
            Space::with_width(Length::Fill),
            acciones,
        ]
        .spacing(espacio::S4)
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([espacio::S3, espacio::S4]))
    .width(Length::Fill);
    let regla_inferior = container(Space::new(Length::Fill, borde::FINO)).style(estilo::bloque(p.border));

    let imagen = ImagenVisor {
        aspecto: item.aspecto,
        handle: item.lamina.imagen(modo).clone(),
        zoom: estado.zoom as f32 / 100.0,
        acercar: m.acercar.clone(),
        alejar: m.alejar.clone(),
    };
    let medio = row![
        container(boton_icono(p, Icono::ChevronIzquierda, "Anterior", m.anterior)).center_x(56),
        Element::new(imagen),
        container(boton_icono(p, Icono::ChevronDerecha, "Siguiente", m.siguiente)).center_x(56),
    ]
    .align_y(Alignment::Center)
    .height(Length::Fill);

    let pie = container(
        row![
            boton_icono(p, Icono::ZoomMenos, "Alejar", m.alejar),
            text(format!("{} % · {} de {}", estado.zoom, estado.indice + 1, total))
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text),
            boton_icono(p, Icono::ZoomMas, "Acercar", m.acercar),
        ]
        .spacing(espacio::S4)
        .align_y(Alignment::Center),
    )
    .center_x(Length::Fill)
    .padding(espacio::S3);

    let cuerpo = column![
        Space::with_height(espacio::S6),
        barra,
        regla_inferior,
        container(medio).padding(Padding::from([espacio::S4, 0.0])).height(Length::Fill),
        pie,
    ];
    opaque(container(cuerpo)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::from([0.0, espacio::S4]))
        .style(estilo::bloque(VELO)))
}

/// Botón fantasma de la acción destructiva: texto cinabrio.
fn fantasma_error(p: Paleta) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_, estado| button::Style {
        background: matches!(estado, button::Status::Hovered | button::Status::Pressed)
            .then(|| Background::Color(p.surface_hover)),
        text_color: p.error,
        border: Border { width: borde::MEDIO, color: Color::TRANSPARENT, radius: 0.0.into() },
        ..Default::default()
    }
}

// ------------------------------------------------------------- imagen

/// Grabado ajustado sin recorte o captura dentro de un marco 16:9.
struct ImagenVisor<M> {
    aspecto: Option<f32>,
    handle: Handle,
    zoom: f32,
    acercar: M,
    alejar: M,
}

#[derive(Default)]
struct EstadoImagen {
    mover: Vector,
    arrastre: Option<Point>,
    imagen: u64,
}

impl<M> ImagenVisor<M> {
    fn marco(&self, caja: Rectangle) -> Rectangle {
        let Some(aspecto) = self.aspecto else { return caja };
        let w = caja.width.min(caja.height * aspecto);
        let h = w / aspecto;
        Rectangle { x: caja.center_x() - w / 2.0, y: caja.center_y() - h / 2.0, width: w, height: h }
    }

    /// Rectángulo donde se dibuja la imagen dentro de `caja`, ya con el
    /// desplazamiento limitado a que la imagen no se salga del todo.
    fn rectangulo(&self, tam: Size<u32>, caja: Rectangle, mover: Vector) -> Rectangle {
        let (iw, ih) = (tam.width as f32, tam.height as f32);
        let ajuste = if self.aspecto.is_some() {
            (caja.width / iw).max(caja.height / ih)
        } else { (caja.width / iw).min(caja.height / ih) };
        let (w, h) = (iw * ajuste * self.zoom, ih * ajuste * self.zoom);
        let tope_x = ((w - caja.width) / 2.0).max(0.0);
        let tope_y = ((h - caja.height) / 2.0).max(0.0);
        Rectangle {
            x: caja.center_x() - w / 2.0 + mover.x.clamp(-tope_x, tope_x),
            y: caja.center_y() - h / 2.0 + mover.y.clamp(-tope_y, tope_y),
            width: w,
            height: h,
        }
    }
}

impl<M: Clone> Widget<M, iced::Theme, iced::Renderer> for ImagenVisor<M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<EstadoImagen>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(EstadoImagen::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(&self, _: &mut Tree, _: &iced::Renderer, limites: &layout::Limits) -> Node {
        layout::atomic(limites, Length::Fill, Length::Fill)
    }

    fn on_event(
        &mut self,
        arbol: &mut Tree,
        evento: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        _: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        _: &Rectangle,
    ) -> event::Status {
        let estado = arbol.state.downcast_mut::<EstadoImagen>();
        if estado.imagen != id_de(&self.handle) {
            *estado = EstadoImagen { imagen: id_de(&self.handle), ..Default::default() };
        }
        let caja = self.marco(layout.bounds());
        let Event::Mouse(raton) = evento else {
            return event::Status::Ignored;
        };
        match raton {
            mouse::Event::WheelScrolled { delta } if cursor.is_over(caja) => {
                let y = match delta {
                    mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => y,
                };
                if y > 0.0 {
                    shell.publish(self.acercar.clone());
                } else if y < 0.0 {
                    shell.publish(self.alejar.clone());
                }
                event::Status::Captured
            }
            mouse::Event::ButtonPressed(mouse::Button::Left) if cursor.is_over(caja) => {
                estado.arrastre = cursor.position();
                event::Status::Captured
            }
            mouse::Event::CursorMoved { position } => {
                if let Some(previo) = estado.arrastre {
                    // Se limita al soltar y al dibujar; aquí solo se acumula.
                    estado.mover = estado.mover + (position - previo);
                    estado.arrastre = Some(position);
                    shell.request_redraw(window::RedrawRequest::NextFrame);
                    return event::Status::Captured;
                }
                event::Status::Ignored
            }
            mouse::Event::ButtonReleased(mouse::Button::Left) if estado.arrastre.is_some() => {
                estado.arrastre = None;
                // Deja guardado el desplazamiento ya limitado.
                let tam = renderer.measure_image(&self.handle);
                let r = self.rectangulo(tam, caja, estado.mover);
                estado.mover = Vector::new(
                    r.center_x() - caja.center_x(),
                    r.center_y() - caja.center_y(),
                );
                event::Status::Captured
            }
            _ => event::Status::Ignored,
        }
    }

    fn mouse_interaction(
        &self,
        arbol: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Rectangle,
        _: &iced::Renderer,
    ) -> mouse::Interaction {
        let estado = arbol.state.downcast_ref::<EstadoImagen>();
        if estado.arrastre.is_some() {
            mouse::Interaction::Grabbing
        } else if self.zoom > 1.0 && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::None
        }
    }

    fn draw(
        &self,
        arbol: &Tree,
        renderer: &mut iced::Renderer,
        _: &iced::Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
        visible: &Rectangle,
    ) {
        let caja = self.marco(layout.bounds());
        let Some(recorte) = caja.intersection(visible) else {
            return;
        };
        let tam = renderer.measure_image(&self.handle);
        if tam.width == 0 || tam.height == 0 {
            return;
        }
        let estado = arbol.state.downcast_ref::<EstadoImagen>();
        // Una imagen distinta empieza centrada.
        let mover = if estado.imagen == id_de(&self.handle) { estado.mover } else { Vector::ZERO };
        let destino = self.rectangulo(tam, caja, mover);
        renderer.with_layer(recorte, |renderer| {
            renderer.draw_image(
                Image {
                    handle: self.handle.clone(),
                    filter_method: iced::advanced::image::FilterMethod::Linear,
                    rotation: iced::Radians(0.0),
                    opacity: 1.0,
                    snap: true,
                },
                destino,
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn las_capturas_usan_marco_16_9_y_los_grabados_su_proporcion() {
        let mut imagen = ImagenVisor {
            aspecto: Some(16.0 / 9.0),
            handle: Handle::from_rgba(1, 1, vec![0; 4]),
            zoom: 1.0,
            acercar: (),
            alejar: (),
        };
        let caja = Rectangle::new(Point::new(12.0, 90.0), Size::new(1200.0, 650.0));
        let marco = imagen.marco(caja);
        assert!((marco.width / marco.height - 16.0 / 9.0).abs() < 0.001);
        assert_eq!(marco.center(), caja.center());
        let dibujo = imagen.rectangulo(Size::new(640, 480), marco, Vector::ZERO);
        assert!(dibujo.width >= marco.width && dibujo.height >= marco.height);
        assert!((dibujo.width / dibujo.height - 4.0 / 3.0).abs() < 0.001);
        imagen.aspecto = None;
        let dibujo = imagen.rectangulo(Size::new(600, 1200), caja, Vector::ZERO);
        assert!((dibujo.width / dibujo.height - 0.5).abs() < 0.001);
    }

    #[test]
    fn el_zoom_avanza_de_25_en_25_con_tope() {
        let mut v = EstadoVisor::nuevo(0, Instant::now());
        v.acercar();
        assert_eq!(v.zoom, 125);
        for _ in 0..20 {
            v.acercar();
        }
        assert_eq!(v.zoom, ZOOM_MAX);
        for _ in 0..20 {
            v.alejar();
        }
        assert_eq!(v.zoom, ZOOM_MIN);
    }

    #[test]
    fn cambiar_de_imagen_vuelve_al_ajuste() {
        let mut v = EstadoVisor::nuevo(0, Instant::now());
        v.acercar();
        v.ir_a(3);
        assert_eq!((v.indice, v.zoom), (3, 100));
    }

}

/// Huella de la imagen, para detectar que cambió.
fn id_de(handle: &Handle) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    handle.id().hash(&mut h);
    h.finish()
}
