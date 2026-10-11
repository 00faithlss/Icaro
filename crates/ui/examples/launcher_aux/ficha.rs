// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Fichas públicas de mods y modpacks, con la estructura del sistema de diseño.

use crate::catalogo::texto_publico as text;
use crate::{catalogo, servicios::Accion, App, Mensaje};
use icaro_core::{
    auth::CancellationToken,
    modrinth::{Kind, Member, Organization, Project, Search, Version},
};
use icaro_ui::{
    componentes::{self, Variante},
    fuentes,
    iconos::Icono,
    tema::{espacio, texto, Paleta},
};
use iced::{
    widget::{button, column, container, keyed_column, row, Space},
    Element, Length, Padding,
};

pub const PAGE_SIZE: usize = 25;
pub const GALLERY_PAGE: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pestana {
    #[default]
    Descripcion,
    Galeria,
    Versiones,
    Cambios,
    Dependencias,
}
impl Pestana {
    pub fn nombre(self) -> &'static str {
        match self {
            Self::Descripcion => "Descripción",
            Self::Galeria => "Galería",
            Self::Versiones => "Versiones",
            Self::Cambios => "Cambios",
            Self::Dependencias => "Dependencias",
        }
    }
}

pub struct Estado {
    pub tab: Pestana,
    pub galeria: usize,
    pub compatibilidad_ampliada: bool,
    pub miembros: Vec<Member>,
    pub miembros_busy: bool,
    pub miembros_error: Option<String>,
    pub organizacion: Option<Organization>,
    pub organizacion_busy: bool,
    pub organizacion_error: Option<String>,
    pub minecraft: String,
    pub loader: String,
    pub pruebas: bool,
    pub pagina: usize,
    pub versiones_id: u64,
    pub versiones_busy: bool,
    pub versiones_error: Option<String>,
    pub versiones_cancel: Option<CancellationToken>,
    pub seleccionada: Option<String>,
    pub cambios_id: u64,
    pub cambios_busy: bool,
    pub cambios_error: Option<String>,
    pub cambios_cancel: Option<CancellationToken>,
    pub cambio: Option<Version>,
    pub documento_cambios: catalogo::Documento,
    pub visor: Option<icaro_ui::visor::EstadoVisor>,
    pub visor_id: u64,
    pub visor_imagen: Option<(String, catalogo::Imagen)>,
    pub visor_busy: bool,
    pub visor_error: Option<String>,
    pub visor_cancel: Option<CancellationToken>,
}
impl Default for Estado {
    fn default() -> Self {
        Self {
            tab: Pestana::Descripcion,
            galeria: 0,
            compatibilidad_ampliada: false,
            miembros: Vec::new(),
            miembros_busy: false,
            miembros_error: None,
            organizacion: None,
            organizacion_busy: false,
            organizacion_error: None,
            minecraft: "Todas las versiones".into(),
            loader: "Todos los loaders".into(),
            pruebas: false,
            pagina: 0,
            versiones_id: 0,
            versiones_busy: false,
            versiones_error: None,
            versiones_cancel: None,
            seleccionada: None,
            cambios_id: 0,
            cambios_busy: false,
            cambios_error: None,
            cambios_cancel: None,
            cambio: None,
            documento_cambios: Default::default(),
            visor: None,
            visor_id: 0,
            visor_imagen: None,
            visor_busy: false,
            visor_error: None,
            visor_cancel: None,
        }
    }
}
impl Drop for Estado {
    fn drop(&mut self) {
        if let Some(c) = &self.versiones_cancel {
            c.cancel();
        }
        if let Some(c) = &self.cambios_cancel {
            c.cancel();
        }
        if let Some(c) = &self.visor_cancel {
            c.cancel();
        }
    }
}
impl Estado {
    pub fn nuevo(filter: Option<&Search>, versions: &[Version]) -> Self {
        let mut state = Self::default();
        if let Some(filter) = filter {
            state.minecraft = filter
                .minecraft
                .clone()
                .unwrap_or_else(|| "Todas las versiones".into());
            state.loader = filter
                .loader
                .clone()
                .map(|l| loader(&l))
                .unwrap_or_else(|| "Todos los loaders".into());
        }
        state.elegir_predeterminada(versions);
        state
    }
    pub fn visibles<'a>(&self, versions: &'a [Version]) -> Vec<&'a Version> {
        versions
            .iter()
            .filter(|v| self.pruebas || v.version_type == "release")
            .collect()
    }
    pub fn elegir_predeterminada(&mut self, versions: &[Version]) {
        let visibles = self.visibles(versions);
        if !visibles
            .iter()
            .any(|v| Some(&v.id) == self.seleccionada.as_ref())
        {
            self.seleccionada = visibles.first().map(|v| v.id.clone());
            self.limpiar_cambios();
        }
        self.pagina = self
            .pagina
            .min(visibles.len().saturating_sub(1) / PAGE_SIZE);
    }
    pub fn limpiar_cambios(&mut self) {
        if let Some(c) = self.cambios_cancel.take() {
            c.cancel();
        }
        self.cambios_id += 1;
        self.cambios_busy = false;
        self.cambios_error = None;
        self.cambio = None;
        self.documento_cambios = Default::default();
    }
    pub fn imagenes(&self, project: &Project, description: &catalogo::Documento) -> Vec<String> {
        let mut urls: Vec<_> = project.icon_url.iter().cloned().collect();
        match self.tab {
            Pestana::Descripcion => urls.extend(description.imagenes()),
            Pestana::Galeria => {
                let inicio = self.galeria / GALLERY_PAGE * GALLERY_PAGE;
                urls.extend(
                    project
                        .gallery
                        .iter()
                        .skip(inicio)
                        .take(GALLERY_PAGE)
                        .map(|g| g.url.clone()),
                );
            }
            Pestana::Cambios => urls.extend(self.documento_cambios.imagenes()),
            _ => {}
        }
        urls
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OpcionVersion {
    id: String,
    nombre: String,
}
impl std::fmt::Display for OpcionVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.nombre)
    }
}
fn opcion(v: &Version) -> OpcionVersion {
    OpcionVersion {
        id: v.id.clone(),
        nombre: format!(
            "{} · {} · {}",
            v.version_number,
            v.loaders
                .iter()
                .map(|l| loader(l))
                .collect::<Vec<_>>()
                .join(", "),
            fecha(&v.date_published)
        ),
    }
}
pub fn loader(value: &str) -> String {
    match value.to_ascii_lowercase().as_str() {
        "fabric" => "Fabric".into(),
        "forge" => "Forge".into(),
        "neoforge" => "NeoForge".into(),
        "quilt" => "Quilt".into(),
        "minecraft" => "Minecraft".into(),
        _ => value.to_owned(),
    }
}
pub fn fecha(iso: &str) -> String {
    let Some(date) = iso.get(..10) else {
        return "Sin fecha".into();
    };
    let parts: Vec<_> = date.split('-').collect();
    if parts.len() != 3 || !parts.iter().all(|p| p.bytes().all(|b| b.is_ascii_digit())) {
        return "Sin fecha".into();
    }
    format!("{}-{}-{}", parts[2], parts[1], parts[0])
}
fn canal(value: &str) -> &str {
    match value {
        "release" => "Estable",
        "beta" => "Beta",
        "alpha" => "Alpha",
        _ => value,
    }
}
fn m(action: Accion) -> Mensaje {
    Mensaje::Servicio(action)
}
fn enlace<'a>(p: Paleta, titulo: &str, url: &str) -> Element<'a, Mensaje> {
    componentes::boton_con_icono(
        p,
        Icono::Externo,
        titulo,
        Variante::Fantasma,
        Some(m(Accion::Enlace(url.into()))),
        false,
    )
}
fn compatibilidad<'a>(p: Paleta, project: &Project, ampliada: bool) -> Element<'a, Mensaje> {
    let juegos = row(project
        .game_versions
        .iter()
        .rev()
        .take(if ampliada { usize::MAX } else { 12 })
        .map(|v| componentes::insignia_contorno(p, v))
        .collect::<Vec<_>>())
    .spacing(espacio::S2)
    .wrap();
    let loaders = row(project
        .loaders
        .iter()
        .map(|v| componentes::insignia_contorno(p, &loader(v)))
        .collect::<Vec<_>>())
    .spacing(espacio::S2)
    .wrap();
    let mut body = column![
        text("Minecraft").color(p.text_muted),
        juegos,
        text("Loaders").color(p.text_muted),
        loaders
    ]
    .spacing(espacio::S2);
    if project.game_versions.len() > 12 {
        body = body.push(componentes::boton_compacto(
            p,
            if ampliada {
                "Ver menos"
            } else {
                "Ver todas las versiones"
            },
            Variante::Secundario,
            Some(m(Accion::CompatibilidadAmpliada)),
        ));
    }
    body.into()
}
fn lateral<'a>(app: &'a App, project: &'a Project) -> Element<'a, Mensaje> {
    let p = app.modo.paleta();
    let s = &app.servicios.ficha;
    let mut miembros = column![].spacing(espacio::S3);
    if let Some(org) = &s.organizacion {
        miembros = miembros.push(
            column![
                text(&org.name).font(fuentes::TITULO).color(p.text),
                text("Organización propietaria")
                    .size(13)
                    .color(p.text_muted),
                enlace(
                    p,
                    "Ver organización",
                    &format!("https://modrinth.com/organization/{}", org.id)
                )
            ]
            .spacing(espacio::S1),
        );
    } else if s.organizacion_busy {
        miembros = miembros.push(text("Cargando organización…").color(p.text_muted));
    } else if let Some(error) = &s.organizacion_error {
        miembros = miembros
            .push(text(error).color(p.warning))
            .push(componentes::boton(
                p,
                "Reintentar creadores",
                Variante::Secundario,
                Some(m(Accion::CargarMiembros)),
            ));
    }
    if s.miembros_busy {
        miembros = miembros.push(text("Cargando creadores…").color(p.text_muted));
    }
    if let Some(error) = &s.miembros_error {
        miembros = miembros
            .push(text(error).color(p.warning))
            .push(componentes::boton(
                p,
                "Reintentar",
                Variante::Secundario,
                Some(m(Accion::CargarMiembros)),
            ));
    } else if !s.miembros_busy && s.miembros.is_empty() {
        miembros = miembros.push(text("Sin creadores publicados.").color(p.text_muted));
    }
    for member in &s.miembros {
        miembros = miembros.push(
            column![
                row![
                    app.servicios
                        .images
                        .vista(p, member.user.avatar_url.as_deref(), 40.),
                    text(member.user.name.as_deref().unwrap_or(&member.user.username))
                        .font(fuentes::TITULO)
                        .color(p.text)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                text(
                    if member.is_owner && member.role.eq_ignore_ascii_case("owner") {
                        "Propietario".into()
                    } else if member.is_owner {
                        format!("Propietario · {}", member.role)
                    } else if member.role.eq_ignore_ascii_case("member") {
                        "Miembro".into()
                    } else {
                        member.role.clone()
                    }
                )
                .size(13)
                .color(p.text_muted),
                enlace(
                    p,
                    "Ver perfil",
                    &format!("https://modrinth.com/user/{}", member.user.id)
                ),
            ]
            .spacing(espacio::S1),
        );
    }
    let mut enlaces = column![enlace(
        p,
        "Ver en Modrinth",
        &format!("https://modrinth.com/project/{}", project.id)
    )]
    .spacing(espacio::S2);
    for (label, url) in [
        ("Reportar problemas", &project.issues_url),
        ("Código fuente", &project.source_url),
        ("Wiki", &project.wiki_url),
        ("Discord", &project.discord_url),
    ] {
        if let Some(url) = url {
            enlaces = enlaces.push(enlace(p, label, url));
        }
    }
    for donation in project.donation_urls.iter().flatten() {
        enlaces = enlaces.push(enlace(
            p,
            &format!("Donar · {}", donation.platform),
            &donation.url,
        ));
    }
    let lado = |value: &str| match value {
        "required" => "Requerido",
        "optional" => "Opcional",
        "unsupported" => "No compatible",
        _ => "No especificado",
    };
    let mut detalles = column![].spacing(10);
    for (label, value) in [
        ("Descargas", catalogo::numero(project.downloads)),
        ("Seguidores", catalogo::numero(project.followers)),
        ("Licencia", project.license.name.clone()),
        ("Publicado", fecha(&project.published)),
        ("Actualizado", fecha(&project.updated)),
        ("Cliente", lado(&project.client_side).into()),
        ("Servidor", lado(&project.server_side).into()),
    ] {
        detalles = detalles.push(
            row![
                text(label)
                    .size(texto::BODY_SM.0)
                    .color(p.text_muted)
                    .width(90),
                text(value)
                    .font(fuentes::MONO)
                    .size(texto::MONO_SM.0)
                    .color(p.text)
                    .width(Length::Fill)
            ]
            .spacing(8),
        );
    }
    let bloques = column![
        componentes::etiqueta(p, "Funciona en"),
        compatibilidad(p, project, s.compatibilidad_ampliada),
        detalles,
        crate::diseno_mods::regla(p),
        componentes::etiqueta(p, "Creador"),
        miembros,
        crate::diseno_mods::regla(p),
        enlaces,
    ]
    .spacing(espacio::S4)
    .width(Length::Fill);
    bloques.into()
}

fn filtros<'a>(app: &'a App, project: &Project) -> Element<'a, Mensaje> {
    let p = app.modo.paleta();
    let s = &app.servicios.ficha;
    if app.editor.is_some() {
        let (mc, loader) = crate::juego_de_instancia(app);
        return row![
            text(format!("Minecraft {mc}")).color(p.text_muted),
            catalogo::insignia_loader(p, &loader),
            componentes::casilla(p, "Mostrar betas y alphas", s.pruebas, |b| m(
                Accion::VersionPruebas(b)
            ))
        ]
        .spacing(espacio::S3)
        .wrap()
        .into();
    }
    let mut juegos = vec!["Todas las versiones".to_owned()];
    juegos.extend(project.game_versions.iter().rev().cloned());
    if !juegos.contains(&s.minecraft) {
        juegos.push(s.minecraft.clone());
    }
    let mut loaders = vec!["Todos los loaders".to_owned()];
    loaders.extend(project.loaders.iter().map(|l| loader(l)));
    if !loaders.contains(&s.loader) {
        loaders.push(s.loader.clone());
    }
    row![
        column![
            text("Minecraft").color(p.text_muted),
            componentes::selector(p, juegos, Some(s.minecraft.clone()), |v| m(
                Accion::VersionMinecraft(v)
            ))
        ]
        .spacing(espacio::S2),
        column![
            text("Loader").color(p.text_muted),
            componentes::selector(p, loaders, Some(s.loader.clone()), |v| m(
                Accion::VersionLoader(v)
            ))
        ]
        .spacing(espacio::S2),
        componentes::casilla(p, "Mostrar betas y alphas", s.pruebas, |b| m(
            Accion::VersionPruebas(b)
        )),
    ]
    .spacing(espacio::S3)
    .wrap()
    .into()
}
fn versiones<'a>(app: &'a App, project: &Project, versions: &'a [Version]) -> Element<'a, Mensaje> {
    versiones_tabla(app, project, versions)
}
fn versiones_tabla<'a>(
    app: &'a App,
    project: &Project,
    versions: &'a [Version],
) -> Element<'a, Mensaje> {
    let p = app.modo.paleta();
    let s = &app.servicios.ficha;
    let visible = s.visibles(versions);
    let headings = row![
        container(componentes::etiqueta(p, "Versión")).width(Length::FillPortion(4)),
        container(componentes::etiqueta(p, "Minecraft")).width(Length::FillPortion(2)),
        container(componentes::etiqueta(p, "Loader")).width(Length::FillPortion(2)),
        container(componentes::etiqueta(p, "Fecha")).width(Length::FillPortion(2)),
        container(componentes::etiqueta(p, "Descargas")).width(Length::FillPortion(2)),
        Space::with_width(128)
    ]
    .spacing(12)
    .padding(8);
    let mut body = column![
        filtros(app, project),
        headings,
        crate::diseno_mods::regla(p)
    ]
    .spacing(12);
    if s.versiones_busy {
        return body
            .push(text("Obteniendo versiones…").color(p.text_muted))
            .into();
    }
    if let Some(e) = &s.versiones_error {
        return body
            .push(text(e).color(p.warning))
            .push(componentes::boton_compacto(
                p,
                "Reintentar versiones",
                Variante::Secundario,
                Some(m(Accion::CargarVersiones)),
            ))
            .into();
    }
    for (index, v) in visible
        .iter()
        .skip(s.pagina * PAGE_SIZE)
        .take(PAGE_SIZE)
        .enumerate()
    {
        let instance = app.editor == Some(2) && index == 0 && s.pagina == 0;
        let mut version = column![button(
            text(&v.version_number)
                .font(fuentes::MONO)
                .size(texto::MONO.0)
                .color(p.text)
        )
        .padding(0)
        .on_press(m(Accion::CambiosVersion(v.id.clone())))
        .style(icaro_ui::estilo::sin_estilo(p.text))]
        .spacing(4);
        if instance {
            version = version.push(componentes::insignia(
                p,
                componentes::Estado::Exito,
                "Para tu instancia",
            ));
        }
        if v.version_type != "release" {
            version = version.push(componentes::insignia(
                p,
                componentes::Estado::Aviso,
                canal(&v.version_type),
            ));
        }
        let action = if app.editor == Some(2) {
            crate::diseno_mods::accion(app, &project.id, Some(&v.id), false)
        } else {
            componentes::boton_con_icono(
                p,
                Icono::Descargas,
                "",
                Variante::Fantasma,
                v.files
                    .iter()
                    .find(|f| f.primary)
                    .or_else(|| v.files.first())
                    .map(|f| m(Accion::Enlace(f.url.clone()))),
                true,
            )
        };
        let row = row![
            container(version).width(Length::FillPortion(4)),
            text(
                v.game_versions
                    .iter()
                    .rev()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted)
            .width(Length::FillPortion(2)),
            text(
                v.loaders
                    .iter()
                    .map(|l| loader(l))
                    .collect::<Vec<_>>()
                    .join(" · ")
            )
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted)
            .width(Length::FillPortion(2)),
            text(fecha(&v.date_published))
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted)
                .width(Length::FillPortion(2)),
            text(catalogo::numero(v.downloads))
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted)
                .width(Length::FillPortion(2)),
            container(action)
                .width(128)
                .align_x(iced::alignment::Horizontal::Right)
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center)
        .padding(8);
        body = body
            .push(container(row).width(Length::Fill).style(move |_| {
                iced::widget::container::Style {
                    background: instance.then_some(iced::Background::Color(p.surface_hover)),
                    border: iced::Border {
                        width: if instance { 1. } else { 0. },
                        color: p.border_strong,
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }))
            .push(crate::diseno_mods::regla(p));
    }
    if visible.is_empty() {
        body = body.push(text("No hay versiones para estos filtros.").color(p.text_muted));
    }
    let pages = visible.len().div_ceil(PAGE_SIZE).max(1);
    body.push(
        row![
            componentes::boton_compacto(
                p,
                "Anterior",
                Variante::Secundario,
                (s.pagina > 0).then(|| m(Accion::VersionPagina(s.pagina - 1)))
            ),
            container(
                text(format!("{} / {}", s.pagina + 1, pages))
                    .font(fuentes::MONO)
                    .color(p.text_muted)
            )
            .center_x(Length::Fill),
            componentes::boton_compacto(
                p,
                "Siguiente",
                Variante::Secundario,
                (s.pagina + 1 < pages).then(|| m(Accion::VersionPagina(s.pagina + 1)))
            )
        ]
        .align_y(iced::Alignment::Center),
    )
    .into()
}
fn cambios<'a>(
    app: &'a App,
    project: &Project,
    versions: &'a [Version],
    dependencias: bool,
) -> Element<'a, Mensaje> {
    let p = app.modo.paleta();
    let s = &app.servicios.ficha;
    let visibles = s.visibles(versions);
    let opciones: Vec<_> = visibles.iter().map(|v| opcion(v)).collect();
    let selected = opciones
        .iter()
        .find(|v| Some(&v.id) == s.seleccionada.as_ref())
        .cloned();
    let mut body = column![
        filtros(app, project),
        text("Versión del proyecto").color(p.text_muted),
        componentes::selector(p, opciones, selected, |v| m(Accion::CambiosSeleccionar(
            v.id
        )))
    ]
    .spacing(espacio::S4)
    .width(Length::Fill);
    if s.versiones_busy {
        return body
            .push(text("Obteniendo versiones…").color(p.text_muted))
            .into();
    }
    if let Some(error) = &s.versiones_error {
        return body
            .push(text(error).color(p.warning))
            .push(componentes::boton(
                p,
                "Reintentar versiones",
                Variante::Secundario,
                Some(m(Accion::CargarVersiones)),
            ))
            .into();
    }
    if dependencias {
        if app.servicios.dependencias.values().any(|v| v.is_err()) {
            body = body.push(componentes::boton(
                p,
                "Reintentar nombres de dependencias",
                Variante::Fantasma,
                Some(m(Accion::ReintentarDependencias)),
            ));
        }
        if let Some(v) = visibles
            .iter()
            .find(|v| Some(&v.id) == s.seleccionada.as_ref())
        {
            if v.dependencies.is_empty() {
                body = body.push(
                    text("Esta versión no declara dependencias en Modrinth.").color(p.text_muted),
                );
            }
            for dep in &v.dependencies {
                let key = crate::servicios::clave_dependencia(dep);
                let resolved = app.servicios.dependencias.get(&key);
                let name = match resolved {
                    Some(Ok((_, name))) => name.as_str(),
                    Some(Err(_)) => dep
                        .file_name
                        .as_deref()
                        .unwrap_or("No se pudo obtener el nombre"),
                    None => {
                        if dep.project_id.is_some() || dep.version_id.is_some() {
                            "Obteniendo nombre…"
                        } else {
                            dep.file_name.as_deref().unwrap_or("Dependencia externa")
                        }
                    }
                };
                let tipo = match dep.dependency_type.as_str() {
                    "required" => "Obligatoria",
                    "optional" => "Opcional",
                    "incompatible" => "Incompatible",
                    "embedded" => "Incluida",
                    x => x,
                };
                let mut d = column![
                    text(tipo).font(fuentes::TITULO).color(p.text),
                    text(name).color(p.text_muted)
                ]
                .spacing(espacio::S2);
                if let Some(id) = resolved
                    .and_then(|r| r.as_ref().ok())
                    .map(|(id, _)| id)
                    .or(dep.project_id.as_ref())
                {
                    d = d.push(componentes::boton_con_icono(
                        p,
                        Icono::Externo,
                        "Ver dependencia",
                        Variante::Fantasma,
                        Some(m(Accion::Proyecto(id.clone()))),
                        false,
                    ));
                }
                body = body.push(
                    container(d)
                        .width(Length::Fill)
                        .padding(espacio::S4)
                        .style(icaro_ui::estilo::marco(p)),
                );
            }
        } else {
            body = body.push(text("No hay una versión seleccionada.").color(p.text_muted));
        }
    } else if s.cambios_busy {
        body = body.push(text("Obteniendo cambios de esta versión…").color(p.text_muted));
    } else if let Some(error) = &s.cambios_error {
        body = body
            .push(text(error).color(p.warning))
            .push(componentes::boton(
                p,
                "Reintentar cambios",
                Variante::Secundario,
                s.seleccionada
                    .clone()
                    .map(|id| m(Accion::CambiosSeleccionar(id))),
            ));
    } else if let Some(v) = &s.cambio {
        body = body.push(text(&v.name).font(fuentes::TITULO).color(p.text));
        if v.changelog.as_deref().is_none_or(|c| c.trim().is_empty()) {
            body = body
                .push(text("El creador no publicó cambios para esta versión.").color(p.text_muted));
        } else {
            body = body.push(s.documento_cambios.vista(p, &app.servicios.images));
        }
    } else {
        body = body.push(text("No hay versiones para estos filtros.").color(p.text_muted));
    }
    body.into()
}
fn galeria<'a>(app: &'a App, project: &'a Project) -> Element<'a, Mensaje> {
    let p = app.modo.paleta();
    let s = &app.servicios.ficha;
    let images = &app.servicios.images;
    if project.gallery.is_empty() {
        return text("El creador no ha publicado imágenes en la galería.")
            .color(p.text_muted)
            .into();
    }
    let sel = s.galeria.min(project.gallery.len() - 1);
    let g = &project.gallery[sel];
    let principal = images.portada_alta(&g.url, 340.).unwrap_or_else(|| {
        container(
            text(if images.pendiente(&g.url) {
                "Cargando imagen…"
            } else {
                "Esta imagen no está disponible."
            })
            .color(p.text_muted),
        )
        .width(Length::Fill)
        .height(340)
        .center_x(Length::Fill)
        .center_y(340)
        .style(icaro_ui::estilo::marco(p))
        .into()
    });
    let principal = button(principal)
        .padding(0)
        .width(Length::Fill)
        .on_press(m(Accion::VisorAbrir(sel)))
        .style(icaro_ui::estilo::sin_estilo(p.text));
    let inicio = sel / GALLERY_PAGE * GALLERY_PAGE;
    let mut thumbs = row![].spacing(espacio::S3);
    for (i, g) in project
        .gallery
        .iter()
        .enumerate()
        .skip(inicio)
        .take(GALLERY_PAGE)
    {
        thumbs = thumbs.push(
            button(
                column![
                    images.vista(p, Some(&g.url), 80.),
                    text(format!("{}", i + 1)).size(13)
                ]
                .spacing(espacio::S1),
            )
            .padding(espacio::S2)
            .on_press(m(Accion::VisorAbrir(i)))
            .style(move |theme, status| {
                if i == sel {
                    icaro_ui::estilo::boton_primario(p)(theme, status)
                } else {
                    icaro_ui::estilo::boton_secundario(p)(theme, status)
                }
            }),
        );
    }
    let mut body = column![
        principal,
        text(g.title.as_deref().unwrap_or("Imagen del proyecto"))
            .font(fuentes::TITULO)
            .color(p.text)
    ]
    .spacing(espacio::S3)
    .width(Length::Fill);
    if let Some(description) = &g.description {
        body = body.push(text(description).color(p.text_muted));
    }
    body.push(
        row![
            componentes::boton(
                p,
                "Anterior",
                Variante::Secundario,
                (sel > 0).then(|| m(Accion::Galeria(sel - 1)))
            ),
            container(text(format!("{} / {}", sel + 1, project.gallery.len())).color(p.text_muted))
                .center_x(56),
            componentes::boton(
                p,
                "Siguiente",
                Variante::Secundario,
                (sel + 1 < project.gallery.len()).then(|| m(Accion::Galeria(sel + 1)))
            ),
        ]
        .spacing(espacio::S3)
        .align_y(iced::Alignment::Center),
    )
    .push(enlace(p, "Abrir original", g.original_url()))
    .push(thumbs.wrap())
    .into()
}

