// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! ActualizacionLauncher del System Design, con releases y notas reales.
use crate::{pantallas, App, Mensaje};
use icaro_core::actualizacion::{self, Disponible, Preparada};
use icaro_ui::{
    componentes::{self, Estado as Senal, Variante},
    fuentes,
    iconos::Icono,
    laminas::{self, Lamina},
    tema::{espacio, texto},
};
use iced::futures::SinkExt;
use iced::{
    widget::{column, container, row, text, Space},
    Alignment, Element, Length, Task,
};

#[derive(Default)]
pub struct Estado {
    pub disponible: Option<Disponible>,
    pub comprobando: bool,
    pub comprobada: bool,
    pub error: Option<String>,
    pub descargando: bool,
    pub avance: f32,
    pub preparada: Option<Preparada>,
    pub al_cerrar: bool,
    anterior: Option<pantallas::Pantalla>,
}
#[derive(Debug, Clone)]
pub enum Accion {
    Comprobar,
    Comprobada(Result<Option<Disponible>, String>),
    Abrir,
    Ahora,
    AlCerrar,
    MasTarde,
    Progreso(u64, u64),
    Descargada(bool, Result<Preparada, String>),
}
pub fn m(a: Accion) -> Mensaje {
    Mensaje::Actualizacion(a)
}
pub fn comprobar() -> Task<Mensaje> {
    Task::perform(
        async {
            actualizacion::comprobar(env!("CARGO_PKG_VERSION"))
                .await
                .map_err(|e| e.to_string())
        },
        |r| m(Accion::Comprobada(r)),
    )
}
fn juego_abierto(app: &App) -> bool {
    !app.servicios.juego.procesos.is_empty()
        || app.servicios.juego.trabajos.values().any(|t| {
            matches!(
                t.estado,
                icaro_ui::descargas::EstadoDescarga::Activa(_)
                    | icaro_ui::descargas::EstadoDescarga::EnCola(_)
            )
        })
}
pub fn actualizar(app: &mut App, accion: Accion) -> Task<Mensaje> {
    match accion {
        Accion::Abrir => {
            if app.pantalla != Some(pantallas::Pantalla::Actualizacion) {
                app.actualizacion.anterior = app.pantalla;
            }
            app.pantalla = Some(pantallas::Pantalla::Actualizacion);
            if !app.actualizacion.comprobada && !app.actualizacion.comprobando {
                app.actualizacion.comprobando = true;
                return comprobar();
            }
        }
        Accion::Comprobar => {
            if !app.actualizacion.comprobando && !app.actualizacion.descargando {
                app.actualizacion.comprobando = true;
                app.actualizacion.error = None;
                return comprobar();
            }
        }
        Accion::Comprobada(r) => {
            app.actualizacion.comprobando = false;
            app.actualizacion.comprobada = true;
            match r {
                Ok(d) => {
                    if app
                        .actualizacion
                        .preparada
                        .as_ref()
                        .is_some_and(|p| d.as_ref().is_none_or(|n| n.version != p.version))
                    {
                        app.actualizacion.preparada = None;
                        app.actualizacion.al_cerrar = false;
                    }
                    app.actualizacion.disponible = d;
                    app.actualizacion.error = None;
                }
                Err(e) => app.actualizacion.error = Some(e),
            }
        }
        Accion::Ahora | Accion::AlCerrar => {
            let ahora = matches!(accion, Accion::Ahora);
            if app.actualizacion.descargando {
                return Task::none();
            }
            if ahora && juego_abierto(app) {
                app.actualizacion.error =
                    Some("Cierra Minecraft y espera a que terminen sus tareas antes de actualizar Ícaro.".into());
                return Task::none();
            }
            if let Some(p) = app.actualizacion.preparada.as_ref() {
                if ahora {
                    match actualizacion::aplicar(p, true) {
                        Ok(()) => return crate::juego_real::cerrar(app),
                        Err(e) => app.actualizacion.error = Some(e.to_string()),
                    }
                } else {
                    app.actualizacion.al_cerrar = true;
                    app.pantalla = app.actualizacion.anterior;
                }
                return Task::none();
            }
            let Some(d) = app.actualizacion.disponible.clone() else {
                return Task::none();
            };
            app.actualizacion.descargando = true;
            app.actualizacion.error = None;
            app.actualizacion.avance = 0.;
            return Task::run(
                iced::stream::channel(32, move |mut salida| async move {
                    let (tx, mut rx) = iced::futures::channel::mpsc::unbounded();
                    let descargar = actualizacion::descargar(d, move |n, total| {
                        let _ = tx.unbounded_send(m(Accion::Progreso(n, total)));
                    });
                    tokio::pin!(descargar);
                    use iced::futures::StreamExt;
                    let resultado = loop {
                        tokio::select! {
                            r = &mut descargar => break r.map_err(|e| e.to_string()),
                            Some(a) = rx.next() => { let _ = salida.send(a).await; }
                        }
                    };
                    let _ = salida.send(m(Accion::Descargada(ahora, resultado))).await;
                }),
                |a| a,
            );
        }
        Accion::MasTarde => {
            if app.actualizacion.descargando {
                return Task::none();
            }
            app.actualizacion.al_cerrar = false;
            app.pantalla = app.actualizacion.anterior;
        }
        Accion::Progreso(n, total) => {
            app.actualizacion.avance = if total == 0 {
                0.
            } else {
                (n as f32 / total as f32).clamp(0., 1.)
            };
        }
        Accion::Descargada(ahora, r) => {
            app.actualizacion.descargando = false;
            match r {
                Err(e) => app.actualizacion.error = Some(e),
                Ok(p) => {
                    app.actualizacion.preparada = Some(p);
                    if ahora && !juego_abierto(app) {
                        return actualizar(app, Accion::Ahora);
                    }
                    app.actualizacion.al_cerrar = true;
                    if ahora {
                        app.actualizacion.error = Some(
                            "Minecraft está abierto. La actualización se aplicará al cerrar Ícaro."
                                .into(),
                        );
                    } else {
                        app.pantalla = app.actualizacion.anterior;
                    }
                }
            }
        }
    }
    Task::none()
}
/// Invocar antes del cierre normal. No fuerza el cierre de un juego.
pub fn al_cerrar(app: &mut App) -> bool {
    if !app.actualizacion.al_cerrar {
        return true;
    }
    if juego_abierto(app) {
        app.actualizacion.error =
            Some("Cierra Minecraft y espera a que terminen sus tareas antes de aplicar la actualización pendiente.".into());
        app.pantalla = Some(pantallas::Pantalla::Actualizacion);
        return false;
    }
    if let Some(p) = app.actualizacion.preparada.as_ref() {
        if let Err(e) = actualizacion::aplicar(p, false) {
            app.actualizacion.error = Some(e.to_string());
            app.pantalla = Some(pantallas::Pantalla::Actualizacion);
            return false;
        }
    }
    app.actualizacion.al_cerrar = false;
    true
}

