// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Consola del juego, fiel a `Consola` del sistema de diseño.
//! La búsqueda resalta coincidencias y conserva el contexto del registro.
//! La lista reutiliza párrafos y dibuja únicamente las filas visibles.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ops::Range;

use iced::advanced::layout::{Layout, Limits, Node};
use iced::advanced::text::{self as texto_avanzado, Paragraph as _, Renderer as _};
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{mouse, renderer, Renderer as _, Widget};
use iced::widget::{button, column, container, row, scrollable, text, text_input, tooltip, Space};
use iced::{
    Alignment, Element, Font, Length, Padding, Pixels, Point, Rectangle, Size, Theme, Vector,
};

use crate::componentes::{boton_compacto, interruptor, Variante};
use crate::iconos::{self, Icono, Tam};
use crate::tema::Paleta;
use crate::{estilo, fuentes};

/// Identificador compartido con las operaciones de autoscroll de la aplicación.
pub const LISTA_CONSOLA: &str = "icaro-consola-viva";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nivel {
    Info,
    Warn,
    Error,
}

impl Nivel {
    const fn palabra(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
    const fn indice(self) -> usize {
        match self {
            Self::Info => 0,
            Self::Warn => 1,
            Self::Error => 2,
        }
    }
}

/// Línea del registro, ya decodificada por el proceso de Minecraft.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Linea {
    pub hora: String,
    pub nivel: Nivel,
    pub mensaje: String,
}

/// Posición real del visor. La aplicación pausa el autoscroll al alejarse del final.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DesplazamientoConsola {
    pub offset: f32,
    pub altura: f32,
    pub contenido: f32,
    pub esta_al_final: bool,
}

impl Default for DesplazamientoConsola {
    fn default() -> Self {
        Self {
            offset: 0.0,
            altura: 0.0,
            contenido: 0.0,
            esta_al_final: true,
        }
    }
}

impl From<scrollable::Viewport> for DesplazamientoConsola {
    fn from(v: scrollable::Viewport) -> Self {
        let offset = v.absolute_offset().y;
        let altura = v.bounds().height;
        let contenido = v.content_bounds().height;
        Self {
            offset,
            altura,
            contenido,
            esta_al_final: esta_al_final(offset, altura, contenido),
        }
    }
}

pub struct VistaConsola<'a> {
    pub nombre: &'a str,
    pub niveles: [bool; 3],
    pub busqueda: &'a str,
    pub autoscroll: bool,
    pub desplazamiento: DesplazamientoConsola,
}

impl Default for VistaConsola<'_> {
    fn default() -> Self {
        Self {
            nombre: "Registro",
            niveles: [true; 3],
            busqueda: "",
            autoscroll: true,
            desplazamiento: DesplazamientoConsola::default(),
        }
    }
}

pub struct MensajesConsolaViva<M> {
    pub nivel: fn(usize) -> M,
    pub buscar: fn(String) -> M,
    pub autoscroll: fn(bool) -> M,
    pub desplazado: fn(DesplazamientoConsola) -> M,
    pub ir_al_final: M,
    pub copiar: M,
    pub limpiar: M,
}

/// API anterior, conservada para las escenas de componentes y demostración.
pub struct MensajesConsola<M> {
    pub filtro: fn(usize) -> M,
    pub buscar: fn(String) -> M,
    pub copiar: M,
    pub limpiar: M,
}

pub fn consola<'a, M: Clone + 'a>(
    p: Paleta,
    lineas: &[Linea],
    filtro: usize,
    busqueda: &str,
    m: MensajesConsola<M>,
) -> Element<'a, M> {
    let niveles = std::array::from_fn(|i| filtro == 0 || filtro == i + 1);
    let barra = barra_consola(
        p,
        "Registro",
        lineas,
        niveles,
        busqueda,
        true,
        |i| (m.filtro)(if filtro == i + 1 { 0 } else { i + 1 }),
        m.buscar,
        None,
        m.copiar,
        m.limpiar,
    );
    panel(p, barra, lista(p, lineas, niveles, busqueda), None)
}

