// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss
//! Menú situado a partir del layout actual del botón, también al redimensionar.
use iced::advanced::{layout, mouse, overlay, renderer, widget::Tree, Clipboard, Layout, Shell, Widget};
use iced::{Element, Event, Length, Point, Rectangle, Size, Theme, Vector};

pub fn menu<'a, M: Clone + 'a>(boton: Element<'a, M>, contenido: Element<'a, M>, abierto: bool, cerrar: M) -> Element<'a, M> {
    Element::new(Ancla { boton, contenido, abierto, cerrar })
}

struct Ancla<'a, M> { boton: Element<'a, M>, contenido: Element<'a, M>, abierto: bool, cerrar: M }
impl<M: Clone> Widget<M, Theme, iced::Renderer> for Ancla<'_, M> {
    fn size(&self) -> Size<Length> { self.boton.as_widget().size() }
    fn children(&self) -> Vec<Tree> { vec![Tree::new(&self.boton), Tree::new(&self.contenido)] }
    fn diff(&self, tree: &mut Tree) { tree.children[0].diff(&self.boton); tree.children[1].diff(&self.contenido); }
    fn layout(&self, tree: &mut Tree, r: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        self.boton.as_widget().layout(&mut tree.children[0], r, limits)
    }
    fn draw(&self, tree: &Tree, r: &mut iced::Renderer, theme: &Theme, style: &renderer::Style, l: Layout<'_>, c: mouse::Cursor, v: &Rectangle) {
        self.boton.as_widget().draw(&tree.children[0], r, theme, style, l, c, v);
    }
    fn operate(&self, tree: &mut Tree, l: Layout<'_>, r: &iced::Renderer, op: &mut dyn iced::advanced::widget::Operation) {
        self.boton.as_widget().operate(&mut tree.children[0], l, r, op);
    }
    fn on_event(&mut self, tree: &mut Tree, e: Event, l: Layout<'_>, c: mouse::Cursor, r: &iced::Renderer, cb: &mut dyn Clipboard, s: &mut Shell<'_, M>, v: &Rectangle) -> iced::event::Status {
        self.boton.as_widget_mut().on_event(&mut tree.children[0], e, l, c, r, cb, s, v)
    }
    fn mouse_interaction(&self, tree: &Tree, l: Layout<'_>, c: mouse::Cursor, v: &Rectangle, r: &iced::Renderer) -> mouse::Interaction {
        self.boton.as_widget().mouse_interaction(&tree.children[0], l, c, v, r)
    }
    fn overlay<'b>(&'b mut self, tree: &'b mut Tree, l: Layout<'_>, _: &iced::Renderer, translation: Vector) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.abierto.then(|| overlay::Element::new(Box::new(Menu {
            contenido: &mut self.contenido, tree: &mut tree.children[1],
            ancla: Point::new(l.bounds().x, l.bounds().y + l.bounds().height) + translation,
            cerrar: self.cerrar.clone(),
        })))
    }
}
fn posicion(ancla: Point, menu: Size, ventana: Size) -> Point {
    Point::new(ancla.x.clamp(8., (ventana.width - menu.width - 8.).max(8.)),
        if ancla.y + menu.height + 8. <= ventana.height { ancla.y + 4. } else { (ancla.y - menu.height - 32.).max(8.) })
}
struct Menu<'a, 'b, M> { contenido: &'a mut Element<'b, M>, tree: &'a mut Tree, ancla: Point, cerrar: M }
impl<M: Clone> overlay::Overlay<M, Theme, iced::Renderer> for Menu<'_, '_, M> {
    fn layout(&mut self, r: &iced::Renderer, bounds: Size) -> layout::Node {
        let mut node = self.contenido.as_widget().layout(self.tree, r, &layout::Limits::new(Size::ZERO, bounds));
        node.move_to_mut(posicion(self.ancla, node.size(), bounds)); node
    }
    fn draw(&self, r: &mut iced::Renderer, t: &Theme, s: &renderer::Style, l: Layout<'_>, c: mouse::Cursor) {
        self.contenido.as_widget().draw(self.tree, r, t, s, l, c, &l.bounds());
    }
    fn on_event(&mut self, e: Event, l: Layout<'_>, c: mouse::Cursor, r: &iced::Renderer, cb: &mut dyn Clipboard, s: &mut Shell<'_, M>) -> iced::event::Status {
        if matches!(e, Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))) && !c.is_over(l.bounds()) {
            s.publish(self.cerrar.clone()); return iced::event::Status::Captured;
        }
        self.contenido.as_widget_mut().on_event(self.tree, e, l, c, r, cb, s, &l.bounds())
    }
    fn mouse_interaction(&self, l: Layout<'_>, c: mouse::Cursor, v: &Rectangle, r: &iced::Renderer) -> mouse::Interaction {
        self.contenido.as_widget().mouse_interaction(self.tree, l, c, v, r)
    }
    fn operate(&mut self, l: Layout<'_>, r: &iced::Renderer, op: &mut dyn iced::advanced::widget::Operation) {
        self.contenido.as_widget().operate(self.tree, l, r, op);
    }
    fn is_over(&self, _: Layout<'_>, _: &iced::Renderer, _: Point) -> bool { true }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sigue_el_boton_al_redimensionar_y_respeta_los_bordes() {
        let menu = Size::new(260., 84.);
        assert_eq!(posicion(Point::new(740., 450.), menu, Size::new(1280., 800.)).x, 740.);
        assert_eq!(posicion(Point::new(1000., 434.), menu, Size::new(1920., 1080.)).x, 1000.);
        let p = posicion(Point::new(1850., 1040.), menu, Size::new(1920., 1080.));
        assert!(p.x + menu.width <= 1912. && p.y + menu.height <= 1072.);
    }
}
