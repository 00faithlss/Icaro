// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Galería de componentes. Ejecutar con `cargo run -p icaro-ui --example galeria`.

use icaro_ui::componentes::{self, Estado, Variante};
use icaro_ui::tema::{espacio, medida, texto, Modo};
use icaro_ui::{estilo, fuentes};
use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Length, Theme};

#[derive(Default)]
struct Galeria {
    modo: Modo,
    nombre: String,
    filtro: usize,
    vista: usize,
    marcada: bool,
    encendido: bool,
    loader: Option<&'static str>,
}

#[derive(Debug, Clone)]
enum Mensaje {
    CambiarTema,
    Nombre(String),
    Filtro(usize),
    Vista(usize),
    Marcar(bool),
    Encender,
    Loader(&'static str),
    Nada,
}

fn actualizar(g: &mut Galeria, m: Mensaje) {
    match m {
        Mensaje::CambiarTema => {
            g.modo = match g.modo {
                Modo::Tinta => Modo::Piedra,
                Modo::Piedra => Modo::Tinta,
            }
        }
        Mensaje::Nombre(n) => g.nombre = n,
        Mensaje::Filtro(i) => g.filtro = i,
        Mensaje::Vista(i) => g.vista = i,
        Mensaje::Marcar(m) => g.marcada = m,
        Mensaje::Encender => g.encendido = !g.encendido,
        Mensaje::Loader(l) => g.loader = Some(l),
        Mensaje::Nada => {}
    }
}

fn vista(g: &Galeria) -> Element<'_, Mensaje> {
    let p = g.modo.paleta();
    let titulo = text("COMPONENTES")
        .font(fuentes::DISPLAY)
        .size(texto::DISPLAY_XL.0)
        .color(p.text);
    let botones = row![
        componentes::boton(
            p,
            "Crear instancia",
            Variante::Primario,
            Some(Mensaje::Nada)
        ),
        componentes::boton(
            p,
            "Exportar modpack",
            Variante::Secundario,
            Some(Mensaje::Nada)
        ),
        componentes::boton(p, "Cancelar", Variante::Fantasma, Some(Mensaje::Nada)),
        componentes::boton(
            p,
            "Eliminar Supervivencia",
            Variante::Peligro,
            Some(Mensaje::Nada)
        ),
        componentes::boton(p, "Deshabilitado", Variante::Primario, None),
    ]
    .spacing(espacio::S2);
    let insignias = row![
        componentes::insignia(p, Estado::Exito, "Lista"),
        componentes::insignia(p, Estado::Aviso, "Desactualizada"),
        componentes::insignia(p, Estado::Error, "Con error"),
        componentes::insignia(p, Estado::Info, "Instalando"),
    ]
    .spacing(espacio::S2);
    let ficha = componentes::tarjeta(
        p,
        false,
        column![
            text("Supervivencia")
                .font(fuentes::CUERPO_NEGRITA)
                .size(texto::HEADING.0),
            text("Fabric 0.16.9 · 1.21.1")
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted),
            text("6.144 MB de memoria")
                .font(fuentes::CUERPO)
                .size(texto::BODY.0),
        ]
        .spacing(espacio::S2),
    );
    let contenido = column![
        titulo,
        componentes::boton(
            p,
            "Cambiar tema",
            Variante::Secundario,
            Some(Mensaje::CambiarTema)
        ),
        componentes::etiqueta(p, "Botones"),
        botones,
        componentes::etiqueta(p, "Estados"),
        insignias,
        componentes::etiqueta(p, "Nombre de la instancia"),
        container(componentes::campo(
            p,
            "Mi mundo",
            &g.nombre,
            Mensaje::Nombre
        ))
        .width(medida::PANEL_LATERAL / 2.0),
        componentes::etiqueta(p, "Filtros"),
        row![
            componentes::chip(p, "Todas", g.filtro == 0, Mensaje::Filtro(0)),
            componentes::chip(p, "Fabric", g.filtro == 1, Mensaje::Filtro(1)),
            componentes::chip(p, "NeoForge", g.filtro == 2, Mensaje::Filtro(2)),
            componentes::segmentado(p, &["Cuadrícula", "Lista"], g.vista, Mensaje::Vista),
        ]
        .spacing(espacio::S2),
        componentes::etiqueta(p, "Controles"),
        row![
            componentes::casilla(p, "Abrir consola al jugar", g.marcada, Mensaje::Marcar),
            componentes::interruptor(p, g.encendido, Mensaje::Encender),
            componentes::selector(
                p,
                vec!["Fabric", "NeoForge", "Quilt"],
                g.loader,
                Mensaje::Loader
            ),
        ]
        .spacing(espacio::S6)
        .align_y(iced::Alignment::Center),
        componentes::etiqueta(p, "Progreso"),
        container(componentes::progreso(p, 0.58)).width(320),
        container(componentes::progreso_bloques(p, 16, 9)).width(320),
        componentes::etiqueta(p, "Tarjeta"),
        container(ficha).width(320),
    ]
    .spacing(espacio::S4);
    container(scrollable(container(contenido).padding(espacio::S12)))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(estilo::fondo(p))
        .into()
}

fn main() -> iced::Result {
    let mut app = iced::application("Ícaro · Componentes", actualizar, vista)
        .theme(|_| Theme::Dark)
        .default_font(fuentes::CUERPO)
        .window_size((1000.0, 800.0));
    for f in fuentes::BYTES {
        app = app.font(f);
    }
    app.run()
}
