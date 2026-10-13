// SPDX-License-Identifier: GPL-3.0-only
//! Lista real de servidores con formularios y consultas independientes de la maqueta.
use crate::{App, Mensaje};
use icaro_core::{
    servidores::{self, Servidor},
    zerotier,
};
use icaro_ui::{
    componentes::{self, Variante},
    fuentes,
    iconos::Icono,
    laminas::{banda_completa, DatoBanda, Lamina, ALTO_BANDA},
    servidores::{AccionesLista, EstadoServidor, VistaFila},
};
use iced::{
    widget::{column, container, pick_list, row, text, text_input, Space},
    Alignment, Element, Length, Task,
};
use std::collections::BTreeMap;
use tokio_util::sync::CancellationToken;

#[derive(Default)]
pub struct Estado {
    loaded: bool,
    loading: bool,
    saving: bool,
    revision: u64,
    request: u64,
    registros: Vec<Servidor>,
    estados: BTreeMap<String, Result<servidores::Estado, String>>,
    cancel: Option<CancellationToken>,
    error: Option<String>,
    editor: Option<Editor>,
    eliminar: Option<String>,
    pub visual: bool,
    zero: Option<zerotier::Estado>,
}
impl Drop for Estado {
    fn drop(&mut self) {
        if let Some(c) = &self.cancel {
            c.cancel();
        }
    }
}
impl Estado {
    pub fn guardados(&self) -> &[Servidor] {
        &self.registros
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instancia {
    id: String,
    nombre: String,
}
impl std::fmt::Display for Instancia {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.nombre)
    }
}
struct Editor {
    id: Option<String>,
    nombre: String,
    direccion: String,
    instancia: Option<Instancia>,
    error: Option<String>,
}
#[derive(Clone, Debug)]
pub enum Accion {
    Abrir,
    Cargados(Result<Vec<Servidor>, String>),
    Consultar,
    Estado(u64, String, Result<servidores::Estado, String>),
    Zero(u64, zerotier::Estado),
    Nuevo,
    Editar(usize),
    Nombre(String),
    Direccion(String),
    Instancia(Instancia),
    Guardar,
    Guardados(u64, Result<Vec<Servidor>, String>),
    Cerrar,
    Eliminar(usize),
    ConfirmarEliminar,
    CancelarEliminar,
    Jugar(usize),
    Subir(usize),
    Bajar(usize),
}
fn m(a: Accion) -> Mensaje {
    Mensaje::Servidores(a)
}
fn persistir(app: &mut App, v: Vec<Servidor>) -> Task<Mensaje> {
    let s = &mut app.servicios.servidores;
    if s.saving || s.error.is_some() {
        return Task::none();
    }
    s.saving = true;
    s.revision += 1;
    let g = s.revision;
    let root = crate::estado::directorio(false);
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                servidores::guardar(&root, &v)?;
                Ok(v)
            })
            .await
            .map_err(|_| "No se pudo guardar la lista.".to_string())
            .and_then(|r| r)
        },
        move |r| m(Accion::Guardados(g, r)),
    )
}
pub fn actualizar(app: &mut App, a: Accion) -> Task<Mensaje> {
    if app.servicios.demo {
        return Task::none();
    }
    if app.servicios.servidores.saving
        && !matches!(
            a,
            Accion::Guardados(..) | Accion::Estado(..) | Accion::Zero(..)
        )
    {
        return Task::none();
    }
    match a {
        Accion::Abrir => {
            let s = &mut app.servicios.servidores;
            if s.loading {
                return Task::none();
            }
            if s.loaded && s.error.is_none() {
                return actualizar(app, Accion::Consultar);
            }
            s.loading = true;
            let root = crate::estado::directorio(false);
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || servidores::cargar(&root))
                        .await
                        .map_err(|_| "No se pudo leer la lista.".to_string())
                        .and_then(|r| r)
                },
                |r| m(Accion::Cargados(r)),
            );
        }
        Accion::Cargados(r) => {
            let s = &mut app.servicios.servidores;
            s.loading = false;
            s.loaded = true;
            match r {
                Ok(v) => {
                    s.registros = v;
                    s.error = None;
                }
                Err(e) => {
                    s.error = Some(e);
                }
            }
            return actualizar(app, Accion::Consultar);
        }
        Accion::Consultar => {
            let s = &mut app.servicios.servidores;
            if s.loading || !s.loaded {
                return Task::none();
            }
            if let Some(c) = s.cancel.take() {
                c.cancel();
            }
            s.request += 1;
            let g = s.request;
            let c = CancellationToken::new();
            s.cancel = Some(c.clone());
            s.estados.clear();
            s.zero = None;
            let slots = std::sync::Arc::new(tokio::sync::Semaphore::new(4));
            let mut tasks = vec![Task::perform(zerotier::detectar(), move |z| {
                m(Accion::Zero(g, z))
            })];
            for server in s.registros.clone() {
                let c = c.clone();
                let slots = slots.clone();
                tasks.push(Task::perform(async move{
                    tokio::select!{
                        _=c.cancelled()=>return Err("Consulta cancelada.".into()),
                        permit=slots.acquire_owned()=>{let _permit=permit.map_err(|_|"Consulta cerrada.")?;servidores::consultar(&server.direccion,c).await}
                    }
                },move|r|m(Accion::Estado(g,server.id.clone(),r))));
            }
            return Task::batch(tasks);
        }
        Accion::Estado(g, id, r) => {
            let s = &mut app.servicios.servidores;
            if g == s.request && s.registros.iter().any(|v| v.id == id) {
                s.estados.insert(id, r);
            }
        }
        Accion::Zero(g, z) => {
            let s = &mut app.servicios.servidores;
            if g == s.request {
                s.zero = Some(z);
            }
        }
        Accion::Nuevo => {
            if app.servicios.servidores.error.is_some() {
                return Task::none();
            }
            app.servicios.servidores.editor = Some(Editor {
                id: None,
                nombre: String::new(),
                direccion: String::new(),
                instancia: None,
                error: None,
            });
        }
        Accion::Editar(i) => {
            if let Some(v) = app.servicios.servidores.registros.get(i) {
                let instancia = v
                    .instancia
                    .as_ref()
                    .and_then(|id| {
                        app.servicios
                            .instancias
                            .registros
                            .iter()
                            .find(|i| &i.id == id)
                    })
                    .map(|i| Instancia {
                        id: i.id.clone(),
                        nombre: i.nombre.clone(),
                    });
                app.servicios.servidores.editor = Some(Editor {
                    id: Some(v.id.clone()),
                    nombre: v.nombre.clone(),
                    direccion: v.direccion.clone(),
                    instancia,
                    error: None,
                });
            }
        }
        Accion::Nombre(v) => {
            if let Some(e) = &mut app.servicios.servidores.editor {
                e.nombre = v;
            }
        }
        Accion::Direccion(v) => {
            if let Some(e) = &mut app.servicios.servidores.editor {
                e.direccion = v;
            }
        }
        Accion::Instancia(v) => {
            if let Some(e) = &mut app.servicios.servidores.editor {
                e.instancia = Some(v);
            }
        }
        Accion::Cerrar => app.servicios.servidores.editor = None,
        Accion::Guardar => {
            let Some(e) = &mut app.servicios.servidores.editor else {
                return Task::none();
            };
            match servidores::nuevo(
                e.nombre.clone(),
                e.direccion.clone(),
                e.instancia.as_ref().map(|i| i.id.clone()),
            ) {
                Err(error) => e.error = Some(error),
                Ok(mut v) => {
                    let mut lista = app.servicios.servidores.registros.clone();
                    if let Some(id) = &e.id {
                        v.id = id.clone();
                        let Some(old) = lista.iter_mut().find(|v| &v.id == id) else {
                            return Task::none();
                        };
                        *old = v;
                    } else {
                        lista.push(v);
                    }
                    return persistir(app, lista);
                }
            }
        }
        Accion::Guardados(g, r) => {
            let s = &mut app.servicios.servidores;
            if g != s.revision {
                return Task::none();
            }
            s.saving = false;
            match r {
                Ok(v) => {
                    s.registros = v;
                    s.editor = None;
                    s.eliminar = None;
                    app.avisar("Lista de servidores guardada.".into());
                    return actualizar(app, Accion::Consultar);
                }
                Err(e) => {
                    if let Some(editor) = &mut s.editor {
                        editor.error = Some(e);
                    } else {
                        app.avisar(format!("No se pudo guardar: {e}"));
                    }
                }
            }
        }
        Accion::Eliminar(i) => {
            app.servicios.servidores.eliminar = app
                .servicios
                .servidores
                .registros
                .get(i)
                .map(|v| v.id.clone());
        }
        Accion::CancelarEliminar => app.servicios.servidores.eliminar = None,
        Accion::ConfirmarEliminar => {
            let s = &app.servicios.servidores;
            let Some(id) = &s.eliminar else {
                return Task::none();
            };
            let lista = s
                .registros
                .iter()
                .filter(|v| &v.id != id)
                .cloned()
                .collect();
            return persistir(app, lista);
        }
        Accion::Subir(i) | Accion::Bajar(i) => {
            let up = matches!(a, Accion::Subir(_));
            let mut v = app.servicios.servidores.registros.clone();
            let target = if up {
                i.checked_sub(1)
            } else {
                i.checked_add(1)
            };
            if let Some(t) = target.filter(|t| i < v.len() && *t < v.len()) {
                v.swap(i, t);
                return persistir(app, v);
            }
        }
        Accion::Jugar(i) => {
            let Some(v) = app.servicios.servidores.registros.get(i).cloned() else {
                return Task::none();
            };
            let Some(index) = v.instancia.as_ref().and_then(|id| {
                app.servicios
                    .instancias
                    .registros
                    .iter()
                    .position(|i| &i.id == id)
            }) else {
                app.avisar("Vincula una instancia al servidor antes de jugar.".into());
                return actualizar(app, Accion::Editar(i));
            };
            if let Some(Ok(status)) = app.servicios.servidores.estados.get(&v.id) {
                let mc = &app.servicios.instancias.registros[index].minecraft;
                // Solo identifica nombres de versiones exactos; proxies pueden publicar etiquetas propias.
                if status.version != *mc
                    && app
                        .servicios
                        .instancias
                        .versiones
                        .iter()
                        .any(|version| version.id == status.version)
                {
                    app.avisar(format!("El servidor anuncia Minecraft {} y la instancia usa {mc}. Cambia la versión o vincula otra instancia.",status.version));
                    return Task::none();
                }
            }
            return crate::juego_real::iniciar_con_servidor(app, index, true, Some(v.direccion));
        }
    }
    Task::none()
}
pub fn vista(app: &App) -> Element<'_, Mensaje> {
    let p = app.paleta();
    let s = &app.servicios.servidores;
    let button = |label: &str, a: Accion| {
        componentes::boton(p, label, Variante::Secundario, (!s.saving).then(|| m(a)))
    };
    let mut body = column![row![
        button(
            if s.error.is_some() {
                "Releer lista"
            } else {
                "Actualizar estado"
            },
            if s.error.is_some() {
                Accion::Abrir
            } else {
                Accion::Consultar
            }
        ),
        Space::with_width(Length::Fill),
        componentes::boton_con_icono(
            p,
            Icono::Mas,
            "Agregar servidor",
            Variante::Primario,
            (!s.saving && s.loaded && s.error.is_none()).then(|| m(Accion::Nuevo)),
            false
        )
    ]
    .spacing(12)
    .align_y(Alignment::Center)]
    .spacing(24);
    let z=match &s.zero{
        None=>"Comprobando ZeroTier…".into(),
        Some(zerotier::Estado::Ausente)=>"ZeroTier no está instalado. Puedes conectar a servidores directos.".into(),
        Some(zerotier::Estado::SinAcceso)=>"ZeroTier detectado; no se pudo consultar el servicio. Comprueba que esté activo y que tu usuario tenga acceso.".into(),
        Some(zerotier::Estado::Disponible{version,online,redes})=>format!("ZeroTier {version} · {} · {} redes",if *online{"En línea"}else{"Sin conexión"},redes.len()),
    };
    if s.editor.is_none() {
        body = body.push(
            container(text(z).size(13).color(p.text_muted))
                .width(Length::Fill)
                .padding(16)
                .style(icaro_ui::estilo::marco(p)),
        );
        if let Some(zerotier::Estado::Disponible { redes, .. }) = &s.zero {
            for red in redes {
                let status = match red.estado.as_str() {
                    "OK" => "Autorizada",
                    "ACCESS_DENIED" => "Esperando autorización",
                    "REQUESTING_CONFIGURATION" => "Solicitando configuración",
                    _ => &red.estado,
                };
                body = body.push(
                    text(format!(
                        "{} · {} · {} · {}",
                        if red.nombre.is_empty() {
                            &red.id
                        } else {
                            &red.nombre
                        },
                        red.id,
                        status,
                        if red.ips.is_empty() {
                            "Sin IP asignada".into()
                        } else {
                            red.ips.join(", ")
                        }
                    ))
                    .font(fuentes::MONO)
                    .size(12)
                    .color(p.text_muted),
                );
            }
        }
    }
    if let Some(error) = &s.error {
        body = body.push(text(error.clone()).color(p.error));
    }
    if let Some(e) = &s.editor {
        let instances: Vec<_> = app
            .servicios
            .instancias
            .registros
            .iter()
            .map(|i| Instancia {
                id: i.id.clone(),
                nombre: i.nombre.clone(),
            })
            .collect();
        let field = |placeholder: &str, value: &str, f: fn(String) -> Accion| {
            text_input(placeholder, value)
                .on_input(move |v| m(f(v)))
                .padding(12)
                .style(icaro_ui::estilo::campo(p))
        };
        let mut form = column![
            text(if e.id.is_some() {
                "Editar servidor"
            } else {
                "Agregar servidor"
            })
            .font(fuentes::TITULO)
            .size(24),
            componentes::etiqueta(p, "Nombre"),
            field("Amigos", &e.nombre, Accion::Nombre),
            componentes::etiqueta(p, "Dirección"),
            field("play.example.net:25565", &e.direccion, Accion::Direccion),
            componentes::etiqueta(p, "Instancia"),
            pick_list(instances, e.instancia.clone(), |v| m(Accion::Instancia(v)))
                .placeholder("Selecciona una instancia")
                .padding(12)
                .width(Length::Fill)
                .style(icaro_ui::estilo::selector(p))
                .menu_style(icaro_ui::estilo::menu_selector(p)),
            text("El servidor se abrirá con la instancia vinculada y sus ajustes de Java.")
                .size(13)
                .color(p.text_muted)
        ]
        .spacing(12);
        if let Some(error) = &e.error {
            form = form.push(text(error.clone()).color(p.error));
        }
        form = form.push(
            row![
                button("Cancelar", Accion::Cerrar),
                componentes::boton(
                    p,
                    if s.saving {
                        "Guardando…"
                    } else {
                        "Guardar servidor"
                    },
                    Variante::Primario,
                    (!s.saving).then(|| m(Accion::Guardar))
                )
            ]
            .spacing(12),
        );
        body = body.push(
            container(form)
                .padding(24)
                .width(Length::Fill)
                .style(icaro_ui::estilo::marco(p)),
        );
    }
    if let Some(id) = &s.eliminar {
        let name = s
            .registros
            .iter()
            .find(|v| &v.id == id)
            .map_or("servidor", |v| v.nombre.as_str());
        body = body.push(
            container(
                column![
                    text(format!("¿Quitar {name} de la lista?"))
                        .font(fuentes::CUERPO_NEGRITA)
                        .size(18),
                    text("Se conserva la instancia y sus archivos.")
                        .size(13)
                        .color(p.text_muted),
                    row![
                        button("Cancelar", Accion::CancelarEliminar),
                        button("Quitar servidor", Accion::ConfirmarEliminar)
                    ]
                    .spacing(12)
                ]
                .spacing(12),
            )
            .padding(20)
            .style(icaro_ui::estilo::marco(p)),
        );
    }
    if s.loading {
        body = body.push(text("Cargando servidores…").color(p.text_muted));
    } else if s.registros.is_empty() && s.error.is_none() {
        body = body.push(
            text("Ningún servidor guardado. Agrega uno para vincularlo a una instancia y jugar.")
                .size(16)
                .color(p.text_muted),
        );
    }
    let acciones = AccionesLista {
        jugar: |i| m(Accion::Jugar(i)),
        tomar: None,
        sobre: |_| Mensaje::Nada,
        editar: Some(|i| m(Accion::Editar(i))),
        eliminar: Some(|i| m(Accion::Eliminar(i))),
        permitir_sin_estado: true,
    };
    for (i, v) in s.registros.iter().enumerate() {
        let estado = s.estados.get(&v.id);
        let name = v
            .instancia
            .as_ref()
            .and_then(|id| {
                app.servicios
                    .instancias
                    .registros
                    .iter()
                    .find(|i| &i.id == id)
            })
            .map(|i| i.nombre.clone());
        let fila = icaro_ui::servidores::Servidor {
            nombre: v.nombre.clone(),
            direccion: v.direccion.clone(),
            motd: estado.and_then(|e| e.as_ref().ok()).map(|e| e.motd.clone()),
            estado: match estado {
                Some(Ok(e)) => EstadoServidor::EnLinea {
                    jugadores: e.jugadores,
                    maximo: e.maximo,
                    ms: e.ms.min(u32::MAX as u64) as u32,
                },
                Some(Err(_)) => EstadoServidor::FueraDeLinea("Sin respuesta".into()),
                None => EstadoServidor::FueraDeLinea("Consultando…".into()),
            },
            vinculo: name.map(|n| (n, false)),
            version_distinta: None,
        };
        let mut section = column![icaro_ui::servidores::tarjeta_servidor(
            p,
            i,
            &fila,
            &acciones,
            VistaFila::default()
        )]
        .spacing(8);
        let detail = match estado {
            Some(Ok(e)) => format!("Minecraft {} · {}", e.version, e.motd.replace('\n', " ")),
            Some(Err(e)) => e.clone(),
            None => String::new(),
        };
        section = section.push(
            row![
                text(detail)
                    .size(12)
                    .color(p.text_muted)
                    .width(Length::Fill),
                componentes::boton(
                    p,
                    "Subir",
                    Variante::Fantasma,
                    (!s.saving && i > 0).then(|| m(Accion::Subir(i)))
                ),
                componentes::boton(
                    p,
                    "Bajar",
                    Variante::Fantasma,
                    (!s.saving && i + 1 < s.registros.len()).then(|| m(Accion::Bajar(i)))
                )
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        );
        body = body.push(section);
    }
    column![
        banda_completa(
            p,
            Lamina::TivoliCiudad,
            Some((0.5, 0.4)),
            "Servidores",
            DatoBanda::Conteo(s.registros.len()),
            ALTO_BANDA,
            icaro_ui::tema::texto::DISPLAY_XL
        ),
        crate::suave(app, body.padding(32))
    ]
    .into()
}
pub fn preparar_diseno(app: &mut App, scene: &str) {
    if scene.ends_with("angosto") {
        app.ventana = iced::Size::new(940., 720.);
    }
    app.seccion = icaro_ui::shell::Seccion::Servidores;
    app.editor = None;
    let i = icaro_core::instancias::Instancia {
        id: "qa-servidores".into(),
        nombre: "Supervivencia".into(),
        minecraft: "26.3".into(),
        loader: "fabric".into(),
        portada: 0,
        creada: 0,
    };
    app.servicios.instancias.registros = vec![i.clone()];
    let s = &mut app.servicios.servidores;
    s.visual = true;
    s.loaded = true;
    if scene == "servidores-vacio" {
        s.zero = Some(zerotier::Estado::Ausente);
        return;
    }
    let a = servidores::nuevo(
        "Amigos".into(),
        "10.147.17.1:25565".into(),
        Some(i.id.clone()),
    )
    .unwrap();
    let b = servidores::nuevo(
        "Creativo de la U".into(),
        "play.example.net:25570".into(),
        None,
    )
    .unwrap();
    s.estados.insert(
        a.id.clone(),
        Ok(servidores::Estado {
            version: "26.3".into(),
            protocolo: 999,
            jugadores: 3,
            maximo: 20,
            motd: "Temporada 4: el nether está abierto".into(),
            ms: 24,
        }),
    );
    s.estados.insert(
        b.id.clone(),
        Err("No se pudo conectar; revisa la dirección, la red o ZeroTier.".into()),
    );
    s.registros = vec![a, b];
    s.zero = Some(zerotier::Estado::Disponible {
        version: "1.16.0".into(),
        online: true,
        redes: vec![zerotier::Red {
            id: "1234567890abcdef".into(),
            nombre: "Amigos".into(),
            estado: "ACCESS_DENIED".into(),
            ips: vec![],
        }],
    });
    if scene.starts_with("servidores-formulario") {
        s.editor = Some(Editor {
            id: None,
            nombre: "Servidor con amigos".into(),
            direccion: "play.example.net".into(),
            instancia: Some(Instancia {
                id: i.id,
                nombre: i.nombre,
            }),
            error: Some("Ese servidor ya está guardado.".into()),
        });
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
        preparar_diseno(&mut a, "servidores");
        a.servicios.demo = false;
        a
    }
    #[test]
    fn respuestas_antiguas_y_servidores_eliminados_no_regresan_a_la_lista() {
        let mut a = app();
        let id = a.servicios.servidores.registros[0].id.clone();
        a.servicios.servidores.request = 7;
        let _ = actualizar(&mut a, Accion::Estado(6, id.clone(), Err("antiguo".into())));
        assert!(a.servicios.servidores.estados[&id].is_ok());
        a.servicios.servidores.registros.remove(0);
        a.servicios.servidores.estados.remove(&id);
        let _ = actualizar(
            &mut a,
            Accion::Estado(7, id.clone(), Err("eliminado".into())),
        );
        assert!(!a.servicios.servidores.estados.contains_key(&id));
    }
    #[test]
    fn fallo_de_guardado_conserva_la_lista_y_no_cierra_el_editor() {
        let mut a = app();
        let previous = a.servicios.servidores.registros.clone();
        let _ = actualizar(&mut a, Accion::Editar(0));
        a.servicios.servidores.saving = true;
        a.servicios.servidores.revision = 4;
        let _ = actualizar(&mut a, Accion::Guardados(4, Err("Disco ocupado".into())));
        assert_eq!(a.servicios.servidores.registros, previous);
        assert!(a
            .servicios
            .servidores
            .editor
            .as_ref()
            .unwrap()
            .error
            .is_some());
        assert!(!a.servicios.servidores.saving);
    }
    #[test]
    fn la_maqueta_no_guarda_servidores_ni_lanza_el_juego() {
        let mut a = app();
        a.servicios.demo = true;
        let _ = actualizar(&mut a, Accion::Jugar(0));
        assert!(a.servicios.juego.trabajos.is_empty());
        let _ = actualizar(&mut a, Accion::Nuevo);
        assert!(a.servicios.servidores.editor.is_none());
    }
}
