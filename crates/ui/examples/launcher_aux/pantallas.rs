// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Pantallas completas, diálogos y páginas de Ajustes del launcher: primera
//! ejecución, inicio de sesión con Microsoft, errores, asistente de
//! instancias, importar, exportar y las secciones de Ajustes.

use std::time::Instant;

use icaro_ui::ajustes::{encabezado_ajustes, fila_ajuste};
use icaro_ui::componentes::{self, casilla, etiqueta, insignia, Estado, Variante};
use icaro_ui::fuentes;
use icaro_ui::iconos::{icono, Icono, Tam};
use icaro_ui::instancias::{DatosInstancia, EstadoInstancia};
use icaro_ui::laminas::{grabado, Lamina};
use icaro_ui::tema::{espacio, texto, Paleta};
use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Element, Length, Padding, Task};

use crate::{arrancar, config_por_defecto, instancias, lista_instancias, App, Mensaje};

/// Pantallas que reemplazan el contenido de la ventana.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pantalla {
    Primera,
    Codigo,
    Fallo,
    ErrorTienda,
    NoVerificados,
    Sincronizar,
    Integridad,
    Actualizacion,
    Cuenta,
    Red,
}

/// Fase del inicio de sesión por código de dispositivo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FaseCodigo {
    Esperando,
    Confirmado(Instant),
    Vencido,
}

/// Preferencias de la ventana de Ajustes.
pub struct Ajustes {
    pub idioma: String,
    pub abrir_al_iniciar: bool,
    pub iniciar_minimizado: bool,
    pub al_jugar: usize,
    pub confirmar_borrar: bool,
    pub consola_al_jugar: bool,
    pub copias_mundos: bool,
    pub limite_mb: i32,
    pub escala: i32,
    pub cache_limpia: bool,
}

