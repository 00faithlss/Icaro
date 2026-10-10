// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Las pantallas hechas hasta ahora, juntas. Ejecutar con
//! `cargo run -p icaro-ui --example launcher`.

use icaro_ui::ajustes::{encabezado_ajustes, fila_ajuste, indice_ajustes};
use icaro_ui::componentes::{self, AvanceParte, Parte, Variante};
use icaro_ui::capturas::{galeria, Captura};
use icaro_ui::consola::{consola, Linea, MensajesConsola, Nivel};
use icaro_ui::editor::{
    encabezado_editor, pestana_archivos, pestana_general, pestana_java, pestana_mundos,
    pestana_recursos, pestanas_editor, CompatPaquete, Copia, DatosEditor, EstadoGeneral,
    InstalacionJava, MensajesEncabezado, MensajesGeneral, Mundo, Paquete, AccionArchivo, ConfigJava,
    MensajesArchivos, MensajesJava, VistaArchivos,
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
#[path = "launcher_aux/archivos.rs"]
mod archivos;

use archivos::Explorador;
use icaro_ui::scroll::desplazable;
use icaro_ui::shell::{app_shell, Cuenta, DescargasActivas, MensajesShell, Seccion};
use icaro_ui::superficies::{con_velo, dialogo, menu_contextual, toast, ElementoMenu};
use icaro_ui::tema::{espacio, texto, Modo};
use iced::widget::{column, container, row, text, Space};
use iced::{window, Alignment, Element, Length, Padding, Point, Size, Task};

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
    configs: Vec<ConfigJava>,
    lista_grupos: Vec<String>,
    grupos: Vec<Option<String>>,
    cerrados: Vec<Option<String>>,
    edicion_grupo: Option<EdicionGrupo>,
    anim: Option<std::time::Instant>,
    anim_menu: Option<std::time::Instant>,
    ahora: std::time::Instant,
    ultimo_clic: Option<std::time::Instant>,
    menu_grupos: bool,
    editor_inst: usize,
    exp: Explorador,
    vista_recursos: usize,
    paquetes_activos: [bool; 3],
    mundo: usize,
    filtro_consola: usize,
    busqueda_consola: String,
    filtro_capturas: usize,
    captura: Option<usize>,
    cursor: Point,
    menu_en: Point,
    ventana: Size,
}

/// Creación o renombrado de un grupo en curso.
struct EdicionGrupo {
    /// Nombre actual si se renombra; `None` si se crea uno nuevo.
    original: Option<String>,
    texto: String,
    /// Instancia que se mueve al grupo recién creado.
    mover: Option<usize>,
}

