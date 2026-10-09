// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Las pantallas hechas hasta ahora, juntas. Ejecutar con
//! `cargo run -p icaro-ui --example launcher`.

use icaro_ui::ajustes::{encabezado_ajustes, fila_ajuste, indice_ajustes};
use icaro_ui::componentes::{self, Variante};
use icaro_ui::capturas::{galeria, Captura};
use icaro_ui::consola::{consola, Linea, MensajesConsola, Nivel};
use icaro_ui::editor::{
    encabezado_editor, pestana_archivos, pestana_general, pestana_java, pestana_mundos,
    pestana_recursos, pestanas_editor, CompatPaquete, Copia, DatosEditor, EstadoGeneral,
    InstalacionJava, MensajesEncabezado, MensajesGeneral, Mundo, Nodo, Paquete,
};
use icaro_ui::descargas::{
    cola_descargas, historial, limites, AccionesCola, Descarga, EntradaHistorial, EstadoDescarga,
};
use icaro_ui::fuentes;
use icaro_ui::iconos::Icono;
use icaro_ui::instancias::{
    tarjeta_instancia, tarjeta_nueva, AccionesTarjeta, DatosInstancia, EstadoInstancia,
};
use icaro_ui::laminas::{banda, Lamina};
use icaro_ui::servidores::{red_privada, tarjeta_servidor, EstadoServidor, Servidor};
use icaro_ui::shell::{app_shell, Cuenta, DescargasActivas, MensajesShell, Seccion};
use icaro_ui::superficies::{con_velo, dialogo, menu_contextual, toast, ElementoMenu};
use icaro_ui::tema::{espacio, texto, Modo};
use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{window, Alignment, Element, Length, Padding, Size, Task};

struct App {
    modo: Modo,
    seccion: Seccion,
    colapsada: bool,
    filtro: usize,
    seleccionada: usize,
    menu: Option<usize>,
    dialogo: bool,
    aviso: Option<String>,
    tema: usize,
    reducir: bool,
    simultaneas: i32,
    ajuste: usize,
    editor: Option<usize>,
    general: EstadoGeneral,
    java_auto: bool,
    vista_recursos: usize,
    paquetes_activos: [bool; 3],
    mundo: usize,
    archivo: usize,
    filtro_consola: usize,
    busqueda_consola: String,
    filtro_capturas: usize,
    captura: Option<usize>,
}