impl Default for Ajustes {
    fn default() -> Self {
        Self {
            idioma: "Español (Chile)".into(),
            abrir_al_iniciar: false,
            iniciar_minimizado: false,
            al_jugar: 0,
            confirmar_borrar: true,
            consola_al_jugar: false,
            copias_mundos: true,
            limite_mb: 0,
            escala: 100,
            cache_limpia: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CampoAjuste {
    AbrirAlIniciar,
    IniciarMinimizado,
    ConfirmarBorrar,
    ConsolaAlJugar,
    CopiasMundos,
}

/// Asistente de creación de instancias, en cuatro pasos.
pub struct Asistente {
    pub paso: usize,
    pub tipo_version: usize,
    pub busqueda: String,
    pub version: usize,
    pub loader: usize,
    pub nombre: String,
    pub portada: usize,
}

pub const PASOS: [&str; 4] = ["Versión", "Loader", "Nombre e icono", "Revisar"];
pub const LOADERS: [(&str, &str, &str); 4] = [
    ("Vanilla", "Minecraft tal cual, sin mods.", ""),
    ("Fabric", "Liviano y rápido. El más usado para mods de rendimiento.", "0.16.9"),
    ("NeoForge", "Para modpacks grandes de tecnología y magia.", "21.4.38"),
    ("Forge", "El clásico de los mods grandes.", "54.0.1"),
];
pub const VERSIONES: [[(&str, &str); 5]; 3] = [
    [("1.21.4", "dic 2024"), ("1.21.3", "oct 2024"), ("1.21.1", "ago 2024"), ("1.20.6", "abr 2024"), ("1.20.1", "jun 2023")],
    [("25w02a", "ene 2025"), ("24w46a", "nov 2024"), ("24w39a", "sep 2024"), ("24w33a", "ago 2024"), ("24w21b", "may 2024")],
    [("1.16.5", "ene 2021"), ("1.12.2", "sep 2017"), ("1.8.9", "dic 2015"), ("1.7.10", "jun 2014"), ("1.6.4", "sep 2013")],
];
pub const PORTADAS: [(&str, Lamina); 4] = [
    ("Castillo", Lamina::CastilloPasarela),
    ("Valle", Lamina::TivoliCascada),
    ("Aldea", Lamina::MercadoAldea),
    ("Ciudad", Lamina::TivoliRocas),
];

/// Importación de una instancia desde un archivo, otro launcher o un clon.
pub struct Importar {
    pub origen: usize,
    pub archivo: Option<&'static str>,
    pub launchers: [bool; 3],
    pub clon_origen: usize,
    pub con_mundos: bool,
}

/// Exportación de una instancia como modpack `.mrpack`.
pub struct Exportar {
    pub instancia: usize,
    pub nombre: String,
    pub version: String,
    pub autor: String,
    pub carpetas: [bool; 4],
}

/// Todo lo que cambian estas pantallas.
#[derive(Debug, Clone)]
pub enum Accion {
    Abrir(Pantalla),
    Cerrar,
    // Ajustes
    Idioma(String),
    Alternar(CampoAjuste),
    AlJugar(String),
    Limite(i32),
    Escala(i32),
    LimpiarCache,
    AbrirCarpetaDatos,
    // Asistente
    NuevaInstancia,
    AsistenteCancelar,
    AsistenteAtras,
    AsistenteSiguiente,
    AsistentePaso(usize),
    AsistenteTipo(usize),
    AsistenteBuscar(String),
    AsistenteVersion(usize),
    AsistenteLoader(usize),
    AsistenteNombre(String),
    AsistentePortada(usize),
    AsistenteCrear,
    // Importar
    Importar,
    ImportarOrigen(usize),
    ImportarArchivo,
    ImportarLauncher(usize),
    ImportarClon(String),
    ImportarMundos(bool),
    ImportarCancelar,
    ImportarConfirmar,
    // Exportar
    Exportar(usize),
    ExportarNombre(String),
    ExportarVersion(String),
    ExportarAutor(String),
    ExportarCarpeta(usize),
    ExportarCancelar,
    ExportarConfirmar,
    // Cuenta y errores
    CopiarCodigo,
    AbrirPaginaMicrosoft,
    PedirOtroCodigo,
    SimularConfirmacion,
    JugarComoInvitado,
    Reintentar,
    DesactivarYJugar,
    CopiarInforme,
    DetalleTecnico,
    CambiarVersion(usize),
    CambiarCancelar,
    CambiarConfirmar(usize),
    VerInstalados,
    Duplicar(usize),
    AbrirCarpetaInstancia(usize),
}

fn nueva_instancia(nombre: &str, version: String, portada: Lamina) -> DatosInstancia {
    DatosInstancia {
        nombre: nombre.to_owned(),
        version,
        ultima_vez: "Nueva".into(),
        mods: "Sin mods".into(),
        portada,
        estado: EstadoInstancia::Lista,
        seleccionada: false,
        detalle: String::new(),
    }
}

/// Nombre que no choque con ninguna instancia existente.
fn nombre_unico(base: &str) -> String {
    let existentes: Vec<String> = instancias().into_iter().map(|d| d.nombre).collect();
    if !existentes.iter().any(|n| n == base) {
        return base.to_owned();
    }
    (2..)
        .map(|n| format!("{base} {n}"))
        .find(|c| !existentes.contains(c))
        .unwrap_or_else(|| base.to_owned())
}

/// Agrega la instancia a la lista y a los arreglos paralelos de la app.
fn agregar_instancia(app: &mut App, d: DatosInstancia) {
    lista_instancias().lock().unwrap().push(d);
    app.grupos.push(None);
    app.configs.push(config_por_defecto());
}

pub fn actualizar(app: &mut App, a: Accion) -> Task<Mensaje> {
    match a {
        Accion::Abrir(p) => {
            app.pantalla = Some(p);
            if p == Pantalla::Codigo {
                app.codigo_inicio = Instant::now();
                app.codigo_fase = FaseCodigo::Esperando;
            }
            if p == Pantalla::ErrorTienda {
                app.intentos = 0;
            }
            if p == Pantalla::Integridad {
                app.codigo_inicio = Instant::now();
            }
            arrancar(app);
        }
        Accion::Cerrar => {
            app.pantalla = None;
            arrancar(app);
        }
        Accion::Idioma(v) => app.aj.idioma = v,
        Accion::Alternar(c) => match c {
            CampoAjuste::AbrirAlIniciar => app.aj.abrir_al_iniciar = !app.aj.abrir_al_iniciar,
            CampoAjuste::IniciarMinimizado => app.aj.iniciar_minimizado = !app.aj.iniciar_minimizado,
            CampoAjuste::ConfirmarBorrar => app.aj.confirmar_borrar = !app.aj.confirmar_borrar,
            CampoAjuste::ConsolaAlJugar => app.aj.consola_al_jugar = !app.aj.consola_al_jugar,
            CampoAjuste::CopiasMundos => app.aj.copias_mundos = !app.aj.copias_mundos,
        },
        Accion::AlJugar(v) => {
            app.aj.al_jugar = ["Mantener abierto", "Minimizar", "Cerrar el launcher"]
                .iter()
                .position(|o| *o == v)
                .unwrap_or(0);
        }
        Accion::Limite(n) => app.aj.limite_mb = n.clamp(0, 100),
        Accion::Escala(n) => app.aj.escala = n.clamp(80, 150),
        Accion::LimpiarCache => {
            app.aj.cache_limpia = true;
            app.aviso = Some("Caché limpiada: se liberaron 380 MB.".into());
        }
        Accion::AbrirCarpetaDatos => {
            let dir = std::env::temp_dir().join("icaro-demo");
            let _ = std::fs::create_dir_all(&dir);
            abrir_en_sistema(&dir);
        }
        // ---- Asistente
        Accion::NuevaInstancia => {
            app.asistente = Some(Asistente {
                paso: 0,
                tipo_version: 0,
                busqueda: String::new(),
                version: 0,
                loader: 0,
                nombre: String::new(),
                portada: 0,
            });
        }
        Accion::AsistenteCancelar => app.asistente = None,
        Accion::AsistenteAtras => {
            if let Some(s) = app.asistente.as_mut() {
                s.paso = s.paso.saturating_sub(1);
            }
        }
        Accion::AsistenteSiguiente => {
            if let Some(s) = app.asistente.as_mut() {
                s.paso = (s.paso + 1).min(3);
                if s.paso == 2 && s.nombre.is_empty() {
                    let v = VERSIONES[s.tipo_version][s.version].0;
                    s.nombre = format!("{} {v}", LOADERS[s.loader].0);
                }
            }
        }
        Accion::AsistentePaso(i) => {
            if let Some(s) = app.asistente.as_mut() {
                if i < s.paso {
                    s.paso = i;
                }
            }
        }
        Accion::AsistenteTipo(i) => {
            if let Some(s) = app.asistente.as_mut() {
                s.tipo_version = i.min(2);
                s.version = 0;
            }
        }
        Accion::AsistenteBuscar(t) => {
            if let Some(s) = app.asistente.as_mut() {
                s.busqueda = t;
            }
        }
        Accion::AsistenteVersion(i) => {
            if let Some(s) = app.asistente.as_mut() {
                s.version = i;
            }
        }
        Accion::AsistenteLoader(i) => {
            if let Some(s) = app.asistente.as_mut() {
                s.loader = i;
            }
        }
        Accion::AsistenteNombre(t) => {
            if let Some(s) = app.asistente.as_mut() {
                s.nombre = t;
            }
        }
        Accion::AsistentePortada(i) => {
            if let Some(s) = app.asistente.as_mut() {
                s.portada = i;
            }
        }
        Accion::AsistenteCrear => {
            if let Some(s) = app.asistente.take() {
                let nombre = nombre_unico(if s.nombre.trim().is_empty() { "Nueva instancia" } else { s.nombre.trim() });
                let version = format!("{} · {}", VERSIONES[s.tipo_version][s.version].0, LOADERS[s.loader].0);
                agregar_instancia(app, nueva_instancia(&nombre, version, PORTADAS[s.portada].1));
                app.aviso = Some(format!("Instancia {nombre} creada."));
                arrancar(app);
            }
        }
        // ---- Importar
        Accion::Importar => {
            app.importar = Some(Importar {
                origen: 0,
                archivo: None,
                launchers: [true, false, false],
                clon_origen: 0,
                con_mundos: false,
            });
        }
        Accion::ImportarOrigen(i) => {
            if let Some(s) = app.importar.as_mut() {
                s.origen = i.min(2);
            }
        }
        Accion::ImportarArchivo => {
            if let Some(s) = app.importar.as_mut() {
                s.archivo = Some("Fabulously Optimized 2.1.0.mrpack");
            }
        }
        Accion::ImportarLauncher(i) => {
            if let Some(s) = app.importar.as_mut() {
                if let Some(c) = s.launchers.get_mut(i) {
                    *c = !*c;
                }
            }
        }
        Accion::ImportarClon(nombre) => {
            if let Some(s) = app.importar.as_mut() {
                s.clon_origen = instancias().iter().position(|d| d.nombre == nombre).unwrap_or(0);
            }
        }
        Accion::ImportarMundos(v) => {
            if let Some(s) = app.importar.as_mut() {
                s.con_mundos = v;
            }
        }
        Accion::ImportarCancelar => app.importar = None,
        Accion::ImportarConfirmar => {
            if let Some(s) = app.importar.take() {
                let mut creadas = Vec::new();
                match s.origen {
                    0 => {
                        if s.archivo.is_some() {
                            creadas.push(nueva_instancia(
                                &nombre_unico("Fabulously Optimized"),
                                "1.21.4 · Fabric".into(),
                                Lamina::CascadaMolino,
                            ));
                        }
                    }
                    1 => {
                        for (i, (nombre, version)) in [
                            ("Mundo de Prism", "1.20.1 · Fabric"),
                            ("Pruebas de MultiMC", "1.19.4 · Forge"),
                            ("Pack de CurseForge", "1.20.1 · NeoForge"),
                        ]
                        .iter()
                        .enumerate()
                        {
                            if s.launchers[i] {
                                creadas.push(nueva_instancia(&nombre_unico(nombre), (*version).into(), Lamina::ValleRocas));
                            }
                        }
                    }
                    _ => {
                        let todas = instancias();
                        if let Some(o) = todas.get(s.clon_origen) {
                            let mut d = o.clone();
                            d.nombre = nombre_unico(&format!("{} (copia)", o.nombre));
                            d.estado = EstadoInstancia::Lista;
                            d.detalle.clear();
                            d.ultima_vez = "Nueva".into();
                            creadas.push(d);
                        }
                    }
                }
                let n = creadas.len();
                for d in creadas {
                    agregar_instancia(app, d);
                }
                app.aviso = Some(if n == 0 { "No se importó nada.".into() } else { format!("{n} instancia(s) importada(s).") });
                arrancar(app);
            }
        }
        // ---- Exportar
        Accion::Exportar(i) => {
            let d = &instancias()[i];
            app.exportar = Some(Exportar {
                instancia: i,
                nombre: d.nombre.clone(),
                version: "1.0.0".into(),
                autor: "Mineral_7".into(),
                carpetas: [true, true, false, false],
            });
            app.menu = None;
        }
        Accion::ExportarNombre(t) => {
            if let Some(s) = app.exportar.as_mut() {
                s.nombre = t;
            }
        }
        Accion::ExportarVersion(t) => {
            if let Some(s) = app.exportar.as_mut() {
                s.version = t;
            }
        }
        Accion::ExportarAutor(t) => {
            if let Some(s) = app.exportar.as_mut() {
                s.autor = t;
            }
        }
        Accion::ExportarCarpeta(i) => {
            if let Some(s) = app.exportar.as_mut() {
                if let Some(c) = s.carpetas.get_mut(i) {
                    *c = !*c;
                }
            }
        }
        Accion::ExportarCancelar => app.exportar = None,
        Accion::ExportarConfirmar => {
            if let Some(s) = app.exportar.take() {
                let dir = std::env::temp_dir().join("icaro-demo").join("exportados");
                let _ = std::fs::create_dir_all(&dir);
                let archivo = dir.join(format!("{}-{}.mrpack", s.nombre.trim(), s.version.trim()));
                let indice = serde_json::json!({
                    "formatVersion": 1,
                    "game": "minecraft",
                    "name": s.nombre,
                    "versionId": s.version,
                    "summary": format!("Exportado por {}", s.autor),
                    "instancia": instancias()[s.instancia.min(instancias().len() - 1)].nombre,
                });
                let _ = std::fs::write(&archivo, serde_json::to_string_pretty(&indice).unwrap_or_default());
                app.aviso = Some(format!("Modpack exportado: {}", archivo.display()));
            }
        }
        // ---- Cuenta y errores
        Accion::CopiarCodigo => return iced::clipboard::write("F7K2-9QXM".to_owned()),
        Accion::AbrirPaginaMicrosoft => {
            let _ = std::process::Command::new("cmd")
                .args(["/C", "start", "", "https://www.microsoft.com/link"])
                .spawn();
        }
        Accion::PedirOtroCodigo => {
            app.codigo_inicio = Instant::now();
            app.codigo_fase = FaseCodigo::Esperando;
        }
        Accion::SimularConfirmacion => app.codigo_fase = FaseCodigo::Confirmado(Instant::now()),
        Accion::JugarComoInvitado => {
            app.pantalla = None;
            app.aviso = Some("Sigues como invitado. Puedes iniciar sesión luego desde la cuenta.".into());
            arrancar(app);
        }
        Accion::Reintentar => app.intentos += 1,
        Accion::DesactivarYJugar => {
            app.pantalla = None;
            app.aviso = Some("Iris Shaders se desactivó. Ya puedes jugar.".into());
            arrancar(app);
        }
        Accion::CopiarInforme => {
            return iced::clipboard::write(
                "Ícaro · El juego se cerró (código 1) · Supervivencia\nCausa probable: Iris Shaders 1.8.1 no es compatible con Sodium 0.6.5"
                    .to_owned(),
            );
        }
        Accion::CambiarVersion(i) => {
            app.menu = None;
            app.cambiar = Some(i);
        }
        Accion::CambiarCancelar => app.cambiar = None,
        Accion::CambiarConfirmar(i) => {
            app.cambiar = None;
            let nombre = instancias()[i].nombre.clone();
            if let Some(d) = lista_instancias().lock().unwrap().get_mut(i) {
                d.version = "1.20.1 · Fabric".into();
            }
            app.aviso = Some(format!("{nombre} ahora usa 1.20.1. Se guardó una copia de los mundos."));
        }
        Accion::DetalleTecnico => app.detalle_fallo = !app.detalle_fallo,
        Accion::Duplicar(i) => {
            app.menu = None;
            let todas = instancias();
            if let Some(o) = todas.get(i) {
                let mut d = o.clone();
                d.nombre = nombre_unico(&format!("{} (copia)", o.nombre));
                d.estado = EstadoInstancia::Lista;
                d.detalle.clear();
                d.ultima_vez = "Nueva".into();
                let nombre = d.nombre.clone();
                agregar_instancia(app, d);
                app.aviso = Some(format!("Se creó {nombre}."));
            }
        }
        Accion::AbrirCarpetaInstancia(i) => {
            app.menu = None;
            if let Some(d) = instancias().get(i) {
                let dir = std::env::temp_dir().join("icaro-demo").join(&d.nombre);
                let _ = std::fs::create_dir_all(&dir);
                abrir_en_sistema(&dir);
            }
        }
        Accion::VerInstalados => {
            app.pantalla = None;
            app.seccion = icaro_ui::shell::Seccion::Instancias;
            arrancar(app);
        }
    }
    Task::none()
}

pub fn abrir_en_sistema(ruta: &std::path::Path) {
    let programa = if cfg!(target_os = "windows") {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    let _ = std::process::Command::new(programa).arg(ruta).spawn();
}

fn m(a: Accion) -> Mensaje {
    Mensaje::Pant(a)
}

// ------------------------------------------------------------------ vistas

/// Mitad izquierda de una pantalla con lámina a toda altura.
fn con_lamina<'a>(p: Paleta, lamina: Lamina, derecha: Element<'a, Mensaje>) -> Element<'a, Mensaje> {
    // Como en el sistema: la lámina va sin marco, en una columna con una regla
    // de 1 px a la derecha; las figuras se muestran enteras y centradas.
    let entera = matches!(lamina, Lamina::Icaro | Lamina::Hermes | Lamina::Faeton | Lamina::HermesHilos | Lamina::ApoloBelvedere);
    let izquierda: Element<Mensaje> = if entera {
        container(icaro_ui::laminas::grabado_entero(p, lamina, Length::Fill, Length::Shrink))
            .padding(espacio::S8)
            .center_y(Length::Fill)
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .into()
    } else {
        container(grabado(p, lamina, Length::Fill, Length::Fill))
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .into()
    };
    row![
        izquierda,
        container(Space::new(1.0, Length::Fill)).style(icaro_ui::estilo::bloque(p.border)),
        container(derecha)
            .width(Length::FillPortion(3))
            .height(Length::Fill)
            .center_y(Length::Fill)
            .padding(Padding::from([espacio::S12, espacio::S12])),
    ]
    .height(Length::Fill)
    .into()
}

pub fn vista_pantalla<'a>(app: &'a App, pantalla: Pantalla) -> Element<'a, Mensaje> {
    let p = app.modo.paleta();
    match pantalla {
        Pantalla::Primera => primera(app, p),
        Pantalla::Codigo => codigo(app, p),
        Pantalla::Fallo => fallo(app, p),
        Pantalla::ErrorTienda => error_tienda(app, p),
        Pantalla::NoVerificados => con_lamina(p, Lamina::TorreReja, crate::desplazable_pub(no_verificados(p))),
        Pantalla::Sincronizar => con_lamina(p, Lamina::Hidra, crate::desplazable_pub(sincronizar(p))),
        Pantalla::Integridad => con_lamina(p, Lamina::Minerva, crate::desplazable_pub(integridad(app, p))),
        Pantalla::Actualizacion => con_lamina(p, Lamina::Saturno, crate::desplazable_pub(actualizacion(p))),
        Pantalla::Cuenta => con_lamina(p, Lamina::ApoloBelvedere, crate::desplazable_pub(cuenta(p))),
        Pantalla::Red => con_lamina(p, Lamina::HermesHilos, crate::desplazable_pub(red_privada(p))),
    }
}

