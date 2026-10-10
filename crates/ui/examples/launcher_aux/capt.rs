// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Capturas, visor, Gabinete y estados vacíos del launcher de ejemplo.
//! Los datos son de muestra: las capturas son recortes de grabados y las
//! acciones de copiar y abrir carpeta son simuladas.

use std::time::{Duration, Instant};

use icaro_ui::capturas::{galeria, Captura, MensajesCapturas, VistaCapturas};
use icaro_ui::fuentes;
use icaro_ui::gabinete::{self, MensajesGabinete};
use icaro_ui::iconos::Icono;
use icaro_ui::laminas::Lamina;
use icaro_ui::movimiento;
use icaro_ui::superficies::toast;
use icaro_ui::tema::{duracion, espacio, texto};
use icaro_ui::vacios::{self, Contexto};
use icaro_ui::visor::{visor, EstadoVisor, ItemVisor, MensajesVisor};
use iced::widget::{column, container, text};
use iced::{Element, Length, Padding, Subscription, Task};

use crate::{instancias, App, Mensaje};

/// Cuánto dura un aviso antes de retirarse solo.
const AVISO_MS: u64 = 5000;

/// De dónde sale la lista que recorre el visor.
#[derive(Debug, Clone, PartialEq)]
pub enum Ambito {
    /// Capturas, todas o de una sola instancia.
    Capturas(Option<String>),
    /// Piezas del Gabinete.
    Gabinete,
}

pub struct Abierto {
    pub estado: EstadoVisor,
    pub ambito: Ambito,
}

pub struct Estado {
    pub capturas: Vec<Captura>,
    pub filtro: usize,
    pub seleccion: Vec<usize>,
    pub encima: Option<usize>,
    pub visor: Option<Abierto>,
    pub gabinete_encima: Option<usize>,
    /// Capturas enviadas a la papelera con su lugar, para Deshacer.
    pub papelera: Vec<(usize, Captura)>,
    pub aviso: Option<Aviso>,
    /// Muestra todas las pantallas sin datos.
    pub vacio: bool,
    /// Inicio de la Prensa de las láminas de la pantalla.
    pub prensa: Option<Instant>,
}

pub struct Aviso {
    pub texto: String,
    pub deshacer: bool,
    pub desde: Instant,
}

#[derive(Debug, Clone)]
pub enum Accion {
    Filtrar(String),
    Abrir(usize),
    Alternar(usize),
    Pasar(Option<usize>),
    Copiar(usize),
    Carpeta,
    Eliminar(usize),
    AbrirCarpeta,
    CopiarSeleccion,
    EliminarSeleccion,
    LimpiarSeleccion,
    VerTodas,
    Deshacer,
    CerrarAviso,
    Tic(Instant),
    // Visor
    VisorPaso(i32),
    VisorAcercar,
    VisorAlejar,
    VisorCerrar,
    VisorCopiar,
    VisorCarpeta,
    VisorEliminar,
    // Gabinete
    GabinetePasar(Option<usize>),
    GabineteAbrir(usize),
    // Estados vacíos
    AlternarVacios,
}

fn captura(archivo: &str, instancia: &str, fecha: &str, mb: f32, imagen: Lamina) -> Captura {
    Captura {
        archivo: archivo.into(),
        instancia: instancia.into(),
        fecha: fecha.into(),
        bytes: (mb * 1024.0 * 1024.0) as u64,
        imagen,
    }
}