pub fn visor<'a>(app: &'a App, project: &Project) -> Option<Element<'a, Mensaje>> {
    let s = &app.servicios.ficha;
    let state = s.visor.as_ref()?;
    let image = project.gallery.get(state.indice)?;
    let full = s
        .visor_imagen
        .as_ref()
        .filter(|(url, _)| url == image.original_url())
        .map(|(_, i)| i.handle());
    let detail = if s.visor_busy {
        format!("{} · Cargando imagen…", project.title)
    } else if let Some(error) = &s.visor_error {
        format!("{} · {}", project.title, error)
    } else {
        image
            .description
            .clone()
            .unwrap_or_else(|| project.title.clone())
    };
    let item = icaro_ui::visor::ItemImagenVisor {
        titulo: image
            .title
            .clone()
            .unwrap_or_else(|| format!("Imagen {}", state.indice + 1)),
        detalle: detail,
        handle: full,
        aviso: s
            .visor_error
            .clone()
            .unwrap_or_else(|| "Cargando imagen…".into()),
    };
    Some(icaro_ui::visor::visor_imagen(
        state,
        &item,
        project.gallery.len(),
        icaro_ui::visor::MensajesVisor {
            cerrar: m(Accion::VisorCerrar),
            anterior: m(Accion::VisorPaso(-1)),
            siguiente: m(Accion::VisorPaso(1)),
            acercar: m(Accion::VisorZoom(true)),
            alejar: m(Accion::VisorZoom(false)),
            copiar: None,
            carpeta: None,
            eliminar: None,
        },
    ))
}

