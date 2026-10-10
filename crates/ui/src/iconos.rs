// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Iconos Pixelarticons (MIT). Se dibujan en una grilla de 24 con píxeles de
//! 2 unidades, así que solo se usan en múltiplos de 12 px.

use iced::widget::svg::{self, Handle, Svg};
use iced::advanced::layout::{Limits, Node};
use iced::advanced::svg::Renderer as _;
use iced::advanced::widget::Tree;
use iced::advanced::{renderer, Layout, Widget};
use iced::{mouse, Color, Element, Length, Radians, Size, Theme};

macro_rules! iconos {
    ($($variante:ident => $archivo:literal),* $(,)?) => {
        /// Iconos disponibles.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Icono { $($variante),* }

        impl Icono {
            pub(crate) const fn bytes(self) -> &'static [u8] {
                match self {
                    $(Icono::$variante => include_bytes!(concat!("../assets/iconos/", $archivo, ".svg"))),*
                }
            }
        }
    };
}

iconos! {
    Instancias => "blocks",
    Servidores => "server",
    Descargas => "download",
    Consola => "terminal",
    Mods => "tool-case",
    Capturas => "camera",
    Ajustes => "sliders",
    Jugar => "play",
    Buscar => "search",
    Mas => "plus",
    Menos => "minus",
    Cerrar => "close",
    Flecha => "chevron-up",
    ChevronAbajo => "chevron-down",
    ChevronDerecha => "chevron-right",
    ChevronIzquierda => "chevron-left",
    ZoomMas => "zoom-in",
    ZoomMenos => "zoom-out",
    Carpeta => "folder",
    Editar => "pencil",
    Subir => "upload",
    Acciones => "more-vertical",
    Actualizar => "reload",
    Aviso => "warning-diamond",
    Check => "check",
    Detener => "stop",
    Espera => "hourglass",
    Arrastrar => "drag-and-drop",
    Alerta => "square-alert",
    Papelera => "trash",
    Copiar => "copy",
    Externo => "external-link",
    Pausa => "pause",
    Candado => "lock",
    Info => "circle-info",
    Pincel => "brush",
    Deshacer => "undo",
    Rehacer => "redo",
    Entrar => "login",
    Salir => "logout",
    Refrescar => "refresh",
    Usuario => "user",
    UsuarioMas => "user-plus",
}

/// Tamaños permitidos, múltiplos del píxel de arte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tam {
    /// 12 px.
    Sm,
    /// 24 px.
    Base,
    /// 36 px.
    Md,
    /// 48 px.
    Lg,
}

impl Tam {
    pub const fn px(self) -> f32 {
        match self {
            Tam::Sm => 12.0,
            Tam::Base => 24.0,
            Tam::Md => 36.0,
            Tam::Lg => 48.0,
        }
    }
}

/// Icono de una sola tinta.
pub fn icono<'a>(icono: Icono, tam: Tam, color: Color) -> Svg<'a, Theme> {
    svg(Handle::from_memory(icono.bytes()))
        .width(Length::Fixed(tam.px()))
        .height(Length::Fixed(tam.px()))
        .style(move |_: &Theme, _| svg::Style { color: Some(color) })
}

fn svg<'a>(handle: Handle) -> Svg<'a, Theme> {
    svg::Svg::new(handle)
}

/// Icono que toma la tinta del texto de su contenedor: dentro de un botón
/// cambia con el hover, el presionado y el deshabilitado igual que su rótulo.
pub fn icono_texto<'a, M: 'a>(glifo: Icono, tam: Tam) -> Element<'a, M> {
    Element::new(IconoTexto {
        handle: Handle::from_memory(glifo.bytes()),
        px: tam.px(),
    })
}

struct IconoTexto {
    handle: Handle,
    px: f32,
}

impl<M> Widget<M, Theme, iced::Renderer> for IconoTexto {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.px), Length::Fixed(self.px))
    }

    fn layout(&self, _arbol: &mut Tree, _renderer: &iced::Renderer, _limites: &Limits) -> Node {
        Node::new(Size::new(self.px, self.px))
    }

    fn draw(
        &self,
        _arbol: &Tree,
        renderer: &mut iced::Renderer,
        _tema: &Theme,
        estilo: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _visible: &iced::Rectangle,
    ) {
        renderer.draw_svg(
            iced::advanced::svg::Svg {
                handle: self.handle.clone(),
                color: Some(estilo.text_color),
                rotation: Radians(0.0),
                opacity: 1.0,
            },
            layout.bounds(),
        );
    }
}
