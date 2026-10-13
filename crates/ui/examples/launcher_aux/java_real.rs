// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

use crate::{App, Mensaje};
use icaro_core::{auth::CancellationToken, java};
use iced::Task;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferencias {
    pub ruta: Option<String>,
    pub descarga: bool,
    pub memoria_min: u32,
    pub memoria_max: u32,
    pub metaspace: u32,
    pub argumentos: String,
}
impl Default for Preferencias {
    fn default() -> Self {
        Self {
            ruta: None,
            descarga: true,
            memoria_min: 2048, memoria_max: 6144, metaspace: 512, argumentos: "-XX:+UseG1GC".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Accion {
    Detectar,
    Detectadas(u64, Result<Vec<java::Java>, String>),
    Heredar(bool),
    HeredarMemoria(bool),
    MemoriaGlobal(u8,String),
    DescargaInstancia(bool),
    DescargaGlobal(bool),
    RutaGlobal(String),
    Agregar(bool),
    Agregada(Option<String>, Option<String>),
    Probar(String),
    Probada(u64, Result<(java::Java, bool), String>),
}

pub fn efectiva(
    config: &icaro_ui::editor::ConfigJava,
    global: &Preferencias,
) -> (Option<PathBuf>, bool) {
    if config.heredar {
        (global.ruta.as_ref().map(PathBuf::from), global.descarga)
    } else {
        (config.java.as_ref().map(PathBuf::from), config.descarga)
    }
}

pub fn detectar(app: &mut App) -> Task<Mensaje> {
    if app.servicios.demo {
        return Task::none();
    }
    app.java_generacion += 1;
    let generation = app.java_generacion;
    app.java_estado = "Comprobando instalaciones de Java…".into();
    let base = crate::estado::directorio(false);
    let mut manual: Vec<_> = app.configs.iter().filter_map(|c| c.java.clone()).collect();
    manual.extend(app.java_global.ruta.clone());
    Task::perform(
        async move {
            let mut local = java::detectar(CancellationToken::new())
                .await
                .map_err(|e| e.to_string())?;
            let managed = tokio::task::spawn_blocking(move || java::gestionados(&base))
                .await
                .map_err(|e| e.to_string())?;
            local.extend(managed);
            for path in manual {
                if let Ok(java) =
                    java::comprobar(&PathBuf::from(path), 0, CancellationToken::new()).await
                {
                    if !local.iter().any(|j| j.executable == java.executable) {
                        local.push(java);
                    }
                }
            }
            Ok(local)
        },
        move |result| Mensaje::Java(Accion::Detectadas(generation, result)),
    )
}

pub fn actualizar(app: &mut App, action: Accion) -> Task<Mensaje> {
    match action {
        Accion::Detectar => return detectar(app),
        Accion::Detectadas(generation, result) => {
            if generation == app.java_generacion {
                match result {
                    Ok(java) => {
                        app.java_estado = format!("{} instalaciones encontradas.", java.len());
                        app.java_detectadas = java;
                    }
                    Err(error) => {
                        app.java_estado = error.clone();
                        app.avisar_error(error);
                    }
                }
            }
        }
        Accion::HeredarMemoria(value) => app.configs[app.editor_inst].heredar_memoria=value,
        Accion::MemoriaGlobal(field,value) => {
            let g=&mut app.java_global;
            if field==3 {g.argumentos=value;}else if let Ok(v)=value.parse::<u32>() {
                match field {0=>{g.memoria_min=v;g.memoria_max=g.memoria_max.max(v);},1=>{g.memoria_max=v;g.memoria_min=g.memoria_min.min(v);},_=>g.metaspace=v}
            }
        }
        Accion::Heredar(value) => app.configs[app.editor_inst].heredar = value,
        Accion::DescargaInstancia(value) => {
            app.configs[app.editor_inst].descarga = value;
            app.configs[app.editor_inst].heredar = false;
        }
        Accion::DescargaGlobal(value) => app.java_global.descarga = value,
        Accion::RutaGlobal(value) => {
            app.java_global.ruta = (!value.trim().is_empty()).then_some(value)
        }
        Accion::Agregar(global) => {
            // Asociar el resultado a la identidad original, aunque cambie el editor.
            let target = if global {
                None
            } else {
                app.servicios
                    .instancias
                    .registros
                    .get(app.editor_inst)
                    .map(|i| i.id.clone())
            };
            if !global && target.is_none() {
                return Task::none();
            }
            return Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Seleccionar ejecutable Java")
                        .pick_file()
                        .await
                        .map(|file| file.path().to_string_lossy().into_owned())
                },
                move |path| Mensaje::Java(Accion::Agregada(target.clone(), path)),
            );
        }
        Accion::Agregada(target, Some(path)) => {
            if let Some(id) = &target {
                if let Some(index) = app
                    .servicios
                    .instancias
                    .registros
                    .iter()
                    .position(|i| &i.id == id)
                {
                    app.configs[index].java = Some(path.clone());
                    app.configs[index].java_requiere_seleccion=false;
                    app.configs[index].heredar = false;
                }
            } else {
                app.java_global.ruta = Some(path.clone());
            }
            return probar(app, path, target);
        }
        Accion::Agregada(_, None) => {}
        Accion::Probar(path) => {
            let target = (app.editor == Some(1))
                .then(|| {
                    app.servicios
                        .instancias
                        .registros
                        .get(app.editor_inst)
                        .map(|i| i.id.clone())
                })
                .flatten();
            return probar(app, path, target);
        }
        Accion::Probada(generation, result) => {
            if generation != app.java_prueba_generacion {
                return Task::none();
            }
            match result {
                Ok((mut java, compatible)) => {
                    java.managed = app
                        .java_detectadas
                        .iter()
                        .any(|j| j.executable == java.executable && j.managed);
                    app.java_estado = format!(
                        "Prueba de memoria y argumentos correcta · {} · Java {} · {} · {} · {}",
                        if compatible {
                            "compatible con el perfil"
                        } else {
                            "compatibilidad del perfil pendiente"
                        },
                        java.version,
                        java.arquitectura,
                        if java.managed { "Ícaro" } else { "Sistema" },
                        icaro_ui::editor::ruta_java_visible(&java.executable.to_string_lossy())
                    );
                    app.avisar(app.java_estado.clone());
                    if !app
                        .java_detectadas
                        .iter()
                        .any(|j| j.executable == java.executable)
                    {
                        app.java_detectadas.push(java);
                    }
                }
                Err(error) => {
                    app.java_estado = error.clone();
                    app.avisar_error(error);
                }
            }
        }
    }
    Task::none()
}

fn probar(app: &mut App, path: String, target: Option<String>) -> Task<Mensaje> {
    let instance = target
        .as_ref()
        .and_then(|id| {
            app.servicios
                .instancias
                .registros
                .iter()
                .find(|i| &i.id == id)
        })
        .cloned();
    if target.is_some() && instance.is_none() {
        return Task::none();
    }
    let config = target
        .as_ref()
        .map(|id| crate::juego_real::config(app, id).0)
        .unwrap_or_else(|| icaro_core::lanzamiento::Config {
            memoria_min_mb: 256,
            memoria_max_mb: 512,
            ..Default::default()
        });
    let prepared = target
        .as_ref()
        .and_then(|id| app.servicios.juego.preparada.get(id))
        .filter(|game| {
            instance
                .as_ref()
                .is_some_and(|i| i.minecraft == game.version)
        })
        .cloned();
    let store = app.servicios.instancias.almacen.clone();
    app.java_prueba_generacion += 1;
    let generation = app.java_prueba_generacion;
    app.java_estado = "Probando Java con memoria y argumentos configurados…".into();
    Task::perform(
        async move {
            let game = if let Some(prepared) = prepared {
                Some(prepared)
            } else if let Some(instance) = instance {
                tokio::task::spawn_blocking(move || icaro_core::minecraft::leer(&store, &instance))
                    .await
                    .map_err(|e| e.to_string())?
            } else {
                None
            };
            let profile = game.as_ref().map(|g| g.perfil_java());
            java::probar(
                &PathBuf::from(path),
                profile.as_ref(),
                &config,
                CancellationToken::new(),
            )
            .await
            .map(|java| (java, profile.is_some()))
            .map_err(|e| e.to_string())
        },
        move |result| Mensaje::Java(Accion::Probada(generation, result)),
    )
}

pub fn memoria_efectiva(config:&icaro_ui::editor::ConfigJava,global:&Preferencias)->icaro_ui::editor::ConfigJava {
    let mut c=config.clone();
    if c.heredar_memoria {c.memoria_min=global.memoria_min;c.memoria_max=global.memoria_max;c.metaspace=global.metaspace;c.argumentos=global.argumentos.clone();}
    c
}

pub fn visible_config(app: &App) -> icaro_ui::editor::ConfigJava {
    let mut config = memoria_efectiva(&app.configs[app.editor_inst], &app.java_global);
    if config.heredar {
        config.java = app.java_global.ruta.clone();
        config.descarga = app.java_global.descarga;
    }
    config
}

pub fn resumen(app: &App) -> String {
    let config = &app.configs[app.editor_inst];
    let (path, download) = efectiva(config, &app.java_global);
    let mode = if config.heredar {
        "Heredado de ajustes globales"
    } else {
        "Configuración de esta instancia"
    };
    let selection = path
        .as_ref()
        .map(|p| icaro_ui::editor::ruta_java_visible(&p.to_string_lossy()))
        .unwrap_or_else(|| {
            if download {
                "Automático: runtime administrado de Ícaro".into()
            } else {
                "Automático: instalaciones disponibles, sin descargar".into()
            }
        });
    let id = app
        .servicios
        .instancias
        .registros
        .get(app.editor_inst)
        .map(|i| &i.id);
    let game = id.and_then(|id| app.servicios.juego.preparada.get(id));
    let candidate = if let Some(path) = &path {
        app.java_detectadas.iter().find(|j| &j.executable == path)
    } else {
        game.and_then(|game| {
            app.java_detectadas
                .iter()
                .find(|j| game.perfil_java().acepta(j.major) && j.managed == download)
        })
    };
    let detail = candidate
        .map(|j| {
            let compatibility = game
                .map(|g| {
                    if g.perfil_java().acepta(j.major) {
                        "compatible".into()
                    } else {
                        format!(
                            "incompatible: el perfil acepta Java {:?}",
                            g.perfil_java().majors
                        )
                    }
                })
                .unwrap_or_else(|| "compatibilidad se comprobará al preparar Minecraft".into());
            format!(
                "Candidato detectado: Java {} · {} · {} · {} · {}",
                j.version,
                j.arquitectura,
                if j.managed { "Ícaro" } else { "Sistema" },
                icaro_ui::editor::ruta_java_visible(&j.executable.to_string_lossy()),
                compatibility
            )
        })
        .unwrap_or_else(|| {
            "La versión y compatibilidad se comprobarán al preparar Minecraft.".into()
        });
    let last = id
        .and_then(|id| app.servicios.juego.java.get(id))
        .map(|j| {
            format!(
                "Última preparación: Java {} · {} · {} · {}",
                j.version,
                j.arquitectura,
                if j.managed { "Ícaro" } else { "Sistema" },
                icaro_ui::editor::ruta_java_visible(&j.executable.to_string_lossy())
            )
        })
        .unwrap_or_default();
    format!(
        "{mode} · {selection}\nDescarga automática: {}\n{detail}\n{last}\n{}",
        if download { "permitida" } else { "desactivada" },
        app.java_estado
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn herencia_y_sustitucion_no_confunden_automatico_con_global() {
        let mut config = crate::config_por_defecto();
        let global = Preferencias {
            ruta: Some("global-java".into()),
            descarga: false,
            ..Default::default()
        };
        assert_eq!(
            efectiva(&config, &global),
            (Some(PathBuf::from("global-java")), false)
        );
        let mut memory_global=global.clone();memory_global.memoria_max=8192;
        assert_eq!(memoria_efectiva(&config,&memory_global).memoria_max,8192);
        config.heredar_memoria=false;config.memoria_max=3072;
        assert_eq!(memoria_efectiva(&config,&memory_global).memoria_max,3072);
        config.heredar = false;
        assert_eq!(efectiva(&config, &global), (None, true));
        config.java = Some("instance-java".into());
        assert_eq!(
            efectiva(&config, &global),
            (Some(PathBuf::from("instance-java")), true)
        );
    }
}