fn primera<'a>(_app: &'a App, p: Paleta) -> Element<'a, Mensaje> {
    let paso = |n: &str, titulo: &str, ayuda: &str, activo: bool| -> Element<'a, Mensaje> {
        row![
            container(Space::new(Length::Fixed(3.0), Length::Fill))
                .style(icaro_ui::estilo::bloque(if activo { p.text } else { p.border })),
            column![
                text(n.to_owned()).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted),
                text(titulo.to_owned()).font(fuentes::TITULO).size(texto::BODY.0 + 2.0).color(p.text),
                text(ayuda.to_owned()).size(texto::BODY_SM.0).color(p.text_muted),
            ]
            .spacing(espacio::S1),
        ]
        .spacing(espacio::S4)
        .height(Length::Shrink)
        .into()
    };
    let derecha = column![
        text("ÍCARO").font(fuentes::DISPLAY).size(80.0).color(p.text),
        text("Vamos a dejar todo listo en tres pasos. Después, solo queda pulsar Jugar.")
            .size(texto::BODY.0)
            .color(p.text_muted),
        Space::with_height(espacio::S4),
        paso("1", "Inicia sesión", "Con tu cuenta Microsoft o como invitado.", true),
        paso("2", "Revisamos Java", "Si falta, Ícaro lo descarga.", false),
        paso("3", "Tu primera instancia", "Vanilla, Fabric o un modpack.", false),
        Space::with_height(espacio::S4),
        row![
            componentes::boton(p, "Iniciar sesión", Variante::Primario, Some(m(Accion::Abrir(Pantalla::Codigo)))),
            componentes::boton(p, "Jugar como invitado", Variante::Fantasma, Some(m(Accion::JugarComoInvitado))),
        ]
        .spacing(espacio::S3),
    ]
    .spacing(espacio::S4);
    con_lamina(p, Lamina::Icaro, derecha.into())
}

fn celda<'a>(p: Paleta, c: char, vencido: bool) -> Element<'a, Mensaje> {
    let tinta = if vencido { p.warning } else { p.text };
    container(text(c.to_string()).font(fuentes::MONO).size(26.0).color(tinta))
        .center_x(44)
        .center_y(56)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            border: iced::Border { color: tinta, width: 2.0, radius: 0.0.into() },
            ..Default::default()
        })
        .into()
}

fn codigo<'a>(app: &'a App, p: Paleta) -> Element<'a, Mensaje> {
    let restante = 900u64.saturating_sub(app.ahora.saturating_duration_since(app.codigo_inicio).as_secs());
    let fase = match app.codigo_fase {
        FaseCodigo::Esperando if restante == 0 => FaseCodigo::Vencido,
        f => f,
    };
    let vencido = fase == FaseCodigo::Vencido;
    let mut celdas = row![].spacing(espacio::S2).align_y(Alignment::Center);
    for (i, c) in "F7K2-9QXM".chars().enumerate() {
        if c == '-' {
            celdas = celdas.push(text("—").size(24.0).color(p.text_muted));
        } else {
            let _ = i;
            celdas = celdas.push(celda(p, c, vencido));
        }
    }
    let mut col = column![
        text("INICIA SESIÓN\nCON MICROSOFT").font(fuentes::DISPLAY).size(texto::DISPLAY.0).color(p.text),
        text("Tu cuenta de Minecraft se confirma en el navegador. Ícaro nunca ve tu contraseña.")
            .size(texto::BODY.0)
            .color(p.text_muted),
        Space::with_height(espacio::S2),
        text("1  Abre microsoft.com/link").font(fuentes::TITULO).size(texto::BODY.0).color(p.text),
        row![componentes::boton(p, "Abrir página", Variante::Secundario, Some(m(Accion::AbrirPaginaMicrosoft)))],
        text("2  Escribe este código").font(fuentes::TITULO).size(texto::BODY.0).color(p.text),
        celdas,
    ]
    .spacing(espacio::S3);
    match fase {
        FaseCodigo::Vencido => {
            col = col
                .push(text("El código venció. Pide otro para continuar.").size(texto::BODY.0).color(p.warning))
                .push(row![componentes::boton(p, "Pedir otro código", Variante::Primario, Some(m(Accion::PedirOtroCodigo)))]);
        }
        FaseCodigo::Confirmado(t) => {
            col = col.push(
                container(
                    row![
                        icono(Icono::Check, Tam::Base, p.success),
                        column![
                            text("Mineral_7").font(fuentes::TITULO).size(texto::BODY.0).color(p.text),
                            text("Cuenta Microsoft confirmada").size(texto::BODY_SM.0).color(p.text_muted),
                        ],
                    ]
                    .spacing(espacio::S3)
                    .align_y(Alignment::Center),
                )
                .padding(espacio::S4)
                .style(icaro_ui::estilo::tarjeta(p, false)),
            );
            let _ = t;
        }
        FaseCodigo::Esperando => {
            col = col
                .push(row![
                    componentes::boton(p, "Copiar código", Variante::Secundario, Some(m(Accion::CopiarCodigo))),
                    componentes::boton(p, "Ya lo confirmé", Variante::Fantasma, Some(m(Accion::SimularConfirmacion))),
                ].spacing(espacio::S3))
                .push(
                    row![
                        icono(Icono::Espera, Tam::Base, p.text_muted),
                        text("Esperando que confirmes en el navegador").size(texto::BODY_SM.0).color(p.text),
                        Space::with_width(Length::Fill),
                        text(format!("Vence en {:02}:{:02}", restante / 60, restante % 60))
                            .font(fuentes::MONO)
                            .size(texto::MONO_SM.0)
                            .color(p.text_muted),
                    ]
                    .spacing(espacio::S3)
                    .align_y(Alignment::Center),
                );
        }
    }
    col = col.push(row![componentes::boton(p, "Volver", Variante::Fantasma, Some(m(Accion::Cerrar)))]);
    con_lamina(p, Lamina::Hermes, col.into())
}

