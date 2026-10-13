// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! CambioVersionInstancia del paquete de componentes entregado por el usuario.
//! La vista sólo consume los resultados asíncronos; nunca abre archivos ni
//! consulta servicios durante el dibujo de las filas.

use super::{m, regla, Accion};
use crate::{App, Mensaje};
use icaro_core::versiones_instancia::{baja, Compatibilidad};
use icaro_ui::{
    componentes::{self, Variante},
    fuentes,
    iconos::{icono, Icono, Tam},
    tema::{texto, Paleta},
};
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding};

const ALTO_FILA: f32 = 64.;
const FILAS_RENDERIZADAS: usize = 14;

fn mono<'a>(valor: impl Into<String>, tam: f32, color: Color) -> iced::widget::Text<'a> {
    text(valor.into())
        .font(fuentes::MONO)
        .size(tam)
        .color(color)
}

fn porcentaje(datos: &Compatibilidad) -> usize {
    if datos.total == 0 {
        100
    } else {
        datos.compatibles.saturating_mul(100) / datos.total
    }
}

fn compat_color(p: Paleta, datos: &Compatibilidad) -> Color {
    match porcentaje(datos) {
        95.. => p.success,
        75.. => p.warning,
        _ => p.error,
    }
}

fn recomendada<'a>(app: &'a App, actual: &str) -> Option<&'a str> {
    let c = app.servicios.instancias.cambio.as_ref()?;
    app.servicios
        .instancias
        .versiones
        .iter()
        .filter(|v| v.kind == "release" && v.id != actual)
        .find(|v| {
            c.compatibilidades
                .get(&v.id)
                .is_some_and(|d| porcentaje(d) >= 95)
        })
        .map(|v| v.id.as_str())
}

/// Diez bloques de diez píxeles de alto, con los tonos exactos de ic-cv-compat.
fn bloques<'a>(p: Paleta, datos: &Compatibilidad) -> Element<'a, Mensaje> {
    let llenos = (porcentaje(datos).min(100) + 5) / 10;
    let tono = compat_color(p, datos);
    let mut bloques = row![].spacing(2);
    for i in 0..10 {
        bloques = bloques.push(container(Space::new(Length::Fill, 10)).style(move |_| {
            container::Style {
                background: Some(Background::Color(if i < llenos {
                    tono
                } else {
                    p.surface_sunken
                })),
                border: Border {
                    color: p.border,
                    width: if i < llenos { 0. } else { 1. },
                    radius: 0.into(),
                },
                ..Default::default()
            }
        }));
    }
    bloques.into()
}

struct Esqueleto {
    p: Paleta,
    fase: f32,
}
impl iced::widget::canvas::Program<Mensaje> for Esqueleto {
    type State = ();
    fn draw(
        &self,
        _: &(),
        renderer: &iced::Renderer,
        _: &iced::Theme,
        bounds: iced::Rectangle,
        _: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        let mut frame = iced::widget::canvas::Frame::new(renderer, bounds.size());
        frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), self.p.surface_sunken);
        let mut y = 0.;
        while y < bounds.height {
            frame.fill_rectangle(
                iced::Point::new(0., y),
                iced::Size::new(bounds.width, 2.),
                self.p.border,
            );
            y += 4.;
        }
        let x = self.fase * (bounds.width + 24.) - 24.;
        let inicio = x.max(0.);
        let ancho = ((x + 24.).min(bounds.width) - inicio).max(0.);
        frame.fill_rectangle(
            iced::Point::new(inicio, 0.),
            iced::Size::new(ancho, bounds.height),
            Color {
                a: 0.18,
                ..self.p.text
            },
        );
        vec![frame.into_geometry()]
    }
}

fn esqueleto<'a>(app: &App) -> Element<'a, Mensaje> {
    let fase = if app.reducir {
        0.5
    } else {
        (app.ahora
            .saturating_duration_since(app.codigo_inicio)
            .as_secs_f32()
            % 1.4)
            / 1.4
    };
    iced::widget::canvas(Esqueleto {
        p: app.paleta(),
        fase,
    })
    .width(Length::Fill)
    .height(10)
    .into()
}

