// SPDX-License-Identifier: GPL-3.0-only
//! SincronizarMods original conectado al índice explícito del anfitrión.
use crate::{pantallas::Pantalla, App, Mensaje};
use icaro_core::{
    instancias::Instancia,
    mods::sincronizacion::{self as sync, Cambio, Plan, Tipo},
    servidores::Servidor,
};
use icaro_ui::{
    componentes::{self, Variante},
    fuentes,
    laminas::{grabado_posicion, Lamina},
};
use iced::futures::SinkExt;
use iced::{
    widget::{column, container, responsive, row, text, Space},
    Element, Length, Task,
};
use tokio_util::sync::CancellationToken;
#[derive(Default)]
pub struct Estado {
    request: u64,
    servidor: Option<Servidor>,
    instancia: Option<Instancia>,
    plan: Option<Plan>,
    cancel: Option<CancellationToken>,
    pub ocupado: bool,
    aplicando: bool,
    cancelar: bool,
    error: Option<String>,
    progreso: usize,
    completados: std::collections::BTreeSet<String>,
    archivo_actual: Option<String>,
    confirmar: bool,
    pub visual: bool,
    fixture: Vec<Cambio>,
}
impl Drop for Estado {
    fn drop(&mut self) {
        if let Some(c) = &self.cancel {
            c.cancel();
        }
    }
}
#[derive(Debug, Clone)]
pub enum Accion {
    Abrir(usize),
    Revisada(u64, Result<Plan, String>),
    Aplicar(bool),
    Progreso(u64, String, bool),
    Terminada(u64, bool, Result<(), String>),
    Reintentar,
    SinCambios,
    ConfirmarSinCambios,
    VolverRevision,
    Cerrar,
}
fn m(a: Accion) -> Mensaje {
    Mensaje::Sincronizacion(a)
}
impl Estado {
    pub fn modal(&self) -> bool {
        self.ocupado || self.confirmar
    }
}
fn tamano(bytes: u64) -> String {
    if bytes < 1_048_576 {
        format!("{} KB", bytes / 1024)
    } else {
        format!("{:.1} MB", bytes as f64 / 1_048_576.).replace('.', ",")
    }
}
fn vigente(app: &App) -> bool {
    app.servicios
        .sincronizacion
        .servidor
        .as_ref()
        .is_some_and(|s| {
            app.servicios
                .servidores
                .registros
                .iter()
                .any(|v| v.id == s.id && v.direccion == s.direccion && v.instancia == s.instancia)
        })
}
fn jugar(app: &mut App) -> Task<Mensaje> {
    if !vigente(app) {
        app.avisar("El servidor o la instancia vinculada cambiaron. Comprueba de nuevo.".into());
        return Task::none();
    }
    let s = &app.servicios.sincronizacion;
    let index = s.instancia.as_ref().and_then(|v| {
        app.servicios
            .instancias
            .registros
            .iter()
            .position(|i| i.id == v.id)
    });
    let address = s.servidor.as_ref().map(|v| v.direccion.clone());
    app.pantalla = None;
    index.map_or(Task::none(), |i| {
        crate::juego_real::iniciar_con_servidor(app, i, true, address)
    })
}
pub fn actualizar(app: &mut App, a: Accion) -> Task<Mensaje> {
    if app.servicios.demo {
        return Task::none();
    }
    match a {
        Accion::Abrir(n) => {
            if app.servicios.sincronizacion.ocupado {
                return Task::none();
            }
            let Some(server) = app.servicios.servidores.registros.get(n).cloned() else {
                return Task::none();
            };
            let Some(instance) = server
                .instancia
                .as_ref()
                .and_then(|id| {
                    app.servicios
                        .instancias
                        .registros
                        .iter()
                        .find(|i| &i.id == id)
                })
                .cloned()
            else {
                return Task::none();
            };
            if app.servicios.juego.ocupada(&instance.id)
                || app.servicios.mods.busy.is_some()
                || app.servicios.instancias.operando
            {
                app.avisar(
                    "Termina la operación o cierra Minecraft antes de comprobar los mods.".into(),
                );
                return Task::none();
            }
            let endpoint = sync::endpoint(&server.direccion);
            let source = app.servicios.catalog.clone();
            let store = app.servicios.instancias.almacen.clone();
            let s = &mut app.servicios.sincronizacion;
            if let Some(c) = s.cancel.take() {
                c.cancel();
            }
            s.request += 1;
            let g = s.request;
            s.plan = None;
            s.error = None;
            s.confirmar = false;
            s.cancelar = false;
            s.progreso = 0;
            s.completados.clear();
            s.archivo_actual = None;
            s.servidor = Some(server);
            s.instancia = Some(instance.clone());
            s.ocupado = true;
            let cancel = CancellationToken::new();
            s.cancel = Some(cancel.clone());
            app.pantalla = Some(Pantalla::Sincronizar);
            app.menu = None;
            return Task::perform(
                async move {
                    let addr = endpoint?;
                    let catalog = source.map_err(|e| e.to_string())?;
                    let index = sync::consultar(addr, cancel.clone()).await?;
                    tokio::select! {_=cancel.cancelled()=>Err("Consulta cancelada.".into()),r=tokio::time::timeout(std::time::Duration::from_secs(90),sync::preparar(&catalog,&store,&instance,index,addr.to_string()))=>r.map_err(|_|"Modrinth no respondió a tiempo. Inténtalo de nuevo.")?}
                },
                move |r| m(Accion::Revisada(g, r)),
            );
        }
        Accion::Revisada(g, r) => {
            let s = &mut app.servicios.sincronizacion;
            if g != s.request || app.pantalla != Some(Pantalla::Sincronizar) {
                return Task::none();
            }
            s.ocupado = false;
            match r {
                Ok(plan) => s.plan = Some(plan),
                Err(e) => s.error = Some(e),
            }
            if s.plan.as_ref().is_some_and(|p| p.cambios.is_empty()) {
                return actualizar(app, Accion::Aplicar(true));
            }
        }
        Accion::Aplicar(play) => {
            if app.pantalla != Some(Pantalla::Sincronizar) {
                return Task::none();
            }
            if !vigente(app) {
                return Task::none();
            }
            let s = &app.servicios.sincronizacion;
            let (Some(plan), Some(instance), Some(server)) =
                (s.plan.clone(), s.instancia.clone(), s.servidor.clone())
            else {
                return Task::none();
            };
            if s.ocupado
                || app.servicios.juego.ocupada(&instance.id)
                || app.servicios.mods.busy.is_some()
                || app.servicios.instancias.operando
            {
                return Task::none();
            }
            let endpoint = sync::endpoint(&server.direccion);
            let store = app.servicios.instancias.almacen.clone();
            let source = app.servicios.catalog.clone();
            let s = &mut app.servicios.sincronizacion;
            s.ocupado = true;
            s.aplicando = true;
            s.error = None;
            s.progreso = 0;
            s.completados.clear();
            s.archivo_actual = None;
            s.cancelar = false;
            let cancel = CancellationToken::new();
            s.cancel = Some(cancel.clone());
            let g = s.request;
            app.servicios.mods.busy = Some(instance.id.clone());
            app.servicios.instancias.operando = true;
            return Task::run(
                iced::stream::channel(16, move |mut output| async move {
                    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
                    let work = async {
                        let addr = endpoint?;
                        let catalog = source.map_err(|e| e.to_string())?;
                        let latest = sync::consultar(addr, cancel.clone()).await?;
                        if latest.revision()? != plan.indice.revision()? {
                            return Err("Los mods del anfitrión cambiaron. Comprueba de nuevo antes de sincronizar.".into());
                        }
                        sync::aplicar(
                            &catalog,
                            &store,
                            &instance,
                            &plan,
                            cancel,
                            move |id, done| {
                                let _ = tx.send((id.to_owned(), done));
                            },
                        )
                        .await
                    };
                    tokio::pin!(work);
                    loop {
                        tokio::select! {Some((id,done))=rx.recv()=>{let _=output.send(m(Accion::Progreso(g,id,done))).await;},r=&mut work=>{let _=output.send(m(Accion::Terminada(g,play,r))).await;break;}}
                    }
                }),
                |v| v,
            );
        }
        Accion::Progreso(g, id, done) => {
            let s = &mut app.servicios.sincronizacion;
            if s.request == g && s.aplicando {
                if done {
                    s.completados.insert(id);
                    s.progreso = s.completados.len();
                    s.archivo_actual = None;
                } else {
                    s.archivo_actual = Some(id);
                }
            }
        }
        Accion::Terminada(g, play, r) => {
            let s = &mut app.servicios.sincronizacion;
            if s.request != g || !s.aplicando {
                return Task::none();
            }
            s.aplicando = false;
            s.ocupado = false;
            app.servicios.mods.busy = None;
            app.servicios.instancias.operando = false;
            if s.cancelar {
                app.pantalla = None;
                s.plan = None;
                return Task::none();
            }
            match r {
                Ok(()) => {
                    s.plan = None;
                    app.avisar("Mods sincronizados con el anfitrión.".into());
                    if play && app.pantalla == Some(Pantalla::Sincronizar) {
                        return jugar(app);
                    }
                    app.pantalla = None;
                }
                Err(e) => {
                    s.error = Some(e);
                    s.plan = None;
                }
            }
        }
        Accion::Reintentar => {
            if app.servicios.sincronizacion.ocupado {
                return Task::none();
            }
            let n = app
                .servicios
                .sincronizacion
                .servidor
                .as_ref()
                .and_then(|s| {
                    app.servicios
                        .servidores
                        .registros
                        .iter()
                        .position(|v| v.id == s.id)
                });
            if let Some(n) = n {
                return actualizar(app, Accion::Abrir(n));
            }
        }
        Accion::SinCambios => {
            if !app.servicios.sincronizacion.ocupado {
                app.servicios.sincronizacion.confirmar = true;
            }
        }
        Accion::ConfirmarSinCambios => {
            if !app.servicios.sincronizacion.ocupado && app.servicios.sincronizacion.confirmar {
                app.servicios.sincronizacion.confirmar = false;
                return jugar(app);
            }
        }
        Accion::VolverRevision => app.servicios.sincronizacion.confirmar = false,
        Accion::Cerrar => {
            let s = &mut app.servicios.sincronizacion;
            if s.confirmar {
                s.confirmar = false;
                return Task::none();
            }
            if let Some(c) = s.cancel.take() {
                c.cancel();
            }
            if s.aplicando {
                s.cancelar = true;
                return Task::none();
            }
            s.request += 1;
            s.ocupado = false;
            s.plan = None;
            app.pantalla = None;
        }
    }
    Task::none()
}
pub fn vista(app: &App) -> Element<'_, Mensaje> {
    let p = app.paleta();
    let s = &app.servicios.sincronizacion;
    let name = s
        .servidor
        .as_ref()
        .map(|v| v.nombre.as_str())
        .unwrap_or("Amigos");
    let changes = s
        .plan
        .as_ref()
        .map(|v| v.cambios.as_slice())
        .unwrap_or(&s.fixture);
    let count = |t: Tipo| changes.iter().filter(|c| c.tipo == t).count();
    let bytes = changes.iter().map(|c| c.bytes).sum::<u64>();
    let mut body = column![
        text(format!("Sincronizar\ncon {name}"))
            .font(fuentes::DISPLAY)
            .size(32)
            .line_height(iced::widget::text::LineHeight::Absolute(34.into())),
        text(format!(
            "{} por agregar · {} por actualizar · {} por quitar · {} por descargar",
            count(Tipo::Agregar),
            count(Tipo::Actualizar),
            count(Tipo::Quitar),
            tamano(bytes)
        ))
        .size(13)
        .color(p.text_muted)
    ]
    .spacing(20);
    if s.ocupado {
        body = body.push(
            text(if s.cancelar {
                "Cancelando; se conservarán los mods anteriores…"
            } else if s.aplicando {
                "Preparando el conjunto de mods…"
            } else {
                "Consultando el índice del anfitrión y verificando Modrinth…"
            })
            .size(13)
            .color(p.text_muted),
        );
        body = body.push(if s.aplicando {
            icaro_ui::progreso::continuo(
                p,
                s.progreso as f32
                    / s.plan.as_ref().map_or(1, |p| {
                        (p.indice.mods.len()
                            + p.cambios.iter().filter(|c| c.tipo == Tipo::Quitar).count())
                        .max(1)
                    }) as f32,
                app.reducir,
            )
        } else {
            icaro_ui::progreso::indeterminado(p, app.reducir)
        });
    }
    body = body.push(
        container(Space::with_height(2))
            .width(Length::Fill)
            .style(icaro_ui::estilo::bloque(p.text)),
    );
    for c in changes {
        let (mark, color, label) = match c.tipo {
            Tipo::Agregar => ("+", p.success, "Agregar"),
            Tipo::Actualizar => ("~", p.info, "Actualizar / activar"),
            Tipo::Quitar => ("−", p.error, "Quitar: el anfitrión ya no lo usa"),
        };
        let version = if c.antes.is_empty() {
            c.despues.clone()
        } else if c.despues.is_empty() {
            c.antes.clone()
        } else {
            format!("{} → {}", c.antes, c.despues)
        };
        let mut line = column![
            row![
                text(mark)
                    .font(fuentes::MONO)
                    .size(18)
                    .color(color)
                    .width(24),
                column![
                    text(c.nombre.clone())
                        .font(fuentes::CUERPO_NEGRITA)
                        .size(14),
                    text(label).size(12).color(p.text_muted)
                ]
                .width(Length::Fill)
                .spacing(4),
                text(version)
                    .font(fuentes::MONO)
                    .size(12)
                    .width(if app.ventana.width < 1200. { 100 } else { 150 })
                    .color(p.text_muted),
                text(if c.bytes == 0 {
                    String::new()
                } else {
                    tamano(c.bytes)
                })
                .font(fuentes::MONO)
                .size(12)
                .width(if app.ventana.width < 1200. { 60 } else { 72 })
                .color(p.text_muted)
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
            container(Space::with_height(1))
                .width(Length::Fill)
                .style(icaro_ui::estilo::bloque(p.border))
        ]
        .spacing(12);
        if s.aplicando
            && (s.completados.contains(&c.proyecto)
                || s.archivo_actual.as_ref() == Some(&c.proyecto))
        {
            line = line.push(if s.completados.contains(&c.proyecto) {
                icaro_ui::progreso::continuo(p, 1., app.reducir)
            } else {
                icaro_ui::progreso::indeterminado(p, app.reducir)
            });
        }
        body = body.push(line);
    }
    if let Some(plan) = &s.plan {
        for aviso in &plan.indice.avisos {
            body=body.push(text(aviso.clone()).size(13).color(p.text_muted));
        }
        body = body.push(
            text(format!(
                "Se mantienen {} mods locales o de cliente: {}.",
                plan.conservados.len(),
                plan.conservados.join(", ")
            ))
            .size(13)
            .color(p.text_muted),
        );
    }
    if let Some(e) = &s.error {
        body = body
            .push(text(e.clone()).size(13).color(p.error))
            .push(componentes::boton(
                p,
                "Comprobar de nuevo",
                Variante::Secundario,
                (!s.ocupado).then(|| m(Accion::Reintentar)),
            ));
    }
    let enabled = !s.ocupado && (s.plan.is_some() || s.visual);
    let footer = container(responsive(move |size| {
        let first = componentes::boton(
            p,
            "Sincronizar y jugar",
            Variante::Primario,
            enabled.then(|| m(Accion::Aplicar(true))),
        );
        let second = componentes::boton(
            p,
            "Solo sincronizar",
            Variante::Secundario,
            enabled.then(|| m(Accion::Aplicar(false))),
        );
        let third = componentes::boton(
            p,
            "Jugar sin sincronizar",
            Variante::Fantasma,
            (!s.ocupado).then(|| m(Accion::SinCambios)),
        );
        if size.width < 560. {
            column![first, second, third].spacing(8).into()
        } else {
            row![first, second, third].spacing(8).into()
        }
    }))
    .height(if app.ventana.width < 1200. { 124 } else { 40 });
    let picture = container(grabado_posicion(
        p,
        Lamina::Hidra,
        Length::Fill,
        Length::Fill,
        (0.5, 0.2),
    ))
    .width(258)
    .height(Length::Fill);
    let border = container(Space::with_width(2))
        .height(Length::Fill)
        .style(icaro_ui::estilo::bloque(p.text));
    let right = container(column![crate::desplazable_pub(body.into()), footer].spacing(20))
        .padding(40)
        .width(Length::Fill)
        .height(Length::Fill);
    let view = row![picture, border, right].height(Length::Fill).into();
    if s.confirmar {
        let dialog=icaro_ui::superficies::dialogo(p,"¿Jugar sin sincronizar?",text("Puede faltar un mod o tener una versión distinta. Minecraft podría rechazar la conexión al servidor.").size(14).into(),vec![componentes::boton(p,"Cancelar",Variante::Secundario,Some(m(Accion::VolverRevision))),componentes::boton(p,"Jugar sin sincronizar",Variante::Primario,Some(m(Accion::ConfirmarSinCambios)))],None,560.,250.,m(Accion::VolverRevision));
        icaro_ui::superficies::con_velo(p, view, dialog, m(Accion::VolverRevision))
    } else {
        view
    }
}
pub fn preparar_diseno(app: &mut App, scene: &str) {
    crate::servidores_reales::preparar_diseno(app, "servidores");
    if scene.ends_with("angosto") {
        app.ventana = iced::Size::new(940., 720.);
    }
    app.pantalla = Some(Pantalla::Sincronizar);
    let s = &mut app.servicios.sincronizacion;
    s.visual = true;
    s.servidor = app.servicios.servidores.registros.first().cloned();
    s.fixture = vec![
        Cambio {
            proyecto: "create".into(),
            tipo: Tipo::Agregar,
            nombre: "Create".into(),
            antes: String::new(),
            despues: "0.5.1j".into(),
            bytes: 18_200_000,
        },
        Cambio {
            proyecto: "lithium".into(),
            tipo: Tipo::Actualizar,
            nombre: "Lithium".into(),
            antes: "0.14.3".into(),
            despues: "0.14.5".into(),
            bytes: 720_000,
        },
        Cambio {
            proyecto: "bettercombat".into(),
            tipo: Tipo::Quitar,
            nombre: "Better Combat".into(),
            antes: "1.8.6".into(),
            despues: String::new(),
            bytes: 0,
        },
    ];
    if scene.contains("error") {
        s.error = Some("El anfitrión todavía no tiene un índice de mods disponible.".into());
    }
    if scene.contains("confirmar") {
        s.confirmar = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app() -> App {
        let (mut a, _) = crate::iniciar_con_guardado(
            icaro_ui::shell::Seccion::Servidores,
            true,
            None,
            false,
            crate::servicios::Estado::nuevo(true),
            None,
        );
        preparar_diseno(&mut a, "sincronizacion-revision");
        a.servicios.demo = false;
        a
    }
    #[test]
    fn cerrar_consulta_descarta_respuestas_y_no_inicia_juego() {
        let mut a = app();
        a.servicios.sincronizacion.request = 7;
        a.servicios.sincronizacion.ocupado = true;
        let token = CancellationToken::new();
        a.servicios.sincronizacion.cancel = Some(token.clone());
        let _ = actualizar(&mut a, Accion::Cerrar);
        assert!(token.is_cancelled());
        assert!(a.pantalla.is_none());
        assert!(!a.servicios.sincronizacion.ocupado);
        let _ = actualizar(&mut a, Accion::Revisada(7, Err("antiguo".into())));
        assert!(a.servicios.sincronizacion.error.is_none());
        assert!(a.servicios.juego.procesos.is_empty());
    }
    #[test]
    fn cancelar_aplicacion_espera_acuse_y_libera_la_instancia() {
        let mut a = app();
        let s = &mut a.servicios.sincronizacion;
        s.request = 4;
        s.ocupado = true;
        s.aplicando = true;
        s.cancel = Some(CancellationToken::new());
        a.servicios.mods.busy = Some("instancia".into());
        a.servicios.instancias.operando = true;
        let _ = actualizar(&mut a, Accion::Cerrar);
        assert!(a.servicios.sincronizacion.ocupado);
        assert!(a.servicios.instancias.operando);
        let _ = actualizar(&mut a, Accion::Terminada(3, true, Ok(())));
        assert!(a.servicios.instancias.operando);
        let _ = actualizar(&mut a, Accion::Terminada(4, true, Err("cancelada".into())));
        assert!(!a.servicios.instancias.operando);
        assert!(a.servicios.mods.busy.is_none());
        assert!(a.pantalla.is_none());
        assert!(a.servicios.juego.procesos.is_empty());
    }
}
