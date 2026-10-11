// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Servicios reales del launcher; ICARO_DEMO=1 conserva la maqueta sin red.

use crate::{pantallas, App, Mensaje};
use icaro_core::{
    auth::{self, CancellationToken, Session},
    modrinth::{self, Catalog, Kind, Project, Results, Search, Version},
    vault::Vault,
};
use icaro_ui::{
    componentes::{self, Variante},
    cuenta::EstadoSesion,
    fuentes,
    shell::Seccion,
    tema::{espacio, texto},
};
use iced::widget::{column, container, row, text, Space};
use iced::{Element, Length, Padding, Task};
use std::sync::Arc;
use std::time::Duration;

pub struct Estado {
    pub demo: bool,
    pub session: Option<Arc<Session>>,
    vault: Arc<Vault>,
    catalog: Result<Catalog, modrinth::Error>,
    auth_id: u64,
    auth_cancel: Option<CancellationToken>,
    pub auth_busy: bool,
    auth_error: Option<String>,
    remembered: bool,
    pub query: String,
    search_id: u64,
    search_cancel: Option<CancellationToken>,
    filter: Option<Search>,
    results: Option<Results>,
    detail: Option<(Project, Vec<Version>)>,
    busy: bool,
    error: Option<String>,
    tab: usize,
}

impl Default for Estado {
    fn default() -> Self {
        Self {
            demo: std::env::var("ICARO_DEMO").as_deref() == Ok("1"),
            session: None,
            vault: Arc::new(Vault::default()),
            catalog: Catalog::new(),
            auth_id: 0,
            auth_cancel: None,
            auth_busy: false,
            auth_error: None,
            remembered: false,
            query: String::new(),
            search_id: 0,
            search_cancel: None,
            filter: None,
            results: None,
            detail: None,
            busy: false,
            error: None,
            tab: 0,
        }
    }
}
impl Estado {
    pub fn nombre<'a>(&'a self, demo_name: &'a str) -> &'a str {
        if self.demo {
            demo_name
        } else {
            self.session
                .as_ref()
                .map_or("Invitado", |s| s.profile.name.as_str())
        }
    }
    pub fn cancel(&mut self) {
        if let Some(cancel) = self.auth_cancel.take() {
            cancel.cancel();
        }
        self.auth_id = self.vault.invalidate();
        self.auth_busy = false;
    }
    pub fn reset_catalog(&mut self) {
        if let Some(cancel) = self.search_cancel.take() {
            cancel.cancel();
        }
        self.search_id += 1;
        self.results = None;
        self.detail = None;
        self.filter = None;
        self.error = None;
        self.busy = false;
        self.query.clear();
    }
}
impl Drop for Estado {
    fn drop(&mut self) {
        self.cancel();
        if let Some(cancel) = &self.search_cancel {
            cancel.cancel();
        }
    }
}

#[derive(Debug, Clone)]
pub enum Accion {
    Inicializar,
    Login,
    CancelarLogin,
    Logout,
    Renovar,
    Autenticada(u64, Result<Option<Arc<Session>>, String>, bool),
    Cerrada(u64, Result<(), String>),
    Consulta(String),
    Buscar,
    Pagina(u32),
    Buscada(u64, Result<Results, String>),
    Proyecto(String),
    ProyectoRecibido(u64, Result<(Project, Vec<Version>), String>),
    Volver,
    Pestana(usize),
    AbrirProyecto,
}
fn m(action: Accion) -> Mensaje {
    Mensaje::Servicio(action)
}

async fn save(
    vault: Arc<Vault>,
    id: u64,
    session: Arc<Session>,
) -> (Result<Option<Arc<Session>>, String>, bool) {
    let copy = session.clone();
    let stored = tokio::task::spawn_blocking(move || vault.remember(id, &copy)).await;
    match stored {
        Ok(Ok(())) => (Ok(Some(session)), true),
        Ok(Err(icaro_core::vault::Error::Stale)) => {
            (Err("Inicio de sesión cancelado.".into()), false)
        }
        _ => (Ok(Some(session)), false),
    }
}

