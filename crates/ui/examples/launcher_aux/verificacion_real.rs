// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! ArchivosNoVerificados conectado al almacén real y preparado para importes.
use crate::{mods_reales, pantallas, App, Mensaje};
use icaro_core::{
    archivos_no_verificados::{self as verificacion, Archivo, Informe, Procedencia, Retirado},
    modrinth::Catalog,
};
use icaro_ui::{
    archivos_no_verificados as vista,
    componentes::{self, Variante},
    fuentes,
    iconos::Icono,
};
use iced::{
    widget::{column, container, row, text, Space},
    Element, Length, Task,
};
use std::sync::Arc;

#[derive(Default)]
pub struct Estado {
    pub informe: Informe,
    pub instancia: Option<String>,
    pub cargando: bool,
    pub operando: bool,
    pub error: Option<String>,
    pub fixture: bool,
    generation: u64,
    anterior: Option<pantallas::Pantalla>,
    retirado: Option<Arc<Retirado>>,
}
#[derive(Debug, Clone)]
pub enum Accion {
    Abrir,
    Revisar,
    Cerrar,
    Revisada(String, u64, Result<Informe, String>),
    Buscar(usize),
    Confiar(usize),
    Quitar(usize),
    Deshacer,
    Terminada(String, u64, Result<Operacion, String>),
    Enlace(Result<(), String>),
}
#[derive(Debug, Clone)]
pub enum Operacion {
    Confianza,
    Retirada(Arc<Retirado>),
    Restauracion,
}
pub fn m(a: Accion) -> Mensaje {
    Mensaje::Verificacion(a)
}

