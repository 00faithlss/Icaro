// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss
//! PantallaCuenta y EstudioSkin originales con perfil y texturas reales.
use crate::{pantallas, sesion, App, Mensaje};
use icaro_core::cuenta::{self, Perfil, Textura};
use icaro_ui::{
    componentes::{self, Variante},
    cuenta::{DatosCuenta, EstadoSesion, MensajesCuenta},
};
use iced::{
    widget::{column, text},
    Element, Task,
};
use std::{path::PathBuf, sync::Arc, time::Instant};

#[derive(Default)]
pub struct Estado {
    generacion: u64,
    pub actual: Option<Textura>,
    pub seleccion: Option<Textura>,
    perfil: Option<Perfil>,
    historial: Vec<(Textura, bool)>,
    cargando: bool,
    error: Option<String>,
}
impl Estado {
    pub fn invalidar(&mut self) {
        let next = self.generacion + 1;
        *self = Self::default();
        self.generacion = next;
    }
}
#[derive(Debug, Clone)]
pub enum Accion {
    Abrir(bool),
    Recibida(
        u64,
        String,
        Result<(Perfil, Option<Textura>, Vec<(Textura, bool)>), String>,
    ),
    ElegirArchivo,
    Archivo(Option<PathBuf>),
    Cargada(u64, Result<(Textura, String), String>),
    Elegir(usize),
    Aplicar(bool),
    Aplicada(u64, String, Textura, bool, Result<Perfil, String>),
}
fn m(a: Accion) -> Mensaje {
    Mensaje::CuentaReal(a)
}
fn historial(id: &str) -> PathBuf {
    assert!(id.len()==32 && id.bytes().all(|b|b.is_ascii_hexdigit()), "Identidad de cuenta validada por autenticación");
    crate::estado::directorio(false)
        .join("accounts")
        .join(id)
        .join("skins")
}
fn leer_historial(id: &str) -> Vec<(Textura, bool)> {
    let mut files = std::fs::read_dir(historial(id))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let hash = name.split('-').next()?;
            if hash.len() == 64
                && hash.bytes().all(|b| b.is_ascii_hexdigit())
                && (name.ends_with("-classic.png") || name.ends_with("-slim.png"))
            {
                Some((e.path(), e.metadata().ok()?.modified().ok()?))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    files.sort_by(|a, b| b.1.cmp(&a.1));
    files
        .into_iter()
        .take(8)
        .filter_map(|(p, _)| {
            cuenta::cargar(&p)
                .ok()
                .map(|t| (t, p.to_string_lossy().ends_with("-slim.png")))
        })
        .collect()
}
fn guardar(id: &str, t: &Textura, slim: bool) -> Result<(), String> {
    // Nombres derivados del contenido; la biblioteca contiene solo PNG, sin tokens.
    let png = t.png.clone();
    let dir = historial(id);
    std::fs::create_dir_all(&dir).map_err(|_| "No se pudo guardar el historial de skins.")?;
    let hash = cuenta::huella(&png);
    std::fs::write(
        dir.join(format!(
            "{hash}-{}.png",
            if slim { "slim" } else { "classic" }
        )),
        &png,
    )
    .map_err(|_| "No se pudo guardar el historial de skins.".into())
}
pub fn actualizar(app: &mut App, a: Accion) -> Task<Mensaje> {
    match a {
        Accion::Abrir(estudio) => {
            app.sesion.menu = false;
            if app.servicios.session.as_ref().is_some_and(|s| s.es_local()) {
                return crate::cuenta_sin_conexion::actualizar(app, crate::cuenta_sin_conexion::Accion::Abrir);
            }
            if estudio && app.servicios.session.is_none() {
                return crate::servicios::actualizar(app, crate::servicios::Accion::Login);
            }
            app.pantalla = Some(if estudio {
                pantallas::Pantalla::Estudio
            } else {
                pantallas::Pantalla::Cuenta
            });
            crate::arrancar(app);
            if app.sesion.subiendo.is_some() || app.sesion.cuenta.cargando {
                return Task::none();
            }
            let Some(session) = app.servicios.session.clone() else {
                return Task::none();
            };
            app.sesion.cuenta.generacion += 1;
            let generation = app.sesion.cuenta.generacion;
            app.sesion.cuenta.cargando = true;
            app.sesion.cuenta.error = None;
            let id = session.profile.id.clone();
            let response_id = id.clone();
            return Task::perform(
                async move {
                    let p = cuenta::perfil(&session).await.map_err(|e| e.to_string())?;
                    let texture = if let Some(skin) = p.skins.iter().find(|s| s.state == "ACTIVE") {
                        Some(
                            cuenta::descargar_skin(skin)
                                .await
                                .map_err(|e| e.to_string())?,
                        )
                    } else {
                        None
                    };
                    let history = tokio::task::spawn_blocking(move || leer_historial(&id))
                        .await
                        .map_err(|_| "No se pudo leer el historial de skins.")?;
                    Ok((p, texture, history))
                },
                move |r| m(Accion::Recibida(generation, response_id.clone(), r)),
            );
        }
        Accion::Recibida(g, id, r)
            if g == app.sesion.cuenta.generacion
                && app
                    .servicios
                    .session
                    .as_ref()
                    .is_some_and(|s| s.profile.id == id) =>
        {
            let s = &mut app.sesion.cuenta;
            s.cargando = false;
            match r {
                Ok((p, t, mut h)) => {
                    let slim = p
                        .skins
                        .iter()
                        .find(|s| s.state == "ACTIVE")
                        .is_some_and(|s| s.variant == "SLIM");
                    if let Some(t) = &t {
                        if !h
                            .iter()
                            .any(|(old, model)| old.png == t.png && *model == slim)
                        {
                            h.insert(0, (t.clone(), slim));
                            h.truncate(8);
                        }
                    }
                    s.actual = t;
                    s.perfil = Some(p);
                    s.historial = h;
                    app.sesion.modelo = usize::from(slim);
                    app.sesion.en_uso = 0;
                    app.sesion.elegida = 0;
                }
                Err(e) => s.error = Some(e),
            }
        }
        Accion::ElegirArchivo if app.sesion.subiendo.is_none() => {
            return Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Cargar skin PNG de 64 × 64")
                        .add_filter("Skin PNG", &["png"])
                        .pick_file()
                        .await
                        .map(|f| f.path().to_owned())
                },
                |p| m(Accion::Archivo(p)),
            );
        }
        Accion::Archivo(Some(path)) if app.sesion.subiendo.is_none() => {
            app.sesion.soltando = false;
            let generation = app.sesion.cuenta.generacion;
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        let texture = cuenta::cargar(&path).map_err(|e| e.to_string())?;
                        let name = path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned();
                        Ok((texture, name))
                    })
                    .await
                    .map_err(|_| "No se pudo abrir la skin.".to_owned())?
                },
                move |r| m(Accion::Cargada(generation, r)),
            );
        }
        Accion::Cargada(g, r)
            if g == app.sesion.cuenta.generacion && app.sesion.subiendo.is_none() =>
        {
            match r {
                Ok((t, n)) => {
                    app.sesion.cuenta.seleccion = Some(t);
                    app.sesion.textura = Some(n);
                    app.sesion.error = None;
                }
                Err(e) => app.sesion.error = Some(e),
            }
        }
        Accion::Elegir(i) if app.sesion.subiendo.is_none() => {
            if i < app.sesion.cuenta.historial.len() {
                app.sesion.elegida = i;
            }
        }
        Accion::Aplicar(historia) if app.sesion.subiendo.is_none() => {
            let Some(session) = app.servicios.session.clone() else {
                return Task::none();
            };
            let selected = if historia {
                app.sesion.cuenta.historial.get(app.sesion.elegida).cloned()
            } else {
                app.sesion
                    .cuenta
                    .seleccion
                    .clone()
                    .or_else(||app.sesion.cuenta.actual.clone())
                    .map(|t| (t, app.sesion.modelo == 1))
            };
            let Some((texture, slim)) = selected else {
                return Task::none();
            };
            app.sesion.subiendo = Some(Instant::now());
            app.ahora = Instant::now();
            app.sesion.error = None;
            let generation = app.sesion.cuenta.generacion;
            let id = session.profile.id.clone();
            let output = texture.clone();
            return Task::perform(
                async move {
                    cuenta::aplicar(&session, &texture, slim)
                        .await
                        .map_err(|e| e.to_string())
                },
                move |r| {
                    m(Accion::Aplicada(
                        generation,
                        id.clone(),
                        output.clone(),
                        slim,
                        r,
                    ))
                },
            );
        }
        Accion::Aplicada(g, id, t, slim, r)
            if g == app.sesion.cuenta.generacion
                && app
                    .servicios
                    .session
                    .as_ref()
                    .is_some_and(|s| s.profile.id == id) =>
        {
            app.sesion.subiendo = None;
            match r {
                Ok(p) => {
                    if let Some(actual)=&app.sesion.cuenta.actual {
                        let old_slim=app.sesion.cuenta.perfil.as_ref().is_some_and(|p|p.skins.iter().any(|s|s.state=="ACTIVE"&&s.variant=="SLIM"));
                        let _=guardar(&id,actual,old_slim);
                    }
                    if let Err(e) = guardar(&id, &t, slim) {
                        app.avisar(e);
                    }
                    let s = &mut app.sesion.cuenta;
                    s.actual = Some(t.clone());
                    s.perfil = Some(p);
                    s.seleccion = None;
                    s.historial
                        .retain(|(old, model)| old.png != t.png || *model != slim);
                    s.historial.insert(0, (t, slim));
                    s.historial.truncate(8);
                    app.sesion.elegida = 0;
                    app.sesion.en_uso = 0;
                    app.sesion.textura = None;
                    app.sesion.modelo = usize::from(slim);
                    app.avisar("Skin aplicada a tu cuenta de Minecraft.".into());
                }
                Err(e) => {
                    app.sesion.error = Some(e.clone());
                    app.sesion.cuenta.error = Some(e);
                }
            }
        }
        _ => {}
    }
    Task::none()
}
pub fn vista(app: &App) -> Element<'_, Mensaje> {
    if app.servicios.session.as_ref().is_some_and(|s| s.es_local()) {
        return column![
            text(app.servicios.nombre("Sin cuenta")).size(32).color(app.paleta().text),
            text("Sin conexión").color(app.paleta().text_muted),
            componentes::boton_con_icono(app.paleta(),icaro_ui::iconos::Icono::Usuario,"Gestionar cuenta sin conexión",Variante::Secundario,Some(crate::cuenta_sin_conexion::m(crate::cuenta_sin_conexion::Accion::Abrir)),false),
        ].spacing(20).into();
    }
    let s = &app.sesion;
    let real = &s.cuenta;
    let p = app.paleta();
    let names = vec!["skin"; real.historial.len()];
    let pixels = real
        .historial
        .iter()
        .map(|(t, _)| Arc::clone(&t.rgba))
        .collect::<Vec<_>>();
    let page = icaro_ui::cuenta::pantalla_cuenta(
        p,
        DatosCuenta {
            estado: if app.servicios.session.is_some() {
                EstadoSesion::Activa
            } else {
                EstadoSesion::SinConexion
            },
        nombre: real.perfil.as_ref().map_or_else(||app.servicios.nombre("Invitado"),|p|p.name.as_str()),
            correo: "Minecraft Java",
            proximo_cambio: "",
            historial: names,
            en_uso: s.en_uso,
            elegida: s.elegida,
            textura: real.actual.as_ref().map(|t| t.rgba.as_ref()),
            historial_real: pixels,
            delgado: s.modelo == 1,
        },
        MensajesCuenta {
            abrir_estudio: Mensaje::Ses(sesion::Accion::AbrirEstudio),
            elegir_skin: Box::new(|i| Mensaje::Ses(sesion::Accion::ElegirSkin(i))),
            usar_de_nuevo: Mensaje::Ses(sesion::Accion::UsarDeNuevo),
            cerrar_sesion: Mensaje::Ses(sesion::Accion::CerrarSesion),
            iniciar_sesion: Mensaje::Ses(sesion::Accion::IniciarSesion),
            volver_a_entrar: Mensaje::Ses(sesion::Accion::VolverAEntrar),
            comprar: Mensaje::Ses(sesion::Accion::Comprar),
            jugar_invitado: Mensaje::Ses(sesion::Accion::JugarComoInvitado),
            cambiar_nombre: Some(Mensaje::Servicio(crate::servicios::Accion::Enlace(
                "https://www.minecraft.net/msaprofile/mygames/editprofile".into(),
            ))),
        },
    );
    let mut view = column![].width(iced::Length::Fill);
    if real.cargando {
        view = view.push(text("Consultando tu perfil y skin de Minecraft…").color(p.text_muted));
    }
    if let Some(error) = &real.error {
        view = view
            .push(componentes::aviso(p, componentes::Estado::Aviso, error))
            .push(componentes::boton_compacto(
                p,
                "Reintentar",
                Variante::Secundario,
                Some(m(Accion::Abrir(false))),
            ));
    }
    view.push(page).into()
}
