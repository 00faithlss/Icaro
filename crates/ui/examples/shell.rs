// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Estructura de la ventana. Ejecutar con `cargo run -p icaro-ui --example shell`.

use icaro_ui::componentes::{self, Variante};
use icaro_ui::fuentes;
use icaro_ui::shell::{app_shell, Cuenta, DescargasActivas, MensajesShell, Seccion};
use icaro_ui::tema::{espacio, texto, Modo};
use iced::widget::{column, text};
use iced::{window, Element, Size, Task};

#[derive(Default)]
struct App {
    modo: Modo,
    seccion: Option<Seccion>,
    colapsada: bool,
}

#[derive(Debug, Clone)]
enum Mensaje {
    Ir(Seccion),
    CambiarTema,
    Colapsar,
    Arrastrar,
    Minimizar,
    Maximizar,
    Cerrar,
    Nada,
}

fn actualizar(app: &mut App, m: Mensaje) -> Task<Mensaje> {
    match m {
        Mensaje::Ir(s) => app.seccion = Some(s),
        Mensaje::CambiarTema => {
            app.modo = match app.modo {
                Modo::Tinta => Modo::Piedra,
                Modo::Piedra => Modo::Tinta,
            }
        }
        Mensaje::Colapsar => app.colapsada = !app.colapsada,
        Mensaje::Arrastrar => return window::get_oldest().and_then(window::drag),
        Mensaje::Minimizar => {
            return window::get_oldest().and_then(|id| window::minimize(id, true))
        }
        Mensaje::Maximizar => return window::get_oldest().and_then(window::toggle_maximize),
        Mensaje::Cerrar => return window::get_oldest().and_then(window::close),
        Mensaje::Nada => {}
    }
    Task::none()
}

fn vista(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    let activa = app.seccion.unwrap_or(Seccion::Instancias);
    let contenido = column![
        text(activa.nombre().to_uppercase())
            .font(fuentes::DISPLAY)
            .size(texto::DISPLAY_XL.0)
            .color(p.text),
        componentes::boton(
            p,
            "Cambiar tema",
            Variante::Secundario,
            Some(Mensaje::CambiarTema)
        ),
        componentes::boton(
            p,
            "Colapsar barra",
            Variante::Secundario,
            Some(Mensaje::Colapsar)
        ),
    ]
    .spacing(espacio::S4)
    .padding(espacio::S8);
    let pie = componentes::boton(p, "Jugar", Variante::Primario, Some(Mensaje::Nada));
    app_shell(
        p,
        activa,
        app.colapsada,
        Cuenta {
            nombre: "Mineral_7",
            proveedor: "Microsoft",
        },
        Some(DescargasActivas {
            cantidad: 2,
            velocidad: "6,1 MB/s".into(),
            avance: 0.58,
        }),
        contenido.into(),
        Some(pie),
        MensajesShell {
            ir_a: Box::new(Mensaje::Ir),
            arrastrar: Mensaje::Arrastrar,
            buscar: Mensaje::Nada,
            minimizar: Mensaje::Minimizar,
            maximizar: Mensaje::Maximizar,
            cerrar: Mensaje::Cerrar,
        },
    )
}

fn main() -> iced::Result {
    let mut app = iced::application("Ícaro", actualizar, vista)
        .theme(|_| iced::Theme::Dark)
        .default_font(fuentes::CUERPO)
        .window(window::Settings {
            decorations: false,
            size: Size::new(1200.0, 760.0),
            min_size: Some(Size::new(940.0, 600.0)),
            ..Default::default()
        });
    for f in fuentes::BYTES {
        app = app.font(f);
    }
    app.run()
}