fn error_tienda<'a>(app: &'a App, p: Paleta) -> Element<'a, Mensaje> {
    let causa = if app.intentos >= 3 {
        "Sigue sin responder. Revisa tu conexión a internet y vuelve a intentarlo."
    } else {
        "Modrinth no responde. Tus instancias y mods instalados siguen funcionando; la búsqueda volverá cuando el servicio responda."
    };
    let derecha = column![
        text("LA TIENDA\nNO CARGÓ").font(fuentes::DISPLAY).size(texto::DISPLAY.0).color(p.text),
        text(causa).size(texto::BODY.0).color(p.text_muted),
        row![
            componentes::boton(p, "Reintentar", Variante::Primario, Some(m(Accion::Reintentar))),
            componentes::boton(p, "Ver mods instalados", Variante::Secundario, Some(m(Accion::VerInstalados))),
        ]
        .spacing(espacio::S3),
        text(format!("api.modrinth.com · HTTP 503 · {} intentos", 3 + app.intentos))
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted),
        text("Faetón, Hendrick Goltzius, 1588. The Met, CC0.").size(texto::BODY_SM.0).color(p.text_muted),
    ]
    .spacing(espacio::S4);
    con_lamina(p, Lamina::Faeton, derecha.into())
}

fn fallo<'a>(app: &'a App, p: Paleta) -> Element<'a, Mensaje> {
    let causa = container(
        row![
            icono(Icono::Alerta, Tam::Base, p.error),
            column![
                text("Causa probable: Iris Shaders 1.8.1 no es compatible con Sodium 0.6.5")
                    .font(fuentes::TITULO)
                    .size(texto::BODY.0 + 1.0)
                    .color(p.text),
                text("Iris necesita Sodium 0.6.3 o anterior. Desactívalo para jugar ahora, o instala Iris 1.8.2 cuando esté disponible.")
                    .size(texto::BODY_SM.0)
                    .color(p.text_muted),
            ]
            .spacing(espacio::S1),
        ]
        .spacing(espacio::S3),
    )
    .padding(espacio::S4)
    .width(Length::Fill)
    .style(move |_: &iced::Theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(p.error_soft)),
        border: iced::Border { color: p.error, width: 2.0, radius: 0.0.into() },
        ..Default::default()
    });
    let mut col = column![
        text("EL JUEGO\nSE CERRÓ").font(fuentes::DISPLAY).size(texto::DISPLAY.0).color(p.text),
        text("Código de salida 1 · 14:02:47 · Supervivencia").font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted),
        causa,
        row![
            componentes::boton(p, "Desactivar Iris y jugar", Variante::Primario, Some(m(Accion::DesactivarYJugar))),
            componentes::boton(p, "Abrir registro", Variante::Secundario, Some(Mensaje::Ir(icaro_ui::shell::Seccion::Consola))),
            componentes::boton(p, "Copiar informe", Variante::Fantasma, Some(m(Accion::CopiarInforme))),
        ]
        .spacing(espacio::S3),
        iced::widget::button(
            row![
                text("Detalle técnico").font(fuentes::TITULO).size(texto::BODY.0).color(p.text).width(Length::Fill),
                icono(if app.detalle_fallo { Icono::Flecha } else { Icono::ChevronAbajo }, Tam::Base, p.text),
            ]
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .on_press(m(Accion::DetalleTecnico))
        .style(icaro_ui::estilo::boton_fantasma(p)),
    ]
    .spacing(espacio::S4);
    if app.detalle_fallo {
        col = col.push(
            container(
                text("java.lang.NoSuchMethodError: 'void net.caffeinemc.mods.sodium.client.render.chunk.ChunkRenderMatrices.<init>'\n  at net.irisshaders.iris.compat.sodium.mixin.MixinChunkRenderer.iris$setup(MixinChunkRenderer.java:58)\n  at net.minecraft.client.renderer.LevelRenderer.renderLevel(LevelRenderer.java:1204)")
                    .font(fuentes::MONO)
                    .size(texto::MONO_SM.0)
                    .color(p.text),
            )
            .padding(espacio::S4)
            .width(Length::Fill)
            .style(icaro_ui::estilo::marco(p)),
        );
    }
    con_lamina(p, Lamina::Rostro, crate::desplazable_pub(col.into()))
}

// ------------------------------------------------------------- diálogos

fn cabecera_pasos<'a>(p: Paleta, actual: usize) -> Element<'a, Mensaje> {
    let mut fila = row![].spacing(espacio::S4);
    for (i, nombre) in PASOS.iter().enumerate() {
        let hecho = i < actual;
        let tinta = if i <= actual { p.text } else { p.text_muted };
        let contenido = column![
            container(Space::new(Length::Fill, 3.0))
                .style(icaro_ui::estilo::bloque(if i <= actual { p.text } else { p.border })),
            text(format!("{} de 4", i + 1)).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted),
            text((*nombre).to_owned()).font(fuentes::TITULO).size(texto::BODY.0).color(tinta),
        ]
        .spacing(espacio::S1)
        .width(Length::Fill);
        fila = fila.push(if hecho {
            iced::widget::button(contenido)
                .padding(0)
                .width(Length::FillPortion(1))
                .on_press(m(Accion::AsistentePaso(i)))
                .style(icaro_ui::estilo::sin_estilo(p.text))
                .into()
        } else {
            Element::from(container(contenido).width(Length::FillPortion(1)))
        });
    }
    fila.into()
}