/// Banner info discreto; nunca abre la actualización por su cuenta.
pub fn banner(app: &App) -> Option<Element<'_, Mensaje>> {
    let d = app.actualizacion.disponible.as_ref()?;
    let p = app.paleta();
    if app.pantalla == Some(pantallas::Pantalla::Actualizacion) {
        return None;
    }
    Some(
        container(
            row![
                text(format!("Ícaro {} disponible", d.version))
                    .size(texto::BODY_SM.0)
                    .color(p.text),
                Space::with_width(Length::Fill),
                componentes::boton(
                    p,
                    "Ver actualización",
                    Variante::Fantasma,
                    Some(m(Accion::Abrir))
                )
            ]
            .align_y(Alignment::Center),
        )
        .padding([espacio::S2, espacio::S4])
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(p.info_soft.into()),
            ..Default::default()
        })
        .into(),
    )
}
pub fn vista(app: &App) -> Element<'_, Mensaje> {
    iced::widget::responsive(move |tam| vista_ancho(app, tam.width)).into()
}
fn vista_ancho(app: &App, ancho: f32) -> Element<'_, Mensaje> {
    let p = app.paleta();
    let e = &app.actualizacion;
    let titulo = e
        .disponible
        .as_ref()
        .map(|d| format!("ÍCARO {}", d.version))
        .unwrap_or_else(|| "ÍCARO".into());
    let detalle = e
        .disponible
        .as_ref()
        .map(|d| {
            format!(
                "Tienes {} · descarga de {:.1} MB",
                env!("CARGO_PKG_VERSION"),
                d.bytes as f64 / 1048576.
            )
        })
        .unwrap_or_else(|| format!("Versión {}", env!("CARGO_PKG_VERSION")));
    let mut notas = column![].spacing(espacio::S2);
    if let Some(d) = &e.disponible {
        for n in &d.notas {
            let insignia = match n.tipo.as_str() {
                "nuevo" => componentes::insignia(p, Senal::Exito, "Nuevo"),
                "mejora" => componentes::insignia(p, Senal::Info, "Mejora"),
                _ => componentes::insignia_contorno(p, "Corrección"),
            };
            notas = notas.push(
                row![
                    container(insignia).width(110),
                    text(n.texto.clone()).size(texto::BODY.0).color(p.text)
                ]
                .spacing(espacio::S4)
                .align_y(Alignment::Center),
            );
        }
    } else {
        notas = notas.push(
            text(if e.comprobando {
                "Comprobando actualizaciones…"
            } else if e.comprobada && e.error.is_none() {
                "Tienes la versión más reciente publicada."
            } else {
                "Comprueba si hay una nueva versión de Ícaro."
            })
            .size(texto::BODY.0)
            .color(p.text_muted),
        );
    }
    if let Some(error) = &e.error {
        notas = notas.push(text(error.clone()).size(texto::BODY_SM.0).color(p.error));
    }
    if e.descargando {
        notas = notas.push(
            text(format!(
                "Descargando actualización · {:.0} %",
                e.avance * 100.
            ))
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted),
        );
    }
    if e.al_cerrar {
        notas = notas.push(
            text("Actualización lista para aplicar al cerrar Ícaro.")
                .size(texto::BODY_SM.0)
                .color(p.success),
        );
    }
    if juego_abierto(app) && e.disponible.is_some() {
        notas = notas.push(
            text("Cierra Minecraft antes de actualizar ahora.")
                .size(texto::BODY_SM.0)
                .color(p.text_muted),
        );
    }
    let acciones: Element<'_, Mensaje> = if e.disponible.is_some() {
        row![
            componentes::boton_con_icono(
                p,
                Icono::Actualizar,
                "Actualizar ahora",
                Variante::Primario,
                (!e.descargando && !juego_abierto(app)).then(|| m(Accion::Ahora)),
                false
            ),
            componentes::boton(
                p,
                "Al cerrar Ícaro",
                Variante::Secundario,
                (!e.descargando).then(|| m(Accion::AlCerrar))
            ),
            componentes::boton(
                p,
                "Más tarde",
                Variante::Fantasma,
                (!e.descargando).then(|| m(Accion::MasTarde))
            )
        ]
        .spacing(espacio::S2)
        .align_y(Alignment::Center)
        .into()
    } else {
        row![
            componentes::boton(
                p,
                "Comprobar actualizaciones",
                Variante::Primario,
                (!e.comprobando).then(|| m(Accion::Comprobar))
            ),
            componentes::boton(p, "Volver", Variante::Fantasma, Some(m(Accion::MasTarde)))
        ]
        .spacing(espacio::S2)
        .into()
    };
    let cuerpo = column![
        column![
            text(titulo)
                .font(fuentes::DISPLAY)
                .size(texto::DISPLAY_XL.0)
                .color(p.text),
            text(detalle)
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted)
        ]
        .spacing(espacio::S2),
        notas,
        Space::with_height(Length::Fill),
        acciones
    ]
    .spacing(espacio::S6)
    .padding(40)
    .width(Length::Fill)
    .height(Length::Fill);
    if ancho < 1000. {
        return cuerpo.into();
    }
    row![
        container(laminas::grabado_posicion(
            p,
            Lamina::Saturno,
            Length::Fill,
            Length::Fill,
            (0.5, 0.3)
        ))
        .width(300)
        .height(Length::Fill)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            border: iced::Border {
                color: p.text,
                width: 2.,
                radius: 0.into()
            },
            ..Default::default()
        }),
        cuerpo
    ]
    .height(Length::Fill)
    .width(Length::Fill)
    .into()
}

