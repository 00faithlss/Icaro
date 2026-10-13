// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! ExportarListaMods del ZIP: código seleccionable, vista previa y seis formatos.
use crate::{App, Mensaje};
use icaro_core::exportar_lista_mods::{self as core, Formato, ModMeta, Opciones};
use icaro_ui::{
    componentes::{self, Variante},
    fuentes,
    iconos::Icono,
    tema::texto,
};
use iced::{
    Alignment, Element, Length, Task,
    widget::{Space, column, container, row, scrollable, text, text_editor},
};
use std::{collections::BTreeSet, path::PathBuf};
#[derive(Default)]
pub struct Estado {
    pub dialogo: Option<Dialogo>,
    generacion: u64,
}
pub struct Dialogo {
    pub instancia: String,
    pub nombre: String,
    pub version: String,
    pub datos: Vec<ModMeta>,
    pub opciones: Opciones,
    pub plantilla: text_editor::Content,
    pub codigo: text_editor::Content,
    pub salida: String,
    pub cargando: bool,
    pub completando: bool,
    pub guardando: bool,
    pub error: Option<String>,
    pub copiado: bool,
    generacion: u64,
    seleccion: Option<BTreeSet<String>>,
}
#[derive(Debug, Clone)]
pub enum Accion {
    Abrir(bool),
    Cerrar,
    Leida(u64, Result<Vec<ModMeta>, String>),
    Formato(usize),
    Campo(usize, bool),
    Activos(bool),
    Editar(text_editor::Action),
    Codigo(text_editor::Action),
    Variable(String),
    Completar,
    Completada(u64, Result<Vec<ModMeta>, String>),
    Copiar,
    Copiada(u64, String, Option<String>),
    FinCopiado(u64),
    Guardar,
    Archivo(u64, Option<PathBuf>),
    Guardada(u64, Result<PathBuf, String>),
}
pub fn m(a: Accion) -> Mensaje {
    Mensaje::ExportarLista(a)
}
fn formato(i: usize) -> Formato {
    [
        Formato::Html,
        Formato::Markdown,
        Formato::Texto,
        Formato::Json,
        Formato::Csv,
        Formato::Personalizado,
    ][i.min(5)]
}
fn indice(f: Formato) -> usize {
    match f {
        Formato::Html => 0,
        Formato::Markdown => 1,
        Formato::Texto => 2,
        Formato::Json => 3,
        Formato::Csv => 4,
        Formato::Personalizado => 5,
    }
}
fn recalcular(d: &mut Dialogo, plantilla: bool) {
    d.salida = core::renderizar(&d.datos, &d.opciones);
    d.codigo = text_editor::Content::with_text(&d.salida);
    if plantilla {
        d.plantilla = text_editor::Content::with_text(&core::plantilla(&d.opciones));
    }
    d.copiado = false;
}
fn nombre(d: &Dialogo) -> String {
    let s: String = d
        .nombre
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, ' ' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("{}-mods.{}", s.trim(), d.opciones.formato.extension())
}
pub fn actualizar(app: &mut App, a: Accion) -> Task<Mensaje> {
    match a {
        Accion::Abrir(only) => {
            let index = app.editor_inst;
            let cards = crate::instancias();
            let Some(card) = cards.get(index) else {
                return Task::none();
            };
            let record = app.servicios.instancias.registros.get(index).cloned();
            let id = record
                .as_ref()
                .map(|i| i.id.clone())
                .unwrap_or_else(|| format!("demo-{index}"));
            let selected = if only {
                Some(
                    app.servicios
                        .mods
                        .diseno
                        .seleccion
                        .get(&id)
                        .cloned()
                        .unwrap_or_default(),
                )
            } else {
                None
            };
            app.exportar_lista.generacion += 1;
            let seq = app.exportar_lista.generacion;
            let mut d = Dialogo {
                instancia: id.clone(),
                nombre: card.nombre.clone(),
                version: card.version.clone(),
                datos: vec![],
                opciones: Opciones::default(),
                plantilla: text_editor::Content::new(),
                codigo: text_editor::Content::new(),
                salida: String::new(),
                cargando: !app.servicios.demo,
                completando: false,
                guardando: false,
                error: None,
                copiado: false,
                generacion: seq,
                seleccion: selected,
            };
            if app.servicios.demo {
                d.datos = fixture();
                recalcular(&mut d, true);
            }
            app.exportar_lista.dialogo = Some(d);
            if !app.servicios.demo {
                let store = app.servicios.instancias.almacen.clone();
                return Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || core::leer(&store, &id))
                            .await
                            .unwrap_or_else(|_| Err("No se pudo leer la lista de mods.".into()))
                    },
                    move |r| m(Accion::Leida(seq, r)),
                );
            }
        }
        Accion::Cerrar => {
            if !app
                .exportar_lista
                .dialogo
                .as_ref()
                .is_some_and(|d| d.guardando)
            {
                app.exportar_lista.dialogo = None;
                app.exportar_lista.generacion += 1;
            }
        }
        Accion::Leida(seq, r) | Accion::Completada(seq, r) => {
            if let Some(d) = app
                .exportar_lista
                .dialogo
                .as_mut()
                .filter(|d| d.generacion == seq)
            {
                d.cargando = false;
                d.completando = false;
                match r {
                    Ok(mut v) => {
                        if let Some(selected) = &d.seleccion {
                            v.retain(|m| selected.contains(&m.filename));
                        }
                        d.datos = v;
                        recalcular(d, true);
                    }
                    Err(e) => d.error = Some(e),
                }
            }
        }
        Accion::Formato(i) => {
            if let Some(d) = app.exportar_lista.dialogo.as_mut().filter(|d| !d.guardando) {
                if i == 5 && d.opciones.formato != Formato::Personalizado {
                    d.opciones.plantilla = d.plantilla.text();
                }
                d.opciones.formato = formato(i);
                recalcular(d, true);
            }
        }
        Accion::Campo(i, v) => {
            if let Some(d) = app
                .exportar_lista
                .dialogo
                .as_mut()
                .filter(|d| !d.guardando && d.opciones.formato != Formato::Personalizado)
            {
                if let Some(c) = d.opciones.campos.get_mut(i) {
                    *c = v;
                }
                recalcular(d, true);
            }
        }
        Accion::Activos(v) => {
            if let Some(d) = app.exportar_lista.dialogo.as_mut().filter(|d| !d.guardando) {
                d.opciones.solo_activos = v;
                recalcular(d, false);
            }
        }
        Accion::Editar(a) => {
            if let Some(d) = app.exportar_lista.dialogo.as_mut().filter(|d| {
                !d.guardando && !matches!(d.opciones.formato, Formato::Json | Formato::Csv)
            }) {
                let editing = matches!(a, text_editor::Action::Edit(_));
                d.plantilla.perform(a);
                if editing {
                    d.opciones.formato = Formato::Personalizado;
                    d.opciones.plantilla = d.plantilla.text();
                    recalcular(d, false);
                }
            }
        }
        Accion::Codigo(a) => {
            if let Some(d) = app.exportar_lista.dialogo.as_mut() {
                if !matches!(a, text_editor::Action::Edit(_)) {
                    d.codigo.perform(a);
                }
            }
        }
        Accion::Variable(v) => {
            return actualizar(
                app,
                Accion::Editar(text_editor::Action::Edit(text_editor::Edit::Paste(
                    std::sync::Arc::new(v),
                ))),
            );
        }
        Accion::Completar => {
            if app.servicios.demo {
                return Task::none();
            }
            let Some(d) = app
                .exportar_lista
                .dialogo
                .as_mut()
                .filter(|d| !d.cargando && !d.completando && !d.guardando)
            else {
                return Task::none();
            };
            d.completando = true;
            d.error = None;
            let seq = d.generacion;
            let store = app.servicios.instancias.almacen.clone();
            let id = d.instancia.clone();
            let items = d.datos.clone();
            return Task::perform(
                async move { core::completar(&store, &id, items).await },
                move |r| m(Accion::Completada(seq, r)),
            );
        }
        Accion::Copiar => {
            if let Some(d) = app.exportar_lista.dialogo.as_ref().filter(|d| !d.cargando) {
                let seq = d.generacion;
                let body = d.salida.clone();
                let expected = body.clone();
                return iced::clipboard::write(body).chain(iced::clipboard::read().map(move |s| {
                    m(Accion::Copiada(seq, expected.clone(), s))
                }));
            }
        }
        Accion::Copiada(seq, expected, r) => {
            if let Some(d) = app
                .exportar_lista
                .dialogo
                .as_mut()
                .filter(|d| d.generacion == seq)
            {
                if r.as_deref() == Some(expected.as_str()) {
                    d.copiado = true;
                    return Task::perform(
                        async move {
                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                            seq
                        },
                        |n| m(Accion::FinCopiado(n)),
                    );
                }
                d.error=Some("No se pudo copiar al portapapeles. Selecciona el código para copiarlo manualmente.".into());
            }
        }
        Accion::FinCopiado(seq) => {
            if let Some(d) = app
                .exportar_lista
                .dialogo
                .as_mut()
                .filter(|d| d.generacion == seq)
            {
                d.copiado = false;
            }
        }
        Accion::Guardar => {
            if let Some(d) = app
                .exportar_lista
                .dialogo
                .as_ref()
                .filter(|d| !d.cargando && !d.guardando)
            {
                let file = nombre(d);
                let ext = d.opciones.formato.extension();
                let seq = d.generacion;
                return Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_file_name(file)
                            .add_filter("Lista de mods", &[ext])
                            .save_file()
                            .await
                            .map(|f| f.path().to_path_buf())
                    },
                    move |p| m(Accion::Archivo(seq, p)),
                );
            }
        }
        Accion::Archivo(seq, Some(path)) => {
            if let Some(d) = app
                .exportar_lista
                .dialogo
                .as_mut()
                .filter(|d| d.generacion == seq && !d.guardando)
            {
                let body = d.salida.clone();
                d.guardando = true;
                return Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || core::guardar(&path, &body))
                            .await
                            .unwrap_or_else(|_| Err("No se pudo guardar la lista.".into()))
                    },
                    move |r| m(Accion::Guardada(seq, r)),
                );
            }
        }
        Accion::Archivo(_, None) => {}
        Accion::Guardada(seq, r) => {
            if app
                .exportar_lista
                .dialogo
                .as_ref()
                .is_some_and(|d| d.generacion == seq)
            {
                match r {
                    Ok(path) => {
                        app.exportar_lista.dialogo = None;
                        app.avisar_con(
                            "Lista de mods guardada.".into(),
                            "Mostrar en carpeta",
                            Mensaje::Pant(crate::pantallas::Accion::MostrarExportacion(path)),
                        );
                    }
                    Err(e) => {
                        if let Some(d) = app.exportar_lista.dialogo.as_mut() {
                            d.guardando = false;
                            d.error = Some(e);
                        }
                    }
                }
            }
        }
    }
    Task::none()
}
fn fixed(d: &Dialogo) -> bool {
    matches!(d.opciones.formato, Formato::Json | Formato::Csv)
}
pub fn vista<'a>(app: &'a App) -> Option<Element<'a, Mensaje>> {
    let d = app.exportar_lista.dialogo.as_ref()?;
    let p = app.paleta();
    let mut fields = row![
        componentes::etiqueta(p, "Incluir"),
        iced::widget::checkbox("Nombre",true).icon(componentes::icono_casilla()).size(16).text_size(texto::BODY_SM.0).style(icaro_ui::estilo::casilla(p))
    ]
    .spacing(18)
    .align_y(Alignment::Center);
    for (n, label) in ["Versión", "Autores", "URL", "Nombre de archivo"]
        .iter()
        .enumerate()
    {
        let checkbox = iced::widget::checkbox(*label, d.opciones.campos[n])
            .icon(componentes::icono_casilla())
            .size(16)
            .text_size(texto::BODY_SM.0)
            .style(icaro_ui::estilo::casilla(p));
        fields = fields.push(if d.opciones.formato != Formato::Personalizado {
            checkbox.on_toggle(move |v| m(Accion::Campo(n, v)))
        } else {
            checkbox
        });
    }
    fields = fields.push(
        iced::widget::checkbox("Solo mods activos", d.opciones.solo_activos)
            .icon(componentes::icono_casilla())
            .size(16)
            .text_size(texto::BODY_SM.0)
            .on_toggle(|v| m(Accion::Activos(v)))
            .style(icaro_ui::estilo::casilla(p)),
    );
    let template = text_editor(&d.plantilla)
        .font(fuentes::MONO)
        .size(texto::MONO_SM.0)
        .height(60)
        .on_action(|a| m(Accion::Editar(a)));
    let mut vars = row![].spacing(6);
    for name in ["{name}", "{version}", "{authors}", "{url}", "{filename}"] {
        vars = vars.push(componentes::boton_compacto(
            p,
            name,
            Variante::Secundario,
            (!fixed(d)).then(|| m(Accion::Variable(name.into()))),
        ));
    }
    let code = column![
        row![
            text("Código")
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted),
            Space::with_width(Length::Fill),
            text(format!("{} líneas", d.salida.lines().count()))
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted)
        ]
        .align_y(Alignment::Center),
        text_editor(&d.codigo)
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .height(Length::Fill)
            .on_action(|a| m(Accion::Codigo(a)))
    ]
    .spacing(8);
    let mut preview = column![
        text("Vista previa")
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted)
    ]
    .spacing(12);
    if fixed(d) {
        let mut table = column![].spacing(10);
        let cols: Vec<_> = std::iter::once(("Nombre", None))
            .chain(
                [
                    ("Versión", Some(0)),
                    ("Autores", Some(1)),
                    ("URL", Some(2)),
                    ("Archivo", Some(3)),
                ]
                .into_iter(),
            )
            .filter(|(_, n)| n.is_none_or(|n| d.opciones.campos[n]))
            .collect();
        let mut header = row![].spacing(14);
        for (label, _) in &cols {
            header = header.push(
                text(*label)
                    .font(fuentes::ETIQUETA)
                    .size(texto::LABEL.0)
                    .color(p.text)
                    .width(140),
            );
        }
        table = table.push(header);
        for item in d
            .datos
            .iter()
            .filter(|m| !d.opciones.solo_activos || m.activo)
        {
            let mut r = row![].spacing(14);
            for (_, field) in &cols {
                let v = match field {
                    None => item.name.clone(),
                    Some(0) => item.version.clone().unwrap_or_default(),
                    Some(1) => item.authors.join(", "),
                    Some(2) => item.url.clone().unwrap_or_default(),
                    _ => item.filename.clone(),
                };
                r = r.push(
                    text(if v.is_empty() { "sin dato".into() } else { v })
                        .size(texto::BODY_SM.0)
                        .color(p.text)
                        .width(140),
                );
            }
            table = table.push(r);
        }
        preview = preview.push(
            scrollable(table)
                .direction(iced::widget::scrollable::Direction::Both {
                    vertical: Default::default(),
                    horizontal: Default::default(),
                })
                .height(Length::Fill),
        );
    } else if matches!(d.opciones.formato, Formato::Html | Formato::Markdown) {
        let mut list = column![].spacing(12);
        for item in d
            .datos
            .iter()
            .filter(|m| !d.opciones.solo_activos || m.activo)
        {
            let mut spans=vec![];
            let mut name=iced::widget::span(item.name.clone());
            if d.opciones.campos[2] {if let Some(url)=&item.url {name=name.link(url.clone()).underline(true).color(p.accent_text);}}
            spans.push(name);
            let mut extra = vec![];
            if d.opciones.campos[0] {
                if let Some(v) = &item.version {
                    extra.push(format!("[{v}]"));
                }
            }
            if d.opciones.campos[1] && !item.authors.is_empty() {
                extra.push(format!("por {}", item.authors.join(", ")));
            }
            if d.opciones.campos[3] {
                extra.push(format!("({})", item.filename));
            }
            if !extra.is_empty(){spans.push(iced::widget::span(format!(" {}",extra.join(" "))));}
            let inline:Element<String>=iced::widget::rich_text(spans).size(texto::BODY_SM.0).font(fuentes::CUERPO).color(p.text).width(Length::Fill).into();
            let inline=inline.map(|url|Mensaje::Pant(crate::pantallas::Accion::AbrirListaEnlace(url)));
            list=list.push(row![text("•").size(texto::BODY_SM.0).color(p.text),inline].spacing(10));
        }
        preview = preview.push(scrollable(list).height(Length::Fill));
    } else {
        preview = preview.push(
            scrollable(
                text(d.salida.clone())
                    .font(fuentes::MONO)
                    .size(texto::MONO_SM.0)
                    .color(p.text),
            )
            .height(Length::Fill),
        );
    }
    let panels = row![
        container(code)
            .padding(12)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .style(icaro_ui::estilo::marco(p)),
        container(preview)
            .padding(12)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .style(icaro_ui::estilo::marco(p))
    ]
    .spacing(16)
    .height(Length::Fill);
    let mut body = column![
        text(format!("{} · {}", d.nombre, d.version))
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted),
        row![
            componentes::etiqueta(p, "Formato"),
            componentes::segmentado(
                p,
                &["HTML", "Markdown", "Texto", "JSON", "CSV", "Personalizado"],
                indice(d.opciones.formato),
                |n| m(Accion::Formato(n))
            )
        ]
        .spacing(18)
        .align_y(Alignment::Center),
        fields
    ]
    .spacing(18);
    if fixed(d) {
        body = body.push(
            text("Estructura fija: las casillas eligen los campos.")
                .size(texto::BODY_SM.0)
                .color(p.text_muted),
        );
    } else {
        body = body.push(
            row![
                componentes::etiqueta(p, "Plantilla"),
                container(
                    column![
                        template,
                        vars,
                        text(if d.opciones.formato == Formato::Personalizado {
                            "Una línea por mod. Las casillas no aplican."
                        } else {
                            "Si la editas, el formato pasa a Personalizado."
                        })
                        .size(texto::BODY_SM.0)
                        .color(p.text_muted)
                    ]
                    .spacing(8)
                )
                .width(Length::Fill)
            ]
            .spacing(18),
        );
    }
    body = body.push(panels);
    let missing: Vec<_> = d
        .datos
        .iter()
        .filter(|m| m.version.is_none() || m.authors.is_empty() || m.url.is_none())
        .map(|m| m.filename.clone())
        .collect();
    if !missing.is_empty() {
        body=body.push(container(row![text(format!("Faltan metadatos de {}. Buscar actualizaciones completa lo que falte; no instala archivos.",missing.iter().take(3).cloned().collect::<Vec<_>>().join(", "))).size(texto::BODY_SM.0).color(p.text).width(Length::Fill),componentes::boton_con_icono(p,Icono::Refrescar,if d.completando{"Consultando…"}else{"Buscar actualizaciones"},Variante::Secundario,(!d.cargando&&!d.completando&&!d.guardando).then(||m(Accion::Completar)),false)].spacing(14).align_y(Alignment::Center)).padding(12).style(move|_|iced::widget::container::Style{background:Some(p.info_soft.into()),border:iced::Border{width:1.,color:p.info,..Default::default()},..Default::default()}));
    }
    if d.cargando {
        body = body.push(
            text("Leyendo mods y sus metadatos…")
                .size(texto::BODY_SM.0)
                .color(p.text_muted),
        );
    }
    if let Some(e) = &d.error {
        body = body.push(componentes::aviso(p, componentes::Estado::Error, e));
    }
    let count = d
        .datos
        .iter()
        .filter(|m| !d.opciones.solo_activos || m.activo)
        .count();
    let footer = row![
        text(format!("{count} mods · {}", nombre(d)))
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted)
            .width(Length::Fill),
        componentes::boton(
            p,
            "Cancelar",
            Variante::Secundario,
            (!d.guardando).then(|| m(Accion::Cerrar))
        ),
        componentes::boton_con_icono(
            p,
            Icono::Copiar,
            if d.copiado { "Copiado" } else { "Copiar" },
            Variante::Secundario,
            (!d.cargando).then(|| m(Accion::Copiar)),
            false
        ),
        componentes::boton_con_icono(
            p,
            Icono::Descargas,
            if d.guardando {
                "Guardando…"
            } else {
                "Guardar…"
            },
            Variante::Primario,
            (!d.cargando && !d.guardando).then(|| m(Accion::Guardar)),
            false
        )
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    Some(crate::pantallas::formulario_icono(
        p,
        &if d.seleccion.is_some() {
            format!("Exportar {} mods", d.datos.len())
        } else {
            "Exportar lista de mods".into()
        },
        body.into(),
        footer.into(),
        (app.ventana.width - 48.).clamp(760., 1100.),
        (app.ventana.height - 80.).clamp(520., 820.),
        m(Accion::Cerrar),
    ))
}
fn fixture() -> Vec<ModMeta> {
    vec![
        ModMeta {
            name: "Fabric API".into(),
            version: Some("0.114.0".into()),
            authors: vec!["FabricMC".into()],
            url: Some("https://modrinth.com/mod/fabric-api".into()),
            filename: "fabric-api-0.114.0+1.21.4.jar".into(),
            activo: true,
            ..Default::default()
        },
        ModMeta {
            name: "Sodium".into(),
            version: Some("0.6.5".into()),
            authors: vec!["CaffeineMC".into()],
            url: Some("https://modrinth.com/mod/sodium".into()),
            filename: "sodium-fabric-0.6.5+mc1.21.4.jar".into(),
            activo: true,
            ..Default::default()
        },
        ModMeta {
            name: "server-utils-local".into(),
            filename: "server-utils-local.jar".into(),
            activo: true,
            ..Default::default()
        },
    ]
}
pub fn preparar_diseno(app: &mut App, scene: &str) {
    let _ = actualizar(app, Accion::Abrir(false));
    if scene == "exportar-lista-json" {
        let _ = actualizar(app, Accion::Formato(3));
    }
    if scene == "exportar-lista-personalizado" {
        let _ = actualizar(app, Accion::Formato(5));
    }
}