pub fn asistente<'a>(app: &'a App, s: &'a Asistente) -> Element<'a, Mensaje> {
    let p = app.modo.paleta();
    let version = VERSIONES[s.tipo_version][s.version];
    let cuerpo: Element<Mensaje> = match s.paso {
        0 => {
            let consulta = s.busqueda.trim().to_lowercase();
            let mut lista = column![].spacing(espacio::S1);
            for (i, (v, fecha)) in VERSIONES[s.tipo_version].iter().enumerate() {
                if !consulta.is_empty() && !v.contains(&consulta) {
                    continue;
                }
                let elegida = s.version == i;
                lista = lista.push(
                    iced::widget::button(
                        container(
                            row![
                                text((*v).to_owned()).font(fuentes::MONO).size(texto::MONO.0).color(if elegida { p.bg } else { p.text }).width(Length::Fill),
                                text((*fecha).to_owned()).font(fuentes::MONO).size(texto::MONO_SM.0).color(if elegida { p.bg } else { p.text_muted }),
                            ]
                            .align_y(Alignment::Center),
                        )
                        .center_y(Length::Fill),
                    )
                    .width(Length::Fill)
                    .height(36)
                    .padding(Padding::from([0.0, espacio::S3]))
                    .on_press(m(Accion::AsistenteVersion(i)))
                    .style(move |_, estado| iced::widget::button::Style {
                        background: (elegida || matches!(estado, iced::widget::button::Status::Hovered))
                            .then_some(iced::Background::Color(if elegida { p.text } else { p.surface_hover })),
                        text_color: if elegida { p.bg } else { p.text },
                        ..Default::default()
                    }),
                );
            }
            let aviso: Element<Mensaje> = if s.tipo_version == 1 {
                row![insignia(p, Estado::Aviso, "Beta"), text("Las snapshots pueden fallar y corromper mundos.").size(texto::BODY_SM.0).color(p.text_muted)]
                    .spacing(espacio::S2)
                    .align_y(Alignment::Center)
                    .into()
            } else {
                text("La última estable viene elegida.").size(texto::BODY_SM.0).color(p.text_muted).into()
            };
            column![
                componentes::segmentado(p, &["Lanzamientos", "Snapshots", "Antiguas"], s.tipo_version, |i| m(Accion::AsistenteTipo(i))),
                componentes::campo(p, "Buscar versión", &s.busqueda, |t| m(Accion::AsistenteBuscar(t))),
                container(lista).padding(espacio::S2).width(Length::Fill).style(icaro_ui::estilo::marco(p)),
                aviso,
            ]
            .spacing(espacio::S3)
            .into()
        }
        1 => {
            let mut tarjetas = column![].spacing(espacio::S2);
            for (i, (nombre, ayuda, rec)) in LOADERS.iter().enumerate() {
                let elegido = s.loader == i;
                // Forge todavía no soporta las versiones más nuevas.
                let bloqueado = *nombre == "Forge" && s.tipo_version == 0 && s.version < 3;
                let cuerpo = row![
                    icono(if elegido { Icono::Check } else { Icono::Carpeta }, Tam::Base, if bloqueado { p.text_disabled } else { p.text }),
                    column![
                        text((*nombre).to_owned()).font(fuentes::TITULO).size(texto::BODY.0 + 1.0).color(if bloqueado { p.text_disabled } else { p.text }),
                        text(if bloqueado { format!("Aún no disponible para {}.", version.0) } else { (*ayuda).to_owned() })
                            .size(texto::BODY_SM.0)
                            .color(p.text_muted),
                    ]
                    .spacing(espacio::S1)
                    .width(Length::Fill),
                    text((*rec).to_owned()).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted),
                ]
                .spacing(espacio::S4)
                .align_y(Alignment::Center);
                let b = iced::widget::button(container(cuerpo).padding(espacio::S3))
                    .width(Length::Fill)
                    .padding(0)
                    .on_press_maybe((!bloqueado).then(|| m(Accion::AsistenteLoader(i))))
                    .style(icaro_ui::estilo::sin_estilo(p.text));
                tarjetas = tarjetas.push(container(b).style(icaro_ui::estilo::tarjeta(p, elegido)));
            }
            tarjetas.into()
        }
        2 => {
            let mut portadas = row![].spacing(espacio::S3);
            for (i, (nombre, lamina)) in PORTADAS.iter().enumerate() {
                portadas = portadas.push(
                    iced::widget::button(
                        column![
                            container(grabado(p, *lamina, Length::Fill, 70)).style(icaro_ui::estilo::marco(p)),
                            text((*nombre).to_owned()).size(texto::BODY_SM.0).color(if s.portada == i { p.text } else { p.text_muted }),
                        ]
                        .spacing(espacio::S1),
                    )
                    .padding(0)
                    .width(Length::FillPortion(1))
                    .on_press(m(Accion::AsistentePortada(i)))
                    .style(icaro_ui::estilo::sin_estilo(p.text)),
                );
            }
            column![
                etiqueta(p, "Nombre de la instancia"),
                componentes::campo(p, "Mi instancia", &s.nombre, |t| m(Accion::AsistenteNombre(t))),
                etiqueta(p, "Portada"),
                portadas,
            ]
            .spacing(espacio::S3)
            .into()
        }
        _ => {
            let fila = |k: &str, v: String| -> Element<'a, Mensaje> {
                row![
                    text(k.to_owned()).size(texto::BODY.0).color(p.text_muted).width(160),
                    text(v).font(fuentes::MONO).size(texto::MONO.0).color(p.text),
                ]
                .into()
            };
            column![
                fila("Nombre", if s.nombre.trim().is_empty() { "Nueva instancia".into() } else { s.nombre.trim().to_owned() }),
                fila("Minecraft", version.0.to_owned()),
                fila("Loader", LOADERS[s.loader].0.to_owned()),
                fila("Java", "Automático".into()),
                fila("Memoria", "2.048 a 6.144 MB".into()),
                text("Todo esto se puede cambiar después desde el editor de la instancia.").size(texto::BODY_SM.0).color(p.text_muted),
            ]
            .spacing(espacio::S3)
            .into()
        }
    };
    let cuerpo = column![cabecera_pasos(p, s.paso), cuerpo].spacing(espacio::S4);
    let mut acciones: Vec<Element<Mensaje>> = vec![componentes::boton(p, "Cancelar", Variante::Fantasma, Some(m(Accion::AsistenteCancelar)))];
    if s.paso > 0 {
        acciones.push(componentes::boton(p, "Atrás", Variante::Secundario, Some(m(Accion::AsistenteAtras))));
    }
    acciones.push(if s.paso == 3 {
        componentes::boton(p, "Crear instancia", Variante::Primario, Some(m(Accion::AsistenteCrear)))
    } else {
        componentes::boton(p, "Siguiente", Variante::Primario, Some(m(Accion::AsistenteSiguiente)))
    });
    icaro_ui::superficies::dialogo(p, "Crear instancia", cuerpo.into(), acciones, None, 700.0, 560.0, m(Accion::AsistenteCancelar))
}

pub fn importar<'a>(app: &'a App, s: &'a Importar) -> Element<'a, Mensaje> {
    let p = app.modo.paleta();
    let cuerpo: Element<Mensaje> = match s.origen {
        0 => {
            let contenido: Element<Mensaje> = match s.archivo {
                None => column![
                    text("Elige un archivo .mrpack o el .zip de un modpack de CurseForge.").size(texto::BODY.0).color(p.text_muted),
                    row![componentes::boton(p, "Examinar", Variante::Secundario, Some(m(Accion::ImportarArchivo)))],
                ]
                .spacing(espacio::S3)
                .into(),
                Some(nombre) => column![
                    text(nombre).font(fuentes::MONO).size(texto::MONO.0).color(p.text),
                    text("Fabulously Optimized · 1.21.4 · Fabric · 62 mods").size(texto::BODY.0).color(p.text_muted),
                    insignia(p, Estado::Exito, "Listo para importar"),
                ]
                .spacing(espacio::S2)
                .into(),
            };
            container(contenido).padding(espacio::S4).width(Length::Fill).style(icaro_ui::estilo::marco(p)).into()
        }
        1 => {
            let mut lista = column![text("Encontramos estas carpetas en tu equipo:").size(texto::BODY.0).color(p.text_muted)].spacing(espacio::S2);
            for (i, (n, c)) in [("Prism Launcher", "1 instancia"), ("MultiMC", "1 instancia"), ("CurseForge App", "1 instancia")].iter().enumerate() {
                lista = lista.push(casilla(p, &format!("{n} · {c}"), s.launchers[i], move |_| m(Accion::ImportarLauncher(i))));
            }
            lista.into()
        }
        _ => {
            let todas = instancias();
            let nombres: Vec<String> = todas.iter().map(|d| d.nombre.clone()).collect();
            let origen = todas.get(s.clon_origen).map_or(String::new(), |d| d.nombre.clone());
            column![
                etiqueta(p, "Instancia a clonar"),
                componentes::selector(p, nombres, Some(origen.clone()), |n| m(Accion::ImportarClon(n))),
                casilla(p, "Copiar también los mundos", s.con_mundos, |v| m(Accion::ImportarMundos(v))),
                text(format!("Se creará «{origen} (copia)».")).size(texto::BODY_SM.0).color(p.text_muted),
            ]
            .spacing(espacio::S3)
            .into()
        }
    };
    let cuerpo = column![
        componentes::segmentado(p, &["Modpack", "Otro launcher", "Clonar"], s.origen, |i| m(Accion::ImportarOrigen(i))),
        cuerpo,
    ]
    .spacing(espacio::S4);
    let listo = match s.origen {
        0 => s.archivo.is_some(),
        1 => s.launchers.iter().any(|x| *x),
        _ => true,
    };
    icaro_ui::superficies::dialogo(
        p,
        "Importar instancia",
        cuerpo.into(),
        vec![
            componentes::boton(p, "Cancelar", Variante::Fantasma, Some(m(Accion::ImportarCancelar))),
            componentes::boton(p, "Importar", Variante::Primario, listo.then(|| m(Accion::ImportarConfirmar))),
        ],
        None,
        620.0,
        420.0,
        m(Accion::ImportarCancelar),
    )
}

pub fn exportar<'a>(app: &'a App, s: &'a Exportar) -> Element<'a, Mensaje> {
    let p = app.modo.paleta();
    let mut carpetas = column![etiqueta(p, "Qué incluir")].spacing(espacio::S2);
    for (i, n) in ["Configuración de mods", "Opciones del juego", "Mundos", "Capturas"].iter().enumerate() {
        carpetas = carpetas.push(casilla(p, n, s.carpetas[i], move |_| m(Accion::ExportarCarpeta(i))));
    }
    let tam = 1.4 + s.carpetas.iter().filter(|c| **c).count() as f32 * 0.8;
    let formulario = column![
        etiqueta(p, "Nombre del pack"),
        componentes::campo(p, "Nombre", &s.nombre, |t| m(Accion::ExportarNombre(t))),
        row![
            column![etiqueta(p, "Versión"), componentes::campo(p, "1.0.0", &s.version, |t| m(Accion::ExportarVersion(t)))].spacing(espacio::S2).width(Length::FillPortion(1)),
            column![etiqueta(p, "Autor"), componentes::campo(p, "Autor", &s.autor, |t| m(Accion::ExportarAutor(t)))].spacing(espacio::S2).width(Length::FillPortion(2)),
        ]
        .spacing(espacio::S3),
        carpetas,
    ]
    .spacing(espacio::S3)
    .width(Length::FillPortion(3));
    let resumen = column![
        etiqueta(p, "Resumen"),
        text("Como referencia, desde Modrinth:").size(texto::BODY_SM.0).color(p.text_muted),
        text("— 198 mods").size(texto::BODY.0).color(p.text),
        row![text("Empaquetados dentro:").size(texto::BODY_SM.0).color(p.text_muted), insignia(p, Estado::Info, "Incluido")].spacing(espacio::S2).align_y(Alignment::Center),
        text("— 3 mods de otras fuentes").size(texto::BODY.0).color(p.text),
        text("Revisa que su licencia permita redistribuirlos.").size(texto::BODY_SM.0).color(p.warning),
        Space::with_height(espacio::S2),
        text(format!("{}-{}.mrpack", s.nombre.trim(), s.version.trim())).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text),
        text(format!("Tamaño estimado: {tam:.1} MB")).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted),
    ]
    .spacing(espacio::S2)
    .width(Length::FillPortion(2));
    icaro_ui::superficies::dialogo(
        p,
        "Exportar modpack",
        row![formulario, container(resumen).padding(espacio::S4).style(icaro_ui::estilo::marco(p)).width(Length::FillPortion(2))]
            .spacing(espacio::S6)
            .into(),
        vec![
            componentes::boton(p, "Cancelar", Variante::Fantasma, Some(m(Accion::ExportarCancelar))),
            componentes::boton(p, "Exportar", Variante::Primario, (!s.nombre.trim().is_empty()).then(|| m(Accion::ExportarConfirmar))),
        ],
        None,
        760.0,
        520.0,
        m(Accion::ExportarCancelar),
    )
}