fn begin_auth(app: &mut App, interactive: bool, restore: bool) -> Task<Mensaje> {
    if app.servicios.auth_busy {
        return Task::none();
    }
    app.servicios.cancel();
    let id = app.servicios.auth_id;
    let cancel = CancellationToken::new();
    app.servicios.auth_cancel = Some(cancel.clone());
    app.servicios.auth_busy = true;
    app.servicios.auth_error = None;
    let vault = app.servicios.vault.clone();
    let previous = app.servicios.session.clone();
    if interactive {
        app.pantalla = Some(pantallas::Pantalla::Codigo);
        app.sesion.menu = false;
    }
    Task::perform(
        async move {
            let result = if interactive {
                auth::login(cancel).await.map(Some)
            } else if restore {
                let read_vault = vault.clone();
                match tokio::task::spawn_blocking(move || read_vault.load()).await {
                Ok(Ok(Some(record))) => {
                    match auth::refresh(record.refresh, cancel).await {
                        Ok(session) if session.profile.id == record.profile_id => Ok(Some(session)),
                        Ok(_) => Err(auth::Error::Callback),
                        Err(error) => Err(error),
                    }
                }
                Ok(Ok(None)) => Ok(None),
                _ => return (Err("No se pudo recuperar la sesión guardada. Puedes iniciar sesión de nuevo.".into()), false),
            }
            } else if let Some(session) = previous {
                auth::renew(&session, cancel).await.map(Some)
            } else {
                Ok(None)
            };
            match result {
                Ok(Some(session)) => save(vault, id, Arc::new(session)).await,
                Ok(None) => (Ok(None), false),
                Err(error) => (Err(error.to_string()), false),
            }
        },
        move |(result, stored)| m(Accion::Autenticada(id, result, stored)),
    )
}

fn context(app: &App) -> Search {
    let kind = if app.seccion == Seccion::Modpacks && app.editor.is_none() {
        Kind::Modpack
    } else if app.editor == Some(3) {
        Kind::Resourcepack
    } else {
        Kind::Mod
    };
    let (minecraft, loader) = if kind == Kind::Modpack {
        (None, None)
    } else {
        let (mc, loader) = crate::juego_de_instancia(app);
        (
            Some(mc),
            (kind == Kind::Mod && !loader.eq_ignore_ascii_case("vanilla"))
                .then(|| loader.to_lowercase()),
        )
    };
    Search {
        query: app.servicios.query.clone(),
        kind,
        minecraft,
        loader,
        offset: 0,
    }
}

pub fn buscar(app: &mut App, offset: u32, debounce: bool) -> Task<Mensaje> {
    if app.servicios.demo {
        return Task::none();
    }
    if let Some(cancel) = app.servicios.search_cancel.take() {
        cancel.cancel();
    }
    app.servicios.search_id += 1;
    let id = app.servicios.search_id;
    let cancel = CancellationToken::new();
    app.servicios.search_cancel = Some(cancel.clone());
    let mut filter = context(app);
    filter.offset = offset;
    app.servicios.filter = Some(filter.clone());
    app.servicios.results = None;
    app.servicios.detail = None;
    app.servicios.busy = true;
    app.servicios.error = None;
    let catalog = app.servicios.catalog.clone();
    Task::perform(
        async move {
            tokio::select! {
                biased;
                _ = cancel.cancelled() => Err("Búsqueda cancelada.".into()),
                result = async {
                    if debounce { tokio::time::sleep(Duration::from_millis(350)).await; }
                    catalog?.search(&filter).await
                } => result.map_err(|e| e.to_string()),
            }
        },
        move |result| m(Accion::Buscada(id, result)),
    )
}