impl Estado {
    pub fn nuevo() -> Self {
        let capturas = vec![
            captura("2026-10-09_01.12.04.png", "Supervivencia", "09-10-2026 01:12", 3.4, Lamina::ParejaAldea),
            captura("2026-10-08_22.40.31.png", "Supervivencia", "08-10-2026 22:40", 2.9, Lamina::CascadaAgua),
            captura("2026-10-07_19.03.55.png", "Skyblock", "07-10-2026 19:03", 4.1, Lamina::SoldadosCastillo),
            captura("2026-10-05_16.20.10.png", "Better MC", "05-10-2026 16:20", 3.8, Lamina::IxionNubes),
            captura("2026-10-02_11.48.09.png", "Supervivencia", "02-10-2026 11:48", 2.6, Lamina::DemogorgonCueva),
            captura("2026-09-28_09.15.42.png", "Mundo de Javier", "28-09-2026 09:15", 3.1, Lamina::MelencoliaCometa),
            captura("2026-09-27_21.07.18.png", "Better MC", "27-09-2026 21:07", 4.4, Lamina::CascadaMolino),
            captura("2026-09-25_18.33.02.png", "Supervivencia", "25-09-2026 18:33", 2.2, Lamina::HuidaBosque),
            captura("2026-09-22_14.55.47.png", "Skyblock", "22-09-2026 14:55", 3.7, Lamina::MercadoAldea),
            captura("2026-09-19_23.10.26.png", "Mundo de Javier", "19-09-2026 23:10", 2.8, Lamina::TivoliCascada),
            captura("2026-09-15_10.02.39.png", "Supervivencia", "15-09-2026 10:02", 3.3, Lamina::ValleBosque),
            captura("2026-09-12_20.41.15.png", "Better MC", "12-09-2026 20:41", 4.0, Lamina::FortalezasValle),
        ];
        Self {
            capturas,
            filtro: 0,
            seleccion: Vec::new(),
            encima: None,
            visor: None,
            gabinete_encima: None,
            papelera: Vec::new(),
            aviso: None,
            vacio: std::env::var_os("ICARO_VACIO").is_some(),
            prensa: None,
        }
    }

    /// Capturas visibles en un ámbito, como índices de `capturas`.
    fn visibles(&self, instancia: Option<&str>) -> Vec<usize> {
        self.capturas
            .iter()
            .enumerate()
            .filter(|(_, c)| instancia.is_none_or(|n| c.instancia == n))
            .map(|(i, _)| i)
            .collect()
    }

    fn avisar(&mut self, texto: impl Into<String>, deshacer: bool) {
        self.aviso = Some(Aviso { texto: texto.into(), deshacer, desde: Instant::now() });
    }

    /// Si hay algo moviéndose que exija seguir dibujando cuadros.
    pub fn animando(&self, ahora: Instant, reducir: bool) -> bool {
        self.visor.as_ref().is_some_and(|v| v.estado.animando(ahora, reducir))
            || (!reducir && prensa_en_curso(self.prensa, ahora))

    }
}

fn prensa_en_curso(inicio: Option<Instant>, ahora: Instant) -> bool {
    inicio.is_some_and(|i| ahora.saturating_duration_since(i).as_millis() < u128::from(duracion::PRENSA))
}

/// Avance de la Prensa de las láminas de la pantalla, de 0 a 1.
fn prensa(app: &App) -> f32 {
    match app.capt.prensa {
        Some(i) if !app.reducir => movimiento::prensa(
            (app.ahora.saturating_duration_since(i).as_millis() as f32 / duracion::PRENSA as f32).clamp(0.0, 1.0),
        ),
        _ => 1.0,
    }
}

/// Contexto de pintura de los estados vacíos de una pantalla.
pub fn contexto(app: &App) -> Contexto {
    Contexto { p: app.modo.paleta(), ventana: app.ventana.width, prensa: prensa(app) }
}

/// Instancia a la que se limitan las capturas según dónde se esté.
fn instancia_actual(app: &App) -> Option<String> {
    if app.editor == Some(5) {
        return instancias().get(app.editor_inst).map(|d| d.nombre.clone());
    }
    let nombres = nombres_filtro();
    (app.capt.filtro > 0).then(|| nombres[app.capt.filtro].clone())
}

fn nombres_filtro() -> Vec<String> {
    std::iter::once("Todas las instancias".to_owned())
        .chain(instancias().into_iter().map(|d| d.nombre))
        .collect()
}

