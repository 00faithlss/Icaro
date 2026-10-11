// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Búsqueda local compartida por la barra de título y Ctrl+K.
use super::{instancias, mods, App, Mensaje};
use icaro_ui::iconos::{icono_texto, Icono, Tam};
use icaro_ui::tema::{espacio, texto};
use icaro_ui::{estilo, fuentes};
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Element, Length};

pub const CAMPO: &str = "busqueda-global";
pub const LISTA: &str = "resultados-globales";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destino {
    Instancia(usize),
    Mod(String),
    Servidor(usize),
}

pub struct Resultado {
    pub titulo: String,
    pub detalle: String,
    pub destino: Destino,
}

#[derive(Default)]
pub struct Estado {
    pub abierta: bool,
    pub consulta: String,
    pub seleccion: usize,
}

fn normalizar(texto: &str) -> String {
    texto
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' => 'a',
            'é' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'ï' => 'i',
            'ó' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            _ => c,
        })
        .collect()
}

fn filtrar(mut entradas: Vec<Resultado>, consulta: &str) -> Vec<Resultado> {
    let consulta = normalizar(consulta.trim());
    let palabras: Vec<_> = consulta.split_whitespace().collect();
    entradas.retain(|r| {
        let texto = normalizar(&format!("{} {}", r.titulo, r.detalle));
        palabras.iter().all(|p| texto.contains(p))
    });
    // Primero los nombres exactos y los que comienzan con la consulta.
    entradas.sort_by_key(|r| {
        let titulo = normalizar(&r.titulo);
        if titulo == consulta {
            0
        } else if titulo.starts_with(&consulta) {
            1
        } else {
            2
        }
    });
    entradas
}

pub fn resultados(app: &App) -> Vec<Resultado> {
    let mut entradas: Vec<_> = instancias()
        .iter()
        .enumerate()
        .map(|(i, d)| Resultado {
            titulo: d.nombre.clone(),
            detalle: format!("Instancia · {}", d.version),
            destino: Destino::Instancia(i),
        })
        .collect();
    entradas.extend(mods::CATALOGO.iter().map(|m| Resultado {
        titulo: m.titulo.into(),
        detalle: format!("Mod · {} · {}", m.id, m.descripcion),
        destino: Destino::Mod(m.id.into()),
    }));
    entradas.extend(
        app.red
            .servidores
            .iter()
            .enumerate()
            .map(|(i, s)| Resultado {
                titulo: s.nombre.clone(),
                detalle: format!("Servidor · {}", s.direccion),
                destino: Destino::Servidor(i),
            }),
    );
    filtrar(entradas, &app.busqueda.consulta)
}

pub fn mover(actual: usize, paso: i32, total: usize) -> usize {
    if total == 0 {
        0
    } else {
        (actual as i64 + i64::from(paso)).rem_euclid(total as i64) as usize
    }
}

pub fn vista(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    let resultados = resultados(app);
    let cantidad = resultados.len();
    let mut lista = column![].spacing(espacio::S1);
    for (i, r) in resultados.into_iter().enumerate() {
        let icono = match r.destino {
            Destino::Instancia(_) => Icono::Instancias,
            Destino::Mod(_) => Icono::Mods,
            Destino::Servidor(_) => Icono::Servidores,
        };
        lista = lista.push(
            button(
                row![
                    icono_texto(icono, Tam::Base),
                    column![
                        text(r.titulo).font(fuentes::TITULO).size(texto::BODY.0),
                        text(r.detalle)
                            .size(texto::BODY_SM.0)
                            .wrapping(iced::widget::text::Wrapping::None),
                    ]
                    .spacing(espacio::S1)
                    .width(Length::Fill),
                ]
                .spacing(espacio::S4)
                .align_y(Alignment::Center),
            )
            .padding(espacio::S3)
            .width(Length::Fill)
            .height(64)
            .on_press(Mensaje::AbrirResultado(r.destino))
            .style(estilo::segmento(p, i == app.busqueda.seleccion)),
        );
    }
    if cantidad == 0 {
        lista = lista.push(
            container(
                text("No hay coincidencias. Prueba con otro nombre o una dirección.")
                    .color(p.text_muted),
            )
            .padding(espacio::S4),
        );
    }
    let campo = text_input(
        "Buscar instancias, mods o servidores",
        &app.busqueda.consulta,
    )
    .id(CAMPO)
    .on_input(Mensaje::BuscarGlobal)
    .on_submit(Mensaje::ConfirmarBusqueda)
    .padding(espacio::S3)
    .style(estilo::campo(p));
    let panel = container(
        column![
            row![
                campo,
                button(icono_texto(Icono::Cerrar, Tam::Base))
                    .on_press(Mensaje::CerrarBusqueda)
                    .padding(espacio::S2)
                    .style(estilo::boton_fantasma(p))
            ]
            .spacing(espacio::S3)
            .align_y(Alignment::Center),
            container(iced::widget::scrollable(lista).id(iced::widget::scrollable::Id::new(LISTA)))
                .height((app.ventana.height - 240.0).clamp(160.0, 360.0)),
            text(format!(
                "{cantidad} resultados · ↑ ↓ elegir · Enter abrir · Esc cerrar"
            ))
            .size(texto::BODY_SM.0)
            .color(p.text_muted),
        ]
        .spacing(espacio::S3),
    )
    .padding(espacio::S4)
    .width(720)
    .style(estilo::flotante(p));
    panel.into()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn entrada(titulo: &str, detalle: &str, destino: Destino) -> Resultado {
        Resultado {
            titulo: titulo.into(),
            detalle: detalle.into(),
            destino,
        }
    }

    #[test]
    fn busca_sin_tildes_con_varias_palabras_y_por_direccion() {
        let entradas = vec![
            entrada(
                "Técnicos",
                "Instancia · Fabric 1.21.4",
                Destino::Instancia(1),
            ),
            entrada(
                "Amigos",
                "Servidor · 10.147.17.1:25565",
                Destino::Servidor(0),
            ),
        ];
        assert_eq!(
            filtrar(entradas, "  TECNICOS fabric ")[0].destino,
            Destino::Instancia(1)
        );
        let servidor = entrada(
            "Amigos",
            "Servidor · 10.147.17.1:25565",
            Destino::Servidor(0),
        );
        assert_eq!(
            filtrar(vec![servidor], "25565")[0].destino,
            Destino::Servidor(0)
        );
    }

    #[test]
    fn prioriza_nombre_y_conserva_el_destino_correcto() {
        let entradas = vec![
            entrada("Otro", "Mod · usa Fabric API", Destino::Mod("otro".into())),
            entrada("Fabric API", "Mod", Destino::Mod("fabric-api".into())),
        ];
        let resultados = filtrar(entradas, "fabric api");
        assert_eq!(resultados[0].destino, Destino::Mod("fabric-api".into()));
        assert_eq!(resultados.len(), 2);
    }

    #[test]
    fn sin_coincidencias_y_flechas_no_producen_indices_invalidos() {
        assert!(filtrar(
            vec![entrada("Amigos", "Servidor", Destino::Servidor(0))],
            "xyz"
        )
        .is_empty());
        assert_eq!(mover(0, -1, 3), 2);
        assert_eq!(mover(2, 1, 3), 0);
        assert_eq!(mover(0, 1, 0), 0);
    }
}