#[derive(Debug, Clone)]
enum Mensaje {
    Editar,
    VolverEditor,
    Pestana(usize),
    Nombre(String),
    Grupo(String),
    Ancho(String),
    Alto(String),
    Completa(bool),
    Servidor(&'static str),
    JavaVersion(Option<String>),
    MemMin(u32),
    MemMax(u32),
    Metaspace(u32),
    ArgsJvm(String),
    VistaRecursos(usize),
    Paquete(usize),
    Mundo(usize),
    AbrirInstancia(usize),
    MoverGrupo(Option<String>),
    AlternarGrupo(Option<String>),
    NuevoGrupo,
    NuevoGrupoParaInstancia,
    RenombrarGrupo(String),
    EliminarGrupo(String),
    TextoGrupo(String),
    GuardarGrupo,
    CancelarGrupo,
    VerGrupos(bool),
    Cuadro(std::time::Instant),
    ElegirArchivo(usize),
    AccionArchivo(AccionArchivo),
    EditarArchivo(iced::widget::text_editor::Action),
    NombreArchivo(String),
    FiltroConsola(usize),
    BuscarConsola(String),
    FiltroCapturas(usize),
    Captura(usize),
    Ir(Seccion),
    Filtro(usize),
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
    Cursor(Point),
    Tamano(Size),
}

/// Inicia la entrada suave del contenido, salvo con movimiento reducido.
fn arrancar(app: &mut App) {
    if !app.reducir {
        app.anim = Some(std::time::Instant::now());
        app.ahora = std::time::Instant::now();
    }
}

/// Progreso de 0 a 1 de una animación que empezó en `inicio`.
fn progreso(inicio: Option<std::time::Instant>, ahora: std::time::Instant, ms: u64) -> f32 {
    inicio.map_or(1.0, |i| {
        (ahora.saturating_duration_since(i).as_millis() as f32 / ms as f32).clamp(0.0, 1.0)
    })
}

const MS_ENTRADA: u64 = 320;
const MS_MENU: u64 = 160;

/// Entrada del contenido: aparece despacio. Un velo parcial del color de fondo
/// se disipa de forma continua (sin pasos, para que no parpadee) y arranca
/// lejos de ser opaco para que el contenido nunca desaparezca del todo.
fn con_entrada<'a>(app: &App, contenido: Element<'a, Mensaje>) -> Element<'a, Mensaje> {
    let p = app.modo.paleta();
    let t = progreso(app.anim, app.ahora, MS_ENTRADA);
    let mut velo = p.bg;
    velo.a = 0.7 * (1.0 - icaro_ui::movimiento::Curva::Lineal.aplicar(t));
    // Se apila siempre para que el árbol de widgets no cambie al terminar.
    iced::widget::stack![
        contenido,
        container(Space::new(Length::Fill, Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(icaro_ui::estilo::bloque(velo)),
    ]
    .into()
}

fn abrir_instancia(app: &mut App, i: usize) {
    let datos = &instancias()[i];
    app.seleccionada = i;
    app.editor_inst = i;
    app.exp = Explorador::abrir(&datos.nombre);
    app.general.nombre = datos.nombre.clone();
    app.general.grupo = app.grupos[i].clone();
    app.menu = None;
    app.editor = Some(0);
    arrancar(app);
}

fn actualizar(app: &mut App, m: Mensaje) -> Task<Mensaje> {
    match m {
        Mensaje::Editar => {
            let i = app.menu.unwrap_or(app.seleccionada);
            abrir_instancia(app, i);
        }
        Mensaje::AbrirInstancia(i) => abrir_instancia(app, i),
        Mensaje::AlternarGrupo(g) => {
            arrancar(app);
            if let Some(pos) = app.cerrados.iter().position(|c| *c == g) {
                app.cerrados.remove(pos);
            } else {
                app.cerrados.push(g);
            }
        }
        Mensaje::MoverGrupo(g) => {
            arrancar(app);
            let i = app.menu.unwrap_or(app.seleccionada);
            app.grupos[i] = g.clone();
            app.menu = None;
            let nombre = instancias()[i].nombre.clone();
            app.aviso = Some(match g {
                Some(g) => format!("{nombre} ahora está en el grupo {g}."),
                None => format!("{nombre} quedó sin grupo."),
            });
        }
        Mensaje::VolverEditor => {
            app.editor = None;
            arrancar(app);
        }
        Mensaje::Pestana(i) => {
            app.editor = Some(i);
            arrancar(app);
        }
        Mensaje::Nombre(v) => app.general.nombre = v,
        Mensaje::Grupo(v) => {
            let g = (v != "Sin grupo").then_some(v);
            app.general.grupo = g.clone();
            app.grupos[app.editor_inst] = g;
        }
        Mensaje::NuevoGrupo => {
            app.edicion_grupo = Some(EdicionGrupo { original: None, texto: String::new(), mover: None });
        }
        Mensaje::NuevoGrupoParaInstancia => {
            let i = app.menu.unwrap_or(app.seleccionada);
            app.menu = None;
            app.edicion_grupo = Some(EdicionGrupo { original: None, texto: String::new(), mover: Some(i) });
        }
        Mensaje::RenombrarGrupo(nombre) => {
            app.edicion_grupo = Some(EdicionGrupo { texto: nombre.clone(), original: Some(nombre), mover: None });
        }
        Mensaje::TextoGrupo(t) => {
            if let Some(e) = app.edicion_grupo.as_mut() {
                e.texto = t;
            }
        }
        Mensaje::CancelarGrupo => app.edicion_grupo = None,
        Mensaje::GuardarGrupo => {
            if let Some(e) = app.edicion_grupo.take() {
                let nuevo = e.texto.trim().to_owned();
                let repetido = app.lista_grupos.iter().any(|g| g.to_lowercase() == nuevo.to_lowercase() && Some(g) != e.original.as_ref());
                if nuevo.is_empty() {
                    app.aviso = Some("El grupo necesita un nombre.".into());
                    app.edicion_grupo = Some(e);
                } else if repetido {
                    app.aviso = Some(format!("Ya existe un grupo llamado {nuevo}."));
                    app.edicion_grupo = Some(e);
                } else {
                    arrancar(app);
                    match e.original {
                        Some(viejo) => {
                            if let Some(pos) = app.lista_grupos.iter().position(|g| *g == viejo) {
                                app.lista_grupos[pos] = nuevo.clone();
                            }
                            for g in app.grupos.iter_mut().chain(app.cerrados.iter_mut()) {
                                if g.as_ref() == Some(&viejo) {
                                    *g = Some(nuevo.clone());
                                }
                            }
                            if app.general.grupo.as_ref() == Some(&viejo) {
                                app.general.grupo = Some(nuevo.clone());
                            }
                            app.aviso = Some(format!("Grupo renombrado a {nuevo}."));
                        }
                        None => {
                            app.lista_grupos.push(nuevo.clone());
                            if let Some(i) = e.mover {
                                app.grupos[i] = Some(nuevo.clone());
                                app.aviso = Some(format!("{} ahora está en el grupo {nuevo}.", instancias()[i].nombre));
                            } else {
                                app.aviso = Some(format!("Grupo {nuevo} creado."));
                            }
                        }
                    }
                }
            }
        }
        Mensaje::EliminarGrupo(nombre) => {
            arrancar(app);
            app.lista_grupos.retain(|g| *g != nombre);
            app.cerrados.retain(|g| g.as_ref() != Some(&nombre));
            let mut afectadas = 0;
            for g in app.grupos.iter_mut() {
                if g.as_ref() == Some(&nombre) {
                    *g = None;
                    afectadas += 1;
                }
            }
            if app.general.grupo.as_ref() == Some(&nombre) {
                app.general.grupo = None;
            }
            app.aviso = Some(format!("Grupo {nombre} eliminado; {afectadas} instancias quedaron sin grupo."));
        }
        Mensaje::Ancho(v) => app.general.ancho = v,
        Mensaje::Alto(v) => app.general.alto = v,
        Mensaje::Completa(v) => app.general.pantalla_completa = v,
        Mensaje::Servidor(v) => app.general.servidor = Some(v),
        Mensaje::JavaVersion(v) => app.configs[app.editor_inst].java = v,
        Mensaje::MemMin(v) => {
            let c = &mut app.configs[app.editor_inst];
            c.memoria_min = v;
            c.memoria_max = c.memoria_max.max(v);
        }
        Mensaje::MemMax(v) => {
            let c = &mut app.configs[app.editor_inst];
            c.memoria_max = v;
            c.memoria_min = c.memoria_min.min(v);
        }
        Mensaje::Metaspace(v) => app.configs[app.editor_inst].metaspace = v,
        Mensaje::ArgsJvm(v) => app.configs[app.editor_inst].argumentos = v,
        Mensaje::VistaRecursos(i) => app.vista_recursos = i,
        Mensaje::Paquete(i) => {
            if let Some(a) = app.paquetes_activos.get_mut(i) {
                *a = !*a;
            }
        }
        Mensaje::Mundo(i) => app.mundo = i,
        Mensaje::ElegirArchivo(i) => app.exp.elegir(i),
        Mensaje::EditarArchivo(a) => app.exp.editar(a),
        Mensaje::NombreArchivo(n) => app.exp.escribir_nombre(n),
        Mensaje::AccionArchivo(a) => match a {
            AccionArchivo::Abrir => app.exp.abrir_en_sistema(),
            AccionArchivo::Renombrar => app.exp.iniciar_renombrar(),
            AccionArchivo::ConfirmarNombre => app.exp.confirmar_nombre(),
            AccionArchivo::CancelarNombre => app.exp.cancelar_nombre(),
            AccionArchivo::Duplicar => app.exp.duplicar(),
            AccionArchivo::Papelera => app.exp.a_la_papelera(),
            AccionArchivo::Guardar => app.exp.guardar(),
            AccionArchivo::Descartar => app.exp.descartar(),
            AccionArchivo::CopiarRuta => {
                if let Some(ruta) = app.exp.ruta_texto() {
                    return iced::clipboard::write(ruta);
                }
            }
        },
        Mensaje::FiltroConsola(i) => app.filtro_consola = i,
        Mensaje::BuscarConsola(v) => app.busqueda_consola = v,
        Mensaje::FiltroCapturas(i) => app.filtro_capturas = i,
        Mensaje::Captura(i) => app.captura = Some(i),
        Mensaje::Ir(s) => {
            app.seccion = s;
            arrancar(app);
            app.editor = None;
        }
        Mensaje::Filtro(i) => {
            app.filtro = i;
            arrancar(app);
        }
        Mensaje::Menu(i) => {
            app.menu = i;
            app.menu_grupos = false;
            if i.is_some() && !app.reducir {
                app.anim_menu = Some(std::time::Instant::now());
                app.ahora = std::time::Instant::now();
            }
            app.menu_en = app.cursor;
        }
        Mensaje::VerGrupos(v) => app.menu_grupos = v,
        Mensaje::Cuadro(t) => app.ahora = t,
        Mensaje::Cursor(c) => app.cursor = c,
        Mensaje::Tamano(s) => app.ventana = s,
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
        Mensaje::Arrastrar => {
            // Doble clic en la barra: maximizar o restaurar.
            let ahora = std::time::Instant::now();
            if app.ultimo_clic.is_some_and(|u| ahora.duration_since(u).as_millis() < 400) {
                app.ultimo_clic = None;
                return window::get_oldest().and_then(window::toggle_maximize);
            }
            app.ultimo_clic = Some(ahora);
            return window::get_oldest().and_then(window::drag);
        }
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
        nueva("Supervivencia", "1.21.4 · Fabric", "Hace 2 h", "214 mods", Lamina::CaballeroCastillo, EstadoInstancia::Lista, ""),
        nueva("Better MC", "1.20.1 · NeoForge", "Hace 6 días", "312 mods", Lamina::CastilloTorres, EstadoInstancia::Desactualizada, ""),
        nueva("Skyblock", "1.21.4 · Quilt", "Nueva", "48 mods", Lamina::IcaroDedalo, EstadoInstancia::Instalando(0.58), "Descargando librerías · 112 de 186"),
        nueva("Mundo de Javier", "1.21.1 · Fabric", "Ahora", "96 mods", Lamina::ProdigoAldea, EstadoInstancia::Jugando, "En ejecución · 00:42:17"),
        nueva("Vanilla", "1.21.4 · Vanilla", "Hace 1 mes", "Sin mods", Lamina::ValleRocas, EstadoInstancia::Error, "Falta Java 21"),
    ]
}

fn suave<'a>(app: &'a App, cuerpo: impl Into<Element<'a, Mensaje>>) -> Element<'a, Mensaje> {
    con_entrada(app, desplazable(cuerpo))
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
        componentes::boton(p, "Nuevo grupo", Variante::Secundario, Some(Mensaje::NuevoGrupo)),
        componentes::boton(p, "Importar", Variante::Secundario, Some(Mensaje::Nada)),
        componentes::boton(p, "Crear instancia", Variante::Primario, Some(Mensaje::Nada)),
    ]
    .spacing(espacio::S2)
    .align_y(Alignment::Center);

    // Columnas que caben: barra lateral, márgenes y separación entre tarjetas.
    let disponible = app.ventana.width - 232.0 - 2.0 * espacio::S12 + espacio::S6;
    let columnas = ((disponible / (icaro_ui::instancias::ANCHO_TARJETA + espacio::S6)).floor() as usize).max(1);
    let todas = instancias();
    let filtro = ["", "fabric", "neoforge", "vanilla"][app.filtro.min(3)];
    let visible = |i: usize| filtro.is_empty() || todas[i].version.to_lowercase().contains(filtro);
    let mut cuadricula = column![].spacing(espacio::S8);
    let mut etiquetas: Vec<Option<String>> = app.lista_grupos.iter().cloned().map(Some).collect();
    etiquetas.push(None);
    let hay_grupos = etiquetas.len() > 1;
    for etiqueta in etiquetas {
        let miembros: Vec<usize> = (0..todas.len())
            .filter(|&i| app.grupos[i] == etiqueta && visible(i))
            .collect();
        if miembros.is_empty() && (!filtro.is_empty() || etiqueta.is_none()) {
            continue;
        }
        let mut seccion = column![].spacing(espacio::S4);
        if hay_grupos {
            let cerrado = app.cerrados.contains(&etiqueta);
            seccion = seccion.push(encabezado_grupo(p, etiqueta.clone(), miembros.len(), cerrado));
            if cerrado {
                cuadricula = cuadricula.push(seccion);
                continue;
            }
        }
        if miembros.is_empty() {
            seccion = seccion.push(
                text("Este grupo no tiene instancias. Muévelas desde el menú de sus tres puntos.")
                    .size(texto::BODY.0)
                    .color(p.text_muted),
            );
        }
        let mut tarjetas: Vec<Element<Mensaje>> = Vec::new();
        for i in miembros {
            let mut d = todas[i].clone();
            d.seleccionada = i == app.seleccionada;
            tarjetas.push(tarjeta_instancia(
                p,
                &d,
                AccionesTarjeta {
                    seleccionar: Mensaje::AbrirInstancia(i),
                    principal: Mensaje::Nada,
                    mas: Mensaje::Menu(Some(i)),
                },
            ));
        }
        let mut it = tarjetas.into_iter();
        let mut rejilla = column![].spacing(espacio::S6);
        loop {
            let mut fila = row![].spacing(espacio::S6);
            let mut n = 0;
            for _ in 0..columnas {
                if let Some(t) = it.next() {
                    fila = fila.push(t);
                    n += 1;
                }
            }
            if n == 0 {
                break;
            }
            rejilla = rejilla.push(fila);
        }
        cuadricula = cuadricula.push(seccion.push(rejilla));
    }
    cuadricula = cuadricula.push(tarjeta_nueva(p, Mensaje::Nada));
    let mut cuerpo_col = column![herramientas].spacing(espacio::S6);
    if let Some(e) = &app.edicion_grupo {
        cuerpo_col = cuerpo_col.push(
            row![
                container(componentes::campo(
                    p,
                    if e.original.is_some() { "Nuevo nombre del grupo" } else { "Nombre del grupo" },
                    &e.texto,
                    Mensaje::TextoGrupo,
                ))
                .width(320),
                componentes::boton(p, "Guardar grupo", Variante::Primario, Some(Mensaje::GuardarGrupo)),
                componentes::boton(p, "Cancelar", Variante::Fantasma, Some(Mensaje::CancelarGrupo)),
            ]
            .spacing(espacio::S2)
            .align_y(Alignment::Center),
        );
    }
    cuerpo_col = cuerpo_col.push(cuadricula);
    let cuerpo = cuerpo_col
        .spacing(espacio::S6)
        .padding(Padding::from([espacio::S6, espacio::S12]));
    column![
        banda(p, Lamina::CaidaCielo, "Instancias", Some(5)),
        suave(app, cuerpo)
    ]
    .into()
}