pub fn actualizar(app: &mut App, a: Accion) -> Task<Mensaje> {
    let ahora = Instant::now();
    match a {
        Accion::Filtrar(nombre) => {
            app.capt.filtro = nombres_filtro().iter().position(|n| *n == nombre).unwrap_or(0);
            app.capt.seleccion.clear();
        }
        Accion::Abrir(i) => {
            if !app.capt.seleccion.is_empty() {
                alternar(&mut app.capt, i);
            } else {
                let instancia = instancia_actual(app);
                let lista = app.capt.visibles(instancia.as_deref());
                if let Some(pos) = lista.iter().position(|&x| x == i) {
                    abrir_visor(app, pos, Ambito::Capturas(instancia));
                }
            }
        }
        Accion::Alternar(i) => alternar(&mut app.capt, i),
        Accion::Pasar(i) => app.capt.encima = i,
        Accion::Copiar(i) => copiar(app, i),
        Accion::Carpeta | Accion::AbrirCarpeta | Accion::VisorCarpeta => abrir_carpeta(),
        Accion::Eliminar(i) => eliminar(app, &[i]),
        Accion::CopiarSeleccion => {
            let n = app.capt.seleccion.len();
            app.capt.avisar(format!("{n} capturas copiadas al portapapeles."), false);
        }
        Accion::EliminarSeleccion => {
            let lista = app.capt.seleccion.clone();
            eliminar(app, &lista);
        }
        Accion::LimpiarSeleccion => app.capt.seleccion.clear(),
        Accion::VerTodas => app.capt.filtro = 0,
        Accion::Deshacer => deshacer(&mut app.capt),
        Accion::CerrarAviso => app.capt.aviso = None,
        Accion::Tic(t) => {
            if app.capt.aviso.as_ref().is_some_and(|v| t.saturating_duration_since(v.desde) > Duration::from_millis(AVISO_MS)) {
                app.capt.aviso = None;
            }
        }
        Accion::VisorPaso(d) => {
            if let Some(v) = &mut app.capt.visor {
                let total = total_visor(&app.capt.capturas, &v.ambito) as i32;
                if total > 0 {
                    let nuevo = (v.estado.indice as i32 + d).rem_euclid(total) as usize;
                    v.estado.ir_a(nuevo);
                }
            }
        }
        Accion::VisorAcercar => {
            if let Some(v) = &mut app.capt.visor {
                v.estado.acercar();
            }
        }
        Accion::VisorAlejar => {
            if let Some(v) = &mut app.capt.visor {
                v.estado.alejar();
            }
        }
        Accion::VisorCerrar => cerrar_visor(app, ahora),
        Accion::VisorCopiar => {
            if let Some(i) = captura_del_visor(app) {
                copiar(app, i);
            }
        }
        Accion::VisorEliminar => {
            if let Some(i) = captura_del_visor(app) {
                eliminar(app, &[i]);
            }
        }
        Accion::GabinetePasar(i) => app.capt.gabinete_encima = i,
        Accion::GabineteAbrir(i) => abrir_visor(app, i, Ambito::Gabinete),
        Accion::AlternarVacios => {
            app.capt.vacio = !app.capt.vacio;
            crate::arrancar(app);
        }
    }
    app.ahora = ahora;
    Task::none()
}

fn alternar(est: &mut Estado, i: usize) {
    if let Some(pos) = est.seleccion.iter().position(|&x| x == i) {
        est.seleccion.remove(pos);
    } else {
        est.seleccion.push(i);
    }
}

fn abrir_visor(app: &mut App, indice: usize, ambito: Ambito) {
    let ahora = Instant::now();
    app.ahora = ahora;
    app.capt.visor = Some(Abierto { estado: EstadoVisor::nuevo(indice, ahora), ambito });
}

fn cerrar_visor(app: &mut App, ahora: Instant) {
    if let Some(v) = &mut app.capt.visor {
        if v.estado.cierre.is_none() {
            v.estado.cierre = Some(ahora);
        }
    }
}

fn total_visor(capturas: &[Captura], ambito: &Ambito) -> usize {
    match ambito {
        Ambito::Gabinete => gabinete::PIEZAS.len(),
        Ambito::Capturas(instancia) => capturas
            .iter()
            .filter(|c| instancia.as_deref().is_none_or(|n| c.instancia == n))
            .count(),
    }
}

/// Índice, en `capturas`, de la captura que muestra el visor.
fn captura_del_visor(app: &App) -> Option<usize> {
    let v = app.capt.visor.as_ref()?;
    let Ambito::Capturas(instancia) = &v.ambito else {
        return None;
    };
    app.capt.visibles(instancia.as_deref()).get(v.estado.indice).copied()
}