// ------------------------------------------------------------- ajustes

pub const SECCIONES: [&str; 8] = [
    "General", "Apariencia", "Java y memoria", "Almacenamiento", "Red", "Comportamiento", "Atajos", "Acerca de",
];

pub fn pagina_ajustes<'a>(app: &'a App) -> Element<'a, Mensaje> {
    let p = app.modo.paleta();
    let a = &app.aj;
    let ajustes = app.ajuste.min(SECCIONES.len() - 1);
    let titulo = SECCIONES[ajustes];
    let interruptor = |v: bool, c: CampoAjuste| componentes::interruptor(p, v, m(Accion::Alternar(c)));
    let mut col = column![encabezado_ajustes(p, titulo, None)].spacing(espacio::S2).width(Length::Fill);
    match ajustes {
        0 => {
            col = col
                .push(fila_ajuste(p, "Idioma", "El idioma de la interfaz.",
                    componentes::selector(p, vec!["Español (Chile)".to_owned(), "English".to_owned()], Some(a.idioma.clone()), |v| m(Accion::Idioma(v)))))
                .push(fila_ajuste(p, "Abrir con el sistema", "Ícaro se inicia al encender el equipo.", interruptor(a.abrir_al_iniciar, CampoAjuste::AbrirAlIniciar)))
                .push(fila_ajuste(p, "Iniciar minimizado", "Arranca sin mostrar la ventana.", interruptor(a.iniciar_minimizado, CampoAjuste::IniciarMinimizado)))
                .push(fila_ajuste(p, "Al iniciar el juego", "Qué hace el launcher cuando Minecraft abre.",
                    componentes::selector(p, vec!["Mantener abierto".to_owned(), "Minimizar".to_owned(), "Cerrar el launcher".to_owned()],
                        Some(["Mantener abierto", "Minimizar", "Cerrar el launcher"][a.al_jugar.min(2)].to_owned()), |v| m(Accion::AlJugar(v)))))
                .push(fila_ajuste(p, "Carpeta de datos", "Instancias, versiones y caché.",
                    componentes::boton(p, "Abrir carpeta", Variante::Secundario, Some(m(Accion::AbrirCarpetaDatos)))));
        }
        1 => {
            col = col
                .push(fila_ajuste(p, "Tema", "Tinta es oscuro, Piedra es claro. Sistema sigue a tu equipo.",
                    componentes::segmentado(p, &["Tinta", "Piedra", "Sistema"], app.tema, Mensaje::Tema)))
                .push(fila_ajuste(p, "Reducir movimiento", "Quita animaciones; el Umbral pasa a ser un fundido.",
                    componentes::interruptor(p, app.reducir, Mensaje::Reducir)))
                .push(fila_ajuste(p, "Escala de la interfaz", "De 80 a 150 %. Se aplica al reiniciar.",
                    row![
                        componentes::contador(p, a.escala, m(Accion::Escala(a.escala - 10)), m(Accion::Escala(a.escala + 10))),
                        text(format!("{} %", a.escala)).font(fuentes::MONO).size(texto::MONO.0).color(p.text),
                    ].spacing(espacio::S3).align_y(Alignment::Center).into()));
        }
        2 => {
            col = col
                .push(fila_ajuste(p, "Java de cada instancia", "La versión y la memoria se eligen por instancia, en la pestaña Java de su editor.", Space::with_width(0).into()))
                .push(fila_ajuste(p, "Gestión automática", "Ícaro elige Java 21, 17 u 8 según la versión del juego.", componentes::interruptor(p, true, Mensaje::Nada)))
                .push(text("Instalaciones detectadas").font(fuentes::TITULO).size(texto::BODY.0 + 1.0).color(p.text));
            for (v, prov, ruta) in [("21.0.5", "Temurin · Ícaro", "…/icaro/java/21"), ("17.0.13", "Temurin · Ícaro", "…/icaro/java/17"), ("8.0.432", "Zulu", "C:/Program Files/Zulu/zulu-8")] {
                col = col.push(
                    container(
                        row![
                            text(format!("Java {v}")).font(fuentes::MONO).size(texto::MONO.0).color(p.text).width(140),
                            text(prov).size(texto::BODY.0).color(p.text_muted).width(200),
                            text(ruta).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted),
                        ]
                        .spacing(espacio::S4)
                        .align_y(Alignment::Center),
                    )
                    .padding(Padding::from([espacio::S3, espacio::S4]))
                    .width(Length::Fill)
                    .style(icaro_ui::estilo::tarjeta(p, false)),
                );
            }
        }
        3 => {
            col = col.push(text("Uso de disco: 4,5 GB").font(fuentes::MONO).size(texto::MONO.0).color(p.text));
            let cache = if a.cache_limpia { 0.0 } else { 0.08 };
            let cache_txt = if a.cache_limpia { "0 MB" } else { "380 MB" };
            for (n, gb, frac) in [("Instancias", "2,4 GB", 0.53f32), ("Versiones de Minecraft", "1,1 GB", 0.24), ("Java", "540 MB", 0.12), ("Caché de mods", cache_txt, cache), ("Capturas", "96 MB", 0.02)] {
                col = col.push(
                    row![
                        text(n).size(texto::BODY.0).color(p.text).width(220),
                        container(componentes::progreso(p, frac)).width(Length::Fill),
                        text(gb).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted).width(90).align_x(iced::alignment::Horizontal::Right),
                    ]
                    .spacing(espacio::S4)
                    .align_y(Alignment::Center),
                );
            }
            col = col.push(Space::with_height(espacio::S3)).push(row![
                componentes::boton(p, "Limpiar caché", Variante::Secundario, (!a.cache_limpia).then(|| m(Accion::LimpiarCache))),
                componentes::boton(p, "Abrir carpeta de datos", Variante::Fantasma, Some(m(Accion::AbrirCarpetaDatos))),
            ].spacing(espacio::S3));
        }
        4 => {
            col = col
                .push(fila_ajuste(p, "Descargas simultáneas", "Con internet lento, bájalo a 2.",
                    componentes::contador(p, app.simultaneas, Mensaje::Simultaneas(app.simultaneas - 1), Mensaje::Simultaneas(app.simultaneas + 1))))
                .push(fila_ajuste(p, "Límite de velocidad", "En MB/s. Con 0 no hay límite.",
                    row![
                        componentes::contador(p, a.limite_mb, m(Accion::Limite(a.limite_mb - 1)), m(Accion::Limite(a.limite_mb + 1))),
                        text(if a.limite_mb == 0 { "Sin límite".to_owned() } else { format!("{} MB/s", a.limite_mb) }).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted),
                    ].spacing(espacio::S3).align_y(Alignment::Center).into()));
        }
        5 => {
            col = col
                .push(fila_ajuste(p, "Confirmar antes de eliminar", "Pide confirmación al borrar una instancia.", interruptor(a.confirmar_borrar, CampoAjuste::ConfirmarBorrar)))
                .push(fila_ajuste(p, "Mostrar la consola al jugar", "Abre los registros cuando inicia el juego.", interruptor(a.consola_al_jugar, CampoAjuste::ConsolaAlJugar)))
                .push(fila_ajuste(p, "Copias automáticas de mundos", "Guarda una copia antes de cambiar la versión.", interruptor(a.copias_mundos, CampoAjuste::CopiasMundos)));
        }
        6 => {
            for (tecla, texto_) in [("Ctrl 1 a 7", "Ir a una sección"), ("Ctrl K", "Buscar"), ("Ctrl Enter", "Jugar la instancia elegida"), ("F1", "Ver los atajos"), ("Enter", "Jugar desde el menú"), ("E", "Editar desde el menú"), ("Supr", "Eliminar desde el menú")] {
                col = col.push(
                    row![
                        container(icaro_ui::shell::kbd(p, tecla)).width(160),
                        text(texto_).size(texto::BODY.0).color(p.text),
                    ]
                    .padding(Padding::from([espacio::S2, 0.0]))
                    .align_y(Alignment::Center),
                );
            }
        }
        _ => {
            col = col
                .push(text("Ícaro 0.1.0").font(fuentes::TITULO).size(texto::HEADING.0).color(p.text))
                .push(text("Un launcher de Minecraft escrito en Rust, ligero y optimizado, para Windows y Arch Linux.").size(texto::BODY.0).color(p.text_muted))
                .push(text("Licencia GPL-3.0. Grabados de dominio público (CC0) de The Met, la National Gallery of Art y el Art Institute of Chicago. Íconos Pixelarticons (MIT).").size(texto::BODY.0).color(p.text_muted))
                .push(Space::with_height(espacio::S3))
                .push(text("Pantallas de ejemplo").font(fuentes::TITULO).size(texto::BODY.0 + 1.0).color(p.text))
                .push(
                    row![
                        componentes::boton(p, "Primera ejecución", Variante::Secundario, Some(m(Accion::Abrir(Pantalla::Primera)))),
                        componentes::boton(p, "Iniciar sesión", Variante::Secundario, Some(m(Accion::Abrir(Pantalla::Codigo)))),
                        componentes::boton(p, "Informe de fallo", Variante::Secundario, Some(m(Accion::Abrir(Pantalla::Fallo)))),
                        componentes::boton(p, "Error de la tienda", Variante::Secundario, Some(m(Accion::Abrir(Pantalla::ErrorTienda)))),
                        componentes::boton(p, "Archivos sin verificar", Variante::Secundario, Some(m(Accion::Abrir(Pantalla::NoVerificados)))),
                        componentes::boton(p, "Sincronizar mods", Variante::Secundario, Some(m(Accion::Abrir(Pantalla::Sincronizar)))),
                        componentes::boton(p, "Verificar integridad", Variante::Secundario, Some(m(Accion::Abrir(Pantalla::Integridad)))),
                        componentes::boton(p, "Actualización", Variante::Secundario, Some(m(Accion::Abrir(Pantalla::Actualizacion)))),
                        componentes::boton(p, "Cuenta", Variante::Secundario, Some(m(Accion::Abrir(Pantalla::Cuenta)))),
                        componentes::boton(p, "Red privada", Variante::Secundario, Some(m(Accion::Abrir(Pantalla::Red)))),
                    ]
                    .spacing(espacio::S3)
                    .wrap(),
                );
        }
    }
    col.into()
}