#[derive(Debug, Clone)]
enum Mensaje {
    Editar,
    VolverEditor,
    Pestana(usize),
    Nombre(String),
    Grupo(&'static str),
    Ancho(String),
    Alto(String),
    Completa(bool),
    Servidor(&'static str),
    Memoria(u32),
    JavaAuto,
    VistaRecursos(usize),
    Paquete(usize),
    Mundo(usize),
    Archivo(usize),
    FiltroConsola(usize),
    BuscarConsola(String),
    FiltroCapturas(usize),
    Captura(usize),
    Ir(Seccion),
    Filtro(usize),
    Seleccionar(usize),
    Menu(Option<usize>),
    PedirEliminar,
    CerrarDialogo,
    Eliminar,
    CerrarAviso,
    Tema(usize),
    Reducir,
    Simultaneas(i32),
    Ajuste(usize),
    Arrastrar,
    Minimizar,
    Maximizar,
    Cerrar,
    Nada,
}

fn actualizar(app: &mut App, m: Mensaje) -> Task<Mensaje> {
    match m {
        Mensaje::Editar => {
            app.menu = None;
            app.editor = Some(0);
        }
        Mensaje::VolverEditor => app.editor = None,
        Mensaje::Pestana(i) => app.editor = Some(i),
        Mensaje::Nombre(v) => app.general.nombre = v,
        Mensaje::Grupo(v) => app.general.grupo = Some(v),
        Mensaje::Ancho(v) => app.general.ancho = v,
        Mensaje::Alto(v) => app.general.alto = v,
        Mensaje::Completa(v) => app.general.pantalla_completa = v,
        Mensaje::Servidor(v) => app.general.servidor = Some(v),
        Mensaje::Memoria(v) => app.general.memoria = v,
        Mensaje::JavaAuto => app.java_auto = !app.java_auto,
        Mensaje::VistaRecursos(i) => app.vista_recursos = i,
        Mensaje::Paquete(i) => {
            if let Some(a) = app.paquetes_activos.get_mut(i) {
                *a = !*a;
            }
        }
        Mensaje::Mundo(i) => app.mundo = i,
        Mensaje::Archivo(i) => app.archivo = i,
        Mensaje::FiltroConsola(i) => app.filtro_consola = i,
        Mensaje::BuscarConsola(v) => app.busqueda_consola = v,
        Mensaje::FiltroCapturas(i) => app.filtro_capturas = i,
        Mensaje::Captura(i) => app.captura = Some(i),
        Mensaje::Ir(s) => {
            app.seccion = s;
            app.editor = None;
        }
        Mensaje::Filtro(i) => app.filtro = i,
        Mensaje::Seleccionar(i) => {
            app.seleccionada = i;
            app.menu = None;
        }
        Mensaje::Menu(i) => app.menu = i,
        Mensaje::PedirEliminar => {
            app.menu = None;
            app.dialogo = true;
        }
        Mensaje::CerrarDialogo => app.dialogo = false,
        Mensaje::Eliminar => {
            app.dialogo = false;
            app.aviso = Some("Se eliminó la instancia Supervivencia.".into());
        }
        Mensaje::CerrarAviso => app.aviso = None,
        Mensaje::Tema(i) => {
            app.tema = i;
            app.modo = if i == 1 { Modo::Piedra } else { Modo::Tinta };
        }
        Mensaje::Reducir => app.reducir = !app.reducir,
        Mensaje::Simultaneas(n) => app.simultaneas = n.clamp(1, 8),
        Mensaje::Ajuste(i) => app.ajuste = i,
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

fn instancias() -> Vec<DatosInstancia> {
    let nueva = |nombre: &str, version: &str, vez: &str, mods: &str, portada, estado, detalle: &str| {
        DatosInstancia {
            nombre: nombre.into(),
            version: version.into(),
            ultima_vez: vez.into(),
            mods: mods.into(),
            portada,
            estado,
            seleccionada: false,
            detalle: detalle.into(),
        }
    };
    vec![
        nueva("Supervivencia", "1.21.4 · Fabric", "Hace 2 h", "214 mods", Lamina::CaballeroYelmo, EstadoInstancia::Lista, ""),
        nueva("Better MC", "1.20.1 · NeoForge", "Hace 6 días", "312 mods", Lamina::CastilloTorres, EstadoInstancia::Desactualizada, ""),
        nueva("Skyblock", "1.21.4 · Quilt", "Nueva", "48 mods", Lamina::IcaroDedalo, EstadoInstancia::Instalando(0.58), "Descargando librerías · 112 de 186"),
        nueva("Mundo de Javier", "1.21.1 · Fabric", "Ahora", "96 mods", Lamina::ProdigoAldea, EstadoInstancia::Jugando, "En ejecución · 00:42:17"),
        nueva("Vanilla", "1.21.4 · Vanilla", "Hace 1 mes", "Sin mods", Lamina::ValleRocas, EstadoInstancia::Error, "Falta Java 21"),
    ]
}

fn pantalla_instancias(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    if let Some(pestana) = app.editor {
        return pantalla_editor(app, pestana);
    }
    let mut chips = row![].spacing(espacio::S2);
    for (i, f) in ["Todas", "Fabric", "NeoForge", "Vanilla"].iter().enumerate() {
        chips = chips.push(componentes::chip(p, f, app.filtro == i, Mensaje::Filtro(i)));
    }
    let herramientas = row![
        chips,
        Space::with_width(Length::Fill),
        componentes::boton(p, "Importar", Variante::Secundario, Some(Mensaje::Nada)),
        componentes::boton(p, "Crear instancia", Variante::Primario, Some(Mensaje::Nada)),
    ]
    .spacing(espacio::S2)
    .align_y(Alignment::Center);

    let mut tarjetas: Vec<Element<Mensaje>> = Vec::new();
    for (i, mut d) in instancias().into_iter().enumerate() {
        d.seleccionada = i == app.seleccionada;
        tarjetas.push(tarjeta_instancia(
            p,
            &d,
            AccionesTarjeta {
                seleccionar: Mensaje::Seleccionar(i),
                principal: Mensaje::Nada,
                mas: Mensaje::Menu(Some(i)),
            },
        ));
    }
    tarjetas.push(tarjeta_nueva(p, Mensaje::Nada));
    let mut cuadricula = column![].spacing(espacio::S6);
    let mut it = tarjetas.into_iter();
    loop {
        let mut fila = row![].spacing(espacio::S6);
        let mut n = 0;
        for _ in 0..3 {
            match it.next() {
                Some(t) => {
                    fila = fila.push(t);
                    n += 1;
                }
                None => fila = fila.push(Space::with_width(Length::Fill)),
            }
        }
        if n == 0 {
            break;
        }
        cuadricula = cuadricula.push(fila);
    }
    let cuerpo = column![herramientas, cuadricula]
        .spacing(espacio::S6)
        .padding(Padding::from([espacio::S6, espacio::S12]));
    column![
        banda(p, Lamina::CaidaCielo, "Instancias", Some(5)),
        scrollable(cuerpo).height(Length::Fill)
    ]
    .into()
}

fn pantalla_editor(app: &App, pestana: usize) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    let datos = DatosEditor {
        nombre: "Supervivencia".into(),
        version: "1.21.4 · Fabric 0.16.9".into(),
        jugado: "41 h".into(),
        mods: "214".into(),
        tamano: "2,4 GB".into(),
        memoria: "6.144 MB".into(),
        portada: Lamina::CastilloTorres,
    };
    let encabezado = encabezado_editor(
        p,
        &datos,
        MensajesEncabezado {
            volver: Mensaje::VolverEditor,
            abrir_carpeta: Mensaje::Nada,
            mas: Mensaje::Nada,
            jugar: Mensaje::Nada,
        },
    );
    let barra = container(pestanas_editor(p, pestana, 214, 3, Mensaje::Pestana))
        .padding(Padding::from([0.0, espacio::S12]));
    let cuerpo: Element<Mensaje> = match pestana {
        0 => pestana_general(
            p,
            &app.general,
            &MensajesGeneral {
                nombre: Mensaje::Nombre,
                grupo: Mensaje::Grupo,
                ancho: Mensaje::Ancho,
                alto: Mensaje::Alto,
                pantalla_completa: Mensaje::Completa,
                servidor: Mensaje::Servidor,
                memoria: Mensaje::Memoria,
            },
            16384,
        ),
        1 => pestana_java(
            p,
            app.java_auto,
            Mensaje::JavaAuto,
            &[
                InstalacionJava { version: "21.0.5".into(), proveedor: "Temurin".into(), gestionada: true, ruta: "…/icaro/java/21".into(), usado_por: "1.20.5 en adelante".into(), existe: true },
                InstalacionJava { version: "17.0.13".into(), proveedor: "Temurin".into(), gestionada: true, ruta: "…/icaro/java/17".into(), usado_por: "1.18 a 1.20.4".into(), existe: true },
                InstalacionJava { version: "19".into(), proveedor: "No encontrado".into(), gestionada: false, ruta: "D:/jdk-19/bin/javaw.exe".into(), usado_por: "Ninguna".into(), existe: false },
            ],
        ),
        3 => {
            let nombres = [("Faithful 32x", "Faithful Team", "32x", CompatPaquete::Compatible), ("Soft Fantasy", "Aeris", "16x", CompatPaquete::HechoPara("1.20".into())), ("Por defecto", "Mojang", "16x", CompatPaquete::Compatible)];
            let paquetes: Vec<Paquete> = nombres
                .into_iter()
                .enumerate()
                .map(|(i, (n, a, r, c))| Paquete {
                    nombre: n.into(),
                    autor: a.into(),
                    resolucion: r.into(),
                    activo: app.paquetes_activos[i],
                    compat: c,
                    fijo: i == 2,
                })
                .collect();
            pestana_recursos(p, app.vista_recursos, Mensaje::VistaRecursos, &paquetes, Mensaje::Paquete)
        }
        4 => pestana_mundos(
            p,
            &[
                Mundo { nombre: "Isla del Faro".into(), modo: "Supervivencia".into(), hardcore: false, ultima_vez: "Hace 2 h".into(), tamano: "1,2 GB".into(), copias: 4 },
                Mundo { nombre: "Prueba Hardcore".into(), modo: "Hardcore".into(), hardcore: true, ultima_vez: "Hace 3 días".into(), tamano: "310 MB".into(), copias: 0 },
                Mundo { nombre: "Creativo".into(), modo: "Creativo".into(), hardcore: false, ultima_vez: "Hace 1 mes".into(), tamano: "880 MB".into(), copias: 2 },
            ],
            app.mundo,
            Mensaje::Mundo,
            &[
                Copia { fecha: "2026-10-09 01:12".into(), tamano: "1,1 GB".into(), motivo: "Automática al jugar".into() },
                Copia { fecha: "2026-10-04 22:40".into(), tamano: "1,0 GB".into(), motivo: "Antes de cambiar versión".into() },
            ],
        ),
        5 => container(consola(p, &lineas_consola(), app.filtro_consola, &app.busqueda_consola, MensajesConsola { filtro: Mensaje::FiltroConsola, buscar: Mensaje::BuscarConsola, copiar: Mensaje::Nada, limpiar: Mensaje::Nada }))
            .padding(Padding::from([espacio::S6, espacio::S12]))
            .into(),
        6 => pestana_archivos(
            p,
            &[
                Nodo { nombre: "mods".into(), profundidad: 0, carpeta: true, bloqueado: false },
                Nodo { nombre: "fabric-api-0.112.0.jar".into(), profundidad: 1, carpeta: false, bloqueado: true },
                Nodo { nombre: "sodium-0.6.5.jar".into(), profundidad: 1, carpeta: false, bloqueado: true },
                Nodo { nombre: "config".into(), profundidad: 0, carpeta: true, bloqueado: false },
                Nodo { nombre: "options.txt".into(), profundidad: 0, carpeta: false, bloqueado: false },
            ],
            app.archivo,
            Mensaje::Archivo,
            "version:3955\nautoJump:false\nrenderDistance:16\nresourcePacks:[\"file/Faithful 32x.zip\"]",
        ),
        _ => pendiente(app),
    };
    column![encabezado, barra, cuerpo].into()
}

fn lineas_consola() -> Vec<Linea> {
    let l = |h: &str, n, m: &str| Linea { hora: h.into(), nivel: n, mensaje: m.into() };
    vec![
        l("14:02:11", Nivel::Info, "[main/INFO]: Loading Minecraft 1.21.4 with Fabric Loader 0.16.9"),
        l("14:02:12", Nivel::Info, "[main/INFO]: Loading 214 mods"),
        l("14:02:19", Nivel::Warn, "[main/WARN]: Mod sodium requests a newer version of fabric-api"),
        l("14:02:24", Nivel::Info, "[Render thread/INFO]: Setting user: Mineral_7"),
        l("14:02:31", Nivel::Error, "[Render thread/ERROR]: Failed to load texture minecraft:textures/block/stone.png"),
        l("14:02:33", Nivel::Info, "[Render thread/INFO]: Reloading ResourceManager"),
    ]
}

fn pantalla_consola(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    column![
        container(
            text("CONSOLA")
                .font(fuentes::DISPLAY)
                .size(texto::DISPLAY_XL.0)
                .color(p.text)
        )
        .padding(Padding::from([espacio::S6, espacio::S12])),
        container(consola(p, &lineas_consola(), app.filtro_consola, &app.busqueda_consola, MensajesConsola { filtro: Mensaje::FiltroConsola, buscar: Mensaje::BuscarConsola, copiar: Mensaje::Nada, limpiar: Mensaje::Nada }))
            .padding(Padding::from([0.0, espacio::S12]))
            .height(Length::Fill),
        Space::with_height(espacio::S6),
    ]
    .into()
}

fn pantalla_capturas(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    let ejemplos = [
        ("2026-10-09_01.12.04.png", "Hoy 01:12", Lamina::CaballeroYelmo),
        ("2026-10-08_22.40.31.png", "Ayer 22:40", Lamina::ValleRocas),
        ("2026-10-07_19.03.55.png", "Hace 2 días", Lamina::CastilloTorres),
        ("2026-10-05_16.20.10.png", "Hace 4 días", Lamina::ProdigoAldea),
        ("2026-10-02_11.48.09.png", "Hace 1 semana", Lamina::DragonCabeza),
        ("2026-09-28_09.15.42.png", "Hace 11 días", Lamina::IcaroDedalo),
    ];
    let capturas: Vec<Captura> = ejemplos
        .iter()
        .map(|(a, f, l)| Captura { archivo: (*a).into(), fecha: (*f).into(), imagen: *l })
        .collect();
    let cuerpo = column![
        container(
            text("CAPTURAS")
                .font(fuentes::DISPLAY)
                .size(texto::DISPLAY_XL.0)
                .color(p.text)
        ),
        galeria(
            p,
            &["Todas", "Supervivencia", "Better MC"],
            app.filtro_capturas,
            Mensaje::FiltroCapturas,
            &capturas,
            app.captura,
            Mensaje::Captura,
        ),
    ]
    .spacing(espacio::S6)
    .padding(Padding::from([espacio::S6, espacio::S12]));
    scrollable(cuerpo).height(Length::Fill).into()
}

fn pantalla_servidores(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    let amigos = Servidor {
        nombre: "Amigos".into(),
        direccion: "10.147.17.1:25565".into(),
        motd: Some("Temporada 4: el nether está abierto".into()),
        estado: EstadoServidor::EnLinea { jugadores: 12, maximo: 20, ms: 24 },
        vinculo: Some(("Supervivencia".into(), true)),
        version_distinta: None,
    };
    let creativo = Servidor {
        nombre: "Creativo de la U".into(),
        direccion: "mc.creativo-u.cl".into(),
        motd: None,
        estado: EstadoServidor::FueraDeLinea("desde las 03:12".into()),
        vinculo: None,
        version_distinta: Some("1.20.1".into()),
    };
    let cuerpo = column![
        red_privada(p, true, "Red Amigos", "10.147.17.4", (5, 7), Mensaje::Nada),
        row![
            text("Arrastra para ordenar").size(texto::BODY_SM.0).color(p.text_muted),
            Space::with_width(Length::Fill),
            componentes::boton(p, "Agregar servidor", Variante::Primario, Some(Mensaje::Nada)),
        ]
        .align_y(Alignment::Center),
        tarjeta_servidor(p, &amigos, Mensaje::Nada),
        tarjeta_servidor(p, &creativo, Mensaje::Nada),
    ]
    .spacing(espacio::S4)
    .padding(Padding::from([espacio::S6, espacio::S12]));
    column![
        banda(p, Lamina::TivoliCiudad, "Servidores", Some(2)),
        scrollable(cuerpo).height(Length::Fill)
    ]
    .into()
}

fn pantalla_descargas(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    let cola = vec![
        Descarga {
            nombre: "Assets de Minecraft 1.21.4".into(),
            destino: "Skyblock".into(),
            estado: EstadoDescarga::Activa(0.6),
            detalle: "412,8 de 687,0 MB · 4,2 MB/s · quedan 1 min 05 s".into(),
        },
        Descarga {
            nombre: "Fabric API 0.112.0".into(),
            destino: "Supervivencia".into(),
            estado: EstadoDescarga::Pausada(0.34),
            detalle: "En pausa · 0,8 de 2,3 MB".into(),
        },
        Descarga {
            nombre: "Iris Shaders 1.8.1".into(),
            destino: "Supervivencia".into(),
            estado: EstadoDescarga::Fallo("Modrinth no responde. Se reintentará en 30 s.".into()),
            detalle: String::new(),
        },
        Descarga {
            nombre: "Java 21 (Temurin)".into(),
            destino: "Sistema".into(),
            estado: EstadoDescarga::EnCola(1),
            detalle: "46,1 MB".into(),
        },
    ];
    let acciones = AccionesCola {
        pausar: |_| Mensaje::Nada,
        cancelar: |_| Mensaje::Nada,
        reintentar: |_| Mensaje::Nada,
    };
    let hist = vec![
        EntradaHistorial { nombre: "Fabric API 0.112.0".into(), destino: "Supervivencia".into(), tamano: "2,3 MB".into(), fecha: "Hoy 01:12".into() },
        EntradaHistorial { nombre: "Java 21 (Temurin)".into(), destino: "Sistema".into(), tamano: "46,1 MB".into(), fecha: "Ayer 22:40".into() },
    ];
    let cuerpo = column![
        cola_descargas(p, "6,1 MB/s", "3 activas · 2 en cola", &cola, &acciones),
        historial(p, &hist, Mensaje::Nada),
        limites(p, app.simultaneas, Mensaje::Simultaneas(app.simultaneas - 1), Mensaje::Simultaneas(app.simultaneas + 1), "Sin límite"),
    ]
    .spacing(espacio::S8)
    .padding(Padding::from([espacio::S6, espacio::S12]));
    column![
        banda(p, Lamina::MercurioFriso, "Descargas", None),
        scrollable(cuerpo).height(Length::Fill)
    ]
    .into()
}

fn pantalla_ajustes(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    let secciones = [
        "General", "Apariencia", "Java y memoria", "Almacenamiento", "Red", "Comportamiento",
        "Atajos", "Acerca de",
    ];
    let pagina = column![
        encabezado_ajustes(p, "Apariencia", None),
        fila_ajuste(
            p,
            "Tema",
            "Tinta es oscuro, Piedra es claro. Sistema sigue a tu equipo.",
            componentes::segmentado(p, &["Tinta", "Piedra", "Sistema"], app.tema, Mensaje::Tema),
        ),
        fila_ajuste(
            p,
            "Reducir movimiento",
            "Quita animaciones; el Umbral pasa a ser un fundido.",
            componentes::interruptor(p, app.reducir, Mensaje::Reducir),
        ),
        fila_ajuste(
            p,
            "Descargas simultáneas",
            "Con internet lento, bájalo a 2.",
            componentes::contador(
                p,
                app.simultaneas,
                Mensaje::Simultaneas(app.simultaneas - 1),
                Mensaje::Simultaneas(app.simultaneas + 1),
            ),
        ),
    ]
    .spacing(espacio::S2)
    .width(Length::Fill);
    let cuerpo = row![
        indice_ajustes(p, &secciones, app.ajuste, Mensaje::Ajuste),
        pagina
    ]
    .spacing(40.0)
    .padding(Padding::from([espacio::S6, espacio::S12]));
    column![
        banda(p, Lamina::FarnesioCielo, "Ajustes", None),
        scrollable(cuerpo).height(Length::Fill)
    ]
    .into()
}

fn pendiente(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    container(
        text(format!("{} · pendiente", app.seccion.nombre().to_uppercase()))
            .font(fuentes::DISPLAY)
            .size(texto::DISPLAY.0)
            .color(p.text_muted),
    )
    .center(Length::Fill)
    .into()
}

fn vista(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    let contenido = match app.seccion {
        Seccion::Instancias => pantalla_instancias(app),
        Seccion::Servidores => pantalla_servidores(app),
        Seccion::Descargas => pantalla_descargas(app),
        Seccion::Ajustes => pantalla_ajustes(app),
        Seccion::Consola => pantalla_consola(app),
        Seccion::Capturas => pantalla_capturas(app),
        _ => pendiente(app),
    };
    let pie = row![
        column![
            text("Supervivencia")
                .font(fuentes::TITULO)
                .size(texto::HEADING.0)
                .color(p.text),
            text("1.21.4 · FABRIC")
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted),
        ],
        Space::with_width(Length::Fill),
        componentes::boton(p, "Jugar", Variante::Primario, Some(Mensaje::Nada)),
    ]
    .align_y(Alignment::Center)
    .padding(Padding::from([espacio::S2, espacio::S12]));
    let mut ventana = app_shell(
        p,
        app.seccion,
        app.colapsada,
        Cuenta { nombre: "Mineral_7", proveedor: "Microsoft" },
        Some(DescargasActivas { cantidad: 2, velocidad: "6,1 MB/s".into(), avance: 0.58 }),
        contenido,
        (app.seccion == Seccion::Instancias && app.editor.is_none()).then(|| pie.into()),
        MensajesShell {
            ir_a: Box::new(Mensaje::Ir),
            arrastrar: Mensaje::Arrastrar,
            buscar: Mensaje::Nada,
            minimizar: Mensaje::Minimizar,
            maximizar: Mensaje::Maximizar,
            cerrar: Mensaje::Cerrar,
        },
    );
    if app.menu.is_some() {
        let menu = menu_contextual(
            p,
            vec![
                ElementoMenu::Opcion { icono: Icono::Jugar, texto: "Jugar", atajo: Some("Enter"), peligro: false, mensaje: Mensaje::Menu(None) },
                ElementoMenu::Opcion { icono: Icono::Carpeta, texto: "Abrir carpeta", atajo: Some("O"), peligro: false, mensaje: Mensaje::Menu(None) },
                ElementoMenu::Opcion { icono: Icono::Editar, texto: "Editar", atajo: Some("E"), peligro: false, mensaje: Mensaje::Editar },
                ElementoMenu::Separador,
                ElementoMenu::Opcion { icono: Icono::Copiar, texto: "Duplicar", atajo: None, peligro: false, mensaje: Mensaje::Menu(None) },
                ElementoMenu::Opcion { icono: Icono::Subir, texto: "Exportar modpack", atajo: None, peligro: false, mensaje: Mensaje::Menu(None) },
                ElementoMenu::Opcion { icono: Icono::Externo, texto: "Crear acceso directo", atajo: None, peligro: false, mensaje: Mensaje::Menu(None) },
                ElementoMenu::Separador,
                ElementoMenu::Opcion { icono: Icono::Papelera, texto: "Eliminar", atajo: Some("Supr"), peligro: true, mensaje: Mensaje::PedirEliminar },
            ],
        );
        ventana = iced::widget::stack![
            ventana,
            iced::widget::mouse_area(
                container(menu).padding(Padding::from([140.0, 420.0])),
            )
            .on_press(Mensaje::Menu(None))
        ]
        .into();
    }
    if app.dialogo {
        let cuerpo = text("Se borrarán la instancia, sus 214 mods y 3 mundos (2,4 GB). Esta acción no se puede deshacer.")
            .size(texto::BODY.0)
            .color(p.text_muted)
            .into();
        ventana = con_velo(
            p,
            ventana,
            dialogo(
                p,
                "¿Eliminar Supervivencia?",
                cuerpo,
                vec![
                    componentes::boton(p, "Cancelar", Variante::Secundario, Some(Mensaje::CerrarDialogo)),
                    componentes::boton(p, "Eliminar Supervivencia", Variante::Peligro, Some(Mensaje::Eliminar)),
                ],
                Some(Lamina::CaidaCielo),
                620.0,
                300.0,
                Mensaje::CerrarDialogo,
            ),
            Mensaje::CerrarDialogo,
        );
    }
    if let Some(a) = &app.aviso {
        ventana = iced::widget::stack![
            ventana,
            container(toast(p, Icono::Check, a, Some(("Deshacer", Mensaje::CerrarAviso)), Some(Mensaje::CerrarAviso), false))
                .align_x(iced::alignment::Horizontal::Left)
                .align_y(iced::alignment::Vertical::Bottom)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(Padding::from([espacio::S6, 256.0]))
        ]
        .into();
    }
    ventana
}

fn main() -> iced::Result {
    let mut app = iced::application("Ícaro", actualizar, vista)
        .theme(|_| iced::Theme::Dark)
        .default_font(fuentes::CUERPO)
        .window(window::Settings {
            decorations: false,
            size: Size::new(1280.0, 800.0),
            min_size: Some(Size::new(940.0, 600.0)),
            ..Default::default()
        });
    for f in fuentes::BYTES {
        app = app.font(f);
    }
    // Variables opcionales para abrir el ejemplo en un estado concreto:
    // ICARO_SECCION (1 a 7), ICARO_PIEDRA, ICARO_MENU, ICARO_DIALOGO.
    let seccion = std::env::var("ICARO_SECCION")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .and_then(|n| Seccion::TODAS.get(n.wrapping_sub(1)).copied())
        .unwrap_or(Seccion::Instancias);
    let piedra = std::env::var_os("ICARO_PIEDRA").is_some();
    let menu = std::env::var_os("ICARO_MENU").is_some().then_some(0);
    let con_dialogo = std::env::var_os("ICARO_DIALOGO").is_some();
    app.run_with(move || {
        (
            App {
                modo: if piedra { Modo::Piedra } else { Modo::Tinta },
                seccion,
                colapsada: false,
                filtro: 0,
                seleccionada: 0,
                menu,
                dialogo: con_dialogo,
                aviso: None,
                tema: usize::from(piedra),
                reducir: false,
                simultaneas: 4,
                ajuste: 1,
                editor: std::env::var("ICARO_EDITOR").ok().and_then(|v| v.parse().ok()),
                general: EstadoGeneral {
                    nombre: "Supervivencia".into(),
                    grupo: Some("Con amigos"),
                    ancho: "1920".into(),
                    alto: "1080".into(),
                    pantalla_completa: false,
                    servidor: Some("Amigos · 10.147.17.1"),
                    memoria: 6144,
                },
                java_auto: true,
                vista_recursos: 0,
                paquetes_activos: [true, false, true],
                mundo: 0,
                archivo: 1,
                filtro_consola: 0,
                busqueda_consola: String::new(),
                filtro_capturas: 0,
                captura: Some(1),
            },
            Task::none(),
        )
    })
}