fn revisar(app: &mut App) -> Task<Mensaje> {
    if app.verificacion.fixture {
        return Task::none();
    }
    let Some(id) = app.verificacion.instancia.clone() else {
        return Task::none();
    };
    if app.verificacion.operando {
        return Task::none();
    }
    app.verificacion.generation += 1;
    let g = app.verificacion.generation;
    app.verificacion.cargando = true;
    app.verificacion.error = None;
    let store = app.servicios.instancias.almacen.clone();
    let result_id = id.clone();
    Task::perform(
        async move {
            let source = Catalog::new().map_err(|e| e.to_string())?;
            verificacion::revisar(&source, &store, &id)
                .await
                .map_err(|e| e.to_string())
        },
        move |r| m(Accion::Revisada(result_id.clone(), g, r)),
    )
}
fn archivo(app: &App, index: usize) -> Option<Archivo> {
    app.verificacion.informe.pendientes().nth(index).cloned()
}
fn disponible(app: &mut App) -> bool {
    if app.verificacion.fixture
        || app.verificacion.operando
        || app.verificacion.cargando
        || app.servicios.mods.busy.is_some()
        || app.servicios.instancias.operando
    {
        return false;
    }
    if app
        .verificacion
        .instancia
        .as_ref()
        .is_some_and(|id| app.servicios.juego.ocupada(id))
    {
        app.verificacion.error = Some("Cierra Minecraft y espera a que termine su instalación antes de modificar estos archivos.".into());
        return false;
    }
    true
}
pub fn actualizar(app: &mut App, accion: Accion) -> Task<Mensaje> {
    match accion {
        Accion::Abrir => {
            if app.verificacion.operando {
                return Task::none();
            }
            let Some(i) = mods_reales::instancia(app).cloned() else {
                app.avisar("Abre una instancia para revisar sus archivos locales.".into());
                return Task::none();
            };
            if app.pantalla != Some(pantallas::Pantalla::NoVerificados) {
                app.verificacion.anterior = app.pantalla;
            }
            if app.verificacion.instancia.as_deref() != Some(&i.id) {
                app.verificacion.informe = Informe::default();
                app.verificacion.retirado = None;
            }
            app.verificacion.instancia = Some(i.id);
            app.verificacion.fixture = false;
            app.pantalla = Some(pantallas::Pantalla::NoVerificados);
            app.mod_panel = None;
            return revisar(app);
        }
        Accion::Revisar => return revisar(app),
        Accion::Cerrar if !app.verificacion.operando => {
            app.verificacion.generation += 1;
            app.verificacion.cargando = false;
            app.pantalla = app.verificacion.anterior.take();
        }
        Accion::Revisada(id, g, result) => {
            if g != app.verificacion.generation
                || app.verificacion.instancia.as_deref() != Some(&id)
            {
                return Task::none();
            }
            app.verificacion.cargando = false;
            match result {
                Ok(informe) => {
                    app.verificacion.error = informe.error_red.clone();
                    app.verificacion.informe = informe;
                }
                Err(error) => app.verificacion.error = Some(error),
            }
        }
        Accion::Buscar(index) => {
            if let Some(a) = archivo(app, index) {
                // Modrinth sólo recibe el nombre de búsqueda, nunca un archivo ni su ruta local.
                return Task::perform(
                    async move {
                        verificacion::buscar_modrinth(&a.nombre)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    |r| m(Accion::Enlace(r)),
                );
            }
        }
        Accion::Confiar(index) | Accion::Quitar(index) => {
            let confianza = matches!(accion, Accion::Confiar(_));
            if !disponible(app) {
                return Task::none();
            }
            let Some(a) = archivo(app, index) else {
                return Task::none();
            };
            let Some(id) = app.verificacion.instancia.clone() else {
                return Task::none();
            };
            let store = app.servicios.instancias.almacen.clone();
            app.verificacion.generation += 1;
            let g = app.verificacion.generation;
            app.verificacion.operando = true;
            app.verificacion.error = None;
            app.servicios.mods.busy = Some(id.clone());
            let result_id = id.clone();
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        if confianza {
                            verificacion::confiar(&store, &id, &a).map(|_| Operacion::Confianza)
                        } else {
                            verificacion::retirar(&store, &id, &a)
                                .map(|t| Operacion::Retirada(Arc::new(t)))
                        }
                    })
                    .await
                    .map_err(|_| "No se pudo completar la revisión.".to_string())?
                    .map_err(|e| e.to_string())
                },
                move |r| m(Accion::Terminada(result_id.clone(), g, r)),
            );
        }
        Accion::Deshacer => {
            if !disponible(app) {
                return Task::none();
            }
            let Some(ticket) = app.verificacion.retirado.clone() else {
                return Task::none();
            };
            let Some(id) = app.verificacion.instancia.clone() else {
                return Task::none();
            };
            let store = app.servicios.instancias.almacen.clone();
            app.verificacion.generation += 1;
            let g = app.verificacion.generation;
            app.verificacion.operando = true;
            app.servicios.mods.busy = Some(id.clone());
            let result_id = id.clone();
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        verificacion::restaurar(&store, &id, &ticket)
                    })
                    .await
                    .map_err(|_| "No se pudo recuperar el archivo.".to_string())?
                    .map(|_| Operacion::Restauracion)
                    .map_err(|e| e.to_string())
                },
                move |r| m(Accion::Terminada(result_id.clone(), g, r)),
            );
        }
        Accion::Terminada(id, g, result) => {
            if g != app.verificacion.generation
                || app.verificacion.instancia.as_deref() != Some(&id)
            {
                return Task::none();
            }
            app.verificacion.operando = false;
            if app.servicios.mods.busy.as_deref() == Some(&id) {
                app.servicios.mods.busy = None;
            }
            match result {
                Ok(operation) => {
                    match operation {
                        Operacion::Confianza => app.avisar(
                            "Confianza guardada para el contenido exacto de este archivo.".into(),
                        ),
                        Operacion::Retirada(ticket) => {
                            app.verificacion.retirado = Some(ticket);
                            app.avisar("Archivo retirado. Puedes deshacer la acción.".into());
                        }
                        Operacion::Restauracion => {
                            app.verificacion.retirado = None;
                            app.avisar("Archivo recuperado.".into());
                        }
                    }
                    mods_reales::recargar(app, &id);
                    return revisar(app);
                }
                Err(error) => app.verificacion.error = Some(error),
            }
        }
        Accion::Enlace(Err(error)) => app.verificacion.error = Some(error),
        _ => {}
    }
    Task::none()
}
pub fn vista(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    let s = &app.verificacion;
    let ocupado = s.operando || s.cargando;
    let filas: Vec<_> = s
        .informe
        .pendientes()
        .map(|a| vista::Archivo {
            nombre: a.nombre.clone(),
            bytes: a.bytes,
            sha512: a.sha512.clone(),
            origen: None,
            evidencia_bloqueo: None,
            alternativa: false,
        })
        .collect();
    let barra = row![
        componentes::boton_con_icono(
            p,
            Icono::Atras,
            "Volver a la instancia",
            Variante::Fantasma,
            (!s.operando).then(|| m(Accion::Cerrar)),
            true
        ),
        Space::with_width(Length::Fill),
        componentes::boton_compacto(
            p,
            "Deshacer",
            Variante::Secundario,
            (!ocupado && s.retirado.is_some()).then(|| m(Accion::Deshacer))
        ),
        componentes::boton_compacto(
            p,
            "Revisar",
            Variante::Secundario,
            (!ocupado && !s.fixture).then(|| m(Accion::Revisar))
        )
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center);
    let cuerpo = vista::vista(p, &filas, ocupado || s.fixture, |a| {
        m(match a {
            vista::Accion::Buscar(i) => Accion::Buscar(i),
            vista::Accion::Confiar(i) => Accion::Confiar(i),
            vista::Accion::Quitar(i) => Accion::Quitar(i),
        })
    });
    let mut contenido = column![container(barra).padding([12., 24.])].spacing(8);
    if let Some(error) = &s.error {
        contenido = contenido.push(
            container(componentes::aviso_detallado(
                p,
                componentes::Estado::Aviso,
                "No se pudo completar la revisión",
                error,
            ))
            .padding([0., 24.]),
        );
    }
    if ocupado {
        contenido = contenido.push(
            container(
                text(if s.cargando {
                    "Consultando procedencia en Modrinth…"
                } else {
                    "Guardando cambios…"
                })
                .font(fuentes::MONO)
                .size(12)
                .color(p.text_muted),
            )
            .padding([0., 24.]),
        );
    }
    contenido.push(cuerpo).height(Length::Fill).into()
}

