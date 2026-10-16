// SPDX-License-Identifier: GPL-3.0-only
//! CuentaSinConexion del diseño entregado: nombres locales, sin credenciales.
use crate::{servicios, App, Mensaje};
use icaro_core::{auth::Session, cuentas_locales::Cuentas};
use icaro_ui::{
    componentes as c, cuenta, estilo, fuentes,
    iconos::{icono, Icono, Tam},
    tema::texto,
};
use iced::{
    widget::{column, container, row, text, text_input, Space},
    Alignment, Element, Length, Task,
};
use std::sync::Arc;

#[derive(Default)]
pub struct Estado {
    pub abierta: bool,
    pub cuentas: Cuentas,
    pub microsoft: Option<Arc<Session>>,
    pub imagenes: std::collections::BTreeMap<String, (icaro_core::skin_local::Imagen, bool)>,
    pub skin: Option<crate::skin_sin_conexion::Estado>,
    pub revision_skin: u64,
    nombre: String,
    estricto: bool,
    pub ocupado: bool,
    error: Option<String>,
    confirmar: Option<String>,
    exito: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Accion {
    Skin(crate::skin_sin_conexion::Accion),
    Imagenes(
        u64,
        u64,
        Result<std::collections::BTreeMap<String, (icaro_core::skin_local::Imagen, bool)>, String>,
    ),
    Abrir,
    Cerrar,
    Nombre(String),
    RevisarNombre,
    Guardar,
    Usar(String),
    UsarMicrosoft,
    Eliminar(String),
    Confirmar,
    CancelarEliminar,
    Cargadas(Result<Cuentas, String>),
    Terminada(u64, Result<Cuentas, String>),
}
pub fn m(a: Accion) -> Mensaje {
    Mensaje::Servicio(servicios::Accion::Local(a))
}

pub fn cargar_imagenes(app: &App) -> Task<Mensaje> {
    let nombres = app.servicios.local.cuentas.nombres.clone();
    let id = app.servicios.auth_id;
    let revision = app.servicios.local.revision_skin;
    let root = crate::estado::directorio(false);
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let mut imagenes = std::collections::BTreeMap::new();
                for nombre in nombres {
                    if let Some(imagen) = icaro_core::skin_local::leer(&root, &nombre)? {
                        imagenes.insert(nombre, imagen);
                    }
                }
                Ok(imagenes)
            })
            .await
            .map_err(|_| "No se pudieron leer las skins locales.".to_string())
            .and_then(|r| r)
        },
        move |r| m(Accion::Imagenes(id, revision, r)),
    )
}

pub fn textura_activa(app: &App) -> Option<&[u8]> {
    if let Some(s) = app.servicios.session.as_ref().filter(|s| s.es_local()) {
        return app
            .servicios
            .local
            .imagenes
            .get(&s.profile.name)
            .map(|(i, _)| i.textura.rgba.as_ref());
    }
    app.sesion.cuenta.actual.as_ref().map(|t| t.rgba.as_ref())
}

fn validar(nombre: &str, cuentas: &[String], estricto: bool) -> Result<(), String> {
    if nombre.is_empty() {
        return Err(String::new());
    }
    if nombre.chars().any(char::is_whitespace) {
        return Err("Sin espacios. Usa _ para separar, como en Jugador_1.".into());
    }
    if let Some(c) = nombre.chars().find(|c| !c.is_ascii()) {
        return Err(format!(
            "Sin acentos ni ñ: cambia «{c}» por una letra simple."
        ));
    }
    if let Some(c) = nombre
        .chars()
        .find(|c| !c.is_ascii_alphanumeric() && *c != '_')
    {
        return Err(format!("Solo letras, números y _. Quita «{c}»."));
    }
    if nombre.len() > 16 {
        return Err(format!("Máximo 16 caracteres; tiene {}.", nombre.len()));
    }
    if cuentas.iter().any(|n| n.eq_ignore_ascii_case(nombre)) {
        return Err(
            "Ya tienes una cuenta sin conexión con ese nombre. Úsala desde la lista.".into(),
        );
    }
    if nombre.len() < 3 {
        return Err(if estricto {
            "Muy corto: mínimo 3 caracteres.".into()
        } else {
            String::new()
        });
    }
    Ok(())
}