pub fn actualizar(app: &mut App, action: Accion) -> Task<Mensaje> {
    if app.servicios.demo {
        return Task::none();
    }
    match action {
        Accion::Inicializar => {
            let auth = begin_auth(app, false, true);
            let catalog = if app.seccion == Seccion::Modpacks || app.mods_vista == 1 {
                buscar(app, 0, false)
            } else {
                Task::none()
            };
            return Task::batch([auth, catalog]);
        }
        Accion::Login => return begin_auth(app, true, false),
        Accion::Renovar => {
            if !app.servicios.auth_busy
                && app
                    .servicios
                    .session
                    .as_ref()
                    .is_some_and(|s| s.needs_refresh())
            {
                return begin_auth(app, false, false);
            }
        }
        Accion::CancelarLogin => {
            app.servicios.cancel();
            app.servicios.auth_error = None;
            app.pantalla = None;
        }
        Accion::Logout => {
            app.servicios.cancel();
            app.servicios.session = None;
            app.servicios.remembered = false;
            app.sesion.estado = EstadoSesion::Invitado;
            app.sesion.menu = false;
            app.servicios.auth_busy = true;
            let id = app.servicios.auth_id;
            let vault = app.servicios.vault.clone();
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || vault.forget(id))
                        .await
                        .map_err(|_| "No se pudo acceder al almacén de credenciales.".to_owned())?
                        .map_err(|e| e.to_string())
                },
                move |result| m(Accion::Cerrada(id, result)),
            );
        }
        Accion::Cerrada(id, result) if id == app.servicios.auth_id => {
            app.servicios.auth_busy = false;
            if let Err(error) = result {
                app.servicios.auth_error = Some(format!(
                    "{error} No se pudo borrar la sesión guardada. Vuelve a pulsar Cerrar sesión."
                ));
            } else {
                app.servicios.auth_error = None;
                app.avisar("Sesión cerrada y credenciales guardadas eliminadas.".into());
            }
        }
        Accion::Autenticada(id, result, stored) if id == app.servicios.auth_id => {
            app.servicios.auth_busy = false;
            app.servicios.auth_cancel = None;
            match result {
                Ok(session) => {
                    app.servicios.remembered = stored;
                    app.sesion.estado = if session.is_some() {
                        EstadoSesion::Activa
                    } else {
                        EstadoSesion::Invitado
                    };
                    app.servicios.session = session;
                    if app.servicios.session.is_some() {
                        if app.pantalla == Some(pantallas::Pantalla::Codigo) {
                            app.pantalla = Some(pantallas::Pantalla::Cuenta);
                        }
                        if !stored {
                            app.servicios.auth_error = Some("Sesión activa solo durante esta apertura: el almacén de credenciales no está disponible.".into());
                        }
                    }
                }
                Err(error) => {
                    app.servicios.auth_error = Some(error);
                    if app.servicios.session.is_some() {
                        app.servicios.session = None;
                        app.sesion.estado = EstadoSesion::Vencida;
                    }
                }
            }
        }
        Accion::Consulta(query) => {
            app.servicios.query = query.chars().take(512).collect();
            return buscar(app, 0, true);
        }
        Accion::Buscar => return buscar(app, 0, false),
        Accion::Pagina(offset) => return buscar(app, offset, false),
        Accion::Buscada(id, result) if id == app.servicios.search_id => {
            app.servicios.busy = false;
            match result {
                Ok(results) => app.servicios.results = Some(results),
                Err(error) => app.servicios.error = Some(error),
            }
        }
        Accion::Proyecto(id) => {
            if let Some(cancel) = app.servicios.search_cancel.take() {
                cancel.cancel();
            }
            app.servicios.search_id += 1;
            let generation = app.servicios.search_id;
            let cancel = CancellationToken::new();
            app.servicios.search_cancel = Some(cancel.clone());
            let filter = context(app);
            app.servicios.filter = Some(filter.clone());
            let catalog = app.servicios.catalog.clone();
            app.servicios.busy = true;
            app.servicios.error = None;
            return Task::perform(
                async move {
                    tokio::select! {
                        biased;
                        _ = cancel.cancelled() => Err("Consulta cancelada.".into()),
                        result = async {
                            let catalog = catalog?;
                            let project = catalog.project(&id).await?;
                            let versions = catalog.versions(&project.id, filter.minecraft.as_deref(), filter.loader.as_deref()).await?;
                            Ok::<_, modrinth::Error>((project, versions))
                        } => result.map_err(|e| e.to_string()),
                    }
                },
                move |result| m(Accion::ProyectoRecibido(generation, result)),
            );
        }
        Accion::ProyectoRecibido(id, result) if id == app.servicios.search_id => {
            app.servicios.busy = false;
            match result {
                Ok(detail) => {
                    app.servicios.detail = Some(detail);
                    app.servicios.tab = 0;
                    crate::arrancar(app);
                }
                Err(error) => app.servicios.error = Some(error),
            }
        }
        Accion::Volver => {
            app.servicios.detail = None;
            app.servicios.error = None;
            crate::arrancar(app);
            if app.servicios.results.is_none() {
                return buscar(app, 0, false);
            }
        }
        Accion::Pestana(tab) => {
            app.servicios.tab = tab;
            crate::arrancar(app);
        }
        Accion::AbrirProyecto => {
            if let Some((project, _)) = &app.servicios.detail {
                let id = project.id.clone();
                return Task::perform(
                    async move { icaro_core::modrinth::open_project(&id).await },
                    |_| Mensaje::Nada,
                );
            }
        }
        _ => {}
    }
    Task::none()
}

