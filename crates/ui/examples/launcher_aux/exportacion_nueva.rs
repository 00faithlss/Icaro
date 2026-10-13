// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! ExportarInstancia: árbol real, selecciones rápidas y ZIP portable.
use super::{m, Accion, App, Exportar, Mensaje};
use icaro_core::exportar::{Arbol, Seleccion};
use icaro_ui::{
    componentes::{self, Estado, Variante},
    fuentes,
    iconos::{Icono, Tam},
    tema::texto,
};
use iced::{
    widget::{button, column, container, row, text, Space},
    Alignment, Element, Length, Padding,
};

pub fn tam(n: u64) -> String {
    let (v, u) = if n >= 1 << 30 {
        (n as f64 / (1u64 << 30) as f64, "GB")
    } else if n >= 1 << 20 {
        (n as f64 / (1u64 << 20) as f64, "MB")
    } else if n >= 1024 {
        (n as f64 / 1024., "KB")
    } else {
        (n as f64, "B")
    };
    if u == "B" {
        format!("{n} B")
    } else {
        format!("{v:.1} {u}").replace('.', ",")
    }
}
fn caja<'a>(
    p: icaro_ui::tema::Paleta,
    (yes, mixed): (bool, bool),
    ruta: &str,
) -> Element<'a, Mensaje> {
    let content: Element<'a, Mensaje> = if mixed {
        container(Space::new(10, 2))
            .style(icaro_ui::estilo::bloque(p.text))
            .into()
    } else if yes {
        icaro_ui::iconos::icono(Icono::Check, Tam::Sm, p.text).into()
    } else {
        Space::new(1, 1).into()
    };
    button(container(content).center(Length::Fill))
        .padding(0)
        .width(16)
        .height(16)
        .on_press(m(Accion::ExportarNodo(ruta.to_owned())))
        .style(move |_: &iced::Theme, _| iced::widget::button::Style {
            background: Some(iced::Background::Color(p.bg)),
            border: iced::Border {
                color: p.border,
                width: 2.,
                radius: 0.into(),
            },
            ..Default::default()
        })
        .into()
}
pub fn vista<'a>(app: &'a App, s: &'a Exportar) -> Element<'a, Mensaje> {
    let p = app.paleta();
    let mut presets = row![].spacing(6).align_y(Alignment::Center);
    for (n, name) in [
        "Recomendada",
        "Todo",
        "Para compartir",
        "Solo mundos",
        "Personalizada",
    ]
    .iter()
    .enumerate()
    {
        presets = presets.push(componentes::boton_compacto(
            p,
            name,
            if n == s.preset {
                Variante::Primario
            } else {
                Variante::Secundario
            },
            (n < 4).then(|| m(Accion::ExportarPreset(n))),
        ));
    }
    let version = crate::instancias()
        .get(s.instancia)
        .map(|i| i.version.clone())
        .unwrap_or_default();
    let mut filas = column![];
    if let Some(tree) = &s.arbol {
        let marcas = tree.marcas();
        let root = row![
            button(icaro_ui::iconos::icono(
                Icono::ChevronAbajo,
                Tam::Sm,
                p.text_muted
            ))
            .padding(0)
            .width(16)
            .style(icaro_ui::estilo::boton_fantasma(p)),
            caja(p, marcas[""], ""),
            icaro_ui::iconos::icono(Icono::Carpeta, Tam::Sm, p.text_muted),
            text("minecraft")
                .font(fuentes::TITULO)
                .size(texto::BODY_SM.0)
                .color(p.text),
            Space::with_width(Length::Fill),
            text(tam(tree
                .nodos
                .iter()
                .filter(|n| !n.carpeta)
                .map(|n| n.bytes)
                .sum()))
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted)
        ]
        .spacing(10)
        .align_y(Alignment::Center);
        filas = filas.push(
            container(root)
                .height(36)
                .padding(Padding::from([0., 12.]))
                .align_y(Alignment::Center),
        );
        for nodo in tree.visibles() {
            let nivel = nodo.ruta.matches('/').count() + 1;
            let arrow: Element<'a, Mensaje> = if nodo.carpeta {
                button(icaro_ui::iconos::icono(
                    if nodo.abierto {
                        Icono::ChevronAbajo
                    } else {
                        Icono::ChevronDerecha
                    },
                    Tam::Sm,
                    p.text_muted,
                ))
                .padding(0)
                .width(16)
                .on_press(m(Accion::ExportarExpandir(nodo.ruta.clone())))
                .style(icaro_ui::estilo::boton_fantasma(p))
                .into()
            } else {
                Space::with_width(16).into()
            };
            let mut fila = row![
                Space::with_width(nivel as f32 * 24.),
                arrow,
                caja(p, marcas[nodo.ruta.as_str()], &nodo.ruta),
                icaro_ui::iconos::icono(
                    if nodo.carpeta {
                        Icono::Carpeta
                    } else {
                        Icono::Documento
                    },
                    Tam::Sm,
                    p.text_muted
                ),
                text(nodo.ruta.rsplit('/').next().unwrap_or(&nodo.ruta))
                    .size(texto::BODY_SM.0)
                    .color(p.text)
                    .width(Length::Fill)
            ]
            .spacing(10)
            .align_y(Alignment::Center);
            if let Some(label) = nodo.etiqueta {
                fila = fila.push(match label {
                    "Mapas" | "Datos de mod" => componentes::insignia(p, Estado::Info, label),
                    "Personal" => componentes::insignia(p, Estado::Aviso, label),
                    _ => componentes::insignia_contorno(p, label),
                });
            }
            fila = fila.push(
                container(
                    text(tam(nodo.bytes))
                        .font(fuentes::MONO)
                        .size(texto::MONO_SM.0)
                        .color(p.text_muted),
                )
                .width(90)
                .align_x(Alignment::End),
            );
            filas = filas.push(
                container(fila)
                    .height(36)
                    .padding(Padding::from([0., 12.]))
                    .align_y(Alignment::Center),
            );
        }
    } else {
        filas = filas.push(
            container(
                text("Calculando archivos y tamaños…")
                    .size(texto::BODY_SM.0)
                    .color(p.text_muted),
            )
            .padding(16),
        );
    }
    let tree = container(iced::widget::scrollable::Scrollable::with_direction(
        container(filas).width(796).height(Length::Shrink),
        iced::widget::scrollable::Direction::Both {
            vertical: Default::default(),
            horizontal: Default::default(),
        },
    ))
    .height(Length::Fill)
    .style(icaro_ui::estilo::marco(p));
    let (count, bytes) = s.arbol.as_ref().map_or((0, 0), Arbol::resumen);
    let formats = ["Formato: .zip de Icaro Launcher", "Formato: Prism / MultiMC"];
    let selector = componentes::selector(
        p,
        formats.to_vec(),
        Some(formats[s.formato.min(1)]),
        move |choice| m(Accion::ExportarFormato(usize::from(choice == formats[1]))),
    );
    let summary = row![
        text(format!(
            "{count} elementos · {} seleccionados · .zip de unos {}",
            tam(bytes),
            tam(bytes * 78 / 100)
        ))
        .size(texto::BODY_SM.0)
        .color(p.text)
        .width(Length::Fill),
        container(selector).width(242)
    ]
    .spacing(16)
    .align_y(Alignment::Center);
    let mut body = column![
        presets,
        text(format!(".minecraft de {} · {}", s.nombre, version))
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted),
        tree,
        summary
    ]
    .spacing(14);
    if bytes >= 4 * 1024 * 1024 * 1024 {
        body = body.push(componentes::aviso(
            p,
            Estado::Aviso,
            "El archivo pesará más de 4 GB; algunos servicios para compartir no lo aceptan.",
        ));
    }
    if let Some(error) = &s.error {
        body = body.push(componentes::aviso(p, Estado::Error, error));
    }
    let footer = row![
        text("Luego eliges dónde guardarlo")
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted)
            .width(Length::Fill),
        componentes::boton(
            p,
            "Cancelar",
            Variante::Secundario,
            Some(m(Accion::ExportarCancelar))
        ),
        componentes::boton_con_icono(
            p,
            Icono::Descargas,
            "Exportar…",
            Variante::Primario,
            (!s.guardando
                && s.arbol
                    .as_ref()
                    .is_some_and(|a| count > 0 || a.nodos.is_empty()))
            .then(|| m(Accion::ExportarConfirmar)),
            false
        )
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    super::formulario(
        p,
        &format!("Exportar {}", s.nombre),
        body.into(),
        footer.into(),
        860.,
        (app.ventana.height - 80.).clamp(420., 820.),
        m(Accion::ExportarCancelar),
    )
}