fn encabezado_grupo(
    p: icaro_ui::tema::Paleta,
    grupo: Option<String>,
    cantidad: usize,
    cerrado: bool,
) -> Element<'static, Mensaje> {
    use icaro_ui::iconos::{icono, Icono, Tam};
    let nombre = grupo.clone().unwrap_or_else(|| "Sin grupo".to_owned());
    let cabeza = iced::widget::button(
        container(
            row![
                icono(if cerrado { Icono::ChevronDerecha } else { Icono::ChevronAbajo }, Tam::Base, p.text),
                text(nombre.to_uppercase())
                    .font(fuentes::ETIQUETA)
                    .size(texto::LABEL.0 + 2.0)
                    .color(p.text),
                text(cantidad.to_string())
                    .font(fuentes::MONO)
                    .size(texto::MONO.0)
                    .color(p.text_muted),
            ]
            .spacing(espacio::S3)
            .align_y(Alignment::Center),
        )
        .center_y(Length::Fill),
    )
    .height(40)
    .padding(Padding::from([0.0, espacio::S2]))
    .on_press(Mensaje::AlternarGrupo(grupo.clone()))
    .style(icaro_ui::estilo::boton_fantasma(p));
    let mut acciones = row![cabeza, Space::with_width(Length::Fill)].align_y(Alignment::Center);
    if let Some(g) = grupo {
        acciones = acciones
            .push(componentes::boton(p, "Renombrar", Variante::Fantasma, Some(Mensaje::RenombrarGrupo(g.clone()))))
            .push(componentes::boton(p, "Eliminar grupo", Variante::Fantasma, Some(Mensaje::EliminarGrupo(g))));
    }
    column![
        acciones,
        container(Space::new(Length::Fill, 2.0)).style(icaro_ui::estilo::bloque(p.text)),
    ]
    .spacing(espacio::S1)
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
        memoria: format!("{} MB", app.configs[app.editor_inst].memoria_max),
        portada: Lamina::MelencoliaReloj,
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
            &EstadoGeneral {
                grupos: app.lista_grupos.iter().cloned().chain(["Sin grupo".to_owned()]).collect(),
                ..app.general.clone()
            },
            &MensajesGeneral {
                nombre: Mensaje::Nombre,
                grupo: Mensaje::Grupo,
                ancho: Mensaje::Ancho,
                alto: Mensaje::Alto,
                pantalla_completa: Mensaje::Completa,
                servidor: Mensaje::Servidor,
            },
        ),
        1 => pestana_java(
            p,
            &app.configs[app.editor_inst],
            &MensajesJava {
                java: Mensaje::JavaVersion,
                memoria_min: Mensaje::MemMin,
                memoria_max: Mensaje::MemMax,
                metaspace: Mensaje::Metaspace,
                argumentos: Mensaje::ArgsJvm,
            },
            16384,
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
        6 => {
            pestana_archivos(
                p,
                &VistaArchivos {
                    arbol: &app.exp.nodos,
                    elegido: app.exp.elegido,
                    contenido: app.exp.contenido.as_ref(),
                    modificado: app.exp.modificado(),
                    nota: app.exp.nota.clone(),
                    renombrando: app.exp.renombrando.as_deref(),
                },
                &MensajesArchivos {
                    elegir: Mensaje::ElegirArchivo,
                    accion: Mensaje::AccionArchivo,
                    editar: Mensaje::EditarArchivo,
                    nombre: Mensaje::NombreArchivo,
                },
            )
        }
        _ => pendiente(app),
    };
    column![encabezado, barra, con_entrada(app, cuerpo)].into()
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
    suave(app, cuerpo).into()
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
        suave(app, cuerpo)
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
        suave(app, cuerpo)
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
        banda(p, Lamina::FaetonPaisaje, "Ajustes", None),
        suave(app, cuerpo)
    ]
    .into()
}

