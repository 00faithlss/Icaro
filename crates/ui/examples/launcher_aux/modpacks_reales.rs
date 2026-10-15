// SPDX-License-Identifier: GPL-3.0-only
//! Revisión de un MRPACK y creación de una instancia independiente.
use crate::{App, Mensaje};
use icaro_core::{instancias::Instancia, modpacks::Plan};
use icaro_ui::{
    componentes::{self, Variante},
    fuentes,
    iconos::Icono,
};
use iced::futures::SinkExt;
use iced::{
    widget::{column, container, row, text},
    Element, Task,
};
use tokio_util::sync::CancellationToken;
#[path = "importar_modpack.rs"]
mod importar;
#[derive(Default)]
pub struct Estado {
    pub abierto: bool,
    busy: bool,
    creando: bool,
    request: u64,
    cancel: Option<CancellationToken>,
    plan: Option<Plan>,
    error: Option<String>,
    local: bool,
    archivo: Option<std::path::PathBuf>,
    minecraft: String,
    loader: String,
    loader_version: String,
    arrastrando: bool,
    sobre: Option<String>,
    eligiendo: bool,
    ruta: String,
    nombre: String,
    grupo: String,
    paso: usize,
    detalles: [String; 5],
    lectura_visible: bool,
    peso_archivo: Option<u64>,
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
    Importar,
    Examinar,
    Ruta(String),
    AbrirRuta,
    Nombre(String),
    Grupo(String),
    Progreso(u64, usize, String),
    MostrarLectura(u64),
    ArchivoMedido(u64, u64),
    Sobre(Option<std::path::PathBuf>),
    Soltar(std::path::PathBuf),
    Archivo(u64, Option<std::path::PathBuf>),
    Minecraft(String),
    Loader(String),
    LoaderVersion(String),
    RevisarLocal,
    Elegir(String),
    Preparar(String, String),
    Revisado(u64, Result<Plan, String>),
    Opcionales(bool),
    Crear,
    Creada(u64, Result<Instancia, String>),
    Cerrar,
}
fn m(a: Accion) -> Mensaje {
    Mensaje::Modpacks(a)
}
fn admitido(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("mrpack") || e.eq_ignore_ascii_case("zip"))
}
pub fn evento(app: &App, evento: iced::Event) -> Option<Accion> {
    let s = &app.servicios.modpacks;
    if !s.abierto || !s.local || s.busy || s.creando || s.eligiendo {
        return None;
    }
    match evento {
        iced::Event::Window(iced::window::Event::FileHovered(path)) => {
            Some(Accion::Sobre(Some(path)))
        }
        iced::Event::Window(iced::window::Event::FilesHoveredLeft) => Some(Accion::Sobre(None)),
        iced::Event::Window(iced::window::Event::FileDropped(path)) => Some(Accion::Soltar(path)),
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter),
            ..
        }) if s.archivo.is_none() && s.ruta.trim().is_empty() => Some(Accion::Examinar),
        _ => None,
    }
}