/// Consola en vivo: barra interior, filtros independientes, búsqueda invertida,
/// interruptor de autoscroll y acciones por iconos, dentro del marco de 2 px.
pub fn consola_viva<'a, M: Clone + 'a>(
    p: Paleta,
    lineas: &[Linea],
    vista: VistaConsola<'_>,
    m: MensajesConsolaViva<M>,
) -> Element<'a, M> {
    let barra = barra_consola(
        p,
        vista.nombre,
        lineas,
        vista.niveles,
        vista.busqueda,
        vista.autoscroll,
        m.nivel,
        m.buscar,
        Some((m.autoscroll)(!vista.autoscroll)),
        m.copiar,
        m.limpiar,
    );
    let filas = lista(p, lineas, vista.niveles, vista.busqueda)
        .on_scroll(move |v| (m.desplazado)(v.into()));
    let final_registro =
        (!vista.autoscroll && !vista.desplazamiento.esta_al_final).then_some(m.ir_al_final);
    panel(p, barra, filas, final_registro)
}

fn barra_consola<'a, M: Clone + 'a>(
    p: Paleta,
    nombre: &str,
    lineas: &[Linea],
    niveles: [bool; 3],
    busqueda: &str,
    autoscroll: bool,
    nivel: impl Fn(usize) -> M,
    buscar: fn(String) -> M,
    cambiar_autoscroll: Option<M>,
    copiar: M,
    limpiar: M,
) -> Element<'a, M> {
    let cantidades = conteos(lineas);
    let mut chips = row![].spacing(8);
    for (i, nombre) in ["INFO", "WARN", "ERROR"].iter().enumerate() {
        let mut contenido = row![text(*nombre).font(fuentes::ETIQUETA).size(12)]
            .spacing(6)
            .align_y(Alignment::Center);
        if cantidades[i] > 0 {
            contenido =
                contenido.push(text(cantidades[i].to_string()).font(fuentes::MONO).size(11));
        }
        chips = chips.push(
            button(container(contenido).center_y(Length::Fill))
                .height(28)
                .padding(Padding::from([0., 12.]))
                .on_press(nivel(i))
                .style(estilo::chip(p, niveles[i])),
        );
    }
    let entrada = text_input("Buscar en el registro", busqueda)
        .on_input(buscar)
        .font(fuentes::CUERPO)
        .size(12)
        .padding(Padding::from([2., 0.]))
        .style(move |tema, estado| {
            let mut s = estilo::campo(p)(tema, estado);
            s.border.width = 0.;
            s
        });
    let campo = container(
        row![
            iconos::icono(Icono::Buscar, Tam::Base, p.text_muted),
            entrada
        ]
        .spacing(4)
        .align_y(Alignment::Center),
    )
    .width(200)
    .height(28)
    .padding(Padding::from([0., 8.]))
    .style(move |_| container::Style {
        background: Some(p.surface_sunken.into()),
        border: iced::Border {
            width: 2.,
            color: p.border_strong,
            ..Default::default()
        },
        ..Default::default()
    });
    let interruptor: Element<'a, M> = if let Some(msg) = cambiar_autoscroll {
        interruptor(p, autoscroll, msg)
    } else {
        let perilla = container(Space::new(12, 12)).style(estilo::perilla(p.on_accent));
        container(row![Space::with_width(Length::Fill), perilla].align_y(Alignment::Center))
            .width(44)
            .height(24)
            .padding(Padding::from([0., 6.]))
            .center_y(24)
            .style(estilo::pista(p, true, false))
            .into()
    };
    container(
        row![
            text(nombre.to_owned())
                .font(fuentes::CUERPO_NEGRITA)
                .size(16)
                .wrapping(iced::widget::text::Wrapping::None),
            chips,
            Space::with_width(Length::Fill),
            campo,
            row![
                interruptor,
                text("Autoscroll").font(fuentes::CUERPO).size(12)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            accion_icono(p, Icono::Copiar, "Copiar registro", copiar),
            accion_icono(p, Icono::Papelera, "Limpiar", limpiar),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding(Padding {
        top: 8.,
        right: 8.,
        bottom: 8.,
        left: 16.,
    })
    .style(estilo::bloque(p.surface))
    .into()
}

fn accion_icono<'a, M: Clone + 'a>(
    p: Paleta,
    icono: Icono,
    etiqueta: &'static str,
    msg: M,
) -> Element<'a, M> {
    tooltip(
        button(iconos::icono_texto(icono, Tam::Base))
            .width(28)
            .height(28)
            .padding(2)
            .on_press(msg)
            .style(estilo::boton_fantasma(p)),
        text(etiqueta).font(fuentes::CUERPO).size(12),
        tooltip::Position::Bottom,
    )
    .style(estilo::tarjeta(p, false))
    .into()
}

fn panel<'a, M: Clone + 'a>(
    p: Paleta,
    barra: Element<'a, M>,
    filas: scrollable::Scrollable<'a, M>,
    ir_al_final: Option<M>,
) -> Element<'a, M> {
    let cuerpo: Element<'a, M> = filas.into();
    let aviso: Element<'a, M> = if let Some(msg) = ir_al_final {
        container(boton_compacto(
            p,
            "Ir al final",
            Variante::Primario,
            Some(msg),
        ))
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Right)
        .align_y(iced::alignment::Vertical::Bottom)
        .into()
    } else {
        Space::new(Length::Fill, Length::Fill).into()
    };
    // La envoltura estable conserva el scroll al aparecer o desaparecer Ir al final.
    let cuerpo = iced::widget::stack![cuerpo, aviso];
    let borde = container(Space::with_height(1))
        .width(Length::Fill)
        .style(estilo::bloque(p.border));
    container(column![barra, borde, cuerpo].height(Length::Fill))
        .padding(2)
        .height(Length::Fill)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(p.surface_sunken.into()),
            border: iced::Border {
                width: 2.,
                color: p.text,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn lista<'a, M: 'a>(
    p: Paleta,
    lineas: &[Linea],
    niveles: [bool; 3],
    busqueda: &str,
) -> scrollable::Scrollable<'a, M> {
    scrollable(Element::new(Filas {
        p,
        lineas: lineas
            .iter()
            .filter(|l| niveles[l.nivel.indice()])
            .cloned()
            .collect(),
        busqueda: busqueda.trim().to_owned(),
    }))
    .id(scrollable::Id::new(LISTA_CONSOLA))
    .height(Length::Fill)
    .direction(scrollable::Direction::Vertical(
        scrollable::Scrollbar::new().width(0).scroller_width(0),
    ))
}