pub fn actualizar(app: &mut App, a: Accion) -> Task<Mensaje> {
    match a {
        Accion::Skin(a) => return crate::skin_sin_conexion::actualizar(app, a),
        Accion::Imagenes(id, revision, r)
            if id == app.servicios.auth_id && revision == app.servicios.local.revision_skin =>
        {
            match r {
                Ok(imagenes) => app.servicios.local.imagenes = imagenes,
                Err(e) => app.servicios.local.error = Some(e),
            }
        }
        Accion::Imagenes(_, _, _) => {}
        Accion::Cargadas(r) => match r {
            Ok(c) => {
                app.servicios.local.cuentas = c;
                return cargar_imagenes(app);
            }
            Err(e) => app.servicios.local.error = Some(e),
        },
        Accion::Terminada(id, r) if id == app.servicios.auth_id && app.servicios.local.ocupado => {
            app.servicios.auth_busy = false;
            app.servicios.local.ocupado = false;
            match r {
                Err(e) => app.servicios.local.error = Some(e),
                Ok(cuentas) => {
                    if app
                        .servicios
                        .session
                        .as_ref()
                        .is_some_and(|s| !s.es_local())
                    {
                        app.servicios.local.microsoft = app.servicios.session.clone();
                    }
                    app.sesion.cuenta.invalidar();
                    app.sesion.textura = None;
                    app.sesion.subiendo = None;
                    app.servicios.session = cuentas
                        .activa
                        .as_deref()
                        .and_then(|n| Session::local(n).ok())
                        .map(Arc::new)
                        .or_else(|| app.servicios.local.microsoft.clone());
                    app.sesion.estado = if app
                        .servicios
                        .session
                        .as_ref()
                        .is_some_and(|s| !s.es_local())
                    {
                        cuenta::EstadoSesion::Activa
                    } else {
                        cuenta::EstadoSesion::SinConexion
                    };
                    app.servicios.local.exito = cuentas.activa.clone();
                    app.servicios.local.cuentas = cuentas;
                    app.servicios.local.nombre.clear();
                    app.servicios.local.estricto = false;
                    app.servicios.local.confirmar = None;
                    // El diálogo permite revisar la cuenta en uso; Volver reanuda Jugar.
                    app.pantalla = None;
                    app.sesion.menu = false;
                    if app.servicios.session.is_none() {
                        return servicios::begin_auth(app, true);
                    }
                    return cargar_imagenes(app);
                }
            }
        }
        Accion::Terminada(_, _) => {}
        _ if app.servicios.auth_busy || app.servicios.restoring || app.servicios.local.ocupado => {}
        Accion::Abrir => {
            app.sesion.menu = false;
            app.servicios.local.abierta = true;
            app.servicios.local.exito = None;
            return iced::widget::text_input::focus(iced::widget::text_input::Id::new(
                "cuenta-local-nombre",
            ));
        }
        Accion::Cerrar if app.servicios.local.skin.is_some() => {
            return crate::skin_sin_conexion::actualizar(
                app,
                crate::skin_sin_conexion::Accion::Cerrar,
            )
        }
        Accion::Cerrar => {
            app.servicios.local.abierta = false;
            if app.servicios.session.is_none() {
                crate::juego_real::cancelar_espera_sesion(app);
                return Task::none();
            }
            return crate::juego_real::reanudar_sesion(app);
        }
        Accion::Nombre(n) => {
            app.servicios.local.nombre = n.chars().take(64).collect();
            app.servicios.local.error = None;
            app.servicios.local.exito = None;
        }
        Accion::RevisarNombre => {
            if !app.servicios.local.nombre.is_empty() {
                app.servicios.local.estricto = true;
            }
        }
        Accion::Eliminar(n) => {
            app.servicios.local.estricto = true;
            app.servicios.local.confirmar = Some(n);
        }
        Accion::CancelarEliminar => app.servicios.local.confirmar = None,
        Accion::Guardar | Accion::Usar(_) | Accion::Confirmar | Accion::UsarMicrosoft => {
            if !app.servicios.juego.permite_cambiar_cuenta() || app.sesion.subiendo.is_some() {
                app.servicios.local.error = Some("Cierra Minecraft y espera a que termine el cambio de skin antes de cambiar de cuenta.".into());
                return Task::none();
            }
            let eliminar = matches!(a, Accion::Confirmar);
            let microsoft = matches!(a, Accion::UsarMicrosoft);
            let nombre = match a {
                Accion::UsarMicrosoft => String::new(),
                Accion::Usar(n) if app.servicios.local.cuentas.nombres.contains(&n) => n,
                Accion::Usar(_) => return Task::none(),
                Accion::Confirmar => match app.servicios.local.confirmar.clone() {
                    Some(n) => n,
                    None => return Task::none(),
                },
                _ => {
                    app.servicios.local.estricto = true;
                    if validar(
                        &app.servicios.local.nombre,
                        &app.servicios.local.cuentas.nombres,
                        true,
                    )
                    .is_err()
                    {
                        return Task::none();
                    }
                    app.servicios.local.nombre.clone()
                }
            };
            app.servicios.cancel();
            app.servicios.auth_busy = true;
            app.servicios.local.ocupado = true;
            app.servicios.local.error = None;
            let id = app.servicios.auth_id;
            let raiz = crate::estado::directorio(false);
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        if microsoft {
                            Cuentas::desactivar(&raiz)?;
                            Cuentas::cargar(&raiz)
                        } else if eliminar {
                            Cuentas::eliminar(&raiz, &nombre)
                        } else {
                            Cuentas::activar(&raiz, &nombre)
                        }
                    })
                    .await
                    .map_err(|_| "No se pudo guardar la cuenta; vuelve a intentarlo.".to_string())
                    .and_then(|r| r)
                },
                move |r| m(Accion::Terminada(id, r)),
            );
        }
    }
    Task::none()
}

