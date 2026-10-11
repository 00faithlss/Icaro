// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Conecta el asistente del diseño con el almacén local y las versiones de Mojang.

use crate::{pantallas, App, Mensaje};
use icaro_core::{
    instancias::{Almacen, Instancia},
    manifest::{self, VersionManifest},
};
use icaro_ui::instancias::{DatosInstancia, EstadoInstancia};
use iced::Task;

pub struct Estado {
    pub almacen: Almacen,
    pub registros: Vec<Instancia>,
    pub versiones: Vec<manifest::VersionEntry>,
    pub cargando: bool,
    pub creando: bool,
    pub error: Option<String>,
    pub eliminar: Option<String>,
    pub operando: bool,
    pub deshacer: Option<Eliminada>,
    request: u64,
}
impl Default for Estado {
    fn default() -> Self {
        Self {
            almacen: Almacen::nuevo(crate::estado::directorio(false).join("instances")),
            registros: vec![],
            versiones: vec![],
            cargando: false,
            creando: false,
            error: None,
            eliminar: None,
            operando: false,
            deshacer: None,
            request: 0,
        }
    }
}

pub struct Eliminada {
    registro: Instancia,
    grupo: Option<String>,
    config: icaro_ui::editor::ConfigJava,
}
impl Estado {
    pub fn cargar(&mut self) {
        match self.almacen.cargar() {
            Ok(report) => {
                self.registros = report.instancias;
                if report.omitidas > 0 {
                    self.error = Some(format!(
                        "No se pudieron leer {} instancias. Sus archivos se conservan en disco.",
                        report.omitidas
                    ));
                }
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }
    pub fn datos(store:&Almacen,i: &Instancia) -> DatosInstancia {
        let ready=icaro_core::minecraft::listo(store,i);
        DatosInstancia {
            nombre: i.nombre.clone(),
            version: format!("{} · {}", i.minecraft, crate::ficha::loader(&i.loader)),
            ultima_vez: "Sin jugar".into(),
            mods: "Sin mods".into(),
            portada: pantallas::PORTADAS[i.portada as usize].1,
            estado: if ready {EstadoInstancia::Lista}else{EstadoInstancia::Pendiente},
            seleccionada: false,
            detalle: if ready {"Lista para jugar"}else{"Minecraft pendiente de instalación"}.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Accion {
    Versiones(u64, Result<VersionManifest, String>),
    Creada(Result<Instancia, String>),
    Eliminada(String, Result<(), String>),
    Restaurada(String, Result<Instancia, String>),
}

pub fn eliminar(app: &mut App) -> Task<Mensaje> {
    if app.servicios.mods.busy.is_some() { app.avisar("Espera a que termine la operación de mods antes de eliminar una instancia.".into()); return Task::none(); }
    let s = &mut app.servicios.instancias;
    if s.operando {
        return Task::none();
    }
    let Some(id) = s.eliminar.clone() else {
        return Task::none();
    };
    if app.servicios.juego.ocupada(&id){app.avisar("Detén el juego o su instalación antes de eliminar esta instancia.".into());return Task::none();}
    if !s.registros.iter().any(|i| i.id == id) {
        return Task::none();
    }
    s.operando = true;
    let store = s.almacen.clone();
    let target = id.clone();
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || store.eliminar(&target))
                .await
                .map_err(|_| "No se pudo eliminar la instancia.".to_owned())?
                .map_err(|e| e.to_string())
        },
        move |result| Mensaje::Instancias(Accion::Eliminada(id.clone(), result)),
    )
}

pub fn deshacer(app: &mut App) -> Task<Mensaje> {
    let s = &mut app.servicios.instancias;
    if s.operando {
        return Task::none();
    }
    let Some(deleted) = &s.deshacer else {
        return Task::none();
    };
    let id = deleted.registro.id.clone();
    let target = id.clone();
    let store = s.almacen.clone();
    s.operando = true;
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || store.restaurar(&target))
                .await
                .map_err(|_| "No se pudo recuperar la instancia.".to_owned())?
                .map_err(|e| e.to_string())
        },
        move |result| Mensaje::Instancias(Accion::Restaurada(id.clone(), result)),
    )
}

