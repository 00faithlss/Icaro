// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Fichas públicas de mods y modpacks, con la estructura del sistema de diseño.

use crate::catalogo::texto_publico as text;
use crate::{catalogo, servicios::Accion, App, Mensaje};
use icaro_core::{
    auth::CancellationToken,
    modrinth::{Kind, Member, Organization, Project, RelatedProject, Search, Version},
};
use icaro_ui::{
    componentes::{self, Variante},
    fuentes,
    iconos::Icono,
    tema::{espacio, texto},
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
    pub info_snapshots: bool,
    pub mostrar_todas: bool,
    pub relacionados: Vec<RelatedProject>,
    pub relacionados_busy: bool,
    pub relacionados_error: Option<String>,
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
            info_snapshots: false,
            mostrar_todas: false,
            relacionados: vec![],
            relacionados_busy: false,
            relacionados_error: None,
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
        "iris" => "Iris".into(),
        "optifine" => "OptiFine".into(),
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
fn lateral<'a>(app: &'a App, project: &'a Project) -> Element<'a, Mensaje> {
    column![crate::info_proyecto::vista(app, project, 287.),crate::info_proyecto::mas_creador(app)].spacing(28).width(Length::Fill).into()
}
fn filtros<'a>(app: &'a App, project: &Project) -> Element<'a, Mensaje> {
    let p=app.modo.paleta(); let s=&app.servicios.ficha;
    let bound=app.editor.is_some() && !s.mostrar_todas;
    let (mc,current_loader)=if app.editor.is_some() {crate::mods_reales::instancia(app).map(|i|(i.minecraft.clone(),i.loader.clone())).unwrap_or_else(||crate::juego_de_instancia(app))}else{(s.minecraft.clone(),s.loader.clone())};
    let mut games=if bound {vec![mc.clone()]} else {vec!["Todas las versiones".to_owned()]};
    if !bound {games.extend(project.game_versions.iter().rev().cloned());}
    let game=if bound {mc} else {s.minecraft.clone()};
    if !games.contains(&game){games.push(game.clone());}
    let mut loaders=row![].spacing(6);
    if !matches!(project.project_type.as_str(),"resourcepack"|"shader") {
        for id in ["fabric","quilt","neoforge","forge"] {
            if project.loaders.iter().any(|l|l==id) {
                let selected=if bound {current_loader.eq_ignore_ascii_case(id)} else {s.loader.eq_ignore_ascii_case(id)};
                loaders=loaders.push(componentes::boton_compacto(p,&loader(id),if selected{Variante::Primario}else{Variante::Secundario},(!bound).then(||m(Accion::VersionLoader(loader(id))))));
            }
        }
    }
    let filters=row![container(componentes::selector(p,games,Some(game),|v|m(Accion::VersionMinecraft(v)))).width(170),loaders.wrap(),Space::with_width(Length::Fill),componentes::casilla(p,"Mostrar beta y alpha",s.pruebas,|b|m(Accion::VersionPruebas(b)))].spacing(12).align_y(iced::Alignment::Center).wrap();
    let mut body=column![filters].spacing(16);
    if app.editor.is_some() { body=body.push(componentes::boton_compacto(p,if s.mostrar_todas{"Filtrar para mi instancia"}else{"Mostrar todas"},Variante::Fantasma,Some(m(Accion::MostrarTodas(!s.mostrar_todas))))); }
    body.into()
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
    let cabecera = |label: &str| text(label.to_uppercase()).font(fuentes::ETIQUETA)
        .size(texto::LABEL.0).color(p.text_muted).wrapping(iced::widget::text::Wrapping::None);
    let headings = row![
        container(cabecera("Versión")).width(Length::FillPortion(11)),
        container(cabecera("Minecraft")).width(Length::FillPortion(6)),
        container(cabecera("Loader")).width(Length::FillPortion(4)),
        container(cabecera("Fecha")).width(Length::FillPortion(7)),
        container(cabecera("Descargas")).width(Length::FillPortion(6)),
        Space::with_width(128)
    ]
    .spacing(8)
    .padding(8);
    let mut body = column![filtros(app, project)].spacing(12);
    let mut tabla = column![headings,crate::diseno_mods::regla(p)].spacing(0);
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
    for v in visible
        .iter()
        .skip(s.pagina * PAGE_SIZE)
        .take(PAGE_SIZE)
    {
        let instance=crate::mods_reales::instancia(app).is_some_and(|i|v.game_versions.contains(&i.minecraft)&&(matches!(project.project_type.as_str(),"resourcepack"|"shader")||v.loaders.contains(&i.loader)));
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
                if v.version_type=="alpha" {componentes::Estado::Error} else {componentes::Estado::Aviso},
                canal(&v.version_type),
            ));
        }
        let action = if app.editor == Some(2) {
            crate::diseno_mods::accion(app, &project.id, Some(&v.id), false)
        } else if app.editor == Some(3) && project.project_type == "resourcepack" {
            crate::mods_reales::accion_recurso(app, &project.id, Some(&v.id))
        } else if app.editor == Some(8) && project.project_type == "shader" {
            crate::mods_reales::packs::accion_instalar(app, icaro_core::mods::TipoPack::Shaders, &project.id, Some(&v.id))
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
            container(version).width(Length::FillPortion(11)),
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
            .width(Length::FillPortion(6)),
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
            .width(Length::FillPortion(4)),
            text(fecha(&v.date_published))
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted)
                .width(Length::FillPortion(7)),
            text(if v.downloads>=1_000_000 {format!("{:.1} M",v.downloads as f64/1_000_000.).replace('.',",")}else if v.downloads>=1_000 {format!("{:.0} k",v.downloads as f64/1_000.)}else{catalogo::numero(v.downloads)})
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted)
                .width(Length::FillPortion(6)),
            container(action)
                .width(128)
                .align_x(iced::alignment::Horizontal::Right)
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .padding(8);
        let decorated=container(row).width(Length::Fill).style(move |_| {
                iced::widget::container::Style {
                    background: instance.then_some(iced::Background::Color(p.accent_soft)),
                    border: iced::Border {
                        width: 0.,
                        color: p.border_strong,
                        ..Default::default()
                    },
                    ..Default::default()
                }
            });
        let decorated: Element<Mensaje> = if instance { iced::widget::stack![decorated,container(Space::new(3,Length::Fill)).style(icaro_ui::estilo::bloque(p.text))].into() }else{decorated.into()};
        tabla = tabla
            .push(decorated)
            .push(crate::diseno_mods::regla(p));
    }
    body=body.push(tabla);
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
fn dependencias<'a>(app:&'a App,versions:&'a [Version])->Element<'a,Mensaje>{
    let p=app.modo.paleta();let s=&app.servicios.ficha;
    let pack=app.servicios.detail.as_ref().is_some_and(|(p,_)|matches!(p.project_type.as_str(),"resourcepack"|"shader"));
    let Some(version)=versions.iter().find(|v|Some(&v.id)==s.seleccionada.as_ref())else{return text("No hay una versión seleccionada.").color(p.text_muted).into();};
    if version.dependencies.is_empty(){return text("Esta versión no declara dependencias en Modrinth.").color(p.text_muted).into();}
    let mut body=column![].spacing(24).max_width(680);
    for (kind,title) in [("required","Obligatorias"),("optional","Opcionales"),("incompatible","Incompatibles"),("embedded","Incluidas")]{
        let deps:Vec<_>=version.dependencies.iter().filter(|d|d.dependency_type==kind).collect();if deps.is_empty(){continue;}
        let mut group=column![componentes::etiqueta(p,title),crate::diseno_mods::regla(p)].spacing(0);
        for dep in deps{
            let resolved=app.servicios.dependencias.get(&crate::servicios::clave_dependencia(dep)).and_then(|r|r.as_ref().ok());
            let id=resolved.map(|(id,_)|id).or(dep.project_id.as_ref());
            let name=resolved.map(|(_,name)|name.as_str()).or(dep.file_name.as_deref()).unwrap_or("Obteniendo nombre…");
            let metadata=id.and_then(|id|app.servicios.mods.diseno.proyectos.get(id));
            let installed=id.is_some_and(|id|crate::mods_reales::lista(app).iter().any(|m|m.project_id==*id&&m.activo));
            let en_lista=id.is_some_and(|id|crate::diseno_mods::cesta(app).is_some_and(|c|c.plan.iter().any(|(v,_)|v.project_id==*id)));
            let mut label=column![button(text(name).font(fuentes::TITULO).size(14).color(p.text)).padding(0).on_press_maybe(id.map(|id|m(Accion::Proyecto(id.clone())))).style(icaro_ui::estilo::sin_estilo(p.text))].spacing(2).width(Length::Fill);
            if let Some((_,author))=metadata{if !author.is_empty(){label=label.push(text(author).size(13).color(p.text_muted));}}
            let state=if kind=="optional"&&!installed&&!pack {
                if let Some(id)=id {let id=id.clone();let v=dep.version_id.clone();componentes::casilla(p,"",en_lista,move|selected|Mensaje::DisenoMods(if selected{crate::diseno_mods::Accion::Agregar(id.clone(),v.clone())}else{crate::diseno_mods::Accion::Quitar(id.clone())}))}else{componentes::insignia_contorno(p,"Opcional")}
            }else{componentes::insignia(p,if kind=="incompatible"{componentes::Estado::Error}else if installed{componentes::Estado::Exito}else{componentes::Estado::Info},if kind=="incompatible"{"No juntos"}else if kind=="embedded"{"Incluida"}else if installed{"Instalada"}else if kind=="optional"{"Opcional"}else if pack{"Obligatoria"}else{"Se instalará"})};
            group=group.push(row![app.servicios.images.vista(p,metadata.and_then(|(p,_)|p.icon_url.as_deref()),32.),label,state].spacing(12).align_y(iced::Alignment::Center).padding([12,0])).push(crate::diseno_mods::regla(p));
        }
        body=body.push(group);
    }
    if app.servicios.dependencias.values().any(|d|d.is_err()){body=body.push(componentes::boton_compacto(p,"Reintentar nombres",Variante::Fantasma,Some(m(Accion::ReintentarDependencias))));}
    body.into()
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
    let p=app.modo.paleta(); let s=&app.servicios.ficha;
    let start=s.galeria/GALLERY_PAGE*GALLERY_PAGE;
    let cell_width=((app.ventana.width-220.-96.-320.-48.).max(240.)-32.)/3.;
    let mut body=column![].spacing(16).width(Length::Fill);
    let mut line=row![].spacing(16);let mut units=0;
    for (i,image) in project.gallery.iter().enumerate().skip(start).take(GALLERY_PAGE) {
        let span=if image.featured{2}else{1};
        if units+span>3 {if units<3{line=line.push(Space::with_width(Length::FillPortion(3-units)));} body=body.push(line);line=row![].spacing(16);units=0;}
        let height=(cell_width*span as f32+16.*(span-1) as f32)*9./16.;
        let preview=app.servicios.images.portada_alta(&image.url,height).unwrap_or_else(||container(text(if app.servicios.images.pendiente(&image.url){"Cargando imagen…"}else{"Imagen no disponible"}).size(13).color(p.text_muted)).width(Length::Fill).height(height).center_x(Length::Fill).center_y(height).into());
        let mut card=column![button(container(preview).width(Length::Fill).height(height).padding(1).style(icaro_ui::estilo::marco(p))).padding(0).width(Length::Fill).on_press(m(Accion::VisorAbrir(i))).style(icaro_ui::estilo::sin_estilo(p.text))].spacing(8);
        if let Some(title)=&image.title{card=card.push(text(title).size(13).color(p.text_muted));}
        line=line.push(card.width(Length::FillPortion(span)));units+=span;
    }
    if units>0{if units<3{line=line.push(Space::with_width(Length::FillPortion(3-units)));}body=body.push(line);}
    if project.gallery.len()>GALLERY_PAGE {body=body.push(row![componentes::boton_compacto(p,"Anterior",Variante::Secundario,(start>0).then(||m(Accion::Galeria(start.saturating_sub(GALLERY_PAGE))))),Space::with_width(Length::Fill),componentes::boton_compacto(p,"Siguiente",Variante::Secundario,(start+GALLERY_PAGE<project.gallery.len()).then(||m(Accion::Galeria(start+GALLERY_PAGE))))].spacing(16));}
    body.into()
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
        Pestana::Versiones,
        Pestana::Cambios,
    ];
    if !project.gallery.is_empty() { tabs.insert(1, Pestana::Galeria); }
    if kind != Kind::Modpack {
        tabs.push(Pestana::Dependencias);
    }
    let numero_dependencias = versions
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
                    Pestana::Dependencias => Some(numero_dependencias),
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
        Pestana::Dependencias => dependencias(app, versions),
    };
    let main = container(crate::con_entrada(app, body)).width(Length::Fill).max_width(if s.tab==Pestana::Descripcion { 680 } else { 10000 });
    let side = lateral(app, project);
    let mut metadata=row![].spacing(6).align_y(iced::Alignment::Center);
    let creator=s.organizacion.as_ref().map(|o|(o.name.as_str(),format!("https://modrinth.com/organization/{}",o.id))).or_else(||s.miembros.iter().find(|m|m.is_owner).or_else(||s.miembros.first()).map(|m|(m.user.name.as_deref().unwrap_or(&m.user.username),format!("https://modrinth.com/user/{}",m.user.id))));
    if let Some((name,url))=creator {metadata=metadata.push(text("por").size(13).color(p.text_muted)).push(button(text(name.to_owned()).size(13).color(p.text)).padding(0).on_press(m(Accion::Enlace(url))).style(icaro_ui::estilo::sin_estilo(p.text)));}
    for category in project.categories.iter().take(3){metadata=metadata.push(text(format!("· {}",crate::diseno_mods::categoria(category))).size(13).color(p.text_muted));}
    let lado=match (project.client_side.as_str(),project.server_side.as_str()) {("required","required")=>Some("Ambos"),("unsupported","required")=>Some("Servidor"),("required","unsupported")=>Some("Cliente"),_=>None};
    if let Some(lado)=lado {metadata=metadata.push(text(format!("· {lado}")).size(13).color(p.text_muted));}
    let metadata=metadata.wrap();
    let header = row![
        app.servicios
            .images
            .vista(p, project.icon_url.as_deref(), 64.),
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
    .spacing(20)
    .align_y(iced::Alignment::End);
    let cabecera = container(
        column![
            componentes::boton_con_icono(
                p,
                Icono::ChevronIzquierda,
                if app.servicios.hay_anterior() {
                    "Volver"
                } else if app.editor.is_some() && !app.servicios.hay_catalogo() {
                    match app.editor {
                        Some(3) => "Volver a mis recursos",
                        Some(8) => "Volver a mis shaders",
                        _ => "Volver a mis mods",
                    }
                } else {
                    "Volver al catálogo"
                },
                Variante::Fantasma,
                Some(m(Accion::Volver)),
                false,
            ),
            header,
            text(&project.description).size(texto::BODY.0).color(p.text_muted),
            barra,
        ]
        .spacing(espacio::S6)
        .width(Length::Fill),
    )
    .padding(Padding {
        top: 0.,
        right: 0.,
        bottom: 0.0,
        left: 0.,
    })
    .width(Length::Fill);
    // La navegación queda fuera del área desplazable. Al cambiar de pestaña,
    // una clave nueva reinicia también el desplazamiento suave de su contenido.
    let ancho = app.ventana.width - if app.colapsada {64.} else {232.} - 96. - 320. - 48.;
    let contenido_principal = if s.tab==Pestana::Versiones && ancho<680. {
        // Una sola área de desplazamiento conserva la medida mínima de las
        // columnas sin partir encabezados, fechas o acciones en dos líneas.
        iced::widget::scrollable(container(main).padding(Padding {top:28.,..Padding::ZERO}).width(680))
            .direction(iced::widget::scrollable::Direction::Both {
                vertical:iced::widget::scrollable::Scrollbar::new().width(0).scroller_width(0),
                horizontal:iced::widget::scrollable::Scrollbar::new().width(8).scroller_width(8),
            }).width(Length::Fill).height(Length::Fill).into()
    }else{crate::desplazable_pub(container(main).padding(Padding {top:28.,..Padding::ZERO}).width(Length::Fill).into())};
    let contenido = keyed_column([(s.tab,contenido_principal)])
    .width(Length::Fill)
    .height(Length::Fill);
    let left = column![cabecera, contenido]
        .width(Length::Fill)
        .height(Length::Fill);
    let compatible: Element<Mensaje> = if let Some(i) = crate::mods_reales::instancia(app) {
        let resource = matches!(project.project_type.as_str(), "resourcepack" | "shader");
        let ok = project.game_versions.contains(&i.minecraft)
            && (resource || project.loaders.contains(&i.loader));
        componentes::aviso(
            p,
            if ok {
                componentes::Estado::Exito
            } else {
                componentes::Estado::Aviso
            },
            &if resource {
                if ok {
                    format!(
                        "Compatible con Minecraft {}. No requiere un loader específico.",
                        i.minecraft
                    )
                } else {
                    format!(
                        "No declara Minecraft {}. Puedes instalarlo y probarlo igualmente.",
                        i.minecraft
                    )
                }
            } else {
                format!(
                    "{} con {} ({}, {}).",
                    if ok { "Compatible" } else { "No compatible" },
                    i.nombre,
                    i.minecraft,
                    loader(&i.loader)
                )
            },
        )
    } else {
        Space::with_height(0).into()
    };
    container(row![
        left,
        container(row![
            container(Space::new(1, Length::Fill)).style(icaro_ui::estilo::bloque(p.border)),
            container(
                column![
                    if app.editor == Some(2) {
                        crate::diseno_mods::accion(app, &project.id, None, true)
                    } else if app.editor == Some(3) && project.project_type == "resourcepack" {
                        crate::mods_reales::accion_recurso(app, &project.id, None)
                    } else if app.editor == Some(8) && project.project_type == "shader" {
                        crate::mods_reales::packs::accion_instalar(app, icaro_core::mods::TipoPack::Shaders, &project.id, None)
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
                left: 32.,
                ..Padding::ZERO
            })
            .width(Length::Fill)
        ])
        .padding(Padding {
            top: 0.,
            right: 0.,
            bottom: 0.,
            left: 0.
        })
        .width(320)
        .height(Length::Fill)
    ]
    .spacing(48)
    .width(Length::Fill)
    .height(Length::Fill))
    .padding([40,48])
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