/// Escena aislada para comparar Tinta/Piedra con el diseño; nunca se usa como oferta real.
pub fn preparar_diseno(app: &mut App) {
    let disponible = serde_json::from_value(serde_json::json!({
        "version": "1.3.0",
        "bytes": 14889779,
        "url": "https://github.com/00faithlss/Icaro/releases/download/v1.3.0/icaro-windows-x86_64.exe",
        "sha256": "00".repeat(32),
        "notas": [
            {"tipo":"nuevo", "texto":"Sincroniza mods con un servidor antes de unirte."},
            {"tipo":"nuevo", "texto":"Estudio de skin con vista de frente y espalda."},
            {"tipo":"mejora", "texto":"La consola maneja 100.000 líneas sin trabarse."},
            {"tipo":"correccion", "texto":"Las descargas pausadas ya no se reinician al cerrar."}
        ]
    })).expect("Escena de ActualizacionLauncher válida");
    app.actualizacion = Estado {
        disponible: Some(disponible),
        comprobada: true,
        ..Default::default()
    };
    app.pantalla = Some(pantallas::Pantalla::Actualizacion);
}

/// Variante visual de la programación al cierre; no contiene ningún archivo ejecutable.
pub fn preparar_diseno_al_cerrar(app: &mut App) {
    preparar_diseno(app);
    app.actualizacion.al_cerrar = true;
}