// ------------------------------------------------------- más pantallas

fn regla<'a>(p: Paleta) -> Element<'a, Mensaje> {
    container(Space::new(Length::Fill, 1.0)).style(icaro_ui::estilo::bloque(p.border)).into()
}

fn no_verificados<'a>(p: Paleta) -> Element<'a, Mensaje> {
    let archivo = |nombre: &str, meta: &str, accion: &str| -> Element<'a, Mensaje> {
        column![
            row![
                column![
                    text(nombre.to_owned()).font(fuentes::MONO).size(texto::MONO.0).color(p.text),
                    text(meta.to_owned()).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted),
                ]
                .spacing(espacio::S1)
                .width(Length::Fill),
                componentes::boton(p, accion, Variante::Secundario, Some(Mensaje::Nada)),
                componentes::boton(p, "Confiar", Variante::Fantasma, Some(Mensaje::Nada)),
            ]
            .spacing(espacio::S3)
            .align_y(Alignment::Center),
            regla(p),
        ]
        .spacing(espacio::S3)
        .into()
    };
    let malicioso = container(
        row![
            column![
                text("free-cosmetics.jar").font(fuentes::MONO).size(texto::MONO.0).color(p.error),
                text("Modrinth lo marcó como malicioso. Lo desactivamos.").size(texto::BODY_SM.0).color(p.error),
            ]
            .spacing(espacio::S1)
            .width(Length::Fill),
            insignia(p, Estado::Error, "Desactivado"),
            componentes::boton(p, "Quitar", Variante::Peligro, Some(Mensaje::Nada)),
        ]
        .spacing(espacio::S3)
        .align_y(Alignment::Center),
    )
    .padding(espacio::S3)
    .width(Length::Fill)
    .style(move |_: &iced::Theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(p.error_soft)),
        ..Default::default()
    });
    column![
        text("3 ARCHIVOS SIN\nVERIFICAR").font(fuentes::DISPLAY).size(texto::DISPLAY.0).color(p.text),
        text("No pudimos verificar el origen de estos mods. Pueden ser legítimos (copiados a mano), pero conviene reemplazarlos por la versión de Modrinth.")
            .size(texto::BODY.0)
            .color(p.text_muted),
        regla(p),
        archivo("custom-hud-1.4.jar", "412 KB · sha1 8f2c…a91e · copiado a mano", "Buscar en Modrinth"),
        archivo("optifine-1.21.4.jar", "7,1 MB · sha1 03be…77d0 · optifine.net", "Buscar alternativa"),
        malicioso,
        row![componentes::boton(p, "Volver", Variante::Fantasma, Some(m(Accion::Cerrar)))],
    ]
    .spacing(espacio::S4)
    .into()
}

fn sincronizar<'a>(p: Paleta) -> Element<'a, Mensaje> {
    let fila = |signo: &str, tono: iced::Color, nombre: &str, accion: &str, version: &str, tam: &str| {
        column![
            row![
                container(text(signo.to_owned()).font(fuentes::MONO).size(texto::MONO.0).color(tono))
                    .padding(Padding::from([espacio::S1, espacio::S2]))
                    .style(move |_: &iced::Theme| iced::widget::container::Style {
                        border: iced::Border { color: tono, width: 2.0, radius: 0.0.into() },
                        ..Default::default()
                    }),
                column![
                    text(nombre.to_owned()).font(fuentes::TITULO).size(texto::BODY.0).color(p.text),
                    text(accion.to_owned()).size(texto::BODY_SM.0).color(p.text_muted),
                ]
                .spacing(espacio::S1)
                .width(Length::Fill),
                text(version.to_owned()).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted),
                text(tam.to_owned()).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted).width(70),
            ]
            .spacing(espacio::S3)
            .align_y(Alignment::Center),
            regla(p),
        ]
        .spacing(espacio::S2)
    };
    let derecha = column![
        text("SINCRONIZAR\nCON AMIGOS").font(fuentes::DISPLAY).size(texto::DISPLAY.0).color(p.text),
        text("3 por agregar · 2 por actualizar · 1 por quitar · 412 MB por descargar").size(texto::BODY_SM.0).color(p.text_muted),
        regla(p),
        fila("+", p.success, "Create", "Agregar", "0.5.1j", "18,2 MB"),
        fila("+", p.success, "Farmer's Delight", "Agregar", "1.2.6", "3,1 MB"),
        fila("~", p.info, "Lithium", "Actualizar", "0.14.3 → 0.14.5", "720 KB"),
        fila("−", p.error, "Better Combat", "Quitar: el servidor ya no lo usa", "1.8.6", ""),
        text("Se mantienen 4 mods solo de cliente: Sodium, Iris, Xaero's Minimap, Mod Menu.").size(texto::BODY_SM.0).color(p.text_muted),
        row![
            componentes::boton(p, "Sincronizar y jugar", Variante::Primario, Some(m(Accion::Cerrar))),
            componentes::boton(p, "Solo sincronizar", Variante::Secundario, Some(m(Accion::Cerrar))),
            componentes::boton(p, "Jugar sin sincronizar", Variante::Fantasma, Some(m(Accion::Cerrar))),
        ]
        .spacing(espacio::S3),
    ]
    .spacing(espacio::S3);
    derecha.into()
}

fn integridad<'a>(app: &'a App, p: Paleta) -> Element<'a, Mensaje> {
    let seg = app.ahora.saturating_duration_since(app.codigo_inicio).as_secs_f32();
    let avance = (seg / 12.0).min(1.0);
    let total = 1482u32;
    let hechos = (avance * total as f32) as u32;
    let celdas = 40usize;
    let rellenas = (avance * (celdas * 6) as f32) as usize;
    let mut cuadricula = column![].spacing(2);
    for f in 0..6 {
        let mut fila = row![].spacing(2);
        for c in 0..celdas {
            let i = f * celdas + c;
            let dañada = i == 28 || i == 107;
            let color = if i < rellenas {
                if dañada { p.error } else { p.text_muted }
            } else {
                p.surface_sunken
            };
            fila = fila.push(container(Space::new(Length::Fill, 12.0)).width(Length::Fill).style(icaro_ui::estilo::bloque(color)));
        }
        cuadricula = cuadricula.push(fila);
    }
    let dañados = if rellenas > 28 { if rellenas > 107 { 2 } else { 1 } } else { 0 };
    let mut col = column![
        text("VERIFICANDO\nSUPERVIVENCIA").font(fuentes::DISPLAY).size(texto::DISPLAY.0).color(p.text),
        row![
            text(if avance >= 1.0 { "Verificación terminada" } else { "Revisando librerías" }).size(texto::BODY_SM.0).color(p.text),
            Space::with_width(Length::Fill),
            text(format!("{hechos} de {total}")).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted),
        ],
        cuadricula,
    ]
    .spacing(espacio::S3);
    if dañados > 0 {
        col = col.push(
            container(
                column![
                    text(format!("{dañados} archivos dañados hasta ahora")).font(fuentes::TITULO).size(texto::BODY.0).color(p.text),
                    text("libraries/org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3.jar").font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text),
                    text("assets/objects/4f/4f8a2c19e0b7….ogg").font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text),
                ]
                .spacing(espacio::S1),
            )
            .padding(espacio::S4)
            .width(Length::Fill)
            .style(move |_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(p.error_soft)),
                border: iced::Border { color: p.error, width: 2.0, radius: 0.0.into() },
                ..Default::default()
            }),
        );
    }
    col = col.push(
        row![
            componentes::boton(p, "Cancelar", Variante::Secundario, Some(m(Accion::Cerrar))),
            componentes::boton(p, "Reparar al terminar", Variante::Primario, (avance >= 1.0).then(|| m(Accion::Cerrar))),
        ]
        .spacing(espacio::S3),
    );
    col.into()
}