fn pendiente(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    let partes: &[Parte] = match app.seccion {
        Seccion::Mods => &[
            Parte { nombre: "Biblioteca de mods", avance: AvanceParte::Pendiente },
            Parte { nombre: "Tienda de mods", avance: AvanceParte::Pendiente },
            Parte { nombre: "Sincronizar mods", avance: AvanceParte::Pendiente },
        ],
        _ => &[Parte { nombre: "Pantalla", avance: AvanceParte::Pendiente }],
    };
    componentes::en_construccion(p, app.seccion.nombre(), partes)
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
        let grupo_menu = app.grupos[app.menu.unwrap_or(0)].clone();
        let mut elementos_grupos: Vec<ElementoMenu<Mensaje>> = app
            .lista_grupos
            .iter()
            .map(|g| ElementoMenu::Opcion {
                icono: if grupo_menu.as_ref() == Some(g) { Icono::Check } else { Icono::Carpeta },
                texto: g.clone().into(),
                atajo: None,
                peligro: false,
                mensaje: Mensaje::MoverGrupo(Some(g.clone())),
            })
            .collect();
        elementos_grupos.push(ElementoMenu::Opcion { icono: if grupo_menu.is_none() { Icono::Check } else { Icono::Carpeta }, texto: "Sin grupo".into(), atajo: None, peligro: false, mensaje: Mensaje::MoverGrupo(None) });
        elementos_grupos.push(ElementoMenu::Separador);
        elementos_grupos.push(ElementoMenu::Opcion { icono: Icono::Mas, texto: "Nuevo grupo".into(), atajo: None, peligro: false, mensaje: Mensaje::NuevoGrupoParaInstancia });
        elementos_grupos.push(ElementoMenu::Opcion { icono: Icono::ChevronDerecha, texto: "Volver".into(), atajo: None, peligro: false, mensaje: Mensaje::VerGrupos(false) });
        let menu = menu_contextual(
            p,
            if app.menu_grupos { elementos_grupos } else { vec![
                ElementoMenu::Opcion { icono: Icono::Jugar, texto: "Jugar".into(), atajo: Some("Enter"), peligro: false, mensaje: Mensaje::Menu(None) },
                ElementoMenu::Opcion { icono: Icono::Carpeta, texto: "Abrir carpeta".into(), atajo: Some("O"), peligro: false, mensaje: Mensaje::Menu(None) },
                ElementoMenu::Opcion { icono: Icono::Editar, texto: "Editar".into(), atajo: Some("E"), peligro: false, mensaje: Mensaje::Editar },
                ElementoMenu::Separador,
                ElementoMenu::Opcion { icono: Icono::Carpeta, texto: "Mover a grupo".into(), atajo: Some(">"), peligro: false, mensaje: Mensaje::VerGrupos(true) },
                ElementoMenu::Separador,
                ElementoMenu::Opcion { icono: Icono::Copiar, texto: "Duplicar".into(), atajo: None, peligro: false, mensaje: Mensaje::Menu(None) },
                ElementoMenu::Opcion { icono: Icono::Subir, texto: "Exportar modpack".into(), atajo: None, peligro: false, mensaje: Mensaje::Menu(None) },
                ElementoMenu::Opcion { icono: Icono::Externo, texto: "Crear acceso directo".into(), atajo: None, peligro: false, mensaje: Mensaje::Menu(None) },
                ElementoMenu::Separador,
                ElementoMenu::Opcion { icono: Icono::Papelera, texto: "Eliminar".into(), atajo: Some("Supr"), peligro: true, mensaje: Mensaje::PedirEliminar },
            ] },
        );
        let caida = (1.0 - icaro_ui::movimiento::Curva::Pasos(4).aplicar(progreso(app.anim_menu, app.ahora, MS_MENU))) * 12.0;
        ventana = iced::widget::stack![
            ventana,
            iced::widget::mouse_area(
                container(menu)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding {
                    top: caida + app.menu_en.y.clamp(0.0, (app.ventana.height - 420.0).max(0.0)),
                    left: app.menu_en.x.clamp(0.0, (app.ventana.width - 268.0).max(0.0)),
                    ..Padding::ZERO
                }),
            )
            .on_press(Mensaje::Menu(None))
            .on_scroll(|_| Mensaje::Menu(None))
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
                Some(Lamina::FaetonMano),
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
    iced::widget::mouse_area(ventana)
        .on_move(Mensaje::Cursor)
        .into()
}