fn copiar(app: &mut App, i: usize) {
    if let Some(c) = app.capt.capturas.get(i) {
        let nombre = c.archivo.clone();
        app.capt.avisar(format!("{nombre} copiada al portapapeles."), false);
    }
}

fn abrir_carpeta() {
    let dir = std::env::temp_dir().join("icaro-demo").join("capturas");
    let _ = std::fs::create_dir_all(&dir);
    crate::pantallas::abrir_en_sistema(&dir);
}

/// Envía capturas a la papelera; se pueden recuperar con Deshacer.
fn eliminar(app: &mut App, indices: &[usize]) {
    let mut orden: Vec<usize> = indices.to_vec();
    orden.sort_unstable();
    orden.dedup();
    let mut quitadas = Vec::new();
    for &i in orden.iter().rev() {
        if i < app.capt.capturas.len() {
            quitadas.push((i, app.capt.capturas.remove(i)));
        }
    }
    if quitadas.is_empty() {
        return;
    }
    quitadas.reverse();
    let n = quitadas.len();
    app.capt.papelera = quitadas;
    app.capt.seleccion.clear();
    app.capt.encima = None;
    app.capt.avisar(
        if n == 1 {
            "Captura enviada a la papelera.".to_owned()
        } else {
            format!("{n} capturas enviadas a la papelera.")
        },
        true,
    );
    // El visor sigue en la que quedó en ese lugar, o se cierra si no queda ninguna.
    let ahora = Instant::now();
    let total = app.capt.visor.as_ref().map(|v| total_visor(&app.capt.capturas, &v.ambito));
    if let (Some(v), Some(total)) = (&mut app.capt.visor, total) {
        if total == 0 {
            if v.estado.cierre.is_none() {
                v.estado.cierre = Some(ahora);
            }
        } else if v.estado.indice >= total {
            v.estado.ir_a(total - 1);
        } else {
            v.estado.ir_a(v.estado.indice);
        }
    }
}

fn deshacer(est: &mut Estado) {
    for (i, c) in std::mem::take(&mut est.papelera) {
        let lugar = i.min(est.capturas.len());
        est.capturas.insert(lugar, c);
    }
    est.aviso = None;
}

// ---------------------------------------------------------------- vistas

fn mensajes() -> MensajesCapturas<Mensaje> {
    MensajesCapturas {
        filtrar: |n| Mensaje::Capt(Accion::Filtrar(n)),
        abrir: |i| Mensaje::Capt(Accion::Abrir(i)),
        alternar: |i| Mensaje::Capt(Accion::Alternar(i)),
        pasar: |i| Mensaje::Capt(Accion::Pasar(i)),
        copiar: |i| Mensaje::Capt(Accion::Copiar(i)),
        carpeta: |_| Mensaje::Capt(Accion::Carpeta),
        eliminar: |i| Mensaje::Capt(Accion::Eliminar(i)),
        abrir_carpeta: Mensaje::Capt(Accion::AbrirCarpeta),
        copiar_seleccion: Mensaje::Capt(Accion::CopiarSeleccion),
        eliminar_seleccion: Mensaje::Capt(Accion::EliminarSeleccion),
        limpiar_seleccion: Mensaje::Capt(Accion::LimpiarSeleccion),
    }
}

/// Ancho que queda para el contenido de una sección.
fn ancho_contenido(app: &App) -> f32 {
    let barra = if app.colapsada { 64.0 } else { 232.0 };
    app.ventana.width - barra - 2.0 * espacio::S12
}

/// Galería (o su estado vacío), limitada a una instancia si se indica.
fn cuerpo<'a>(app: &'a App, instancia: Option<String>, con_filtro: bool) -> Element<'a, Mensaje> {
    let p = app.modo.paleta();
    let ctx = contexto(app);
    let est = &app.capt;
    if est.vacio || est.capturas.is_empty() {
        return vacios::capturas(ctx, Mensaje::Ir(icaro_ui::shell::Seccion::Instancias));
    }
    let visibles: Vec<usize> = est.visibles(instancia.as_deref());
    // Las seleccionadas que ya no se ven dejan de contar.
    let seleccion: Vec<usize> = est.seleccion.iter().copied().filter(|i| visibles.contains(i)).collect();
    let vista = VistaCapturas {
        p,
        ancho: ancho_contenido(app),
        instancias: if con_filtro { nombres_filtro() } else { Vec::new() },
        filtro: est.filtro,
        capturas: &est.capturas,
        visibles: &visibles,
        seleccion: &seleccion,
        encima: est.encima,
    };
    let galeria = galeria(&vista, &mensajes());
    if visibles.is_empty() {
        let nombre = instancia.unwrap_or_default();
        return column![
            galeria,
            vacios::capturas_de_instancia(ctx, &nombre, Mensaje::Capt(Accion::VerTodas)),
        ]
        .spacing(espacio::S6)
        .into();
    }
    galeria
}

