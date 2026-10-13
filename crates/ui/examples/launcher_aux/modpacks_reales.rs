// SPDX-License-Identifier: GPL-3.0-only
//! Revisión de un MRPACK y creación de una instancia independiente.
use crate::{App, Mensaje};
use icaro_core::{instancias::Instancia, modpacks::Plan, modrinth::Catalog};
use icaro_ui::{
    componentes::{self, Variante},
    fuentes,
    iconos::Icono,
};
use iced::{
    widget::{column, container, row, text},
    Element, Task,
};
use tokio_util::sync::CancellationToken;
#[derive(Default)]
pub struct Estado {
    pub abierto: bool,
    busy: bool,
    creando: bool,
    request: u64,
    cancel: Option<CancellationToken>,
    plan: Option<Plan>,
    error: Option<String>,
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
    Preparar(String, Option<String>),
    Revisado(u64, Result<Plan, String>),
    Opcionales(bool),
    Crear,
    Creada(u64, Result<Instancia, String>),
    Cerrar,
}
fn m(a: Accion) -> Mensaje {
    Mensaje::Modpacks(a)
}
pub fn accion<'a>(app: &App, id: &str, version: Option<&str>) -> Element<'a, Mensaje> {
    componentes::boton_con_icono(
        app.paleta(),
        Icono::Descargas,
        if version.is_some() { "" } else { "Instalar modpack" },
        if version.is_some() {Variante::Fantasma} else {Variante::Primario},
        (!app.servicios.modpacks.busy && !app.servicios.modpacks.creando)
            .then(|| m(Accion::Preparar(id.into(), version.map(str::to_owned)))),
        version.is_some(),
    )
}
pub fn actualizar(app: &mut App, a: Accion) -> Task<Mensaje> {
    match a {
        Accion::Preparar(project, version) => {
            if app.servicios.demo || app.servicios.modpacks.creando {
                return Task::none();
            }
            let s = &mut app.servicios.modpacks;
            if let Some(c) = s.cancel.take() {
                c.cancel();
            }
            s.request += 1;
            let generation = s.request;
            let cancel = CancellationToken::new();
            s.cancel = Some(cancel.clone());
            s.abierto = true;
            s.busy = true;
            s.plan = None;
            s.error = None;
            let root = crate::estado::directorio(false);
            return Task::perform(
                async move {
                    let catalog = Catalog::new().map_err(|e| e.to_string())?;
                    let selected = if let Some(id) = version {
                        catalog.version(&id).await.map_err(|e| e.to_string())?
                    } else {
                        let mut versions = catalog
                            .versions(&project, None, None)
                            .await
                            .map_err(|e| e.to_string())?;
                        versions.retain(|v| {
                            v.version_type == "release"
                                && v.files.iter().any(|f| f.filename.ends_with(".mrpack"))
                        });
                        versions.sort_by(|a, b| b.date_published.cmp(&a.date_published));
                        versions
                            .into_iter()
                            .next()
                            .ok_or("No hay una versión estable MRPACK; elige una en Versiones.")?
                    };
                    if selected.project_id != project {
                        return Err("La versión no pertenece al modpack seleccionado.".into());
                    }
                    icaro_core::modpacks::preparar(&root, &selected, cancel).await
                },
                move |r| m(Accion::Revisado(generation, r)),
            );
        }
        Accion::Revisado(g, result) => {
            let s = &mut app.servicios.modpacks;
            if g != s.request || !s.abierto {
                return Task::none();
            }
            s.busy = false;
            match result {
                Ok(plan) => s.plan = Some(plan),
                Err(error) => s.error = Some(error),
            }
        }
        Accion::Opcionales(value) => {
            if !app.servicios.modpacks.creando {
                if let Some(plan) = &mut app.servicios.modpacks.plan {
                    plan.opcionales = value;
                }
            }
        }
        Accion::Crear => {
            if app.servicios.demo || app.servicios.modpacks.busy || app.servicios.modpacks.creando {
                return Task::none();
            }
            let Some(plan) = app.servicios.modpacks.plan.clone() else {
                return Task::none();
            };
            let s = &mut app.servicios.modpacks;
            s.creando = true;
            s.error = None;
            let generation = s.request;
            let store = app.servicios.instancias.almacen.clone();
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || icaro_core::modpacks::crear(&store, &plan))
                        .await
                        .map_err(|_| "No se pudo crear la instancia del modpack.".to_owned())?
                },
                move |r| m(Accion::Creada(generation, r)),
            );
        }
        Accion::Creada(g, result) => {
            let s = &mut app.servicios.modpacks;
            if g != s.request {
                return Task::none();
            }
            s.creando = false;
            match result {
                Ok(instance) => {
                    s.abierto = false;
                    s.plan = None;
                    return crate::instancias_reales::actualizar(
                        app,
                        crate::instancias_reales::Accion::Creada(Ok(instance)),
                    );
                }
                Err(error) => s.error = Some(error),
            }
        }
        Accion::Cerrar => {
            let s = &mut app.servicios.modpacks;
            if s.creando {
                return Task::none();
            }
            if let Some(c) = s.cancel.take() {
                c.cancel();
            }
            s.request += 1;
            s.abierto = false;
            s.busy = false;
            s.plan = None;
            s.error = None;
        }
    }
    Task::none()
}
pub fn vista(app: &App) -> Element<'_, Mensaje> {
    let s = &app.servicios.modpacks;
    let p = app.paleta();
    let mut body = column![text("Instalar modpack")
        .font(fuentes::TITULO)
        .size(26)
        .color(p.text)]
    .spacing(20);
    if s.busy {
        body = body.push(text("Leyendo y verificando el MRPACK…").color(p.text_muted));
    }
    if let Some(plan) = &s.plan {
        body = body.push(
            text(format!("{} · {}", plan.nombre, plan.version))
                .font(fuentes::CUERPO_NEGRITA)
                .size(18)
                .color(p.text),
        );
        body = body.push(
            text(format!(
                "Minecraft {}\n{} {}\n{} archivos referenciados de cliente",
                plan.minecraft,
                crate::ficha::loader(&plan.loader),
                plan.loader_version.as_deref().unwrap_or(""),
                plan.archivos_cliente()
            ))
            .font(fuentes::MONO)
            .size(13)
            .color(p.text),
        );
        if plan.archivos_opcionales() > 0 {
            body = body.push(componentes::casilla(
                p,
                &format!("Incluir {} archivos opcionales", plan.archivos_opcionales()),
                plan.opcionales,
                |v| m(Accion::Opcionales(v)),
            ));
        }
        body=body.push(text("Se creará una nueva instancia. Los archivos incluidos se aplicarán antes de preparar Minecraft y Java. Puedes pausar o cancelar la instalación desde Descargas.").size(13).color(p.text_muted));
    }
    if let Some(error) = &s.error {
        body = body.push(text(error).size(13).color(p.error));
    }
    body = body.push(
        row![
            componentes::boton(
                p,
                "Cancelar",
                Variante::Secundario,
                (!s.creando).then(|| m(Accion::Cerrar))
            ),
            componentes::boton(
                p,
                if s.creando {
                    "Creando…"
                } else {
                    "Instalar en nueva instancia"
                },
                Variante::Primario,
                (!s.busy && !s.creando && s.plan.is_some()).then(|| m(Accion::Crear))
            )
        ]
        .spacing(12),
    );
    container(body)
        .padding(28)
        .width((app.ventana.width - 48.).clamp(600., 700.))
        .style(icaro_ui::estilo::flotante(p))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cerrar_descarta_resultados_tardios_y_respeta_la_creacion_en_curso() {
        let (mut app,_)=crate::iniciar_con_guardado(icaro_ui::shell::Seccion::Modpacks,false,None,false,crate::servicios::Estado::nuevo(true),None);
        let token=CancellationToken::new();
        app.servicios.modpacks.abierto=true;app.servicios.modpacks.busy=true;app.servicios.modpacks.cancel=Some(token.clone());
        let generation=app.servicios.modpacks.request;
        let _=actualizar(&mut app,Accion::Cerrar);
        assert!(token.is_cancelled());
        let _=actualizar(&mut app,Accion::Revisado(generation,Err("Resultado anterior".into())));
        assert!(!app.servicios.modpacks.abierto&&!app.servicios.modpacks.busy&&app.servicios.modpacks.error.is_none());
        app.servicios.modpacks.abierto=true;app.servicios.modpacks.creando=true;
        let generation=app.servicios.modpacks.request;
        let _=actualizar(&mut app,Accion::Cerrar);
        assert!(app.servicios.modpacks.abierto);assert_eq!(app.servicios.modpacks.request,generation);
    }
}