pub fn vista<'a>(
    app: &'a App,
    project: &'a Project,
    versions: &'a [Version],
    kind: Kind,
) -> Element<'a, Mensaje> {
    let p = app.modo.paleta();
    let s = &app.servicios.ficha;
    let mut tabs = vec![
        Pestana::Descripcion,
        Pestana::Galeria,
        Pestana::Versiones,
        Pestana::Cambios,
    ];
    if kind != Kind::Modpack {
        tabs.push(Pestana::Dependencias);
    }
    let dependencias = versions
        .iter()
        .find(|v| Some(&v.id) == s.seleccionada.as_ref())
        .map_or(0, |v| v.dependencies.len());
    let nombres: Vec<_> = tabs
        .iter()
        .map(|t| {
            (
                t.nombre(),
                match t {
                    Pestana::Galeria => Some(project.gallery.len()),
                    Pestana::Versiones => Some(versions.len()),
                    Pestana::Dependencias => Some(dependencias),
                    _ => None,
                },
            )
        })
        .collect();
    let activa = tabs.iter().position(|t| *t == s.tab).unwrap_or(0);
    let barra = componentes::pestanas(p, &nombres, activa, move |i| {
        m(Accion::FichaPestana(tabs[i]))
    });
    let body = match s.tab {
        Pestana::Descripcion => app.servicios.document.vista(p, &app.servicios.images),
        Pestana::Galeria => galeria(app, project),
        Pestana::Versiones => versiones(app, project, versions),
        Pestana::Cambios => cambios(app, project, versions, false),
        Pestana::Dependencias => cambios(app, project, versions, true),
    };
    let main = container(crate::con_entrada(app, body)).width(Length::Fill);
    let side = lateral(app, project);
    let creators = s
        .miembros
        .iter()
        .map(|m| m.user.username.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let metadata = text(format!(
        "{}{}{}",
        creators,
        if creators.is_empty() { "" } else { " · " },
        project
            .categories
            .iter()
            .take(3)
            .map(|c| crate::diseno_mods::categoria(c).to_owned())
            .collect::<Vec<_>>()
            .join(" · ")
    ))
    .size(texto::BODY_SM.0)
    .color(p.text_muted);
    let header = row![
        app.servicios
            .images
            .vista(p, project.icon_url.as_deref(), 48.),
        column![
            text(project.title.to_uppercase())
                .font(fuentes::DISPLAY)
                .size(40)
                .color(p.text),
            metadata,
        ]
        .spacing(espacio::S3)
        .width(Length::Fill)
    ]
    .spacing(espacio::S5);
    let cabecera = container(
        column![
            componentes::boton_con_icono(
                p,
                Icono::ChevronIzquierda,
                if app.servicios.hay_anterior() {
                    "Volver"
                } else if app.editor.is_some() && !app.servicios.hay_catalogo() {
                    "Volver a mis mods"
                } else {
                    "Volver al catálogo"
                },
                Variante::Fantasma,
                Some(m(Accion::Volver)),
                false,
            ),
            header,
            barra,
        ]
        .spacing(espacio::S6)
        .width(Length::Fill),
    )
    .padding(Padding {
        top: espacio::S6,
        right: espacio::S6,
        bottom: 0.0,
        left: espacio::S6,
    })
    .width(Length::Fill);
    // La navegación queda fuera del área desplazable. Al cambiar de pestaña,
    // una clave nueva reinicia también el desplazamiento suave de su contenido.
    let contenido = keyed_column([(
        s.tab,
        crate::desplazable_pub(
            container(main)
                .padding(espacio::S6)
                .width(Length::Fill)
                .into(),
        ),
    )])
    .width(Length::Fill)
    .height(Length::Fill);
    let left = column![cabecera, contenido]
        .width(Length::Fill)
        .height(Length::Fill);
    let compatible: Element<Mensaje> = if let Some(i) = crate::mods_reales::instancia(app) {
        let ok =
            project.game_versions.contains(&i.minecraft) && project.loaders.contains(&i.loader);
        componentes::aviso(
            p,
            if ok {
                componentes::Estado::Exito
            } else {
                componentes::Estado::Aviso
            },
            &format!(
                "{} con {} ({}, {}).",
                if ok { "Compatible" } else { "No compatible" },
                i.nombre,
                i.minecraft,
                loader(&i.loader)
            ),
        )
    } else {
        Space::with_height(0).into()
    };
    row![
        left,
        container(row![
            container(Space::new(1, Length::Fill)).style(icaro_ui::estilo::bloque(p.border)),
            container(
                column![
                    if app.editor == Some(2) {
                        crate::diseno_mods::accion(app, &project.id, None, true)
                    } else {
                        componentes::boton_con_icono(
                            p,
                            Icono::Externo,
                            "Ver en Modrinth",
                            Variante::Primario,
                            Some(m(Accion::Enlace(format!(
                                "https://modrinth.com/project/{}",
                                project.id
                            )))),
                            false,
                        )
                    },
                    compatible,
                    crate::desplazable_pub(side)
                ]
                .spacing(20)
            )
            .padding(Padding {
                left: 24.,
                ..Padding::ZERO
            })
            .width(Length::Fill)
        ])
        .padding(Padding {
            top: 24.,
            right: 24.,
            bottom: 24.,
            left: 0.
        })
        .width(300)
        .height(Length::Fill)
    ]
    .spacing(16)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn version(id: &str, channel: &str) -> Version {
        serde_json::from_value(serde_json::json!({
            "id": id, "project_id": "pack", "name": id, "version_number": "1.0", "version_type": channel,
            "game_versions": ["1.21.4"], "loaders": ["fabric"], "files": [], "dependencies": [],
            "date_published": "2026-10-11T03:00:00Z"
        })).unwrap()
    }
    #[test]
    fn channel_filter_invalidates_hidden_selection_and_clamps_pages() {
        let stable = version("stable", "release");
        let beta = version("beta", "beta");
        let versions = vec![beta.clone(), stable.clone()];
        let mut s = Estado::nuevo(None, &versions);
        assert_eq!(s.seleccionada.as_deref(), Some("stable"));
        s.pruebas = true;
        s.seleccionada = Some(beta.id.clone());
        s.cambio = Some(beta);
        let cancel = CancellationToken::new();
        s.cambios_cancel = Some(cancel.clone());
        s.pagina = 20;
        s.pruebas = false;
        s.elegir_predeterminada(&versions);
        assert_eq!(s.seleccionada.as_deref(), Some("stable"));
        assert!(s.cambio.is_none() && cancel.is_cancelled());
        assert_eq!(s.pagina, 0);
        let many = vec![stable; PAGE_SIZE * 2 + 1];
        s.pagina = 3;
        s.elegir_predeterminada(&many);
        assert_eq!(s.pagina, 2);
        s.elegir_predeterminada(&[]);
        assert!(s.seleccionada.is_none());
        assert_eq!(s.pagina, 0);
    }
    #[test]
    fn gallery_fetches_only_the_visible_page_and_changes_fetch_their_own_media() {
        let project: Project = serde_json::from_value(serde_json::json!({
            "id":"pack", "slug":"pack", "title":"Pack", "description":"Resumen", "body":"", "downloads":1,
            "license":{"id":"MIT","name":"MIT"}, "icon_url":"https://cdn.modrinth.com/data/pack/icon.png",
            "gallery": (0..12).map(|i| serde_json::json!({"url":format!("https://cdn.modrinth.com/data/pack/{i}.png"), "featured":false})).collect::<Vec<_>>()
        })).unwrap();
        let description = catalogo::Documento::parse(
            "![Descripción](https://cdn.modrinth.com/data/pack/body.png)",
        );
        let mut s = Estado::default();
        s.tab = Pestana::Galeria;
        s.galeria = 7;
        let images = s.imagenes(&project, &description);
        assert_eq!(images.len(), 7);
        assert!(images.iter().any(|url| url.ends_with("/7.png")));
        assert!(!images
            .iter()
            .any(|url| url.ends_with("/0.png") || url.ends_with("/body.png")));
        s.tab = Pestana::Cambios;
        s.documento_cambios =
            catalogo::Documento::parse("![Cambio](https://cdn.modrinth.com/data/pack/change.png)");
        assert!(s
            .imagenes(&project, &description)
            .iter()
            .any(|url| url.ends_with("/change.png")));
        assert_eq!(fecha("2026-10-11T03:00:00Z"), "11-10-2026");
        assert_eq!(fecha("dato inválido"), "Sin fecha");
        assert_eq!(loader("NEOFORGE"), "NeoForge");
    }
}