/// Pantalla Capturas.
pub fn pantalla(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    let nombre = instancia_actual(app);
    let contenido = column![
        container(
            text("CAPTURAS")
                .font(fuentes::DISPLAY)
                .size(texto::DISPLAY_XL.0)
                .color(p.text)
        ),
        cuerpo(app, nombre, true),
    ]
    .spacing(espacio::S6)
    .padding(Padding::from([espacio::S6, espacio::S12]));
    crate::suave(app, contenido)
}

/// Pestaña Capturas del editor, limitada a la instancia abierta.
pub fn pestana(app: &App) -> Element<'_, Mensaje> {
    let nombre = instancias().get(app.editor_inst).map(|d| d.nombre.clone());
    icaro_ui::scroll::desplazable(
        container(cuerpo(app, nombre, false)).padding(Padding::from([espacio::S6, espacio::S12])),
    )
}

/// Gabinete de grabados, con su encabezado.
pub fn pared(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    column![
        gabinete::encabezado(p),
        gabinete::gabinete(
            p,
            app.capt.gabinete_encima,
            None,
            app.reducir,
            &MensajesGabinete {
                pasar: |i| Mensaje::Capt(Accion::GabinetePasar(i)),
                abrir: |i| Mensaje::Capt(Accion::GabineteAbrir(i)),
                usar: None,
            },
        ),
    ]
    .spacing(espacio::S8)
    .into()
}

/// Apila el visor y el aviso sobre la ventana.
pub fn capa<'a>(app: &'a App, ventana: Element<'a, Mensaje>) -> Element<'a, Mensaje> {
    let p = app.modo.paleta();
    let mut resultado = ventana;
    if let Some(v) = &app.capt.visor {
        let (item, total, acciones) = match &v.ambito {
            Ambito::Gabinete => {
                let piezas = gabinete::items_visor();
                let total = piezas.len();
                let item = piezas.into_iter().nth(v.estado.indice).unwrap_or(ItemVisor {
                    titulo: String::new(),
                    detalle: String::new(),
                    lamina: Lamina::Umbral,
                });
                (item, total, false)
            }
            Ambito::Capturas(instancia) => {
                let lista = app.capt.visibles(instancia.as_deref());
                let c = lista.get(v.estado.indice).map(|&i| &app.capt.capturas[i]);
                let item = c.map_or(
                    ItemVisor { titulo: String::new(), detalle: String::new(), lamina: Lamina::Umbral },
                    |c| ItemVisor {
                        titulo: c.archivo.clone(),
                        detalle: format!("{} · {}", c.instancia, c.fecha),
                        lamina: c.imagen,
                    },
                );
                (item, lista.len(), true)
            }
        };
        let m = |a: Accion| Mensaje::Capt(a);
        resultado = iced::widget::stack![
            resultado,
            visor(
                p.modo,
                &v.estado,
                app.ahora,
                app.reducir,
                &item,
                total,
                MensajesVisor {
                    cerrar: m(Accion::VisorCerrar),
                    anterior: m(Accion::VisorPaso(-1)),
                    siguiente: m(Accion::VisorPaso(1)),
                    acercar: m(Accion::VisorAcercar),
                    alejar: m(Accion::VisorAlejar),
                    copiar: acciones.then(|| m(Accion::VisorCopiar)),
                    carpeta: acciones.then(|| m(Accion::VisorCarpeta)),
                    eliminar: acciones.then(|| m(Accion::VisorEliminar)),
                },
            ),
        ]
        .into();
    }
    if let Some(a) = &app.capt.aviso {
        resultado = iced::widget::stack![
            resultado,
            container(toast(
                p,
                Icono::Check,
                &a.texto,
                a.deshacer.then_some(("Deshacer", Mensaje::Capt(Accion::Deshacer))),
                Some(Mensaje::Capt(Accion::CerrarAviso)),
                false,
            ))
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Bottom)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from([espacio::S6, 256.0]))
        ]
        .into();
    }
    resultado
}