fn fila_version<'a>(
    app: &'a App,
    indice: usize,
    version: String,
    fecha: String,
    actual: &str,
    recomendada: Option<&str>,
) -> Element<'a, Mensaje> {
    let p = app.paleta();
    let c = app.servicios.instancias.cambio.as_ref().unwrap();
    let es_actual = version == actual;
    let selected = version == c.destino && !es_actual;
    let color = if es_actual { p.text_muted } else { p.text };
    let badges: Element<'a, Mensaje> = if es_actual {
        componentes::insignia_contorno(p, "Actual")
    } else if recomendada == Some(version.as_str()) {
        componentes::insignia(p, componentes::Estado::Exito, "Recomendada")
    } else {
        Space::new(0, 0).into()
    };
    let compat: Element<'a, Mensaje> = if let Some(datos) = c.compatibilidades.get(&version) {
        column![
            bloques(p, datos),
            mono(
                format!("{} de {} mods", datos.compatibles, datos.total),
                11.,
                p.text_muted
            )
        ]
        .spacing(6)
        .into()
    } else if c.errores_compatibilidad.contains_key(&version) {
        mono("Sin datos de compatibilidad", 10., p.warning).into()
    } else {
        column![esqueleto(app), mono("Consultando…", 11., p.text_muted)]
            .spacing(6)
            .into()
    };
    let inner = row![
        column![
            mono(version.clone(), 15., color),
            mono(fecha, 11., p.text_muted)
        ]
        .spacing(4)
        .width(96),
        container(badges).width(Length::Fill),
        container(compat).width(150)
    ]
    .spacing(16)
    .align_y(Alignment::Center);
    let contenido =
        column![
            row![
                container(Space::new(3, Length::Fill)).style(icaro_ui::estilo::bloque(
                    if selected { p.text } else { Color::TRANSPARENT }
                )),
                container(inner)
                    .padding(Padding {
                        top: 12.,
                        bottom: 11.,
                        left: 7.,
                        right: 10.
                    })
                    .width(Length::Fill)
            ],
            regla(p)
        ];
    button(contenido)
        .width(Length::Fill)
        .height(ALTO_FILA)
        .padding(0)
        .on_press_maybe((!es_actual).then(|| m(Accion::CambiarDestino(indice))))
        .style(move |_, status| button::Style {
            background: Some(Background::Color(if selected {
                p.accent_soft
            } else if matches!(status, button::Status::Hovered) {
                p.surface_hover
            } else {
                Color::TRANSPARENT
            })),
            text_color: color,
            border: Border::default(),
            ..Default::default()
        })
        .into()
}

fn lista<'a>(app: &'a App, actual: &str) -> Element<'a, Mensaje> {
    let p = app.paleta();
    let c = app.servicios.instancias.cambio.as_ref().unwrap();
    let consulta = c.busqueda.trim().to_lowercase();
    let filtradas: Vec<_> = crate::instancias_reales::versiones_cambio(app, c.tipo)
        .into_iter()
        .enumerate()
        .filter(|(_, (id, _))| consulta.is_empty() || id.to_lowercase().contains(&consulta))
        .collect();
    let recomendada = recomendada(app, actual);
    let anterior = filtradas
        .iter()
        .position(|(_, (id, _))| baja(&app.servicios.instancias.versiones, actual, id));
    let offset = app.servicios.instancias.cambio_offset;
    let first = ((offset / ALTO_FILA) as usize)
        .saturating_sub(2)
        .min(filtradas.len().saturating_sub(1));
    let end = (first + FILAS_RENDERIZADAS).min(filtradas.len());
    let tiene_separador_antes = anterior.is_some_and(|pos| pos < first);
    let mut filas = column![Space::with_height(
        first as f32 * ALTO_FILA + if tiene_separador_antes { 48. } else { 0. }
    )];
    let total = filtradas.len();
    for (pos, (indice, (id, fecha))) in filtradas
        .into_iter()
        .enumerate()
        .skip(first)
        .take(end - first)
    {
        if anterior == Some(pos) {
            filas = filas.push(
                container(componentes::etiqueta(p, "Volver a una versión anterior"))
                    .padding(Padding {
                        top: 16.,
                        bottom: 8.,
                        left: 0.,
                        right: 0.,
                    })
                    .height(48),
            );
        }
        filas = filas.push(fila_version(app, indice, id, fecha, actual, recomendada));
    }
    let separador_despues = anterior.is_some_and(|pos| pos >= end);
    filas = filas.push(Space::with_height(
        (total - end) as f32 * ALTO_FILA + if separador_despues { 48. } else { 0. },
    ));
    if total == 0 {
        filas = filas.push(
            text(if app.servicios.instancias.cargando {
                "Consultando versiones…"
            } else {
                "No hay versiones que coincidan."
            })
            .size(texto::BODY_SM.0)
            .color(p.text_muted),
        );
    }
    let lista = scrollable(filas)
        .direction(scrollable::Direction::Vertical(scrollable::Scrollbar::new().width(0).scroller_width(0).margin(0)))
        .id(scrollable::Id::new("version-cambio"))
        .on_scroll(|v| m(Accion::VersionScroll(true, v.absolute_offset().y)))
        .height(Length::Fill);
    let filtros =
        componentes::segmentado_compacto(p, &["Estables", "Snapshots", "Todas"], c.tipo, |t| {
            m(Accion::CambiarTipo(t))
        });
    let buscar =
        componentes::campo_con_icono(p, Icono::Buscar, "Buscar versión", &c.busqueda, |s| {
            m(Accion::CambiarBuscar(s))
        });
    container(
        column![
            row![filtros, container(buscar).width(180)]
                .spacing(12)
                .wrap(),
            container(column![
                container(Space::new(Length::Fill, 2)).style(icaro_ui::estilo::bloque(p.text)),
                lista
            ])
            .height(Length::Fill)
        ]
        .spacing(16),
    )
    .padding([24., 28.])
    .width(Length::FillPortion(3))
    .height(Length::Fill)
    .into()
}