pub fn preparar_diseno(app: &mut App, escena: &str) {
    let s = &mut app.servicios.modpacks;
    s.abierto = true;
    s.local = true;
    s.archivo = Some("Amigos - Minecraft 1.21.1.zip".into());
    s.minecraft = "1.21.1".into();
    s.loader = "fabric".into();
    s.loader_version = "0.19.5".into();
    s.ruta = "D:\\Modpacks\\Amigos - Minecraft 1.21.1.zip".into();
    s.peso_archivo = Some(191260262);
    if escena.contains("leyendo") {
        s.busy = true;
        s.lectura_visible = true;
        s.paso = 3;
        s.detalles = [
            "182,4 MB".into(),
            "Modrinth · modrinth.index.json".into(),
            "112 mods".into(),
            "112 referencias".into(),
            String::new(),
        ];
    } else if escena.contains("listo") {
        let plan: Plan = serde_json::from_value(serde_json::json!({
            "nombre":"Amigos SMP", "version":"2.3", "minecraft":"1.21.1", "loader":"fabric", "loader_version":"0.19.5", "opcionales":true,
            "fuente":{"url":"","sha512":"0".repeat(128),"size":191260262},
            "indice":{"formatVersion":1,"game":"minecraft","name":"Amigos SMP","versionId":"2.3","dependencies":{"minecraft":"1.21.1","fabric-loader":"0.19.5"},"files":[{"path":"mods/sodium.jar","hashes":{},"downloads":[],"fileSize":2097152}]},
            "local":{"sha512":"0".repeat(128),"bytes":191260262,"formato":"Modrinth","entradas":[
                {"origen":"overrides/config/a.json","destino":"config/a.json","bytes":1024,"crc":0},
                {"origen":"overrides/resourcepacks/a.zip","destino":"resourcepacks/a.zip","bytes":41943040,"crc":0},
                {"origen":"overrides/mods/lib.jar","destino":"mods/lib.jar","bytes":1048576,"crc":0}
            ]}
        })).expect("fixture de ImportarModpack");
        s.nombre = plan.nombre.clone();
        s.plan = Some(plan);
    } else if escena.contains("problema") {
        s.error = Some(
            "El paquete no es un ZIP válido. Vuelve a descargarlo y elige la copia completa."
                .into(),
        );
    }
    if escena.contains("reposo") || escena.contains("arrastre") {
        s.archivo = None;
    }
    if escena.contains("arrastre") {
        s.arrastrando = true;
        s.sobre = Some("fabulously-optimized-6.4.mrpack · 48,2 MB".into());
    }
    if escena.ends_with("angosto") {
        app.ventana = iced::Size::new(940., 720.);
    }
}
pub fn accion<'a>(app: &App, id: &str, version: Option<&str>) -> Element<'a, Mensaje> {
    componentes::boton_con_icono(
        app.paleta(),
        Icono::Descargas,
        if version.is_some() {
            "Instalar"
        } else {
            "Elegir versión"
        },
        if version.is_some() {
            Variante::Secundario
        } else {
            Variante::Primario
        },
        (!app.servicios.modpacks.busy && !app.servicios.modpacks.creando).then(|| {
            m(match version {
                Some(version) => Accion::Preparar(id.into(), version.into()),
                None => Accion::Elegir(id.into()),
            })
        }),
        false,
    )
}
pub fn actualizar(app: &mut App, a: Accion) -> Task<Mensaje> {
    match a {
        Accion::Importar => {
            let s = &mut app.servicios.modpacks;
            if s.busy || s.creando || app.servicios.demo {
                return Task::none();
            }
            if let Some(c) = s.cancel.take() {
                c.cancel();
            }
            s.request += 1;
            s.abierto = true;
            s.local = true;
            s.error = None;
            s.plan = None;
            s.archivo = None;
            s.minecraft.clear();
            s.loader = "fabric".into();
            s.loader_version.clear();
            s.arrastrando = false;
            s.sobre = None;
            s.eligiendo = false;
            s.ruta.clear();
            s.nombre.clear();
            s.grupo.clear();
        }
        Accion::Ruta(value) => {
            let s = &mut app.servicios.modpacks;
            if s.abierto && s.local && !s.busy && !s.creando && !s.eligiendo {
                s.ruta = value;
            }
        }
        Accion::AbrirRuta => {
            let s = &app.servicios.modpacks;
            if !s.abierto
                || !s.local
                || s.busy
                || s.creando
                || s.eligiendo
                || s.ruta.trim().is_empty()
            {
                return Task::none();
            }
            return actualizar(
                app,
                Accion::Archivo(s.request, Some(s.ruta.trim().trim_matches('"').into())),
            );
        }
        Accion::Nombre(value) => {
            let s = &mut app.servicios.modpacks;
            if !s.busy && !s.creando {
                s.nombre = value.chars().take(80).collect();
            }
        }
        Accion::Grupo(value) => {
            let s = &mut app.servicios.modpacks;
            if !s.busy && !s.creando {
                s.grupo = value;
            }
        }
        Accion::Progreso(g, paso, detalle) => {
            let s = &mut app.servicios.modpacks;
            if s.abierto && s.busy && g == s.request && paso < 5 {
                s.paso = paso;
                s.detalles[paso] = detalle;
            }
        }
        Accion::MostrarLectura(g) => {
            let s = &mut app.servicios.modpacks;
            if s.abierto && s.busy && s.request == g {
                s.lectura_visible = true;
            }
        }
        Accion::ArchivoMedido(g, bytes) => {
            let s = &mut app.servicios.modpacks;
            if s.abierto && s.busy && s.request == g {
                s.peso_archivo = Some(bytes);
            }
        }
        Accion::Examinar => {
            let s = &mut app.servicios.modpacks;
            if !s.abierto || !s.local || s.busy || s.creando || s.eligiendo || app.servicios.demo {
                return Task::none();
            }
            s.request += 1;
            let generation = s.request;
            s.eligiendo = true;
            return Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Modpack", &["mrpack", "zip"])
                        .pick_file()
                        .await
                        .map(|f| f.path().to_owned())
                },
                move |path| m(Accion::Archivo(generation, path)),
            );
        }
        Accion::Archivo(g, path) => {
            let s = &mut app.servicios.modpacks;
            if g != s.request || !s.abierto || !s.local {
                return Task::none();
            }
            s.eligiendo = false;
            if path.is_none() {
                return Task::none();
            }
            if !admitido(path.as_ref().unwrap()) {
                s.error = Some("Elige un archivo .mrpack o .zip completo.".into());
                return Task::none();
            }
            if s.archivo.as_ref() != path.as_ref() {
                s.minecraft.clear();
                s.loader = "fabric".into();
                s.loader_version.clear();
            }
            s.ruta = path.as_ref().unwrap().to_string_lossy().into_owned();
            s.peso_archivo = None;
            s.archivo = path;
            return actualizar(app, Accion::RevisarLocal);
        }
        Accion::Sobre(path) => {
            let s = &mut app.servicios.modpacks;
            if !s.abierto || !s.local || s.busy || s.creando || s.eligiendo {
                return Task::none();
            }
            s.arrastrando = path.as_ref().is_some_and(|p| admitido(p));
            s.sobre = path.and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()));
        }
        Accion::Soltar(path) => {
            let s = &mut app.servicios.modpacks;
            if !s.abierto || !s.local || s.busy || s.creando || s.eligiendo || app.servicios.demo {
                return Task::none();
            }
            s.arrastrando = false;
            s.sobre = None;
            if !admitido(&path) {
                s.error = Some("Elige un archivo .mrpack o .zip completo.".into());
                return Task::none();
            }
            let generation = s.request;
            return actualizar(app, Accion::Archivo(generation, Some(path)));
        }
        Accion::Minecraft(value) => {
            let s = &mut app.servicios.modpacks;
            if !s.busy && !s.creando {
                s.minecraft = value;
                s.plan = None;
            }
        }
        Accion::Loader(value) => {
            let s = &mut app.servicios.modpacks;
            if !s.busy && !s.creando {
                s.loader = value;
                s.plan = None;
            }
        }
        Accion::LoaderVersion(value) => {
            let s = &mut app.servicios.modpacks;
            if !s.busy && !s.creando {
                s.loader_version = value;
                s.plan = None;
            }
        }
        Accion::RevisarLocal => {
            let s = &mut app.servicios.modpacks;
            if !s.local || s.busy || s.creando || app.servicios.demo {
                return Task::none();
            }
            let Some(path) = s.archivo.clone() else {
                return Task::none();
            };
            if let Some(c) = s.cancel.take() {
                c.cancel();
            }
            s.request += 1;
            let generation = s.request;
            let cancel = CancellationToken::new();
            s.cancel = Some(cancel.clone());
            s.busy = true;
            s.plan = None;
            s.error = None;
            s.arrastrando = false;
            s.sobre = None;
            s.eligiendo = false;
            s.paso = 0;
            s.detalles = Default::default();
            s.lectura_visible = false;
            let (mc, loader, version) = (
                s.minecraft.clone(),
                s.loader.clone(),
                s.loader_version.clone(),
            );
            let root = crate::estado::directorio(false);
            let trabajo = Task::run(
                iced::stream::channel(16, async move |mut output| {
                    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
                    let worker = tokio::task::spawn_blocking(move || {
                        if let Ok(meta) = std::fs::symlink_metadata(&path) {
                            if meta.is_file() {
                                let _ = tx.send(Accion::ArchivoMedido(generation, meta.len()));
                            }
                        }
                        icaro_core::modpacks::local::preparar_con_progreso(
                            &root,
                            &path,
                            &mc,
                            &loader,
                            &version,
                            &cancel,
                            move |paso, detalle| {
                                let _ = tx.send(Accion::Progreso(generation, paso, detalle));
                            },
                        )
                    });
                    // Drenar las fases antes del resultado evita perder las últimas al terminar rápido.
                    while let Some(evento) = rx.recv().await {
                        let _ = output.send(evento).await;
                    }
                    let result = worker
                        .await
                        .unwrap_or_else(|_| Err("No se pudo revisar el paquete local.".into()));
                    let _ = output.send(Accion::Revisado(generation, result)).await;
                }),
                m,
            );
            return Task::batch([
                trabajo,
                Task::perform(
                    async {
                        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                    },
                    move |_| m(Accion::MostrarLectura(generation)),
                ),
            ]);
        }
        Accion::Elegir(project) => {
            if app.servicios.demo || app.servicios.modpacks.creando {
                return Task::none();
            }
            if app
                .servicios
                .detail
                .as_ref()
                .is_some_and(|(p, _)| p.id == project)
            {
                return crate::servicios::actualizar(
                    app,
                    crate::servicios::Accion::FichaPestana(crate::ficha::Pestana::Versiones),
                );
            }
            return crate::servicios::actualizar(
                app,
                crate::servicios::Accion::ProyectoVersiones(project),
            );
        }
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
            s.local = false;
            s.grupo.clear();
            let root = crate::estado::directorio(false);
            let source = app.servicios.catalog.clone();
            return Task::perform(
                async move {
                    let catalog = source.map_err(|e| e.to_string())?;
                    let selected = catalog.version(&version).await.map_err(|e| e.to_string())?;
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
                Ok(plan) => {
                    s.nombre = plan.nombre.clone();
                    s.plan = Some(plan);
                    return iced::widget::text_input::focus("importar-nombre");
                }
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
            let Some(mut plan) = app.servicios.modpacks.plan.clone() else {
                return Task::none();
            };
            let s = &mut app.servicios.modpacks;
            if s.local {
                if s.nombre.trim().is_empty() {
                    return Task::none();
                }
                plan.nombre = s.nombre.trim().into();
            }
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
                    let grupo = s.grupo.clone();
                    s.abierto = false;
                    s.plan = None;
                    let task = crate::instancias_reales::actualizar(
                        app,
                        crate::instancias_reales::Accion::Creada(Ok(instance)),
                    );
                    if !grupo.is_empty() && app.lista_grupos.contains(&grupo) {
                        if let Some(g) = app.grupos.last_mut() {
                            *g = Some(grupo);
                        }
                    }
                    return task;
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
    if s.local {
        return importar::vista(app);
    }
    let p = app.paleta();
    let mut body = column![text(if s.local {
        "Importar modpack"
    } else {
        "Instalar modpack"
    })
    .font(fuentes::TITULO)
    .size(26)
    .color(p.text)]
    .spacing(20);
    if s.busy {
        body = body
            .push(text("Leyendo y verificando el modpack… Puedes cancelar.").color(p.text_muted));
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
                "Minecraft {}\n{} {}\n{} archivos referenciados · {} archivos incluidos",
                plan.minecraft,
                crate::ficha::loader(&plan.loader),
                plan.loader_version.as_deref().unwrap_or(""),
                plan.archivos_cliente(),
                plan.archivos_incluidos()
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
    container(iced::widget::scrollable(body).height(iced::Length::Shrink))
        .padding(28)
        .width((app.ventana.width - 48.).clamp(280., 700.))
        .max_height((app.ventana.height - 48.).max(280.))
        .style(icaro_ui::estilo::flotante(p))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn progreso_y_edicion_respetan_generacion_y_revision_terminada() {
        let (mut app, _) = crate::iniciar_con_guardado(
            icaro_ui::shell::Seccion::Instancias,
            false,
            None,
            false,
            crate::servicios::Estado::nuevo(true),
            None,
        );
        preparar_diseno(&mut app, "importar-local-leyendo");
        let g = app.servicios.modpacks.request;
        let _ = actualizar(
            &mut app,
            Accion::Progreso(g + 1, 4, "Resultado viejo".into()),
        );
        assert_eq!(app.servicios.modpacks.paso, 3);
        let _ = actualizar(&mut app, Accion::Progreso(g, 4, "Copia lista".into()));
        assert_eq!(app.servicios.modpacks.detalles[4], "Copia lista");
        let _ = actualizar(&mut app, Accion::Revisado(g, Err("ZIP inválido".into())));
        let _ = actualizar(&mut app, Accion::Progreso(g, 0, "Tardío".into()));
        let _ = actualizar(&mut app, Accion::MostrarLectura(g));
        assert!(!app.servicios.modpacks.busy);
        assert_eq!(app.servicios.modpacks.paso, 4);
        let _ = actualizar(&mut app, Accion::Nombre("Mi instancia".into()));
        assert_eq!(app.servicios.modpacks.nombre, "Mi instancia");
    }
    #[test]
    fn ruta_pegada_y_selector_comparten_validacion_sin_crear_instancias() {
        let (mut app, _) = crate::iniciar_con_guardado(
            icaro_ui::shell::Seccion::Instancias,
            false,
            None,
            false,
            crate::servicios::Estado::nuevo(true),
            None,
        );
        preparar_diseno(&mut app, "importar-local-reposo");
        let _ = actualizar(&mut app, Accion::Ruta("C:\\Mods\\archivo.rar".into()));
        let _ = actualizar(&mut app, Accion::AbrirRuta);
        assert!(app
            .servicios
            .modpacks
            .error
            .as_ref()
            .unwrap()
            .contains(".mrpack"));
        assert!(app.servicios.modpacks.archivo.is_none());
        assert!(!app.servicios.modpacks.busy);
    }
    #[test]
    fn zona_soltar_reacciona_solo_dentro_de_una_importacion_disponible() {
        let (mut app, _) = crate::iniciar_con_guardado(
            icaro_ui::shell::Seccion::Instancias,
            false,
            None,
            false,
            crate::servicios::Estado::nuevo(true),
            None,
        );
        let hover = iced::Event::Window(iced::window::Event::FileHovered("pack.MRPACK".into()));
        assert!(evento(&app, hover.clone()).is_none());
        app.servicios.modpacks.abierto = true;
        app.servicios.modpacks.local = true;
        let a = evento(&app, hover.clone()).unwrap();
        let _ = actualizar(&mut app, a);
        assert!(app.servicios.modpacks.arrastrando);
        assert_eq!(app.servicios.modpacks.sobre.as_deref(), Some("pack.MRPACK"));
        let _ = actualizar(&mut app, Accion::Sobre(None));
        assert!(!app.servicios.modpacks.arrastrando);
        app.servicios.modpacks.busy = true;
        assert!(evento(&app, hover).is_none());
        app.servicios.modpacks.busy = false;
        app.servicios.demo = false;
        let _ = actualizar(&mut app, Accion::Soltar("imagen.png".into()));
        assert!(app
            .servicios
            .modpacks
            .error
            .as_ref()
            .unwrap()
            .contains(".mrpack"));
        assert!(app.servicios.modpacks.archivo.is_none());
    }
    #[test]
    fn cancelar_selector_de_archivo_conserva_la_zona_abierta() {
        let (mut app, _) = crate::iniciar_con_guardado(
            icaro_ui::shell::Seccion::Instancias,
            false,
            None,
            false,
            crate::servicios::Estado::nuevo(true),
            None,
        );
        let s = &mut app.servicios.modpacks;
        s.abierto = true;
        s.local = true;
        s.eligiendo = true;
        s.archivo = Some("anterior.zip".into());
        let g = s.request;
        let _ = actualizar(&mut app, Accion::Archivo(g, None));
        let s = &app.servicios.modpacks;
        assert!(s.abierto);
        assert!(!s.eligiendo);
        assert_eq!(
            s.archivo.as_deref(),
            Some(std::path::Path::new("anterior.zip"))
        );
    }
    #[test]
    fn selector_local_descarta_archivos_tardios_y_cambios_invalidan_revision() {
        let (mut app, _) = crate::iniciar_con_guardado(
            icaro_ui::shell::Seccion::Instancias,
            false,
            None,
            false,
            crate::servicios::Estado::nuevo(true),
            None,
        );
        app.servicios.modpacks.local = true;
        app.servicios.modpacks.abierto = true;
        let generation = app.servicios.modpacks.request;
        let _ = actualizar(&mut app, Accion::Cerrar);
        let _ = actualizar(
            &mut app,
            Accion::Archivo(generation, Some("viejo.zip".into())),
        );
        assert!(!app.servicios.modpacks.abierto);
        assert!(app.servicios.modpacks.archivo.is_none());
        let _ = actualizar(&mut app, Accion::Minecraft("1.21.1".into()));
        assert_eq!(app.servicios.modpacks.minecraft, "1.21.1");
        assert!(app.servicios.modpacks.plan.is_none());
    }
    #[test]
    fn elegir_abre_versiones_y_conserva_filtros_sin_preparar_una_instalacion() {
        let (mut app, _) = crate::iniciar_con_guardado(
            icaro_ui::shell::Seccion::Modpacks,
            false,
            None,
            false,
            crate::servicios::Estado::nuevo(true),
            None,
        );
        crate::pruebas_diseno::preparar_modpacks(&mut app, "modpacks-versiones");
        app.servicios.ficha.tab = crate::ficha::Pestana::Descripcion;
        app.servicios.ficha.minecraft = "1.20.1".into();
        app.servicios.ficha.pruebas = true;
        let _ = actualizar(&mut app, Accion::Elegir("optimized-pack".into()));
        assert_eq!(app.servicios.ficha.tab, crate::ficha::Pestana::Versiones);
        assert_eq!(app.servicios.ficha.minecraft, "1.20.1");
        assert!(app.servicios.ficha.pruebas);
        assert!(!app.servicios.modpacks.abierto && !app.servicios.modpacks.busy);
    }
    #[test]
    fn cerrar_descarta_resultados_tardios_y_respeta_la_creacion_en_curso() {
        let (mut app, _) = crate::iniciar_con_guardado(
            icaro_ui::shell::Seccion::Modpacks,
            false,
            None,
            false,
            crate::servicios::Estado::nuevo(true),
            None,
        );
        let token = CancellationToken::new();
        app.servicios.modpacks.abierto = true;
        app.servicios.modpacks.busy = true;
        app.servicios.modpacks.cancel = Some(token.clone());
        let generation = app.servicios.modpacks.request;
        let _ = actualizar(&mut app, Accion::Cerrar);
        assert!(token.is_cancelled());
        let _ = actualizar(
            &mut app,
            Accion::Revisado(generation, Err("Resultado anterior".into())),
        );
        assert!(
            !app.servicios.modpacks.abierto
                && !app.servicios.modpacks.busy
                && app.servicios.modpacks.error.is_none()
        );
        app.servicios.modpacks.abierto = true;
        app.servicios.modpacks.creando = true;
        let generation = app.servicios.modpacks.request;
        let _ = actualizar(&mut app, Accion::Cerrar);
        assert!(app.servicios.modpacks.abierto);
        assert_eq!(app.servicios.modpacks.request, generation);
    }
}