fn main() -> iced::Result {
    let mut app = iced::application("Ícaro", actualizar, vista)
        .theme(|_| iced::Theme::Dark)
        .default_font(fuentes::CUERPO)
        .subscription(|app| {
            let mut subs = vec![window::resize_events().map(|(_, s)| Mensaje::Tamano(s))];
            if progreso(app.anim, app.ahora, MS_ENTRADA) < 1.0
                || progreso(app.anim_menu, app.ahora, MS_MENU) < 1.0
            {
                subs.push(window::frames().map(Mensaje::Cuadro));
            }
            iced::Subscription::batch(subs)
        })
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
                    grupo: Some("Con amigos".to_owned()),
                    grupos: vec!["Con amigos".into(), "Técnicos".into(), "Sin grupo".into()],
                    ancho: "1920".into(),
                    alto: "1080".into(),
                    pantalla_completa: false,
                    servidor: Some("Amigos · 10.147.17.1"),
                },
                cerrados: vec![],
                anim: None,
                anim_menu: None,
                ahora: std::time::Instant::now(),
                ultimo_clic: None,
                menu_grupos: false,
                lista_grupos: vec!["Con amigos".into(), "Técnicos".into()],
                edicion_grupo: None,
                grupos: vec![Some("Con amigos".into()), Some("Técnicos".into()), Some("Con amigos".into()), None, None],
                configs: (0..5)
                    .map(|_| ConfigJava {
                        java: None,
                        memoria_min: 2048,
                        memoria_max: 6144,
                        metaspace: 512,
                        argumentos: "-XX:+UseG1GC".into(),
                    })
                    .collect(),
                editor_inst: 0,
                exp: Explorador::abrir("Supervivencia"),
                vista_recursos: 0,
                paquetes_activos: [true, false, true],
                mundo: 0,
                filtro_consola: 0,
                busqueda_consola: String::new(),
                filtro_capturas: 0,
                captura: Some(1),
                cursor: Point::ORIGIN,
                menu_en: Point::new(420.0, 140.0),
                ventana: Size::new(1280.0, 800.0),
            },
            Task::none(),
        )
    })
}