fn conteos(lineas: &[Linea]) -> [usize; 3] {
    let mut conteos = [0; 3];
    for l in lineas {
        conteos[l.nivel.indice()] += 1;
    }
    conteos
}

fn esta_al_final(offset: f32, altura: f32, contenido: f32) -> bool {
    offset + altura + 2. >= contenido
}

type Parrafo = <iced::Renderer as texto_avanzado::Renderer>::Paragraph;
type Plano = texto_avanzado::paragraph::Plain<Parrafo>;

#[derive(Default)]
struct EstadoFilas {
    parrafos: HashMap<u64, FilaMedida>,
    orden: Vec<u64>,
    posiciones: Vec<f32>,
    // Los resaltados se crean únicamente al dibujar una fila visible.
    resaltados: RefCell<HashMap<(u64, usize), Resaltado>>,
    ultima_busqueda: String,
    ancho: f32,
    modo: Option<crate::tema::Modo>,
}

struct FilaMedida {
    linea: Linea,
    hora: Plano,
    nivel: Plano,
    mensaje: Plano,
    altura: f32,
}
struct Resaltado {
    parrafo: Parrafo,
    tramos: Vec<usize>,
}
struct Filas {
    p: Paleta,
    lineas: Vec<Linea>,
    busqueda: String,
}