/// Sólo datos visuales, sin tocar carpetas del usuario ni realizar consultas.
pub fn preparar_diseno(app: &mut App) {
    app.verificacion.fixture = true;
    app.verificacion.informe = Informe {
        archivos: ["custom-hud-1.4.jar", "optifine-1.21.4.jar"]
            .into_iter()
            .enumerate()
            .map(|(i, nombre)| Archivo {
                nombre: nombre.into(),
                bytes: [412 * 1024, 7_400_000][i],
                sha512: format!("{:0>128}", if i == 0 { "8f2ca91e" } else { "03be77d0" }),
                activo: true,
                procedencia: Procedencia::Desconocida,
            })
            .collect(),
        error_red: None,
    };
    app.pantalla = Some(pantallas::Pantalla::NoVerificados);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app() -> App {
        let (mut a, _) = crate::iniciar_con_guardado(
            icaro_ui::shell::Seccion::Instancias,
            false,
            None,
            false,
            crate::servicios::Estado::nuevo(true),
            None,
        );
        a.pantalla = None;
        a
    }
    #[test]
    fn late_review_does_not_replace_another_instance_or_reopen_closed_screen() {
        let mut a = app();
        a.verificacion.instancia = Some("current".into());
        a.verificacion.generation = 3;
        a.verificacion.cargando = true;
        a.pantalla = Some(pantallas::Pantalla::NoVerificados);
        let _ = actualizar(
            &mut a,
            Accion::Revisada("previous".into(), 3, Err("wrong instance".into())),
        );
        assert!(a.verificacion.cargando);
        assert!(a.verificacion.error.is_none());
        let _ = actualizar(
            &mut a,
            Accion::Revisada("current".into(), 2, Err("old generation".into())),
        );
        assert!(a.verificacion.error.is_none());
        let _ = actualizar(&mut a, Accion::Cerrar);
        assert_eq!(a.pantalla, None);
        let _ = actualizar(
            &mut a,
            Accion::Revisada("current".into(), 3, Err("late".into())),
        );
        assert_eq!(a.pantalla, None);
        assert!(a.verificacion.error.is_none());
        assert!(!a.verificacion.cargando);
    }
    #[test]
    fn visual_fixture_cannot_trust_remove_or_launch_real_disk_operations() {
        let mut a = app();
        preparar_diseno(&mut a);
        let files = a.verificacion.informe.archivos.len();
        let _ = actualizar(&mut a, Accion::Confiar(0));
        let _ = actualizar(&mut a, Accion::Quitar(0));
        let _ = actualizar(&mut a, Accion::Revisar);
        assert!(!a.verificacion.operando && !a.verificacion.cargando);
        assert!(a.servicios.mods.busy.is_none());
        assert_eq!(a.verificacion.informe.archivos.len(), files);
        let _ = vista(&a);
    }
}