fn aviso_info(p: icaro_ui::tema::Paleta, mensaje: &str) -> Element<'_, Mensaje> {
    container(
        row![
            icono(Icono::InfoCaja, Tam::Base, p.info),
            text(mensaje).size(texto::BODY_SM.0).color(p.text)
        ]
        .spacing(14),
    )
    .padding([14, 18])
    .width(Length::Fill)
    .style(move |_: &iced::Theme| container::Style {
        background: Some(p.info_soft.into()),
        border: iced::Border {
            color: p.info,
            width: 1.,
            radius: 0.into(),
        },
        ..Default::default()
    })
    .into()
}

pub fn vista(app: &App) -> Element<'_, Mensaje> {
    if app.servicios.local.skin.is_some() {
        return crate::skin_sin_conexion::vista(app);
    }
    let s = &app.servicios.local;
    let p = app.paleta();
    let validacion = validar(&s.nombre, &s.cuentas.nombres, s.estricto);
    let error_nombre = validacion.as_ref().err().filter(|e| !e.is_empty());
    let ayuda = error_nombre
        .map(String::as_str)
        .unwrap_or(if validacion.is_ok() {
            "Nombre válido."
        } else {
            "Entre 3 y 16 caracteres: letras sin acentos, números y _."
        });
    let hay_error = error_nombre.is_some();
    let color = if hay_error {
        p.error
    } else if validacion.is_ok() {
        p.success
    } else {
        p.text_muted
    };
    let habilitado = !s.ocupado && !app.servicios.auth_busy;
    let campo = text_input("Jugador_1", &s.nombre)
        .id(iced::widget::text_input::Id::new("cuenta-local-nombre"))
        .font(fuentes::MONO)
        .size(texto::MONO.0)
        .on_input_maybe(habilitado.then(|| |n| m(Accion::Nombre(n))))
        .on_submit_maybe(habilitado.then(|| m(Accion::Guardar)))
        .padding(0)
        .style(move |theme, status| {
            let mut estilo = estilo::campo(p)(theme, status);
            estilo.border.width = 0.;
            estilo.background = iced::Background::Color(p.surface_sunken);
            estilo
        });
    let caja = container(
        row![
            icono(Icono::Usuario, Tam::Base, p.text_muted),
            campo,
            text(format!("{}/16", s.nombre.chars().count()))
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted)
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .padding([8, 12])
    .style(move |_: &iced::Theme| container::Style {
        background: Some(p.surface_sunken.into()),
        border: iced::Border {
            color: if hay_error { p.error } else { p.border_strong },
            width: 1.,
            radius: 0.into(),
        },
        ..Default::default()
    });
    let mut cuerpo = column![
        aviso_info(p, "Para jugar en mundos locales y servidores que admitan cuentas sin conexión. Los servidores con autenticación requieren una cuenta Microsoft."),
        column![c::etiqueta(p,"Nombre de jugador"), caja, row![icono(if error_nombre.is_some() {Icono::Aviso} else {Icono::Check},Tam::Sm,color),text(ayuda.to_owned()).size(texto::BODY_SM.0).color(color)].spacing(6)].spacing(8),
        container(row![container(Space::new(3,Length::Fill)).style(estilo::bloque(p.text)),
            container(row![icono(Icono::Rama,Tam::Sm,p.text_muted), iced::widget::rich_text([iced::widget::text::Span::new("Cada nombre usa una identidad distinta dentro del juego.").font(fuentes::CUERPO_NEGRITA).color(p.text), iced::widget::text::Span::new(" Si juegas con otro nombre, el mundo no mostrará tu inventario ni tu progreso. No se pierden: vuelven al usar el nombre anterior.").color(p.text_muted)]).size(texto::BODY_SM.0)].spacing(12)).padding([14,16]).width(Length::Fill)]).height(88).style(estilo::bloque(p.surface_sunken)),
    ].spacing(28);
    if let Some(e) = &s.error {
        cuerpo = cuerpo.push(c::aviso_detallado(
            p,
            c::Estado::Error,
            "No se pudo guardar la cuenta",
            e,
        ));
    }
    if let Some(n) = &s.exito {
        cuerpo = cuerpo.push(c::aviso(
            p,
            c::Estado::Exito,
            &format!("{n} está en uso. La barra lateral ya lo muestra."),
        ));
    }
    let cabecera = row![
        c::etiqueta(p, "Cuentas guardadas"),
        Space::with_width(Length::Fill),
        text(format!("{} cuentas", s.cuentas.nombres.len()))
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted)
    ];
    let mut lista = column![].spacing(0);
    if s.cuentas.nombres.is_empty() {
        lista = lista.push(
            row![
                cuenta::cabeza_vacia(p, 32.),
                column![
                    text("Todavía no tienes cuentas sin conexión")
                        .size(texto::BODY_SM.0)
                        .color(p.text),
                    text("La que guardes aparecerá aquí y en el selector de cuentas.")
                        .size(texto::BODY_SM.0)
                        .color(p.text_muted)
                ]
            ]
            .spacing(12),
        );
    }
    for nombre in &s.cuentas.nombres {
        let activa = app
            .servicios
            .session
            .as_ref()
            .is_some_and(|session| session.es_local() && session.profile.name == *nombre);
        let confirmando = s.confirmar.as_ref() == Some(nombre);
        let mut fila = if confirmando {
            row![
                text(format!(
                    "Quitar {nombre} del launcher. Tus mundos e instancias no se tocan."
                ))
                .size(texto::BODY_SM.0)
                .color(p.text)
                .width(Length::Fill),
                c::boton_compacto(
                    p,
                    "Cancelar",
                    c::Variante::Secundario,
                    habilitado.then(|| m(Accion::CancelarEliminar))
                ),
                c::boton_con_icono(
                    p,
                    Icono::Papelera,
                    "Quitar",
                    c::Variante::Peligro,
                    habilitado.then(|| m(Accion::Confirmar)),
                    true
                )
            ]
        } else {
            let mut fila = row![
                if let Some((imagen, _)) = s.imagenes.get(nombre) {
                    cuenta::cabeza_textura(p, &imagen.textura.rgba, 32., false)
                } else {
                    cuenta::cabeza_vacia(p, 32.)
                },
                column![
                    text(nombre.clone())
                        .font(fuentes::TITULO)
                        .size(14)
                        .color(p.text),
                    c::insignia_contorno(p, "Sin conexión")
                ]
                .spacing(4)
                .width(Length::Fill)
            ];
            fila = fila.push(if activa {
                row![
                    icono(Icono::Check, Tam::Sm, p.accent_text),
                    text("En uso").size(texto::BODY_SM.0).color(p.accent_text)
                ]
                .spacing(6)
                .into()
            } else {
                c::boton_compacto(
                    p,
                    "Usar",
                    c::Variante::Secundario,
                    habilitado.then(|| m(Accion::Usar(nombre.clone()))),
                )
            });
            fila = fila.push(c::boton_con_icono(
                p,
                Icono::Pincel,
                "Skin",
                c::Variante::Secundario,
                habilitado.then(|| {
                    m(Accion::Skin(crate::skin_sin_conexion::Accion::Abrir(
                        nombre.clone(),
                    )))
                }),
                true,
            ));
            fila.push(iced::widget::tooltip(
                cuenta::boton_icono(
                    p,
                    Some(Icono::Papelera),
                    "",
                    cuenta::Tipo::Fantasma,
                    28.,
                    habilitado.then(|| m(Accion::Eliminar(nombre.clone()))),
                ),
                text(format!("Eliminar {nombre}")).size(12),
                iced::widget::tooltip::Position::Top,
            ))
        };
        fila = fila.spacing(12).align_y(Alignment::Center);
        lista = lista.push(
            container(row![
                container(Space::new(3, Length::Fill)).style(estilo::bloque(if confirmando {
                    p.error
                } else if activa {
                    p.text
                } else {
                    p.bg
                })),
                container(fila).padding([10, 12]).width(Length::Fill)
            ])
            .height(68)
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(
                    if confirmando {
                        p.error_soft
                    } else if activa {
                        p.accent_soft
                    } else {
                        p.bg
                    }
                    .into(),
                ),
                ..Default::default()
            }),
        );
    }
    cuerpo = cuerpo.push(
        column![
            cabecera,
            Space::with_height(10),
            container(Space::new(Length::Fill, 2)).style(estilo::bloque(p.text)),
            lista
        ]
        .spacing(0),
    );
    let boton = c::boton_con_icono(
        p,
        if s.ocupado {
            Icono::Espera
        } else {
            Icono::Check
        },
        if s.ocupado {
            "Guardando…"
        } else {
            "Guardar y usar cuenta"
        },
        c::Variante::Primario,
        (habilitado && validacion.is_ok()).then(|| m(Accion::Guardar)),
        false,
    );
    let pie = row![
        Space::with_width(Length::Fill),
        c::boton(
            p,
            "Volver",
            c::Variante::Secundario,
            habilitado.then(|| m(Accion::Cerrar))
        ),
        boton
    ]
    .spacing(12)
    .align_y(Alignment::Center);
    container(column![
        container(
            row![
                text("Cuenta sin conexión")
                    .font(fuentes::TITULO)
                    .size(24)
                    .color(p.text)
                    .width(Length::Fill),
                cuenta::boton_icono(
                    p,
                    Some(Icono::Cerrar),
                    "",
                    cuenta::Tipo::Fantasma,
                    28.,
                    habilitado.then(|| m(Accion::Cerrar))
                )
            ]
            .align_y(Alignment::Center)
        )
        .padding([24, 32]),
        container(crate::desplazable_pub(
            iced::widget::mouse_area(cuerpo)
                .on_press(m(Accion::RevisarNombre))
                .into()
        ))
        .padding([0, 32])
        .height(Length::Fill),
        container(pie).padding([20, 32]),
    ])
    .width((app.ventana.width - 48.).clamp(320., 720.))
    .height((app.ventana.height - 80.).clamp(350., 760.))
    .style(estilo::flotante(p))
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn los_estados_del_dialogo_se_construyen_sin_desbordar_el_eje_desplazable() {
        let (mut app, _) = crate::iniciar_con_guardado(
            icaro_ui::shell::Seccion::Instancias,
            false,
            None,
            false,
            servicios::Estado::nuevo(true),
            None,
        );
        for escena in [
            "cuenta-local",
            "cuenta-local-invalido",
            "cuenta-local-vacio",
            "cuenta-local-guardando",
            "cuenta-local-error",
            "cuenta-local-eliminar",
        ] {
            preparar_diseno(&mut app, escena);
            let _ = crate::vista(&app);
        }
    }
    #[test]
    fn valida_el_nombre_sin_rechazar_la_primera_letra_y_detecta_duplicados() {
        assert_eq!(validar("Ju", &[], false), Err(String::new()));
        assert!(validar("Ju", &[], true).unwrap_err().contains("mínimo"));
        assert!(validar("Jugador_1", &[], false).is_ok());
        assert!(validar("jugador_1", &["Jugador_1".into()], true)
            .unwrap_err()
            .contains("Ya tienes"));
        assert!(validar("José Luis", &[], true)
            .unwrap_err()
            .contains("espacios"));
        assert!(validar("José", &[], true).unwrap_err().contains("acentos"));
        assert!(validar("Jugador-1", &[], true)
            .unwrap_err()
            .contains("Quita"));
        assert!(validar("abcdefghijklmnopq", &[], true)
            .unwrap_err()
            .contains("Máximo"));
    }

    #[test]
    fn guardar_bloquea_duplicados_y_errores_o_resultados_tardios_conservan_la_cuenta() {
        let (mut app, _) = crate::iniciar_con_guardado(
            icaro_ui::shell::Seccion::Instancias,
            false,
            None,
            false,
            servicios::Estado::nuevo(false),
            None,
        );
        app.servicios.cancel();
        app.servicios.session = Some(Arc::new(Session::local("Anterior").unwrap()));
        app.servicios.local.nombre = "NuevaCuenta".into();
        app.servicios.local.ocupado = true;
        app.servicios.auth_busy = true;
        let id = app.servicios.auth_id;
        let _ = actualizar(&mut app, Accion::Guardar);
        let _ = actualizar(&mut app, Accion::Nombre("NoDebeCambiar".into()));
        assert_eq!(app.servicios.local.nombre, "NuevaCuenta");
        let _ = actualizar(&mut app, Accion::Terminada(id + 1, Ok(Cuentas::default())));
        assert!(app.servicios.local.ocupado);
        let _ = actualizar(&mut app, Accion::Terminada(id, Err("Disco lleno".into())));
        assert_eq!(
            app.servicios.session.as_ref().unwrap().profile.name,
            "Anterior"
        );
        assert_eq!(app.servicios.local.nombre, "NuevaCuenta");
        assert!(!app.servicios.auth_busy && app.servicios.local.error.is_some());
        app.servicios.local.ocupado = true;
        let _ = actualizar(
            &mut app,
            Accion::Terminada(
                id,
                Ok(Cuentas {
                    nombres: vec!["NuevaCuenta".into()],
                    skins: Default::default(),
                    activa: Some("NuevaCuenta".into()),
                }),
            ),
        );
        assert_eq!(
            app.servicios.session.as_ref().unwrap().profile.name,
            "NuevaCuenta"
        );
        assert!(app.servicios.local.nombre.is_empty());
        assert_eq!(app.sesion.estado, cuenta::EstadoSesion::SinConexion);
        let _ = actualizar(&mut app, Accion::Terminada(id, Ok(Cuentas::default())));
        assert_eq!(
            app.servicios.session.as_ref().unwrap().profile.name,
            "NuevaCuenta"
        );
    }
}

pub fn preparar_diseno(app: &mut App, escena: &str) {
    app.servicios.local = Estado::default();
    let s = &mut app.servicios.local;
    s.abierta = true;
    s.cuentas.nombres = vec!["Jugador_1".into(), "Constructor_88".into()];
    s.cuentas.activa = Some("Jugador_1".into());
    app.servicios.session = Some(Arc::new(Session::local("Jugador_1").unwrap()));
    app.sesion.estado = cuenta::EstadoSesion::SinConexion;
    s.nombre = if escena.contains("invalido") {
        "José Luis"
    } else {
        "Piedra_Roja"
    }
    .into();
    s.estricto = escena.contains("invalido");
    if escena.contains("vacio") {
        s.cuentas = Cuentas::default();
        s.nombre.clear();
        app.servicios.session = None;
    }
    if escena.contains("guardando") {
        s.ocupado = true;
    }
    if escena.contains("error") {
        s.error=Some("No se pudo escribir el archivo de cuentas. El nombre sigue escrito; puedes reintentar.".into());
    }
    if escena.contains("eliminar") {
        s.confirmar = Some("Jugador_1".into());
    }
}
