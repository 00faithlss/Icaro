// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss
//! InfoProyecto del paquete de componentes del usuario; solo datos publicados.
use crate::{catalogo, ficha, servicios::Accion, App, Mensaje};
use icaro_core::{manifest::VersionEntry, modrinth::Project};
use icaro_ui::{
    componentes::{self, Estado, Variante},
    fuentes,
    iconos::{icono, Icono, Tam},
    tema::Paleta,
};
use iced::{
    widget::{button, column, container, row, text, Space},
    Alignment, Element, Length,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, PartialEq, Eq)]
pub struct VersionChip {
    pub label: String,
    pub members: Vec<String>,
}
fn family(id: &str) -> Option<String> {
    let parts: Vec<_> = id.split('.').collect();
    (parts.len() >= 2
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|c| c.is_ascii_digit())))
    .then(|| format!("{}.{}", parts[0], parts[1]))
}
fn release(id: &str, manifest: &[VersionEntry]) -> bool {
    manifest
        .iter()
        .find(|e| e.id == id)
        .map_or_else(|| family(id).is_some(), |e| e.kind == "release")
}
/// Una familia solo se comprime cuando el manifiesto oficial completo demuestra
/// que no falta ninguna release. Las tres releases más nuevas siguen sueltas.
pub fn version_chips(
    versions: &[String],
    manifest: &[VersionEntry],
    snapshots: bool,
) -> Vec<VersionChip> {
    let mut sorted: Vec<_> = versions
        .iter()
        .filter(|v| snapshots || release(v, manifest))
        .cloned()
        .collect();
    sorted.sort_by(|a, b| {
        let index = |id: &str| manifest.iter().position(|e| e.id == id);
        match (index(a), index(b)) {
            (Some(a), Some(b)) => a.cmp(&b),
            _ => {
                let key = |id: &str| {
                    id.split('.')
                        .map(|s| s.parse::<u32>().unwrap_or(0))
                        .collect::<Vec<_>>()
                };
                key(b).cmp(&key(a)).then_with(|| b.cmp(a))
            }
        }
    });
    sorted.dedup();
    let newest: BTreeSet<_> = sorted
        .iter()
        .filter(|id| release(id, manifest))
        .take(3)
        .cloned()
        .collect();
    let supported: BTreeSet<_> = versions.iter().cloned().collect();
    let mut official: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for entry in manifest.iter().filter(|e| e.kind == "release") {
        if let Some(f) = family(&entry.id) {
            official.entry(f).or_default().push(entry.id.clone());
        }
    }
    let mut emitted = BTreeSet::new();
    let mut chips = vec![];
    for id in sorted {
        let f = family(&id);
        let group = f.as_ref().and_then(|f| official.get(f)).filter(|members| {
            members.len() > 1
                && members
                    .iter()
                    .all(|v| supported.contains(v) && !newest.contains(v))
        });
        if let (Some(f), Some(members)) = (f, group) {
            if emitted.insert(f.clone()) {
                chips.push(VersionChip {
                    label: format!("{f}.x"),
                    members: members.clone(),
                });
            }
        } else {
            chips.push(VersionChip {
                label: id.clone(),
                members: vec![id],
            });
        }
    }
    chips
}
fn section<'a>(p: Paleta, title: &str, body: Element<'a, Mensaje>) -> Element<'a, Mensaje> {
    column![
        crate::diseno_mods::regla(p),
        container(column![componentes::etiqueta(p, title), body].spacing(12)).padding(
            iced::Padding {
                top: 14.,
                ..iced::Padding::ZERO
            }
        )
    ]
    .spacing(0)
    .width(Length::Fill)
    .into()
}
fn glyph<'a>(p: Paleta, id: Icono) -> Element<'a, Mensaje> {
    icono(id, Tam::Sm, p.text_muted).width(16).height(16).into()
}
fn link<'a>(p: Paleta, id: Icono, label: &str, url: &str) -> Element<'a, Mensaje> {
    button(
        row![
            glyph(p, id),
            text(label.to_owned())
                .size(13)
                .color(p.text)
                .width(Length::Fill),
            glyph(p, Icono::Externo)
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .padding([4, 0])
    .width(Length::Fill)
    .on_press(Mensaje::Servicio(Accion::Enlace(url.into())))
    .style(icaro_ui::estilo::sin_estilo(p.text))
    .into()
}
fn datum<'a>(
    p: Paleta,
    id: Icono,
    label: &str,
    value: Element<'a, Mensaje>,
) -> Element<'a, Mensaje> {
    row![
        glyph(p, id),
        text(label.to_owned())
            .size(13)
            .color(p.text_muted)
            .width(Length::Fill),
        value
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}
fn mono<'a>(p: Paleta, value: String) -> Element<'a, Mensaje> {
    text(value)
        .font(fuentes::MONO)
        .size(12)
        .color(p.text)
        .into()
}
fn creator<'a>(
    app: &'a App,
    name: &str,
    role: &str,
    url: &str,
    avatar: Option<&str>,
    owner: bool,
) -> Element<'a, Mensaje> {
    let p = app.paleta();
    button(
        row![
            app.servicios.images.vista(p, avatar, 32.),
            column![
                text(name.to_owned())
                    .font(fuentes::TITULO)
                    .size(13)
                    .color(p.text),
                text(role.to_owned())
                    .font(fuentes::MONO)
                    .size(12)
                    .color(p.text_muted)
            ]
            .spacing(2)
            .width(Length::Fill),
            if owner {
                componentes::insignia_contorno(p, "Dueño")
            } else {
                Space::with_width(0).into()
            }
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .padding([6, 0])
    .width(Length::Fill)
    .on_press(Mensaje::Servicio(Accion::Enlace(url.into())))
    .style(icaro_ui::estilo::sin_estilo(p.text))
    .into()
}
pub fn vista<'a>(app: &'a App, project: &'a Project, width: f32) -> Element<'a, Mensaje> {
    let p = app.paleta();
    let s = &app.servicios.ficha;
    let mut sections = column![].spacing(28).width(Length::Fill);
    if !project.game_versions.is_empty() {
        let chips = version_chips(
            &project.game_versions,
            &app.servicios.instancias.versiones,
            s.info_snapshots,
        );
        let current = crate::mods_reales::instancia(app).map(|i| i.minecraft.as_str());
        let mut body =
            column![text("Minecraft: Java Edition").size(13).color(p.text_muted)].spacing(12);
        let mut rows: Vec<Vec<&VersionChip>> = vec![vec![]];
        let mut used = 0.;
        for chip in &chips {
            let chip_width = chip.label.chars().count() as f32 * 7.2 + 16.;
            if used > 0. && used + 6. + chip_width > width {
                rows.push(vec![]);
                used = 0.;
            }
            rows.last_mut().unwrap().push(chip);
            used += chip_width + 6.;
        }
        let visible = rows.iter().take(3).map(|r| r.len()).sum::<usize>();
        for chunk in rows.iter().take(3).filter(|r| !r.is_empty()) {
            body = body.push(
                row(chunk
                    .iter()
                    .map(|chip| {
                        let actual = current.is_some_and(|v| chip.members.iter().any(|m| m == v));
                        container(
                            text(chip.label.clone())
                                .font(fuentes::MONO)
                                .size(12)
                                .line_height(iced::widget::text::LineHeight::Absolute(16.0.into()))
                                .wrapping(iced::widget::text::Wrapping::None)
                                .color(if actual { p.bg } else { p.text }),
                        )
                        .padding([4, 8])
                        .style(move |_| iced::widget::container::Style {
                            background: actual.then_some(p.text.into()),
                            border: iced::Border {
                                width: 1.,
                                color: if actual { p.text } else { p.border_strong },
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .into()
                    })
                    .collect::<Vec<Element<Mensaje>>>())
                .spacing(6),
            );
        }
        if visible < chips.len() {
            body = body.push(componentes::boton_compacto(
                p,
                &format!("Ver las {} versiones", project.game_versions.len()),
                Variante::Fantasma,
                Some(Mensaje::Servicio(Accion::InfoVersiones)),
            ));
        }
        body = body.push(componentes::casilla(
            p,
            "Mostrar snapshots y pre-releases",
            s.info_snapshots,
            |b| Mensaje::Servicio(Accion::InfoSnapshots(b)),
        ));
        sections = sections.push(section(p, "Compatibilidad", body.into()));
    }
    if project.project_type == "resourcepack" {
        sections = sections.push(section(
            p,
            "Plataformas",
            text("No necesita loader: funciona en cualquier instancia.")
                .size(13)
                .color(p.text)
                .into(),
        ));
    } else {
        let loaders: Vec<_> = project
            .loaders
            .iter()
            .filter(|l| {
                if project.project_type == "shader" {
                    matches!(l.as_str(), "iris" | "optifine")
                } else {
                    matches!(l.as_str(), "fabric" | "quilt" | "neoforge" | "forge")
                }
            })
            .collect();
        if !loaders.is_empty() {
            sections = sections.push(section(
                p,
                "Plataformas",
                row(loaders
                    .iter()
                    .map(|l| componentes::insignia_contorno(p, &ficha::loader(l)))
                    .collect::<Vec<_>>())
                .spacing(6)
                .wrap()
                .into(),
            ));
        }
    }
    let mut environment = column![].spacing(12);
    let mut environment_count = 0;
    for (id, label, value) in [
        (Icono::Monitor, "Cliente", project.client_side.as_str()),
        (Icono::Servidores, "Servidor", project.server_side.as_str()),
    ] {
        let state = match value {
            "required" => Some(componentes::insignia(p, Estado::Exito, "Requerido")),
            "optional" => Some(componentes::insignia(p, Estado::Info, "Opcional")),
            "unsupported" => Some(componentes::insignia_contorno(p, "No se usa")),
            _ => None,
        };
        if let Some(state) = state {
            environment = environment.push(datum(p, id, label, state));
            environment_count += 1;
        }
    }
    if environment_count > 0 {
        sections = sections.push(section(p, "Entornos", environment.into()));
    }
    let selected = app.servicios.detail.as_ref().and_then(|(_, v)| {
        v.iter()
            .find(|v| Some(&v.id) == s.seleccionada.as_ref())
            .or_else(|| v.first())
    });
    if project.project_type == "resourcepack" {
        let resolution = project.categories.iter().find(|c| {
            c.strip_suffix('x')
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        });
        let file = selected.and_then(|v| {
            v.files
                .iter()
                .find(|f| f.primary)
                .or_else(|| v.files.first())
        });
        let mut data = column![].spacing(12);
        let mut count = 0;
        if let Some(value) = resolution {
            data = data.push(datum(
                p,
                Icono::Cuadricula,
                "Resolución",
                mono(p, value.clone()),
            ));
            count += 1;
        }
        if let Some(file) = file {
            data = data.push(datum(
                p,
                Icono::Paquete,
                "Tamaño",
                mono(
                    p,
                    format!("{:.1} MB", file.size as f64 / 1048576.).replace('.', ","),
                ),
            ));
            count += 1;
        }
        if count > 0 {
            sections = sections.push(section(p, "Paquete", data.into()));
        }
    } else if project.project_type == "modpack" {
        if let Some(v) = selected {
            sections = sections.push(section(
                p,
                "Contenido",
                datum(
                    p,
                    Icono::Paquete,
                    "Versión del pack",
                    mono(p, v.version_number.clone()),
                ),
            ));
        }
    }
    let mut links = column![].spacing(0);
    let mut link_count = 0;
    for (id, label, url) in [
        (Icono::Bug, "Reportar un problema", &project.issues_url),
        (Icono::Codigo, "Código fuente", &project.source_url),
        (Icono::Notas, "Wiki", &project.wiki_url),
        (Icono::Usuarios, "Servidor de Discord", &project.discord_url),
    ] {
        if let Some(url) = url.as_deref().filter(|u| !u.trim().is_empty()) {
            links = links.push(link(p, id, label, url));
            link_count += 1;
        }
    }
    for donation in project.donation_urls.iter().flatten() {
        if !donation.url.trim().is_empty() {
            links = links.push(link(
                p,
                Icono::Corazon,
                &format!("Donar · {}", donation.platform),
                &donation.url,
            ));
            link_count += 1;
        }
    }
    if link_count > 0 {
        sections = sections.push(section(p, "Enlaces", links.into()));
    }
    if !project.categories.is_empty() {
        sections = sections.push(section(
            p,
            "Etiquetas",
            row(project
                .categories
                .iter()
                .map(|c| componentes::insignia_contorno(p, crate::diseno_mods::categoria(c)))
                .collect::<Vec<_>>())
            .spacing(6)
            .wrap()
            .into(),
        ));
    }
    let mut creators = column![].spacing(0);
    let mut creator_count = 0;
    if let Some(org) = &s.organizacion {
        let role = if org.members.is_empty() {
            "Organización".into()
        } else {
            format!("Organización · {} miembros", org.members.len())
        };
        creators = creators.push(creator(
            app,
            &org.name,
            &role,
            &format!("https://modrinth.com/organization/{}", org.id),
            org.icon_url.as_deref(),
            true,
        ));
        creator_count += 1;
    }
    for member in &s.miembros {
        let role = match member.role.to_ascii_lowercase().as_str() {
            "owner" => "Dueño",
            "member" => "Miembro",
            "maintainer" => "Mantenimiento",
            "project lead" => "Dirección del proyecto",
            _ => &member.role,
        };
        creators = creators.push(creator(
            app,
            member.user.name.as_deref().unwrap_or(&member.user.username),
            role,
            &format!("https://modrinth.com/user/{}", member.user.id),
            member.user.avatar_url.as_deref(),
            member.is_owner,
        ));
        creator_count += 1;
    }
    if s.miembros_busy || s.organizacion_busy {
        creators = creators.push(text("Cargando creadores…").size(13).color(p.text_muted));
        creator_count += 1;
    }
    if let Some(error) = s.miembros_error.as_ref().or(s.organizacion_error.as_ref()) {
        creators =
            creators
                .push(text(error).size(13).color(p.warning))
                .push(componentes::boton_compacto(
                    p,
                    "Reintentar",
                    Variante::Secundario,
                    Some(Mensaje::Servicio(Accion::CargarMiembros)),
                ));
        creator_count += 1;
    }
    if creator_count > 0 {
        sections = sections.push(section(p, "Creadores", creators.into()));
    }
    let mut details = column![].spacing(12);
    if !project.license.name.trim().is_empty() {
        let value = if let Some(url) = &project.license.url {
            button(text(project.license.name.clone()).size(13).color(p.text))
                .padding(0)
                .on_press(Mensaje::Servicio(Accion::Enlace(url.clone())))
                .style(icaro_ui::estilo::sin_estilo(p.text))
                .into()
        } else {
            text(project.license.name.clone())
                .size(13)
                .color(p.text)
                .into()
        };
        details = details.push(datum(p, Icono::Documento, "Licencia", value));
    }
    details = details
        .push(datum(
            p,
            Icono::Descargas,
            "Descargas",
            mono(p, catalogo::numero(project.downloads)),
        ))
        .push(datum(
            p,
            Icono::Corazon,
            "Seguidores",
            mono(p, catalogo::numero(project.followers)),
        ));
    for (id, label, date) in [
        (Icono::Calendario, "Publicado", &project.published),
        (Icono::Refrescar, "Actualizado", &project.updated),
    ] {
        if ficha::fecha(date) != "Sin fecha" {
            details = details.push(datum(p, id, label, mono(p, ficha::fecha(date))));
        }
    }
    sections.push(section(p, "Detalles", details.into())).into()
}
pub fn mas_creador<'a>(app: &'a App) -> Element<'a, Mensaje> {
    let p = app.paleta();
    let s = &app.servicios.ficha;
    if s.relacionados.is_empty() && !s.relacionados_busy && s.relacionados_error.is_none() {
        return Space::with_height(0).into();
    }
    let name = s
        .organizacion
        .as_ref()
        .map(|o| o.name.as_str())
        .or_else(|| {
            s.miembros
                .iter()
                .find(|m| m.is_owner)
                .map(|m| m.user.username.as_str())
        });
    let mut body = column![].spacing(0);
    for project in &s.relacionados {
        body = body.push(
            button(
                row![
                    app.servicios
                        .images
                        .vista(p, project.icon_url.as_deref(), 32.),
                    column![
                        text(&project.title).size(13).color(p.text),
                        text(catalogo::numero(project.downloads))
                            .font(fuentes::MONO)
                            .size(12)
                            .color(p.text_muted)
                    ]
                    .spacing(0)
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .padding([6, 0])
            .width(Length::Fill)
            .on_press(Mensaje::Servicio(Accion::Proyecto(project.id.clone())))
            .style(icaro_ui::estilo::sin_estilo(p.text)),
        );
    }
    if s.relacionados_busy {
        body = body.push(text("Cargando proyectos…").size(13).color(p.text_muted));
    }
    if let Some(error) = &s.relacionados_error {
        body = body
            .push(text(error).size(13).color(p.warning))
            .push(componentes::boton_compacto(
                p,
                "Reintentar",
                Variante::Secundario,
                Some(Mensaje::Servicio(Accion::CargarRelacionados)),
            ));
    }
    let url = s
        .organizacion
        .as_ref()
        .map(|o| format!("https://modrinth.com/organization/{}", o.id))
        .or_else(|| {
            s.miembros
                .iter()
                .find(|m| m.is_owner)
                .map(|m| format!("https://modrinth.com/user/{}", m.user.id))
        });
    if let Some(url) = url {
        body = body.push(componentes::boton_compacto(
            p,
            "Ver perfil",
            Variante::Secundario,
            Some(Mensaje::Servicio(Accion::Enlace(url))),
        ));
    }
    section(
        p,
        &name.map_or_else(
            || "Más de este creador".into(),
            |name| format!("Más de {name}"),
        ),
        body.into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn manifest(ids: &[&str]) -> Vec<VersionEntry> {
        ids.iter()
            .map(|id| VersionEntry {
                id: (*id).into(),
                kind: if id.contains("RC") {
                    "snapshot"
                } else {
                    "release"
                }
                .into(),
                url: String::new(),
                sha1: String::new(),
                release_time: String::new(),
            })
            .collect()
    }
    #[test]
    fn grouping_requires_all_official_versions_and_preserves_latest_three() {
        let official = manifest(&[
            "26.3", "26.2", "26.1.2", "1.21.4", "1.21.3", "1.21.2", "1.21.1", "1.21",
        ]);
        let complete: Vec<_> = official.iter().map(|v| v.id.clone()).collect();
        let chips = version_chips(&complete, &official, false);
        assert_eq!(
            chips.iter().map(|c| c.label.as_str()).collect::<Vec<_>>(),
            vec!["26.3", "26.2", "26.1.2", "1.21.x"]
        );
        let partial: Vec<_> = complete.into_iter().filter(|v| v != "1.21.2").collect();
        assert!(!version_chips(&partial, &official, false)
            .iter()
            .any(|c| c.label.ends_with(".x")));
        assert!(!version_chips(&partial, &[], false)
            .iter()
            .any(|c| c.label.ends_with(".x")));
    }
    #[test]
    fn snapshots_hidden_by_default_and_family_marks_actual_member() {
        let official = manifest(&[
            "26.3-RC-2",
            "26.2",
            "26.1.2",
            "26.1.1",
            "1.21.2",
            "1.21.1",
            "1.21",
        ]);
        let supported: Vec<_> = official.iter().map(|v| v.id.clone()).collect();
        assert!(!version_chips(&supported, &official, false)
            .iter()
            .any(|c| c.label.contains("RC")));
        assert!(version_chips(&supported, &official, true)
            .iter()
            .any(|c| c.label == "26.3-RC-2"));
        let grouped = version_chips(&supported, &official, false);
        assert!(grouped
            .iter()
            .find(|c| c.label == "1.21.x")
            .unwrap()
            .members
            .iter()
            .any(|v| v == "1.21.1"));
    }
}