/// Teclado del visor y reloj del aviso.
pub fn suscripcion(app: &App) -> Subscription<Mensaje> {
    let mut subs = Vec::new();
    if app.capt.visor.as_ref().is_some_and(|v| v.estado.cierre.is_none()) {
        subs.push(iced::keyboard::on_key_press(|tecla, _| {
            use iced::keyboard::key::Named;
            use iced::keyboard::Key;
            let a = match tecla.as_ref() {
                Key::Named(Named::Escape) => Accion::VisorCerrar,
                Key::Named(Named::ArrowLeft) => Accion::VisorPaso(-1),
                Key::Named(Named::ArrowRight) => Accion::VisorPaso(1),
                Key::Character("+" | "=") => Accion::VisorAcercar,
                Key::Character("-") => Accion::VisorAlejar,
                _ => return None,
            };
            Some(Mensaje::Capt(a))
        }));
    }
    if app.capt.aviso.is_some() {
        subs.push(iced::time::every(Duration::from_millis(500)).map(|t| Mensaje::Capt(Accion::Tic(t))));
    }
    Subscription::batch(subs)
}

/// Quita el visor cuando el Rastrillo terminó de bajar.
pub fn cuadro(app: &mut App) {
    let reducir = app.reducir;
    let ahora = app.ahora;
    if app.capt.visor.as_ref().is_some_and(|v| v.estado.cerrado(ahora, reducir)) {
        app.capt.visor = None;
    }
}

// ------------------------------------------------------ estados vacíos

/// Pantalla con su banda y el vacío debajo.
fn con_banda<'a>(
    app: &'a App,
    lamina: Lamina,
    titulo: &str,
    vacio: Element<'a, Mensaje>,
) -> Element<'a, Mensaje> {
    let p = app.modo.paleta();
    column![
        icaro_ui::laminas::banda(p, lamina, titulo, Some(0)),
        crate::suave(app, container(vacio).padding(vacios::margen())),
    ]
    .into()
}

pub fn vacio_instancias(app: &App) -> Element<'_, Mensaje> {
    con_banda(
        app,
        Lamina::CaidaCielo,
        "Instancias",
        vacios::instancias(
            contexto(app),
            Mensaje::Pant(crate::pantallas::Accion::NuevaInstancia),
            Mensaje::Pant(crate::pantallas::Accion::Importar),
        ),
    )
}

pub fn vacio_servidores(app: &App) -> Element<'_, Mensaje> {
    con_banda(app, Lamina::TivoliCiudad, "Servidores", vacios::servidores(contexto(app), Mensaje::Nada))
}

pub fn vacio_descargas(app: &App) -> Element<'_, Mensaje> {
    con_banda(
        app,
        Lamina::MercurioFriso,
        "Descargas",
        vacios::descargas(contexto(app), Mensaje::Ir(icaro_ui::shell::Seccion::Modpacks)),
    )
}

pub fn vacio_consola(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    column![
        container(
            text("CONSOLA")
                .font(fuentes::DISPLAY)
                .size(texto::DISPLAY_XL.0)
                .color(p.text)
        )
        .padding(Padding::from([espacio::S6, espacio::S12])),
        crate::suave(
            app,
            container(vacios::consola(contexto(app), Mensaje::Ir(icaro_ui::shell::Seccion::Instancias)))
                .padding(vacios::margen())
        ),
    ]
    .into()
}

/// Búsqueda sin resultados dentro de un panel: la lámina va arriba.
pub fn vacio_busqueda<'a>(app: &App, consulta: &str, limpiar: Mensaje) -> Element<'a, Mensaje> {
    let ctx = Contexto { ventana: 0.0, ..contexto(app) };
    vacios::busqueda(ctx, consulta, limpiar)
}