fn dato<'a>(p: Paleta, label: &str, valor: String, alerta: bool) -> Element<'a, Mensaje> {
    column![
        container(
            row![
                text(label.to_owned()).size(13).color(p.text),
                Space::with_width(Length::Fill),
                mono(valor, 13., if alerta { p.warning } else { p.text })
            ]
            .spacing(12)
            .align_y(Alignment::Center)
        )
        .padding([10., 0.]),
        regla(p)
    ]
    .into()
}

fn radio<'a>(p: Paleta, label: String, checked: bool, copia: bool) -> Element<'a, Mensaje> {
    button(
        row![
            componentes::rombo(p, checked, false),
            text(label).size(13).color(p.text)
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding(0)
    .width(Length::Fill)
    .on_press(m(Accion::CambiarModo(copia)))
    .style(icaro_ui::estilo::boton_fantasma(p))
    .into()
}

fn aviso<'a>(p: Paleta, label: String, riesgo: usize) -> Element<'a, Mensaje> {
    let (color, fondo, icon) = match riesgo {
        2 => (p.error, p.error_soft, Icono::Alerta),
        1 => (p.warning, p.warning_soft, Icono::Aviso),
        _ => (p.info, p.info_soft, Icono::Info),
    };
    container(
        row![
            icono(icon, Tam::Base, color),
            text(label).size(13).color(p.text)
        ]
        .spacing(12)
        .align_y(Alignment::Start),
    )
    .padding([12., 16.])
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(fondo)),
        border: Border {
            color,
            width: 1.,
            radius: 0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn resumen<'a>(app: &'a App, record: &icaro_core::instancias::Instancia) -> Element<'a, Mensaje> {
    let p = app.paleta();
    let c = app.servicios.instancias.cambio.as_ref().unwrap();
    let datos = c.compatibilidades.get(&c.destino);
    let angosto = app.ventana.width < 1050.;
    let down = !c.destino.is_empty()
        && baja(
            &app.servicios.instancias.versiones,
            &record.minecraft,
            &c.destino,
        );
    let loader = c.loader_actual_version.as_ref().map_or_else(
        || crate::ficha::loader(&record.loader),
        |version| format!("{} {}", crate::ficha::loader(&record.loader), version),
    );
    let destinos: Vec<String> = app
        .servicios
        .instancias
        .loaders
        .get(&c.destino)
        .map(|l| {
            super::LOADERS
                .iter()
                .enumerate()
                .filter(|(idx, _)| *idx == 0 || !l.versiones[*idx].is_empty())
                .map(|(index, (name, _, _))| {
                    l.versiones[index]
                        .first()
                        .map_or_else(|| name.to_string(), |version| format!("{name} {version}"))
                })
                .collect()
        })
        .unwrap_or_default();
    let selector: Element<'a, Mensaje> = if destinos.len() > 1 {
        let elegida = destinos
            .iter()
            .find(|opcion| {
                opcion
                    .split_whitespace()
                    .next()
                    .is_some_and(|nombre| nombre.eq_ignore_ascii_case(&c.loader_destino))
            })
            .cloned();
        iced::widget::pick_list(destinos, elegida, |s: String| {
            m(Accion::CambiarLoader(
                s.split_whitespace()
                    .next()
                    .unwrap_or("vanilla")
                    .to_lowercase(),
            ))
        })
        .font(fuentes::MONO)
        .text_size(11)
        .padding([4., 8.])
        .width(Length::Fill)
        .style(icaro_ui::estilo::selector(p))
        .menu_style(icaro_ui::estilo::menu_selector(p))
        .into()
    } else {
        mono(
            if c.destino.is_empty() {
                "—".to_owned()
            } else {
                crate::ficha::loader(&c.loader_destino).to_owned()
            },
            11.,
            p.text_muted,
        )
        .into()
    };
    let antes_despues = container(
        row![
            column![
                componentes::etiqueta(p, "Ahora"),
                mono(&record.minecraft, 18., p.text),
                mono(loader, 11., p.text_muted)
            ]
            .spacing(4)
            .width(Length::Fill),
            icono(Icono::Adelante, Tam::Sm, p.text_muted),
            column![
                componentes::etiqueta(p, "Después"),
                mono(
                    if c.destino.is_empty() {
                        "—"
                    } else {
                        &c.destino
                    },
                    18.,
                    p.text
                ),
                selector
            ]
            .spacing(4)
            .width(150)
        ]
        .spacing(if angosto { 8 } else { 12 })
        .align_y(Alignment::Center),
    )
    .padding([18., if angosto { 8. } else { 20. }])
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(p.surface_sunken)),
        border: Border {
            color: p.border,
            width: 1.,
            radius: 0.into(),
        },
        ..Default::default()
    });
    let mut body = column![antes_despues].spacing(22);
    if let Some(datos) = datos {
        let desactivar = if c.actualizar_mods {
            &datos.desactivar
        } else {
            &datos.desactivar_sin_actualizar
        };
        let nombres = if c.actualizar_mods {
            &datos.nombres
        } else {
            &datos.nombres_sin_actualizar
        };
        let actualizar = if c.actualizar_mods {
            datos.actualizar.len()
        } else {
            0
        };
        let sin_cambio = datos.total.saturating_sub(actualizar + desactivar.len());
        let mut cifras = column![
            dato(p, "Se actualizarán", actualizar.to_string(), false),
            dato(p, "No cambian", sin_cambio.to_string(), false)
        ];
        let detalle = button(
            row![
                icono(
                    if c.detalle_desactivar {
                        Icono::ChevronAbajo
                    } else {
                        Icono::ChevronDerecha
                    },
                    Tam::Sm,
                    p.warning
                ),
                text("Se desactivarán").size(13).color(p.text),
                Space::with_width(Length::Fill),
                mono(desactivar.len().to_string(), 13., p.warning)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([10., 0.])
        .width(Length::Fill)
        .on_press(m(Accion::CambiarDetalle))
        .style(icaro_ui::estilo::boton_fantasma(p));
        cifras = cifras.push(detalle);
        if c.detalle_desactivar && !nombres.is_empty() {
            cifras = cifras.push(
                container(text(nombres.join(", ")).size(13).color(p.text_muted)).padding(Padding {
                    top: 0.,
                    right: 0.,
                    bottom: 8.,
                    left: 0.,
                }),
            );
        }
        body = body.push(cifras.push(regla(p)));
        let actualizar_label =
            format!("Actualizar los {} mods compatibles", datos.actualizar.len());
        let actualizar_checkbox = iced::widget::checkbox(actualizar_label, c.actualizar_mods)
            .on_toggle(|b| m(Accion::CambiarActualizar(b)))
            .icon(componentes::icono_casilla())
            .font(fuentes::CUERPO)
            .size(20)
            .text_size(13)
            .spacing(8)
            .style(icaro_ui::estilo::casilla(p));
        let respaldo_checkbox: Element<'a, Mensaje> = if down {
            iced::widget::checkbox("Copia de seguridad de los mundos (obligatoria)", true)
                .icon(componentes::icono_casilla())
                .font(fuentes::CUERPO)
                .size(20)
                .text_size(13)
                .spacing(8)
                .style(icaro_ui::estilo::casilla(p))
                .into()
        } else {
            iced::widget::checkbox("Copia de seguridad de los mundos", c.respaldo)
                .on_toggle(|b| m(Accion::CambiarRespaldo(b)))
                .icon(componentes::icono_casilla())
                .font(fuentes::CUERPO)
                .size(20)
                .text_size(13)
                .spacing(8)
                .style(icaro_ui::estilo::casilla(p))
                .into()
        };
        body = body.push(column![actualizar_checkbox, respaldo_checkbox].spacing(12));
        body = body.push(
            column![
                regla(p),
                radio(p, "Cambiar esta instancia".into(), !c.copia, false),
                radio(
                    p,
                    format!("Crear una copia con {} y dejar esta igual", c.destino),
                    c.copia,
                    true
                )
            ]
            .spacing(10),
        );
        let (mensaje, riesgo) = if down {
            (format!("Bajar a {} puede dañar tus mundos. Se hará una copia de seguridad antes de cambiar.", c.destino), 2)
        } else if !desactivar.is_empty() {
            (format!("{} mods no tienen versión para {} y quedarán desactivados hasta que la tengan.", desactivar.len(), c.destino), 1)
        } else {
            (
                format!(
                    "La instancia usará Minecraft {}. Los mundos y archivos se conservarán.",
                    c.destino
                ),
                0,
            )
        };
        body = body.push(aviso(p, mensaje, riesgo));
    } else if c.destino.is_empty() {
        body = body.push(
            text("Elige una versión para revisar qué cambia.")
                .size(13)
                .color(p.text_muted),
        );
    } else if let Some(error) = c.errores_compatibilidad.get(&c.destino) {
        body = body.push(aviso(p, error.clone(), 1));
    } else {
        body = body.push(
            column![
                esqueleto(app),
                text("Consultando compatibilidad de los mods…")
                    .size(13)
                    .color(p.text_muted)
            ]
            .spacing(12),
        );
    }
    if let Some(error) = &c.error {
        body = body.push(text(error).size(13).color(p.error));
    }
    container(crate::desplazable_pub(body.into()))
        .padding([24., 28.])
        .width(Length::FillPortion(2))
        .height(Length::Fill)
        .into()
}