pub fn login_view(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    let mut content = column![
        text("INICIA SESIÓN\nCON MICROSOFT")
            .font(fuentes::DISPLAY)
            .size(texto::DISPLAY.0)
            .color(p.text),
        text("Confirma tu cuenta en el navegador. Al terminar volverás automáticamente a Ícaro.")
            .color(p.text_muted),
        text(if app.servicios.auth_busy {
            "Esperando la autorización y comprobando tu perfil de Minecraft…"
        } else {
            "Inicia sesión para conectar tu cuenta de Minecraft Java."
        })
        .color(p.text),
        componentes::boton(
            p,
            "Abrir Microsoft",
            Variante::Primario,
            (!app.servicios.auth_busy).then(|| m(Accion::Login))
        ),
        componentes::boton(
            p,
            "Cancelar",
            Variante::Fantasma,
            Some(m(Accion::CancelarLogin))
        ),
    ]
    .spacing(espacio::S4);
    if let Some(error) = &app.servicios.auth_error {
        content = content.push(text(error).color(p.warning));
    }
    crate::pantallas::con_lamina(p, icaro_ui::laminas::Lamina::Hermes, content.into())
}

pub fn account_view(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    let s = &app.servicios;
    let mut content = column![text("CUENTA")
        .font(fuentes::DISPLAY)
        .size(texto::DISPLAY.0)
        .color(p.text)]
    .spacing(espacio::S4);
    if let Some(session) = &s.session {
        content = content
            .push(
                text(&session.profile.name)
                    .font(fuentes::TITULO)
                    .size(texto::HEADING.0)
                    .color(p.text),
            )
            .push(text("Microsoft · Minecraft Java").color(p.text_muted))
            .push(
                text(if s.remembered {
                    "Sesión recordada en este equipo."
                } else {
                    "Sesión activa durante esta apertura."
                })
                .color(p.text_muted),
            )
            .push(componentes::boton(
                p,
                "Cerrar sesión",
                Variante::Peligro,
                (!s.auth_busy).then(|| m(Accion::Logout)),
            ));
    } else {
        content = content
            .push(
                text(if s.auth_busy {
                    "Recuperando tu sesión…"
                } else {
                    "No has iniciado sesión."
                })
                .color(p.text),
            )
            .push(componentes::boton(
                p,
                "Iniciar sesión",
                Variante::Primario,
                (!s.auth_busy).then(|| m(Accion::Login)),
            ));
    }
    if let Some(error) = &s.auth_error {
        content = content
            .push(text(error).color(p.warning))
            .push(componentes::boton(
                p,
                "Cerrar sesión guardada",
                Variante::Peligro,
                (!s.auth_busy).then(|| m(Accion::Logout)),
            ));
    }
    container(content).padding(espacio::S12).into()
}