impl<M> Widget<M, Theme, iced::Renderer> for Filas {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<EstadoFilas>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(EstadoFilas::default())
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(&self, tree: &mut Tree, _: &iced::Renderer, limites: &Limits) -> Node {
        let ancho = limites.max().width;
        let ancho_mensaje = (ancho - 188.).max(1.);
        let estado = tree.state.downcast_mut::<EstadoFilas>();
        if estado.ultima_busqueda != self.busqueda
            || estado.ancho != ancho
            || estado.modo != Some(self.p.modo)
        {
            estado.resaltados.get_mut().clear();
            estado.ultima_busqueda.clone_from(&self.busqueda);
            estado.ancho = ancho;
            estado.modo = Some(self.p.modo);
        }
        estado.orden.clear();
        estado.posiciones.clear();
        estado.posiciones.push(8.);
        let mut vigentes = HashSet::with_capacity(self.lineas.len());
        let mut altura = 8.;
        for linea in &self.lineas {
            let clave = clave_linea(linea);
            vigentes.insert(clave);
            let fila = estado.parrafos.entry(clave).or_insert_with(|| FilaMedida {
                linea: linea.clone(),
                hora: Plano::default(),
                nivel: Plano::default(),
                mensaje: Plano::default(),
                altura: 20.,
            });
            if fila.linea != *linea {
                fila.linea.clone_from(linea);
            }
            fila.hora
                .update(config_texto(linea.hora.as_str(), 76., fuentes::MONO));
            fila.nivel.update(config_texto(
                linea.nivel.palabra(),
                56.,
                fuentes::MONO_MEDIO,
            ));
            fila.mensaje.update(config_texto(
                linea.mensaje.as_str(),
                ancho_mensaje,
                fuentes::MONO,
            ));
            fila.altura = fila.mensaje.min_bounds().height.max(20.);
            altura += fila.altura;
            estado.orden.push(clave);
            estado.posiciones.push(altura);
        }
        estado.parrafos.retain(|k, _| vigentes.contains(k));
        estado
            .resaltados
            .get_mut()
            .retain(|(k, _), _| vigentes.contains(k));
        Node::new(Size::new(ancho, altura + 8.))
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let Some(visible) = bounds.intersection(viewport) else {
            return;
        };
        let estado = tree.state.downcast_ref::<EstadoFilas>();
        let rango = filas_visibles(&estado.posiciones, visible.y - bounds.y, visible.height);
        let ancho_mensaje = (bounds.width - 188.).max(1.);
        renderer.with_layer(visible, |renderer| {
            for i in rango {
                let clave = estado.orden[i];
                let fila = &estado.parrafos[&clave];
                let y = bounds.y + estado.posiciones[i];
                let caja = Rectangle {
                    x: bounds.x,
                    y,
                    width: bounds.width,
                    height: fila.altura,
                };
                if fila.linea.nivel == Nivel::Error {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: caja,
                            ..Default::default()
                        },
                        self.p.error_soft,
                    );
                } else if cursor.is_over(caja) && cursor.is_over(visible) {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: caja,
                            ..Default::default()
                        },
                        self.p.surface_hover,
                    );
                }
                let tinta = match fila.linea.nivel {
                    Nivel::Info => self.p.text_muted,
                    Nivel::Warn => self.p.warning,
                    Nivel::Error => self.p.error,
                };
                let mensaje_tinta = if fila.linea.nivel == Nivel::Info {
                    self.p.text
                } else {
                    tinta
                };
                for (campo, texto, x, ancho, fuente, color) in [
                    (
                        0,
                        fila.linea.hora.as_str(),
                        16.,
                        76.,
                        fuentes::MONO,
                        self.p.text_muted,
                    ),
                    (
                        1,
                        fila.linea.nivel.palabra(),
                        104.,
                        56.,
                        fuentes::MONO_MEDIO,
                        tinta,
                    ),
                    (
                        2,
                        fila.linea.mensaje.as_str(),
                        172.,
                        ancho_mensaje,
                        fuentes::MONO,
                        mensaje_tinta,
                    ),
                ] {
                    let punto = Point::new(bounds.x + x, y);
                    let tiene_coincidencia = !self.busqueda.is_empty()
                        && !coincidencias(texto, &self.busqueda).is_empty();
                    if tiene_coincidencia {
                        let mut cache = estado.resaltados.borrow_mut();
                        let resaltado = cache.entry((clave, campo)).or_insert_with(|| {
                            crear_resaltado(texto, &self.busqueda, ancho, fuente, self.p)
                        });
                        for tramo in &resaltado.tramos {
                            for caja in resaltado.parrafo.span_bounds(*tramo) {
                                renderer.fill_quad(
                                    renderer::Quad {
                                        bounds: caja + Vector::new(punto.x, punto.y),
                                        ..Default::default()
                                    },
                                    self.p.text,
                                );
                            }
                        }
                        renderer.fill_paragraph(&resaltado.parrafo, punto, color, visible);
                    } else {
                        let parrafo = match campo {
                            0 => fila.hora.raw(),
                            1 => fila.nivel.raw(),
                            _ => fila.mensaje.raw(),
                        };
                        renderer.fill_paragraph(parrafo, punto, color, visible);
                    }
                }
            }
        });
    }
}

fn config_texto<T>(content: T, ancho: f32, font: Font) -> texto_avanzado::Text<T> {
    texto_avanzado::Text {
        content,
        bounds: Size::new(ancho, f32::INFINITY),
        size: Pixels(12.),
        line_height: texto_avanzado::LineHeight::Absolute(Pixels(20.)),
        font,
        horizontal_alignment: iced::alignment::Horizontal::Left,
        vertical_alignment: iced::alignment::Vertical::Top,
        shaping: texto_avanzado::Shaping::Advanced,
        wrapping: texto_avanzado::Wrapping::WordOrGlyph,
    }
}

fn clave_linea(linea: &Linea) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    linea.hash(&mut h);
    h.finish()
}