pub(super) fn vista<'a>(app: &'a App) -> Element<'a, Mensaje> {
    let p = app.paleta();
    let Some(c) = app.servicios.instancias.cambio.as_ref() else {
        return Space::new(0, 0).into();
    };
    let Some(record) = app
        .servicios
        .instancias
        .registros
        .iter()
        .find(|i| i.id == c.instancia)
    else {
        return Space::new(0, 0).into();
    };
    let enabled = !app.servicios.instancias.operando
        && !c.revisando
        && c.destino != record.minecraft
        && c.compatibilidades.contains_key(&c.destino);
    let label = if c.revisando {
        "Revisando…".to_owned()
    } else if c.destino.is_empty() {
        "Cambiar versión".to_owned()
    } else {
        format!("Cambiar a {}", c.destino)
    };
    let cabeza = row![
        text(format!("Cambiar versión de {}", record.nombre))
            .font(fuentes::TITULO)
            .size(24)
            .color(p.text),
        Space::with_width(Length::Fill),
        button(icono(Icono::Cerrar, Tam::Base, p.text_muted))
            .padding(8)
            .on_press(m(Accion::CambiarCancelar))
            .style(icaro_ui::estilo::boton_fantasma(p))
    ]
    .spacing(16)
    .align_y(Alignment::Start);
    let cuerpo = row![
        lista(app, &record.minecraft),
        container(Space::new(1, Length::Fill)).style(icaro_ui::estilo::bloque(p.border)),
        resumen(app, record)
    ]
    .height(Length::Fill);
    let pie = row![
        Space::with_width(Length::Fill),
        componentes::boton(
            p,
            "Cancelar",
            Variante::Secundario,
            Some(m(Accion::CambiarCancelar))
        ),
        componentes::boton_con_icono(
            p,
            Icono::Refrescar,
            &label,
            Variante::Primario,
            enabled.then(|| m(Accion::CambiarRevisar)),
            false
        )
    ]
    .spacing(8)
    .width(Length::Fill);
    container(column![
        container(cabeza).padding(Padding {
            top: 24.,
            right: 24.,
            bottom: 0.,
            left: 24.
        }),
        cuerpo,
        regla(p),
        container(pie).padding([16., 24.])
    ])
    .width((app.ventana.width - 48.).clamp(640., 980.))
    .height((app.ventana.height - 96.).clamp(400., 700.))
    .style(icaro_ui::estilo::flotante(p))
    .into()
}