pub fn selector<'a>(app: &'a App, window: Element<'a, Mensaje>) -> Element<'a, Mensaje> {
    if !app.sesion.menu {
        return window;
    }
    let p = app.modo.paleta();
    let panel = column![
        text(app.servicios.nombre("Invitado"))
            .font(fuentes::TITULO)
            .color(p.text),
        componentes::boton(
            p,
            "Gestionar cuenta",
            Variante::Fantasma,
            Some(Mensaje::Ses(crate::sesion::Accion::Gestionar))
        ),
        componentes::boton(
            p,
            "Iniciar sesión",
            Variante::Secundario,
            (!app.servicios.auth_busy).then(|| m(Accion::Login))
        ),
        componentes::boton(
            p,
            "Cerrar sesión",
            Variante::Peligro,
            (!app.servicios.auth_busy).then(|| m(Accion::Logout))
        ),
    ]
    .spacing(espacio::S3)
    .padding(espacio::S4);
    let outside = iced::widget::mouse_area(Space::new(Length::Fill, Length::Fill))
        .on_press(Mensaje::Ses(crate::sesion::Accion::Menu(false)));
    iced::widget::stack![
        window,
        outside,
        container(container(panel).style(icaro_ui::estilo::marco(p)))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(iced::alignment::Vertical::Bottom)
            .padding(Padding {
                left: espacio::S2,
                bottom: 72.0,
                ..Padding::ZERO
            })
    ]
    .into()
}