/// Dos búsquedas binarias delimitan el dibujo, respetando filas multilínea.
/// `posiciones` incluye el final de la última fila.
fn filas_visibles(posiciones: &[f32], y: f32, altura: f32) -> Range<usize> {
    let total = posiciones.len().saturating_sub(1);
    if total == 0 || altura <= 0. {
        return 0..0;
    }
    let inicio = posiciones[1..].partition_point(|fin| *fin <= y).min(total);
    let fin = posiciones[..total]
        .partition_point(|inicio| *inicio < y + altura)
        .min(total);
    inicio..fin.max(inicio)
}

/// Rangos del texto original, conservando límites UTF-8 cuando minúsculas cambia
/// la longitud de un carácter (por ejemplo İ -> i + punto).
fn coincidencias(texto: &str, consulta: &str) -> Vec<Range<usize>> {
    let consulta = consulta.trim().to_lowercase();
    if consulta.is_empty() {
        return vec![];
    }
    let mut normalizado = String::new();
    let mut mapa = Vec::new();
    for (inicio, c) in texto.char_indices() {
        let fin = inicio + c.len_utf8();
        for c in c.to_lowercase() {
            let antes = normalizado.len();
            normalizado.push(c);
            mapa.extend(std::iter::repeat_n(
                (inicio, fin),
                normalizado.len() - antes,
            ));
        }
    }
    let mut rangos: Vec<Range<usize>> = Vec::new();
    for (inicio, _) in normalizado.match_indices(&consulta) {
        let fin = inicio + consulta.len();
        let rango = mapa[inicio].0..mapa[fin - 1].1;
        if let Some(anterior) = rangos.last_mut().filter(|r| rango.start <= r.end) {
            anterior.end = anterior.end.max(rango.end);
        } else {
            rangos.push(rango);
        }
    }
    rangos
}

fn crear_resaltado(texto: &str, consulta: &str, ancho: f32, fuente: Font, p: Paleta) -> Resaltado {
    let mut spans: Vec<texto_avanzado::Span<'_, ()>> = Vec::new();
    let mut tramos = Vec::new();
    let mut anterior = 0;
    for rango in coincidencias(texto, consulta) {
        if anterior < rango.start {
            spans.push(texto_avanzado::Span::new(&texto[anterior..rango.start]));
        }
        tramos.push(spans.len());
        spans.push(texto_avanzado::Span::new(&texto[rango.clone()]).color(p.bg));
        anterior = rango.end;
    }
    if anterior < texto.len() {
        spans.push(texto_avanzado::Span::new(&texto[anterior..]));
    }
    Resaltado {
        parrafo: Parrafo::with_spans(config_texto(spans.as_slice(), ancho, fuente)),
        tramos,
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn coincidencias_conservan_contexto_y_limites_unicode() {
        let texto = "[Server thread] Guardó CHUNK y chunk de José, İcaro";
        let rangos = coincidencias(texto, "ChUnK");
        assert_eq!(
            rangos.iter().map(|r| &texto[r.clone()]).collect::<Vec<_>>(),
            ["CHUNK", "chunk"]
        );
        assert_eq!(&texto[coincidencias(texto, "josé")[0].clone()], "José");
        assert_eq!(&texto[coincidencias(texto, "i")[0].clone()], "İ");
        assert!(coincidencias(texto, "  ").is_empty());
        assert!(coincidencias(texto, "no existe").is_empty());
    }

    #[test]
    fn virtualizacion_acota_el_dibujo_y_respeta_lineas_largas() {
        let posiciones: Vec<f32> = (0..=10_000).map(|i| 8. + i as f32 * 20.).collect();
        assert_eq!(filas_visibles(&posiciones, 100_008., 260.), 5000..5013);
        assert_eq!(filas_visibles(&[8., 28., 88., 108.], 35., 20.), 1..2);
        assert_eq!(filas_visibles(&[8.], 0., 260.), 0..0);
        assert_eq!(filas_visibles(&posiciones, 300_000., 260.), 10_000..10_000);
    }

    #[test]
    fn autoscroll_distingue_subida_del_final_con_tolerancia() {
        assert!(esta_al_final(1740., 260., 2000.));
        assert!(esta_al_final(1739., 260., 2000.));
        assert!(!esta_al_final(1700., 260., 2000.));
        assert!(esta_al_final(0., 260., 100.));
    }

    #[test]
    fn conteos_no_dependen_del_filtro() {
        let lineas = [Nivel::Info, Nivel::Warn, Nivel::Error, Nivel::Warn].map(|nivel| Linea {
            hora: "14:02:31".into(),
            nivel,
            mensaje: "Minecraft".into(),
        });
        assert_eq!(conteos(&lineas), [1, 2, 1]);
    }
}
