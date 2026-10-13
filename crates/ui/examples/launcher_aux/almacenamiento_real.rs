// SPDX-License-Identifier: GPL-3.0-only
use crate::{App, Mensaje};
use icaro_core::almacenamiento;
use iced::Task;
#[derive(Default)]
pub struct Estado {
    pub informe: Option<almacenamiento::Informe>,
    pub ocupado: bool,
    pub error: Option<String>,
}
#[derive(Debug, Clone)]
pub enum Accion {
    Medir,
    Limpiar,
    Trasladar,
    Recuperar,
    Terminado(Result<(almacenamiento::Informe, String), String>),
}
pub fn actualizar(app: &mut App, accion: Accion) -> Task<Mensaje> {
    if app.servicios.demo {
        return Task::none();
    }
    if let Accion::Terminado(result) = accion {
        app.almacenamiento.ocupado = false;
        match result {
            Ok((report, note)) => {
                app.almacenamiento.informe = Some(report);
                app.almacenamiento.error = None;
                if !note.is_empty() {
                    app.avisar(note);
                }
            }
            Err(e) => {
                app.almacenamiento.error = Some(e.clone());
                app.avisar(e);
            }
        }
        return Task::none();
    }
    if app.almacenamiento.ocupado {
        return Task::none();
    }
    if !matches!(accion, Accion::Medir)
        && (app.servicios.mods.busy.is_some()
            || app.servicios.instancias.operando
            || app
                .servicios
                .instancias
                .registros
                .iter()
                .any(|i| app.servicios.juego.ocupada(&i.id)))
    {
        app.avisar(
            "Espera a que terminen las operaciones y cierra el juego antes de limpiar.".into(),
        );
        return Task::none();
    }
    app.almacenamiento.ocupado = true;
    let root = crate::estado::directorio(false);
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let note = match accion {
                    Accion::Limpiar => format!(
                        "Caché limpiada: {} liberados.",
                        tamano(almacenamiento::limpiar_cache(&root)?)
                    ),
                    Accion::Recuperar => format!(
                        "{} operaciones recuperadas.",
                        almacenamiento::recuperar_operaciones(&root)?
                    ),
                    Accion::Trasladar => format!(
                        "{} elementos trasladados a la papelera del sistema.",
                        almacenamiento::trasladar_papeleras(&root)?
                    ),
                    _ => String::new(),
                };
                if !note.is_empty() {
                    let _ = almacenamiento::registrar(&root, &note);
                }
                Ok((almacenamiento::medir(&root)?, note))
            })
            .await
            .map_err(|_| "No se pudo revisar el almacenamiento".to_owned())?
        },
        |r| Mensaje::Almacenamiento(Accion::Terminado(r)),
    )
}
pub fn tamano(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / (1024. * 1024. * 1024.))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024. * 1024.))
    } else {
        format!("{} KB", bytes / 1024)
    }
}