fn actualizacion<'a>(p: Paleta) -> Element<'a, Mensaje> {
    let nota = |tono: Estado, etiqueta_: &str, texto_: &str| -> Element<'a, Mensaje> {
        row![
            container(insignia(p, tono, etiqueta_)).width(140),
            text(texto_.to_owned()).size(texto::BODY_SM.0).color(p.text),
        ]
        .spacing(espacio::S3)
        .align_y(Alignment::Center)
        .into()
    };
    column![
        text("ÍCARO 1.3").font(fuentes::DISPLAY).size(texto::DISPLAY.0).color(p.text),
        text("Tienes 1.2.4 · descarga de 14,2 MB").font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted),
        nota(Estado::Exito, "Nuevo", "Sincroniza mods con un servidor antes de unirte."),
        nota(Estado::Exito, "Nuevo", "Estudio de skin con vista de frente y de espalda."),
        nota(Estado::Info, "Mejora", "La consola maneja 200.000 líneas sin trabarse."),
        nota(Estado::Aviso, "Corrección", "Las descargas pausadas ya no se reinician al abrir."),
        row![
            componentes::boton(p, "Actualizar ahora", Variante::Primario, Some(m(Accion::Cerrar))),
            componentes::boton(p, "Al cerrar el juego", Variante::Secundario, Some(m(Accion::Cerrar))),
            componentes::boton(p, "Más tarde", Variante::Fantasma, Some(m(Accion::Cerrar))),
        ]
        .spacing(espacio::S3),
    ]
    .spacing(espacio::S4)
    .into()
}

/// Diálogo para cambiar la versión de una instancia, con aviso de riesgo.
pub fn cambiar_version<'a>(app: &'a App, i: usize) -> Element<'a, Mensaje> {
    let p = app.modo.paleta();
    let d = &instancias()[i];
    let cuerpo = column![
        row![
            column![etiqueta(p, "Ahora"), text(d.version.clone()).font(fuentes::MONO).size(texto::MONO.0).color(p.text)].spacing(espacio::S1).width(Length::Fill),
            icono(Icono::ChevronDerecha, Tam::Base, p.text_muted),
            column![etiqueta(p, "Después"), text("1.20.1 · Fabric").font(fuentes::MONO).size(texto::MONO.0).color(p.text)].spacing(espacio::S1).width(Length::Fill),
        ]
        .align_y(Alignment::Center),
        container(
            row![
                icono(Icono::Alerta, Tam::Base, p.error),
                column![
                    text("Bajar de versión puede dañar tus mundos").font(fuentes::TITULO).size(texto::BODY.0).color(p.text),
                    text("Los 3 mundos se crearon en 1.21.4. Minecraft no garantiza que funcionen en 1.20.1, y el mod de este tamaño no se puede desactivar.")
                        .size(texto::BODY_SM.0)
                        .color(p.text_muted),
                ]
                .spacing(espacio::S1),
            ]
            .spacing(espacio::S3),
        )
        .padding(espacio::S4)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(p.error_soft)),
            border: iced::Border { color: p.error, width: 2.0, radius: 0.0.into() },
            ..Default::default()
        }),
        casilla(p, "Hacer copia de seguridad de los mundos (obligatorio)", true, |_| Mensaje::Nada),
    ]
    .spacing(espacio::S4);
    icaro_ui::superficies::dialogo(
        p,
        "Cambiar versión",
        cuerpo.into(),
        vec![
            componentes::boton(p, "Crear una copia en su lugar", Variante::Fantasma, Some(m(Accion::Duplicar(i)))),
            componentes::boton(p, "Cancelar", Variante::Secundario, Some(m(Accion::CambiarCancelar))),
            componentes::boton(p, "Cambiar a 1.20.1", Variante::Peligro, Some(m(Accion::CambiarConfirmar(i)))),
        ],
        Some(Lamina::CaidaDeFaeton),
        640.0,
        420.0,
        m(Accion::CambiarCancelar),
    )
}

// ------------------------------------------------- cuenta y red privada

fn skin<'a>(p: Paleta, espalda: bool) -> Element<'a, Mensaje> {
    // Muñeco de píxeles: cabeza, torso, brazos y piernas.
    let c = |w: f32, h: f32, color: iced::Color| container(Space::new(w, h)).style(icaro_ui::estilo::bloque(color));
    let piel = p.text_muted;
    let ropa = if espalda { p.border_strong } else { p.text };
    let pelo = p.border;
    column![
        c(64.0, 64.0, if espalda { pelo } else { piel }),
        row![c(32.0, 96.0, piel), c(64.0, 96.0, ropa), c(32.0, 96.0, piel)],
        row![c(32.0, 96.0, p.border_strong), c(32.0, 96.0, p.border_strong)],
    ]
    .align_x(Alignment::Center)
    .into()
}

fn cuenta<'a>(p: Paleta) -> Element<'a, Mensaje> {
    let cabezas = row![
        skin_mini(p, 1), skin_mini(p, 2), skin_mini(p, 3), skin_mini(p, 4),
    ]
    .spacing(espacio::S3);
    column![
        text("CUENTA").font(fuentes::DISPLAY).size(texto::DISPLAY.0).color(p.text),
        text("mineral7@outlook.com · Microsoft").font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted),
        row![skin(p, false), skin(p, true)].spacing(espacio::S6),
        componentes::boton(p, "Abrir estudio", Variante::Secundario, Some(Mensaje::Nada)),
        icaro_ui::componentes::etiqueta(p, "Nombre de usuario"),
        text("Mineral_7").font(fuentes::MONO).size(texto::MONO.0).color(p.text),
        text("Puedes cambiarlo de nuevo en 30 días.").size(texto::BODY_SM.0).color(p.text_muted),
        icaro_ui::componentes::etiqueta(p, "Historial de skins"),
        cabezas,
        componentes::boton(p, "Cerrar sesión de esta cuenta", Variante::Peligro, Some(m(Accion::Cerrar))),
        componentes::boton(p, "Volver", Variante::Fantasma, Some(m(Accion::Cerrar))),
    ]
    .spacing(espacio::S3)
    .into()
}

fn skin_mini<'a>(p: Paleta, n: u32) -> Element<'a, Mensaje> {
    let color = [p.text, p.text_muted, p.border_strong, p.border][(n as usize) % 4];
    container(Space::new(40.0, 40.0)).style(icaro_ui::estilo::bloque(color)).into()
}

fn red_privada<'a>(p: Paleta) -> Element<'a, Mensaje> {
    let paso = |n: &str, t: &str, d: &str| -> Element<'a, Mensaje> {
        column![
            text(format!("{n}  {t}")).font(fuentes::TITULO).size(texto::BODY.0).color(p.text),
            text(d.to_owned()).size(texto::BODY_SM.0).color(p.text_muted),
        ]
        .spacing(espacio::S1)
        .into()
    };
    let miembro = |n: &str, ip: &str, en_linea: bool| -> Element<'a, Mensaje> {
        row![
            text(n.to_owned()).size(texto::BODY.0).color(p.text).width(Length::Fill),
            text(ip.to_owned()).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted),
            if en_linea { insignia(p, Estado::Exito, "En línea") } else { insignia(p, Estado::Info, "Desconectado") },
        ]
        .spacing(espacio::S3)
        .align_y(Alignment::Center)
        .into()
    };
    column![
        text("JUEGA CON AMIGOS\nSIN ABRIR PUERTOS").font(fuentes::DISPLAY).size(texto::DISPLAY.0 - 8.0).color(p.text),
        paso("1", "ZeroTier instalado", "Versión 1.14.2 · servicio activo."),
        paso("2", "Únete a una red", "Pide el ID de red a quien la creó."),
        row![
            container(text("8056c2e21c000001").font(fuentes::MONO).size(texto::MONO.0).color(p.text)).padding(espacio::S3).width(Length::Fill).style(icaro_ui::estilo::marco(p)),
            componentes::boton(p, "Unirse", Variante::Primario, Some(Mensaje::Nada)),
            componentes::boton(p, "Crear una red nueva", Variante::Secundario, Some(Mensaje::Nada)),
        ]
        .spacing(espacio::S2),
        paso("3", "Espera la autorización", "Te avisaremos cuando el dueño de la red te autorice."),
        icaro_ui::componentes::etiqueta(p, "Miembros"),
        miembro("Amigos · Javier", "10.147.17.1", true),
        miembro("Camila", "10.147.17.8", true),
        miembro("Nico", "10.147.17.12", false),
        componentes::boton(p, "Volver", Variante::Fantasma, Some(m(Accion::Cerrar))),
    ]
    .spacing(espacio::S3)
    .into()
}