pub fn store_view(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    let s = &app.servicios;
    let kind = context(app).kind;
    let mut content = column![].spacing(espacio::S4);
    if let Some((project, versions)) = &s.detail {
        content = content
            .push(componentes::boton(
                p,
                "Volver al catálogo",
                Variante::Fantasma,
                Some(m(Accion::Volver)),
            ))
            .push(
                text(&project.title)
                    .font(fuentes::DISPLAY)
                    .size(40)
                    .color(p.text),
            )
            .push(text(&project.description).color(p.text_muted))
            .push(
                row![
                    componentes::chip(p, "Descripción", s.tab == 0, m(Accion::Pestana(0))),
                    componentes::chip(p, "Versiones", s.tab == 1, m(Accion::Pestana(1)))
                ]
                .spacing(espacio::S3),
            );
        let mut body = column![].spacing(espacio::S3);
        if s.tab == 0 {
            body = body
                .push(
                    text(format!(
                        "Licencia: {} · {} descargas",
                        project.license.id, project.downloads
                    ))
                    .color(p.text_muted),
                )
                .push(text(project.body.chars().take(30_000).collect::<String>()).color(p.text));
        } else {
            if versions.is_empty() {
                body = body.push(
                    text("No hay versiones compatibles con esta instancia.").color(p.text_muted),
                );
            }
            for version in versions.iter().take(50) {
                body = body.push(
                    container(
                        column![
                            text(&version.name).font(fuentes::TITULO).color(p.text),
                            text(format!(
                                "{} · {} · {}",
                                version.version_number,
                                version.version_type,
                                version.loaders.join(", ")
                            ))
                            .color(p.text_muted),
                        ]
                        .spacing(espacio::S2),
                    )
                    .padding(espacio::S4)
                    .width(Length::Fill)
                    .style(icaro_ui::estilo::marco(p)),
                );
            }
            if versions.len() > 50 {
                body = body.push(
                    text("Se muestran las 50 primeras versiones. Consulta el resto en Modrinth.")
                        .color(p.text_muted),
                );
            }
        }
        content = content
            .push(crate::con_entrada(app, body.into()))
            .push(componentes::boton(
                p,
                "Ver en Modrinth",
                Variante::Secundario,
                Some(m(Accion::AbrirProyecto)),
            ));
    } else {
        content = content
            .push(
                text(if kind == Kind::Modpack {
                    "MODPACKS"
                } else {
                    "MODRINTH"
                })
                .font(fuentes::DISPLAY)
                .size(40)
                .color(p.text),
            )
            .push(
                row![
                    container(componentes::campo(
                        p,
                        "Buscar en Modrinth",
                        &s.query,
                        |q| m(Accion::Consulta(q))
                    ))
                    .width(Length::Fill),
                    componentes::boton(
                        p,
                        "Buscar",
                        Variante::Secundario,
                        (!s.busy).then(|| m(Accion::Buscar))
                    )
                ]
                .spacing(espacio::S3),
            );
        if let Some(filter) = &s.filter {
            if let Some(mc) = &filter.minecraft {
                content = content.push(
                    text(format!(
                        "Minecraft {mc} · {}",
                        filter.loader.as_deref().unwrap_or("todos los loaders")
                    ))
                    .color(p.text_muted),
                );
            }
        }
        if let Some(results) = &s.results {
            content = content
                .push(text(format!("{} resultados", results.total_hits)).color(p.text_muted));
            for hit in &results.hits {
                content = content.push(
                    container(
                        column![
                            text(hit.title.chars().take(256).collect::<String>())
                                .font(fuentes::TITULO)
                                .size(texto::HEADING.0)
                                .color(p.text),
                            text(hit.description.chars().take(1000).collect::<String>())
                                .color(p.text_muted),
                            row![
                                text(format!("{} · {} descargas", hit.author, hit.downloads))
                                    .color(p.text_muted),
                                Space::with_width(Length::Fill),
                                componentes::boton(
                                    p,
                                    "Ver proyecto",
                                    Variante::Secundario,
                                    Some(m(Accion::Proyecto(hit.project_id.clone())))
                                )
                            ]
                            .spacing(espacio::S3),
                        ]
                        .spacing(espacio::S3),
                    )
                    .padding(espacio::S4)
                    .width(Length::Fill)
                    .style(icaro_ui::estilo::marco(p)),
                );
            }
            content = content.push(
                row![
                    componentes::boton(
                        p,
                        "Anterior",
                        Variante::Secundario,
                        (results.offset > 0 && !s.busy).then(|| m(Accion::Pagina(
                            results.offset.saturating_sub(modrinth::PAGE_SIZE)
                        )))
                    ),
                    componentes::boton(
                        p,
                        "Siguiente",
                        Variante::Secundario,
                        (results.offset.saturating_add(results.hits.len() as u32)
                            < results.total_hits
                            && !s.busy)
                            .then(|| m(Accion::Pagina(
                                results.offset.saturating_add(modrinth::PAGE_SIZE)
                            )))
                    ),
                ]
                .spacing(espacio::S3),
            );
        }
    }
    if s.busy {
        content = content.push(text("Consultando Modrinth…").color(p.text_muted));
    }
    if let Some(error) = &s.error {
        content = content
            .push(text(error).color(p.warning))
            .push(componentes::boton(
                p,
                "Reintentar",
                Variante::Secundario,
                Some(m(Accion::Buscar)),
            ));
    }
    content = content.push(
        text("La instalación desde el catálogo todavía está en desarrollo.").color(p.text_muted),
    );
    crate::desplazable_pub(container(content).padding(espacio::S6).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn no_demo_account_is_reported_as_authenticated() {
        let mut state = Estado::default();
        state.demo = false;
        assert_eq!(state.nombre("Mineral_7"), "Invitado");
    }
}