pub fn preset(n: usize) -> Seleccion {
    match n {
        0 => Seleccion::Recomendada,
        1 => Seleccion::Todo,
        2 => Seleccion::Compartir,
        3 => Seleccion::Mundos,
        _ => Seleccion::Personalizada,
    }
}

/// Solo para las vistas de revisión del diseño; no escribe archivos.
pub fn demo() -> Arbol {
    use icaro_core::exportar::Nodo;
    let mut a = Arbol::default();
    for (name, dir, bytes, label) in [
        (".bobby", true, 0, Some("Caché")),
        (".bobby/chunk.bin", false, 851443712, Some("Caché")),
        ("config", true, 0, None),
        ("config/sodium-options.json", false, 5033164, None),
        (
            "Distant_Horizons_server_data",
            true,
            0,
            Some("Datos de mod"),
        ),
        (
            "Distant_Horizons_server_data/cache.sqlite",
            false,
            1503238553,
            Some("Datos de mod"),
        ),
        ("mods", true, 0, None),
        ("mods/sodium.jar", false, 327155712, None),
        ("resourcepacks", true, 0, None),
        ("resourcepacks/recursos.zip", false, 195035136, None),
        ("saves", true, 0, None),
        ("saves/Temporada 4", true, 0, None),
        ("saves/Temporada 4/level.dat", false, 2040109465, None),
        ("saves/Ciudad de bloques", true, 0, None),
        ("saves/Ciudad de bloques/level.dat", false, 432013312, None),
        ("screenshots", true, 0, None),
        ("screenshots/captura.png", false, 566231040, None),
        ("shaderpacks", true, 0, None),
        ("shaderpacks/shader.zip", false, 50331648, None),
        ("command_history.txt", false, 1095, Some("Personal")),
        ("icon.png", false, 15114, None),
        ("options.txt", false, 11171, None),
        ("servers.dat", false, 12308, None),
        ("servers.dat_old", false, 12308, Some("Respaldo")),
    ] {
        a.nodos.push(Nodo {
            ruta: name.into(),
            carpeta: dir,
            bytes,
            etiqueta: label,
            seleccionado: true,
            abierto: name == "saves",
        });
    }
    for i in (0..a.nodos.len()).rev() {
        if a.nodos[i].carpeta {
            let prefix = format!("{}/", a.nodos[i].ruta);
            a.nodos[i].bytes = a
                .nodos
                .iter()
                .filter(|n| !n.carpeta && n.ruta.starts_with(&prefix))
                .map(|n| n.bytes)
                .sum();
        }
    }
    a.seleccionar(Seleccion::Recomendada);
    a
}

/// Fila reutilizable de ColaDescargas; el empaquetado no ofrece pausa falsa.
pub fn trabajo<'a>(app: &'a App) -> Option<Element<'a, Mensaje>> {
    let s = app.exportar.as_ref().filter(|s| s.guardando)?;
    let p = app.paleta();
    let progress = if s.total == 0 {
        0.
    } else {
        s.hechos as f32 / s.total as f32
    };
    Some(
        container(
            column![
                row![
                    icaro_ui::iconos::icono(Icono::Subir, Tam::Base, p.text),
                    text(format!("Exportando {}", s.nombre))
                        .font(fuentes::TITULO)
                        .color(p.text)
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                text(format!(
                    "{} de {} · {}",
                    tam(s.hechos),
                    tam(s.total),
                    s.destino
                ))
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted),
                iced::widget::progress_bar(0.0..=1.0, progress)
                    .height(4)
                    .style(icaro_ui::estilo::progreso(p))
            ]
            .spacing(12),
        )
        .padding(24)
        .width(Length::Fill)
        .style(icaro_ui::estilo::marco(p))
        .into(),
    )
}
