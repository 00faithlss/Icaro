// SPDX-License-Identifier: GPL-3.0-only
//! Primer uso de ZeroTier con los pasos y la lámina originales.
use crate::{App, Mensaje};
use icaro_core::zerotier::{self, Estado as EstadoZero};
use icaro_ui::{
    componentes::{self, Variante},
    fuentes,
    iconos::Icono,
    laminas::{grabado_posicion, Lamina},
};
use iced::{
    widget::{column, container, row, text, text_input, Space},
    Element, Length, Task,
};
#[derive(Default)]
pub struct Estado {
    pub actual: Option<EstadoZero>,
    id: String,
    busy: bool,
    request: u64,
    error: Option<String>,
    solicitada: Option<String>,
}
#[derive(Debug, Clone)]
pub enum Accion {
    Abrir,
    Consultar,
    Resultado(u64, EstadoZero),
    Id(String),
    Unirse,
    Hecha(u64, Result<EstadoZero, String>),
    Central,
    Descargar,
    Volver,
}
fn m(a: Accion) -> Mensaje {
    Mensaje::RedPrivada(a)
}
pub fn actualizar(app: &mut App, a: Accion) -> Task<Mensaje> {
    if app.servicios.demo {
        return Task::none();
    }
    if app.servicios.red_privada.busy && !matches!(a, Accion::Hecha(..) | Accion::Resultado(..)) {
        return Task::none();
    }
    match a {
        Accion::Abrir | Accion::Consultar => {
            let s = &mut app.servicios.red_privada;
            s.request += 1;
            let g = s.request;
            return Task::perform(zerotier::detectar(), move |z| m(Accion::Resultado(g, z)));
        }
        Accion::Resultado(g, z) => {
            let s = &mut app.servicios.red_privada;
            if s.request == g {
                let accepted=s.solicitada.as_ref().is_some_and(|id|matches!(&z,EstadoZero::Disponible{redes,..} if redes.iter().any(|r|&r.id==id&&r.estado=="OK")));
                s.actual = Some(z);
                if accepted {
                    s.solicitada = None;
                    app.avisar("Tu dispositivo está autorizado en la red ZeroTier.".into());
                }
            }
        }
        Accion::Id(v) => app.servicios.red_privada.id = v,
        Accion::Unirse => {
            let s = &mut app.servicios.red_privada;
            let id = match zerotier::id_red(&s.id) {
                Ok(id) => id,
                Err(e) => {
                    s.error = Some(e);
                    return Task::none();
                }
            };
            s.busy = true;
            s.solicitada = Some(id.clone());
            s.error = None;
            s.request += 1;
            let g = s.request;
            return Task::perform(
                async move {
                    zerotier::cambiar_red(
                        &id,
                        zerotier::Accion::Unirse,
                        tokio_util::sync::CancellationToken::new(),
                    )
                    .await
                },
                move |r| m(Accion::Hecha(g, r)),
            );
        }
        Accion::Hecha(g, r) => {
            let s = &mut app.servicios.red_privada;
            if g != s.request {
                return Task::none();
            }
            s.busy = false;
            match r {
                Ok(z) => {
                    let accepted=s.solicitada.as_ref().is_some_and(|id|matches!(&z,EstadoZero::Disponible{redes,..} if redes.iter().any(|r|&r.id==id&&r.estado=="OK")));
                    s.actual = Some(z);
                    if accepted {
                        s.solicitada = None;
                        app.avisar("Tu dispositivo está autorizado en la red ZeroTier.".into());
                        return Task::none();
                    }
                    app.avisar(
                        "Solicitud de red enviada. Se comprobará la autorización del anfitrión."
                            .into(),
                    );
                }
                Err(e) => s.error = Some(e),
            }
        }
        Accion::Central => {
            return crate::servicios::actualizar(
                app,
                crate::servicios::Accion::Enlace("https://central.zerotier.com/".into()),
            )
        }
        Accion::Descargar => {
            return crate::servicios::actualizar(
                app,
                crate::servicios::Accion::Enlace("https://www.zerotier.com/download/".into()),
            )
        }
        Accion::Volver => {
            app.pantalla = None;
            return crate::servidores_reales::actualizar(
                app,
                crate::servidores_reales::Accion::Consultar,
            );
        }
    }
    Task::none()
}
pub fn suscripcion(app: &App) -> iced::Subscription<Mensaje> {
    if !app.servicios.demo
        && app.pantalla == Some(crate::pantallas::Pantalla::Red)
        && !app.servicios.red_privada.busy
    {
        iced::time::every(std::time::Duration::from_secs(10)).map(|_| m(Accion::Consultar))
    } else {
        iced::Subscription::none()
    }
}
pub fn vista(app: &App) -> Element<'_, Mensaje> {
    let s = &app.servicios.red_privada;
    let p = app.paleta();
    let description=match &s.actual{
        None=>"Comprobando instalación y servicio…".to_string(),
        Some(EstadoZero::Ausente)=>"ZeroTier no está instalado.".into(),
        Some(EstadoZero::SinAcceso)=>"El servicio de ZeroTier no responde o no permite consultar. Comprueba el servicio y sus permisos.".into(),
        Some(EstadoZero::Disponible{version,online,..})=>format!("Versión {version} · {}",if *online{"servicio conectado."}else{"servicio sin conexión."}),
    };
    let step = |n: &str, title: &str, detail: String| {
        column![
            row![
                container(text(n.to_owned()).font(fuentes::MONO).size(14))
                    .padding(8)
                    .style(icaro_ui::estilo::marco(p)),
                text(title.to_owned())
                    .font(fuentes::CUERPO_NEGRITA)
                    .size(16)
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
            text(detail).size(13).color(p.text_muted)
        ]
        .spacing(8)
    };
    let usable = matches!(s.actual, Some(EstadoZero::Disponible { .. }));
    let mut body = column![
        text("Juega con amigos\nsin abrir puertos")
            .font(fuentes::DISPLAY)
            .size(32)
            .line_height(iced::widget::text::LineHeight::Absolute(34.into())),
        step("1", "ZeroTier instalado", description),
    ]
    .spacing(24)
    .padding(40);
    if matches!(s.actual, Some(EstadoZero::Ausente)) {
        body = body.push(componentes::boton(
            p,
            "Instalar ZeroTier",
            Variante::Primario,
            (!s.busy).then(|| m(Accion::Descargar)),
        ));
    } else if matches!(s.actual, Some(EstadoZero::SinAcceso)) {
        body = body.push(componentes::boton(
            p,
            "Comprobar de nuevo",
            Variante::Secundario,
            (!s.busy).then(|| m(Accion::Consultar)),
        ));
    }
    let mut input = text_input("ID de red de 16 caracteres", &s.id)
        .font(fuentes::MONO)
        .padding(12)
        .width(Length::Fill)
        .style(icaro_ui::estilo::campo(p));
    if !s.busy {
        input = input.on_input(|v| m(Accion::Id(v)));
    }
    body = body.push(
        column![
            step(
                "2",
                "Únete a una red",
                "Pide el ID de red a quien la creó.".into()
            ),
            input,
            row![
                componentes::boton(
                    p,
                    if s.busy { "Uniéndose…" } else { "Unirse" },
                    Variante::Primario,
                    (usable && !s.busy && zerotier::id_red(&s.id).is_ok())
                        .then(|| m(Accion::Unirse))
                ),
                componentes::boton(
                    p,
                    "Crear una red nueva",
                    Variante::Fantasma,
                    (!s.busy).then(|| m(Accion::Central))
                )
            ]
            .spacing(8)
        ]
        .spacing(12),
    );
    body = body.push(step(
        "3",
        "Espera la autorización",
        "Te avisaremos cuando el dueño de la red te autorice.".into(),
    ));
    if let Some(EstadoZero::Disponible { redes, .. }) = &s.actual {
        for red in redes {
            let status = match red.estado.as_str() {
                "OK" => "Red autorizada",
                "ACCESS_DENIED" => "Aún no te autorizan en la red",
                "REQUESTING_CONFIGURATION" => "Esperando configuración del anfitrión",
                _ => &red.estado,
            };
            body = body.push(
                column![
                    text(if red.nombre.is_empty() {
                        red.id.clone()
                    } else {
                        red.nombre.clone()
                    })
                    .font(fuentes::CUERPO_NEGRITA)
                    .size(14),
                    text(format!("{} · {status}", red.id))
                        .font(fuentes::MONO)
                        .size(12)
                        .color(p.text_muted),
                    text(red.ips.join(", "))
                        .font(fuentes::MONO)
                        .size(12)
                        .color(p.text_muted)
                ]
                .spacing(4),
            );
        }
    }
    if let Some(e) = &s.error {
        body = body.push(text(e.clone()).size(13).color(p.error));
    }
    body = body.push(componentes::boton_con_icono(
        p,
        Icono::Atras,
        "Volver",
        Variante::Fantasma,
        (!s.busy).then(|| m(Accion::Volver)),
        false,
    ));
    let picture = container(grabado_posicion(
        p,
        Lamina::HilosSol,
        Length::Fill,
        Length::Fill,
        (0.5, 0.45),
    ))
    .width(360)
    .height(Length::Fill);
    let border = container(Space::with_width(2))
        .height(Length::Fill)
        .style(icaro_ui::estilo::bloque(p.text));
    row![crate::desplazable_pub(body.into()), border, picture]
        .height(Length::Fill)
        .into()
}
pub fn preparar_diseno(app: &mut App, scene: &str) {
    app.pantalla = Some(crate::pantallas::Pantalla::Red);
    if scene.ends_with("angosto") {
        app.ventana = iced::Size::new(940., 720.);
    }
    app.servicios.red_privada.id = "1234567890abcdef".into();
    app.servicios.red_privada.actual = Some(if scene.contains("ausente") {
        EstadoZero::Ausente
    } else {
        EstadoZero::Disponible {
            version: "1.16.0".into(),
            online: true,
            redes: vec![zerotier::Red {
                id: "1234567890abcdef".into(),
                nombre: "Amigos".into(),
                estado: if scene.contains("autorizada") {
                    "OK".into()
                } else {
                    "ACCESS_DENIED".into()
                },
                ips: if scene.contains("autorizada") {
                    vec!["10.147.17.4/24".into()]
                } else {
                    vec![]
                },
            }],
        }
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn respuestas_tardias_no_reemplazan_una_solicitud_nueva() {
        let (mut app, _) = crate::iniciar_con_guardado(
            icaro_ui::shell::Seccion::Servidores,
            true,
            None,
            false,
            crate::servicios::Estado::nuevo(true),
            None,
        );
        app.servicios.demo = false;
        app.servicios.red_privada.request = 3;
        let _ = actualizar(&mut app, Accion::Resultado(2, EstadoZero::Ausente));
        assert!(app.servicios.red_privada.actual.is_none());
        app.servicios.red_privada.busy = true;
        let _ = actualizar(&mut app, Accion::Hecha(2, Err("antiguo".into())));
        assert!(app.servicios.red_privada.busy);
        let _ = actualizar(&mut app, Accion::Hecha(3, Err("sin permisos".into())));
        assert!(!app.servicios.red_privada.busy);
        assert_eq!(
            app.servicios.red_privada.error.as_deref(),
            Some("sin permisos")
        );
    }
}