pub fn versiones(app: &App, tipo: usize) -> Vec<(String, String)> {
    if app.servicios.demo {
        return pantallas::VERSIONES[tipo.min(2)]
            .iter()
            .map(|(v, d)| ((*v).into(), (*d).into()))
            .collect();
    }
    app.servicios
        .instancias
        .versiones
        .iter()
        .filter(|v| match tipo {
            0 => v.kind == "release",
            1 => v.kind == "snapshot",
            _ => matches!(v.kind.as_str(), "old_alpha" | "old_beta"),
        })
        .map(|v| (v.id.clone(), crate::ficha::fecha(&v.release_time)))
        .collect()
}
pub fn elegida(app: &App) -> Option<String> {
    let s = app.asistente.as_ref()?;
    versiones(app, s.tipo_version)
        .get(s.version)
        .map(|v| v.0.clone())
}
pub fn abrir(app: &mut App) -> Task<Mensaje> {
    let s = &mut app.servicios.instancias;
    if !s.versiones.is_empty() || s.cargando {
        return Task::none();
    }
    s.request += 1;
    let request = s.request;
    s.cargando = true;
    s.error = None;
    let cache = crate::estado::directorio(false).join("cache/minecraft-versions.json");
    Task::perform(
        async move {
            manifest::fetch_manifest_cached(&cache)
                .await
                .map_err(|e| e.to_string())
        },
        move |r| Mensaje::Instancias(Accion::Versiones(request, r)),
    )
}
pub fn crear(app: &mut App) -> Task<Mensaje> {
    if app.servicios.instancias.creando {
        return Task::none();
    }
    let Some(version) = elegida(app) else {
        app.avisar("Selecciona una versión de Minecraft.".into());
        return Task::none();
    };
    let Some(s) = app.asistente.as_ref() else {
        return Task::none();
    };
    if s.paso != 3 {
        return Task::none();
    }
    let nombre = if s.nombre.trim().is_empty() {
        format!("{} {version}", pantallas::LOADERS[s.loader.min(3)].0)
    } else {
        s.nombre.trim().into()
    };
    let portada = s.portada.min(3) as u8;
    let loader = pantallas::LOADERS[s.loader.min(3)].0.to_lowercase();
    let store = app.servicios.instancias.almacen.clone();
    app.servicios.instancias.creando = true;
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || store.crear_con_loader(&nombre, &version, &loader, portada))
                .await
                .map_err(|_| "No se pudo crear la instancia.".to_owned())?
                .map_err(|e| e.to_string())
        },
        |r| Mensaje::Instancias(Accion::Creada(r)),
    )
}
pub fn actualizar(app: &mut App, action: Accion) -> Task<Mensaje> {
    match action {
        Accion::Eliminada(id, result) => {
            app.servicios.instancias.operando = false;
            match result {
                Ok(()) => {
                    let Some(index) = app
                        .servicios
                        .instancias
                        .registros
                        .iter()
                        .position(|i| i.id == id)
                    else {
                        return Task::none();
                    };
                    let record = app.servicios.instancias.registros.remove(index);
                    let name = record.nombre.clone();
                    crate::lista_instancias().lock().unwrap().remove(index);
                    app.servicios.instancias.deshacer = Some(Eliminada {
                        registro: record,
                        grupo: app.grupos.remove(index),
                        config: app.configs.remove(index),
                    });
                    app.seleccionada = app.seleccionada.and_then(|i| {
                        if i == index {
                            None
                        } else {
                            Some(i - usize::from(i > index))
                        }
                    });
                    if app.editor_inst == index {
                        app.editor = None;
                    }
                    app.editor_inst = app
                        .editor_inst
                        .saturating_sub(usize::from(app.editor_inst > index));
                    app.servicios.instancias.eliminar = None;
                    app.dialogo = false;
                    app.menu = None;
                    app.busqueda.abierta = false;
                    app.picar = None;
                    app.colocada = None;
                    app.avisar(format!("{name} se movió a la papelera."));
                    crate::arrancar(app);
                }
                Err(error) => {
                    app.picar = None;
                    app.avisar(error);
                }
            }
        }
        Accion::Restaurada(id, result) => {
            app.servicios.instancias.operando = false;
            match result {
                Ok(record) => {
                    if !app
                        .servicios
                        .instancias
                        .deshacer
                        .as_ref()
                        .is_some_and(|i| i.registro.id == id)
                    {
                        return Task::none();
                    }
                    let previous = app.servicios.instancias.deshacer.take().unwrap();
                    let index = app
                        .servicios
                        .instancias
                        .registros
                        .partition_point(|i| (i.creada, &i.id) < (record.creada, &record.id));
                    if let Some(i) = &mut app.seleccionada {
                        *i += usize::from(*i >= index);
                    }
                    if app.editor.is_some() {
                        app.editor_inst += usize::from(app.editor_inst >= index);
                    }
                    crate::lista_instancias()
                        .lock()
                        .unwrap()
                        .insert(index, Estado::datos(&app.servicios.instancias.almacen,&record));
                    let name = record.nombre.clone();
                    app.servicios.instancias.registros.insert(index, record);
                    app.grupos.insert(index, previous.grupo);
                    app.configs.insert(index, previous.config);
                    app.menu = None;
                    app.busqueda.abierta = false;
                    app.colocada = Some((index, std::time::Instant::now()));
                    app.avisar(format!("Instancia {name} recuperada."));
                    crate::arrancar(app);
                }
                Err(error) => app.avisar(error),
            }
        }
        Accion::Versiones(id, result) if id == app.servicios.instancias.request => {
            app.servicios.instancias.cargando = false;
            match result {
                Ok(m) => {
                    app.servicios.instancias.versiones = m.versions;
                    app.servicios.instancias.error = None;
                    if let Some(s) = &mut app.asistente {
                        s.version = 0;
                    }
                }
                Err(e) => app.servicios.instancias.error = Some(e),
            }
        }
        Accion::Creada(result) => {
            app.servicios.instancias.creando = false;
            match result {
                Ok(i) => {
                    let name = i.nombre.clone();
                    crate::lista_instancias()
                        .lock()
                        .unwrap()
                        .push(Estado::datos(&app.servicios.instancias.almacen,&i));
                    app.servicios.instancias.registros.push(i);
                    app.grupos.push(None);
                    app.configs.push(crate::config_por_defecto());
                    app.asistente = None;
                    app.editor = None;
                    app.seccion = icaro_ui::shell::Seccion::Instancias;
                    app.avisar(format!("Instancia {name} creada y guardada."));
                    crate::arrancar(app);
                    let index=app.servicios.instancias.registros.len()-1;
                    if app.servicios.instancias.registros[index].loader=="vanilla"{return crate::juego_real::iniciar(app,index,false);}
                }
                Err(e) => app.avisar(e),
            }
        }
        _ => {}
    }
    Task::none()
}
pub fn renombrar(app: &mut App, nombre: &str) -> Result<(), String> {
    let Some(record) = app.servicios.instancias.registros.get(app.editor_inst) else {
        return Ok(());
    };
    if record.nombre == nombre.trim() {
        return Ok(());
    }
    let mut record = record.clone();
    record.nombre = nombre.trim().into();
    app.servicios
        .instancias
        .almacen
        .guardar(&record)
        .map_err(|e| e.to_string())?;
    if let Some(d) = crate::lista_instancias()
        .lock()
        .unwrap()
        .get_mut(app.editor_inst)
    {
        d.nombre = record.nombre.clone();
    }
    app.servicios.instancias.registros[app.editor_inst] = record;
    Ok(())
}
