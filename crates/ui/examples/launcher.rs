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
    pestanas_editor, Copia, DatosEditor, EstadoGeneral,
    InstalacionJava, MensajesEncabezado, MensajesGeneral, Mundo, AccionArchivo, ConfigJava,
    MensajesArchivos, MensajesJava, VistaArchivos,
};
use icaro_ui::descargas::{
    cola_descargas, historial, limites, AccionesCola, Descarga, EntradaHistorial, EstadoDescarga,
};
use icaro_ui::fuentes;
use icaro_ui::iconos::{icono, Icono, Tam};
use icaro_ui::instancias::{
    tarjeta_instancia, tarjeta_nueva, AccionesTarjeta, DatosInstancia, EstadoInstancia,
};
use icaro_ui::laminas::{banda, Lamina};
use icaro_ui::servidores::{red_privada, tarjeta_servidor, EstadoServidor, Servidor};
#[path = "launcher_aux/archivos.rs"]
mod archivos;
#[path = "launcher_aux/estado.rs"]
mod estado;
#[path = "launcher_aux/mods.rs"]
mod mods;
#[path = "launcher_aux/modpacks.rs"]
mod modpacks;

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
    busqueda_mods: String,
    mods_vista: usize,
    modpack_filtro: usize,
    modpack_pagina: Option<usize>,
    modpack_pestana: usize,
    modpack_galeria: usize,
    busqueda_modpacks: String,
    instalando: Vec<(String, std::time::Instant)>,
    mod_pagina: Option<String>,
    mod_filtro_mc: String,
    mod_filtro_loader: String,
    mod_betas: bool,
    mod_galeria: usize,
    mod_opcionales: Vec<String>,
    mod_pestana: usize,
    ultimo_guardado: String,
    anim: Option<std::time::Instant>,
    anim_menu: Option<std::time::Instant>,
    ahora: std::time::Instant,
    ultimo_clic: Option<std::time::Instant>,
    menu_grupos: bool,
    editor_inst: usize,
    exp: Explorador,
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
    BuscarMods(String),
    AlternarMod(String),
    VistaMods(usize),
    FiltroModpacks(usize),
    AbrirModpack(usize),
    CerrarModpack,
    PestanaModpack(usize),
    GaleriaModpack(usize),
    BuscarModpacks(String),
    InstalarModpack(usize),
    AbrirModPagina(String),
    CerrarModPagina,
    PestanaMod(usize),
    FiltroMcMod(String),
    FiltroLoaderMod(String),
    BetasMod,
    GaleriaMod(usize),
    OpcionalMod(String),
    VersionMod(String, String),
    InstalarMod(String, String),
    EliminarMod(String),
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
    app.mod_pagina = None;
    app.menu = None;
    app.editor = Some(0);
    arrancar(app);
}

fn actualizar(app: &mut App, m: Mensaje) -> Task<Mensaje> {
    mods::fijar_recurso(app.editor == Some(3));
    let tarea = actualizar_estado(app, m);
    guardar_si_cambio(app);
    tarea
}

/// Escribe el estado organizado por el usuario solo cuando cambia.
fn guardar_si_cambio(app: &mut App) {
    let texto = estado::Guardado::nuevo(&app.lista_grupos, &app.grupos, &app.cerrados, &app.configs).texto();
    if texto != app.ultimo_guardado {
        estado::Guardado::escribir(&texto);
        app.ultimo_guardado = texto;
    }
}

fn actualizar_estado(app: &mut App, m: Mensaje) -> Task<Mensaje> {
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
        Mensaje::BuscarMods(t) => app.busqueda_mods = t,
        Mensaje::AlternarMod(archivo) => mods::alternar(&instancias()[app.editor_inst].nombre, &archivo),
        Mensaje::VistaMods(v) => {
            app.mods_vista = v;
            arrancar(app);
        }
        Mensaje::AbrirModPagina(id) => {
            app.mod_pagina = Some(id);
            app.mod_pestana = 0;
            app.mod_filtro_mc = "Todas".into();
            app.mod_filtro_loader = "Todos".into();
            app.mod_betas = false;
            app.mod_galeria = 0;
            app.mod_opcionales.clear();
            arrancar(app);
        }
        Mensaje::CerrarModPagina => {
            app.mod_pagina = None;
            arrancar(app);
        }
        Mensaje::FiltroMcMod(v) => app.mod_filtro_mc = v,
        Mensaje::FiltroLoaderMod(v) => app.mod_filtro_loader = v,
        Mensaje::BetasMod => app.mod_betas = !app.mod_betas,
        Mensaje::GaleriaMod(i) => app.mod_galeria = i,
        Mensaje::OpcionalMod(id) => {
            if let Some(pos) = app.mod_opcionales.iter().position(|o| *o == id) {
                app.mod_opcionales.remove(pos);
            } else {
                app.mod_opcionales.push(id);
            }
        }
        Mensaje::PestanaMod(i) => {
            app.mod_pestana = i;
            arrancar(app);
        }
        Mensaje::VersionMod(archivo, version) => {
            let inst = instancias()[app.editor_inst].nombre.clone();
            let id = mods::listar(&inst).into_iter().find(|m| m.archivo == archivo).map(|m| m.id);
            if let Some(id) = id {
                if mods::cambiar_version(&inst, &archivo, &id, &version).is_some() {
                    app.aviso = Some(format!("{id} cambió a la versión {version}."));
                }
            }
        }
        Mensaje::InstalarMod(id, version) => {
            let inst = instancias()[app.editor_inst].nombre.clone();
            mods::instalar(&inst, &id, &version);
            // Dependencias obligatorias que falten y opcionales marcadas.
            if let Some(c) = mods::en_catalogo(&id) {
                let puestos = mods::listar(&inst);
                let opcionales = app.mod_opcionales.clone();
                for dep in c.dependencias.iter().copied().chain(opcionales.iter().map(String::as_str)) {
                    if puestos.iter().all(|m| m.id != dep) {
                        if let Some(d) = mods::en_catalogo(dep) {
                            mods::instalar(&inst, dep, d.lanzamientos[0].version);
                        }
                    }
                }
            }
            app.mod_opcionales.clear();
            app.aviso = Some(format!("{id} {version} instalado."));
        }
        Mensaje::EliminarMod(archivo) => {
            mods::eliminar(&instancias()[app.editor_inst].nombre, &archivo);
            app.aviso = Some(format!("{archivo} se movió a la papelera."));
        }
        Mensaje::Cuadro(t) => {
            app.ahora = t;
            avanzar_instalaciones(app);
        }
        Mensaje::FiltroModpacks(i) => {
            app.modpack_filtro = i;
            arrancar(app);
        }
        Mensaje::BuscarModpacks(t) => app.busqueda_modpacks = t,
        Mensaje::AbrirModpack(i) => {
            app.modpack_pagina = Some(i);
            app.modpack_pestana = 0;
            app.modpack_galeria = 0;
            arrancar(app);
        }
        Mensaje::CerrarModpack => {
            app.modpack_pagina = None;
            arrancar(app);
        }
        Mensaje::PestanaModpack(i) => {
            app.modpack_pestana = i;
            arrancar(app);
        }
        Mensaje::GaleriaModpack(i) => app.modpack_galeria = i,
        Mensaje::InstalarModpack(i) => {
            let m = &modpacks::CATALOGO[i];
            if instancias().iter().any(|d| d.nombre == m.nombre) {
                return Task::none();
            }
            lista_instancias().lock().unwrap().push(DatosInstancia {
                nombre: m.nombre.into(),
                version: format!("{} · {}", m.minecraft, m.cargador),
                ultima_vez: "Nueva".into(),
                mods: format!("{} mods", m.mods),
                portada: m.portada,
                estado: EstadoInstancia::Instalando(0.0),
                seleccionada: false,
                detalle: format!("Descargando mods · 0 de {}", m.mods),
            });
            app.grupos.push(None);
            app.configs.push(config_por_defecto());
            app.instalando.push((m.nombre.to_owned(), std::time::Instant::now()));
            app.ahora = std::time::Instant::now();
            app.aviso = Some(format!("Instalando {} como una instancia nueva.", m.nombre));
        }
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

fn instancias_base() -> Vec<DatosInstancia> {
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

/// Lista viva de instancias: arranca con las de ejemplo y crece al instalar modpacks.
fn lista_instancias() -> &'static std::sync::Mutex<Vec<DatosInstancia>> {
    static LISTA: std::sync::OnceLock<std::sync::Mutex<Vec<DatosInstancia>>> = std::sync::OnceLock::new();
    LISTA.get_or_init(|| std::sync::Mutex::new(instancias_base()))
}

fn instancias() -> Vec<DatosInstancia> {
    lista_instancias().lock().unwrap().clone()
}

fn config_por_defecto() -> ConfigJava {
    ConfigJava {
        java: None,
        memoria_min: 2048,
        memoria_max: 6144,
        metaspace: 512,
        argumentos: "-XX:+UseG1GC".into(),
    }
}

/// Hace avanzar las instalaciones de modpacks en curso (5 s de ejemplo).
fn avanzar_instalaciones(app: &mut App) {
    let ahora = app.ahora;
    let mut terminadas = Vec::new();
    {
        let mut lista = lista_instancias().lock().unwrap();
        for (nombre, inicio) in &app.instalando {
            let avance = (ahora.saturating_duration_since(*inicio).as_secs_f32() / 5.0).min(1.0);
            if let Some(d) = lista.iter_mut().find(|d| &d.nombre == nombre) {
                let total = modpacks::CATALOGO.iter().find(|m| m.nombre == nombre).map_or(0, |m| m.mods);
                if avance >= 1.0 {
                    d.estado = EstadoInstancia::Lista;
                    d.detalle.clear();
                    d.ultima_vez = "Ahora".into();
                    terminadas.push(nombre.clone());
                } else {
                    d.estado = EstadoInstancia::Instalando(avance);
                    d.detalle = format!("Descargando mods · {} de {total}", (avance * total as f32) as u32);
                }
            }
        }
    }
    if let Some(n) = terminadas.last() {
        app.aviso = Some(format!("{n} quedó instalado."));
    }
    app.instalando.retain(|(n, _)| !terminadas.contains(n));
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

/// Pestaña Mods del editor: lista de los mods de la instancia y descarga de
/// nuevos; al pulsar el nombre de un mod se abre su página de detalle.
fn pestana_mods(app: &App) -> Element<'_, Mensaje> {
    match &app.mod_pagina {
        Some(id) => pagina_mod(app, id),
        None => lista_mods(app),
    }
}

fn lista_mods(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    let instancia = instancias()[app.editor_inst].nombre.clone();
    let instalados = mods::listar(&instancia);
    let consulta = app.busqueda_mods.trim().to_lowercase();
    let coincide = |texto: &str| consulta.is_empty() || texto.to_lowercase().contains(&consulta);

    let barra = row![
        componentes::segmentado(p, &["Instalados", "Descargar"], app.mods_vista, Mensaje::VistaMods),
        container(componentes::campo(
            p,
            if app.mods_vista == 0 { "Buscar entre los instalados" } else { "Buscar en el catálogo" },
            &app.busqueda_mods,
            Mensaje::BuscarMods,
        ))
        .width(340),
        Space::with_width(Length::Fill),
        text(format!(
            "{} de {} activos",
            instalados.iter().filter(|m| m.activo).count(),
            instalados.len()
        ))
        .font(fuentes::MONO)
        .size(texto::MONO_SM.0)
        .color(p.text_muted),
    ]
    .spacing(espacio::S4)
    .align_y(Alignment::Center);

    let mut lista = column![].spacing(espacio::S2);
    if app.mods_vista == 0 {
        for m in instalados.iter().filter(|m| coincide(&m.archivo)) {
            let titulo = mods::en_catalogo(&m.id).map_or_else(|| m.id.clone(), |c| c.titulo.to_owned());
            let nombre = iced::widget::button(
                container(
                    row![
                        text(titulo)
                            .size(texto::BODY.0)
                            .color(if m.activo { p.text } else { p.text_muted })
                            .width(Length::Fill),
                        text(m.version.clone()).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted),
                        icono(Icono::ChevronDerecha, Tam::Base, p.text_muted),
                    ]
                    .spacing(espacio::S4)
                    .align_y(Alignment::Center),
                )
                .center_y(Length::Fill),
            )
            .width(Length::Fill)
            .height(52)
            .padding(Padding::from([0.0, espacio::S4]))
            .on_press(Mensaje::AbrirModPagina(m.id.clone()))
            .style(icaro_ui::estilo::boton_fantasma(p));
            let estado = if m.activo { icaro_ui::componentes::Estado::Exito } else { icaro_ui::componentes::Estado::Info };
            lista = lista.push(
                container(
                    row![
                        nombre,
                        icaro_ui::componentes::insignia(p, estado, if m.activo { "Activo" } else { "Desactivado" }),
                        componentes::interruptor(p, m.activo, Mensaje::AlternarMod(m.archivo.clone())),
                    ]
                    .spacing(espacio::S4)
                    .padding(Padding::from([0.0, espacio::S4]))
                    .align_y(Alignment::Center),
                )
                .style(icaro_ui::estilo::tarjeta(p, false)),
            );
        }
        if instalados.is_empty() {
            lista = lista.push(
                text("Esta instancia no tiene nada instalado aquí. Ve a Descargar para agregar algo.")
                    .size(texto::BODY.0)
                    .color(p.text_muted),
            );
        }
    } else {
        for c in mods::catalogo_actual().iter().filter(|c| coincide(c.titulo) || coincide(c.descripcion)) {
            let puesto = instalados.iter().find(|m| m.id == c.id);
            let accion: Element<Mensaje> = match puesto {
                Some(m) => icaro_ui::componentes::insignia(p, icaro_ui::componentes::Estado::Exito, &format!("Instalado {}", m.version)),
                None => componentes::boton(
                    p,
                    &format!("Instalar {}", c.lanzamientos[0].version),
                    Variante::Primario,
                    Some(Mensaje::InstalarMod(c.id.to_owned(), c.lanzamientos[0].version.to_owned())),
                ),
            };
            let titulo = iced::widget::button(
                text(c.titulo).font(fuentes::TITULO).size(texto::BODY.0 + 1.0).color(p.text),
            )
            .padding(0)
            .on_press(Mensaje::AbrirModPagina(c.id.to_owned()))
            .style(icaro_ui::estilo::sin_estilo(p.text));
            lista = lista.push(
                container(
                    row![
                        column![
                            titulo,
                            text(c.descripcion).size(texto::BODY_SM.0).color(p.text_muted),
                            text(format!("{} · {} descargas · {}", c.autor, c.descargas, c.cargadores.join(", ")))
                                .font(fuentes::MONO)
                                .size(texto::MONO_SM.0)
                                .color(p.text_muted),
                        ]
                        .spacing(espacio::S1)
                        .width(Length::Fill),
                        accion,
                    ]
                    .spacing(espacio::S4)
                    .align_y(Alignment::Center),
                )
                .padding(espacio::S4)
                .width(Length::Fill)
                .style(icaro_ui::estilo::tarjeta(p, false)),
            );
        }
    }
    column![barra, lista]
        .spacing(espacio::S6)
        .padding(Padding::from([espacio::S6, espacio::S12]))
        .into()
}

/// Días desde 1970 de una fecha civil (algoritmo de Howard Hinnant).
fn dias_civil(a: i64, m: i64, d: i64) -> i64 {
    let a = if m <= 2 { a - 1 } else { a };
    let era = a.div_euclid(400);
    let aoe = a - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = aoe * 365 + aoe / 4 - aoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// "hace 3 meses" a partir de una fecha `AAAA-MM-DD`.
fn hace(fecha: &str) -> String {
    let partes: Vec<i64> = fecha.split('-').filter_map(|v| v.parse().ok()).collect();
    let [a, m, d] = partes[..] else {
        return fecha.to_owned();
    };
    let hoy = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |t| (t.as_secs() / 86_400) as i64);
    let dias = (hoy - dias_civil(a, m, d)).max(0);
    let (n, unidad) = if dias < 1 {
        return "hoy".into();
    } else if dias < 30 {
        (dias, "día")
    } else if dias < 365 {
        (dias / 30, "mes")
    } else {
        (dias / 365, "año")
    };
    let plural = match (n, unidad) {
        (1, u) => u.to_owned(),
        (_, "mes") => "meses".to_owned(),
        (_, u) => format!("{u}s"),
    };
    format!("hace {n} {plural}")
}

/// Etiqueta pequeña con contorno, como las de compatibilidad y etiquetas.
fn chip_dato<'a>(p: icaro_ui::tema::Paleta, texto: &str) -> Element<'a, Mensaje> {
    container(text(texto.to_owned()).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text))
        .padding(Padding::from([espacio::S1, espacio::S2]))
        .style(icaro_ui::estilo::marco(p))
        .into()
}

/// Chips en filas de hasta tres, porque iced no parte las filas solo.
fn chips_en_filas<'a>(p: icaro_ui::tema::Paleta, items: &[String]) -> Element<'a, Mensaje> {
    let mut col = column![].spacing(espacio::S2);
    for grupo in items.chunks(3) {
        let mut fila = row![].spacing(espacio::S2);
        for i in grupo {
            fila = fila.push(chip_dato(p, i));
        }
        col = col.push(fila);
    }
    col.into()
}

/// Tarjeta de la columna lateral, con título opcional.
fn tarjeta_lateral<'a>(
    p: icaro_ui::tema::Paleta,
    titulo: Option<&str>,
    cuerpo: Element<'a, Mensaje>,
) -> Element<'a, Mensaje> {
    let mut col = column![].spacing(espacio::S3);
    if let Some(t) = titulo {
        col = col.push(text(t.to_owned()).font(fuentes::TITULO).size(texto::BODY.0 + 2.0).color(p.text));
    }
    container(col.push(cuerpo))
        .padding(espacio::S4)
        .width(Length::Fill)
        .style(icaro_ui::estilo::tarjeta(p, false))
        .into()
}

/// Fila de la tarjeta Enlaces: texto con el icono de enlace externo.
fn enlace_externo<'a>(p: icaro_ui::tema::Paleta, texto: &str) -> Element<'a, Mensaje> {
    iced::widget::button(
        row![
            text(texto.to_owned()).size(texto::BODY.0).color(p.text).width(Length::Fill),
            icono(Icono::Externo, Tam::Base, p.text_muted),
        ]
        .spacing(espacio::S2)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(Padding::from([espacio::S1, 0.0]))
    .on_press(Mensaje::Nada)
    .style(icaro_ui::estilo::sin_estilo(p.text))
    .into()
}

/// Fila de la tarjeta Detalles: icono y texto.
fn fila_detalle<'a>(p: icaro_ui::tema::Paleta, glifo: Icono, texto: String) -> Element<'a, Mensaje> {
    row![
        icono(glifo, Tam::Base, p.text_muted),
        text(texto).size(texto::BODY.0).color(p.text),
    ]
    .spacing(espacio::S3)
    .align_y(Alignment::Center)
    .into()
}

/// Tarjetas Enlaces, Etiquetas, Creadores y Detalles, comunes a mods y modpacks.
#[allow(clippy::too_many_arguments)]
fn tarjetas_informacion<'a>(
    p: icaro_ui::tema::Paleta,
    etiquetas: &[String],
    creador: (&str, &str, String, Vec<&str>),
    licencia: &str,
    publicado: &str,
    actualizado: &str,
    extra: Vec<(Icono, String)>,
) -> Vec<Element<'a, Mensaje>> {
    use icaro_ui::componentes::etiqueta;
    let enlaces = column![
        enlace_externo(p, "Reportar problemas"),
        enlace_externo(p, "Ver código fuente"),
        enlace_externo(p, "Unirse al servidor de Discord"),
        container(Space::new(Length::Fill, 1.0)).style(icaro_ui::estilo::bloque(p.border)),
        enlace_externo(p, "Donar"),
    ]
    .spacing(espacio::S1);
    let (nombre, rol, resumen, top) = creador;
    let mut ficha = column![
        row![
            container(icono(Icono::Instancias, Tam::Base, p.text))
                .padding(espacio::S2)
                .style(icaro_ui::estilo::marco(p)),
            column![
                text(nombre.to_owned()).font(fuentes::TITULO).size(texto::BODY.0 + 1.0).color(p.text),
                text(rol.to_owned()).size(texto::BODY_SM.0).color(p.text_muted),
            ]
            .spacing(espacio::S1),
        ]
        .spacing(espacio::S3)
        .align_y(Alignment::Center),
        text(resumen).size(texto::BODY_SM.0).color(p.text_muted),
    ]
    .spacing(espacio::S2);
    if !top.is_empty() {
        ficha = ficha.push(etiqueta(p, "Más descargados"));
        for t in top {
            ficha = ficha.push(text(t.to_owned()).size(texto::BODY_SM.0).color(p.text));
        }
    }
    ficha = ficha.push(componentes::boton(p, "Ver perfil", Variante::Secundario, Some(Mensaje::Nada)));
    let mut detalles = column![
        fila_detalle(p, Icono::Copiar, format!("Licencia {licencia}")),
        fila_detalle(p, Icono::Check, format!("Publicado {}", hace(publicado))),
        fila_detalle(p, Icono::Actualizar, format!("Actualizado {}", hace(actualizado))),
    ]
    .spacing(espacio::S3);
    for (g, t) in extra {
        detalles = detalles.push(fila_detalle(p, g, t));
    }
    vec![
        tarjeta_lateral(p, Some("Enlaces"), enlaces.into()),
        tarjeta_lateral(p, Some("Etiquetas"), chips_en_filas(p, etiquetas)),
        tarjeta_lateral(p, Some("Creadores"), ficha.into()),
        tarjeta_lateral(p, Some("Detalles"), detalles.into()),
    ]
}

/// Índice de la versión que sirve para la instancia: la más nueva con su
/// versión del juego y su cargador (y estable, salvo que se pidan pruebas).
fn para_instancia(c: &mods::ModCatalogo, mc: &str, cargador: &str, pruebas: bool) -> Option<usize> {
    c.lanzamientos.iter().position(|l| {
        l.minecraft == mc
            && (mods::es_recurso() || c.cargadores_de(l).iter().any(|x| x.eq_ignore_ascii_case(cargador)))
            && (pruebas || l.canal == "release")
    })
}

/// Página completa de un mod: encabezado, pestañas y columna lateral fija con
/// la acción principal, la compatibilidad, los datos y el creador.
fn pagina_mod<'a>(app: &'a App, id: &str) -> Element<'a, Mensaje> {
    use icaro_ui::componentes::{casilla, etiqueta, insignia, Estado};
    let p = app.modo.paleta();
    let datos = &instancias()[app.editor_inst];
    let instancia = datos.nombre.clone();
    let (mc_inst, cargador_inst) = {
        let mut partes = datos.version.split(" · ");
        (partes.next().unwrap_or("").to_owned(), partes.next().unwrap_or("").to_owned())
    };
    let todos = mods::listar(&instancia);
    let instalado = todos.iter().find(|m| m.id == id).cloned();
    let catalogo = mods::en_catalogo(id);
    let extras = mods::extras(id);
    let titulo = catalogo.map_or_else(|| id.to_owned(), |c| c.titulo.to_owned());

    let encabezado = column![
        row![componentes::boton(p, if mods::es_recurso() { "Volver a los resourcepacks" } else { "Volver a los mods" }, Variante::Fantasma, Some(Mensaje::CerrarModPagina))],
        text(titulo.to_uppercase()).font(fuentes::DISPLAY).size(texto::DISPLAY.0).color(p.text),
        text(catalogo.map_or("Mod sin ficha en el catálogo.", |c| c.descripcion))
            .size(texto::BODY.0)
            .color(p.text_muted),
    ]
    .spacing(espacio::S2);

    // Las pestañas dependen del mod: sin imágenes no hay Galería.
    let mut nombres = vec!["Descripción"];
    if !extras.galeria.is_empty() {
        nombres.push("Galería");
    }
    nombres.extend(["Versiones", "Cambios"]);
    if !mods::es_recurso() {
        nombres.push("Dependencias");
    }
    let activa = app.mod_pestana.min(nombres.len() - 1);
    let barra = componentes::pestanas(
        p,
        &nombres.iter().map(|n| (*n, None)).collect::<Vec<_>>(),
        activa,
        Mensaje::PestanaMod,
    );

    let idx_para = catalogo.and_then(|c| para_instancia(c, &mc_inst, &cargador_inst, app.mod_betas));
    let idx_instalado = catalogo.zip(instalado.as_ref()).and_then(|(c, m)| {
        c.lanzamientos.iter().position(|l| l.version == m.version)
    });

    let contenido: Element<Mensaje> = match (catalogo, nombres[activa]) {
        (None, _) => text("Este mod no está en el catálogo, así que no hay más datos que mostrar.")
            .size(texto::BODY.0)
            .color(p.text_muted)
            .into(),
        (Some(c), "Descripción") => {
            let mut cat = row![].spacing(espacio::S2);
            for k in c.categorias {
                cat = cat.push(insignia(p, Estado::Info, k));
            }
            column![
                text(c.detalle).size(texto::BODY.0).color(p.text),
                text(if mods::es_recurso() {
                    format!("Resolución {} para Minecraft {}.", c.cargadores.join(", "), c.minecraft().join(", "))
                } else {
                    format!("Funciona con {} en {}.", c.cargadores.join(", "), c.minecraft().join(", "))
                })
                    .size(texto::BODY.0)
                    .color(p.text_muted),
                cat,
            ]
            .spacing(espacio::S4)
            .into()
        }
        (Some(_), "Galería") => {
            let laminas = [
                Lamina::TivoliCiudad,
                Lamina::ValleRocas,
                Lamina::ProdigoAldea,
                Lamina::CastilloTorres,
                Lamina::MelencoliaReloj,
                Lamina::FaetonPaisaje,
            ];
            let sel = app.mod_galeria.min(extras.galeria.len() - 1);
            let principal = column![
                container(icaro_ui::laminas::grabado(p, laminas[sel % laminas.len()], Length::Fill, 300))
                    .style(icaro_ui::estilo::marco(p)),
                text(extras.galeria[sel]).size(texto::BODY.0).color(p.text),
            ]
            .spacing(espacio::S2);
            let mut miniaturas = row![].spacing(espacio::S4);
            for (i, titulo) in extras.galeria.iter().enumerate().take(3) {
                miniaturas = miniaturas.push(
                    iced::widget::button(
                        column![
                            container(icaro_ui::laminas::grabado(p, laminas[i % laminas.len()], Length::Fill, 110))
                                .style(icaro_ui::estilo::marco(p)),
                            text(*titulo).size(texto::BODY_SM.0).color(if i == sel { p.text } else { p.text_muted }),
                        ]
                        .spacing(espacio::S1),
                    )
                    .width(Length::FillPortion(1))
                    .padding(0)
                    .on_press(Mensaje::GaleriaMod(i))
                    .style(icaro_ui::estilo::sin_estilo(p.text)),
                );
            }
            column![
                principal,
                miniaturas,
                text(format!("Imágenes de ejemplo; las capturas reales llegarán con la conexión al catálogo."))
                    .size(texto::BODY_SM.0)
                    .color(p.text_muted),
            ]
            .spacing(espacio::S4)
            .into()
        }
        (Some(c), "Versiones") => {
            let mut mc_opciones = vec!["Todas".to_owned()];
            mc_opciones.extend(c.minecraft().iter().map(|v| (*v).to_owned()));
            let mut loaders = vec!["Todos".to_owned()];
            for l in c.lanzamientos {
                for x in c.cargadores_de(l) {
                    if !loaders.iter().any(|y| y == x) {
                        loaders.push((*x).to_owned());
                    }
                }
            }
            let filtros = row![
                container(componentes::selector(p, mc_opciones, Some(app.mod_filtro_mc.clone()), Mensaje::FiltroMcMod)).width(160),
                container(componentes::selector(p, loaders, Some(app.mod_filtro_loader.clone()), Mensaje::FiltroLoaderMod)).width(160),
                casilla(p, "Mostrar betas y alphas", app.mod_betas, |_| Mensaje::BetasMod),
            ]
            .spacing(espacio::S4)
            .align_y(Alignment::Center);
            let mut tabla = column![container(
                row![
                    container(etiqueta(p, "Versión")).width(Length::FillPortion(2)),
                    container(etiqueta(p, "Juego")).width(Length::FillPortion(2)),
                    container(etiqueta(p, if mods::es_recurso() { "Resolución" } else { "Cargadores" })).width(Length::FillPortion(3)),
                    container(etiqueta(p, "Fecha")).width(Length::FillPortion(3)),
                    Space::with_width(Length::FillPortion(8)),
                ]
            )
            .padding(Padding::from([espacio::S2, espacio::S3]))]
            .spacing(espacio::S1);
            let mut filas = 0;
            for (i, l) in c.lanzamientos.iter().enumerate() {
                let cargadores = c.cargadores_de(l);
                if (app.mod_filtro_mc != "Todas" && l.minecraft != app.mod_filtro_mc)
                    || (app.mod_filtro_loader != "Todos" && !cargadores.contains(&app.mod_filtro_loader.as_str()))
                    || (!app.mod_betas && l.canal != "release")
                {
                    continue;
                }
                filas += 1;
                let es_para = idx_para == Some(i);
                let actual = idx_instalado == Some(i);
                let mut derecha = row![].spacing(espacio::S2).align_y(Alignment::Center);
                if actual {
                    derecha = derecha.push(insignia(p, Estado::Exito, "Instalada"));
                }
                if es_para {
                    derecha = derecha.push(insignia(p, Estado::Info, "Para tu instancia"));
                    if !actual {
                        let (texto, msg) = match &instalado {
                            Some(m) => ("Usar", Mensaje::VersionMod(m.archivo.clone(), l.version.to_owned())),
                            None => ("Agregar", Mensaje::InstalarMod(c.id.to_owned(), l.version.to_owned())),
                        };
                        derecha = derecha.push(componentes::boton(p, texto, Variante::Primario, Some(msg)));
                    }
                }
                let mut version = row![text(l.version).font(fuentes::MONO).size(texto::MONO.0).color(p.text)]
                    .spacing(espacio::S2)
                    .align_y(Alignment::Center);
                if l.canal != "release" {
                    version = version.push(insignia(p, Estado::Aviso, l.canal));
                }
                tabla = tabla.push(
                    container(
                        row![
                            container(version).width(Length::FillPortion(2)),
                            text(l.minecraft).font(fuentes::MONO).size(texto::MONO.0).color(p.text).width(Length::FillPortion(2)),
                            text(cargadores.join(", ")).size(texto::BODY_SM.0).color(p.text_muted).width(Length::FillPortion(3)),
                            text(l.fecha).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted).width(Length::FillPortion(3)),
                            container(derecha).width(Length::FillPortion(8)),
                        ]
                        .align_y(Alignment::Center),
                    )
                    .padding(Padding::from([espacio::S2, espacio::S3]))
                    .style(icaro_ui::estilo::tarjeta(p, es_para)),
                );
            }
            if filas == 0 {
                tabla = tabla.push(
                    text("Ninguna versión coincide con los filtros.").size(texto::BODY.0).color(p.text_muted),
                );
            }
            column![filtros, tabla].spacing(espacio::S4).into()
        }
        (Some(c), "Cambios") => {
            let mut lista = column![].spacing(espacio::S5);
            for (i, l) in c.lanzamientos.iter().enumerate() {
                let mut cabeza = row![
                    text(format!("{} · {}", l.version, l.fecha)).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text),
                ]
                .spacing(espacio::S2)
                .align_y(Alignment::Center);
                if l.canal != "release" {
                    cabeza = cabeza.push(insignia(p, Estado::Aviso, l.canal));
                }
                if idx_instalado == Some(i) {
                    cabeza = cabeza.push(insignia(p, Estado::Exito, "Instalada"));
                } else if idx_instalado.is_some_and(|ins| i < ins) {
                    cabeza = cabeza.push(insignia(p, Estado::Info, "Nueva"));
                }
                let mut bloque = column![cabeza].spacing(espacio::S1);
                for linea in l.cambios {
                    bloque = bloque.push(text(format!("— {linea}")).size(texto::BODY.0).color(p.text_muted));
                }
                lista = lista.push(bloque);
            }
            lista.into()
        }
        (Some(c), _) => {
            let nombre_de = |dep: &str| mods::en_catalogo(dep).map_or_else(|| dep.to_owned(), |d| d.titulo.to_owned());
            let mut lista = column![].spacing(espacio::S3);
            lista = lista.push(etiqueta(p, "Obligatorias"));
            if c.dependencias.is_empty() {
                lista = lista.push(text("Este mod no necesita otros.").size(texto::BODY.0).color(p.text_muted));
            }
            for dep in c.dependencias {
                let puesta = todos.iter().any(|m| m.id == *dep);
                lista = lista.push(
                    row![
                        text(nombre_de(dep)).size(texto::BODY.0).color(p.text).width(Length::Fill),
                        if puesta { insignia(p, Estado::Exito, "Instalada") } else { insignia(p, Estado::Aviso, "Se instalará") },
                    ]
                    .align_y(Alignment::Center),
                );
            }
            if !extras.opcionales.is_empty() {
                lista = lista.push(Space::with_height(espacio::S2)).push(etiqueta(p, "Opcionales"));
                for dep in extras.opcionales {
                    let puesta = todos.iter().any(|m| m.id == *dep);
                    let marcada = puesta || app.mod_opcionales.iter().any(|o| o == dep);
                    let id_dep = (*dep).to_owned();
                    lista = lista.push(
                        row![
                            casilla(p, &nombre_de(dep), marcada, move |_| Mensaje::OpcionalMod(id_dep.clone())),
                            Space::with_width(Length::Fill),
                            if puesta { insignia(p, Estado::Exito, "Instalada") } else { Space::with_width(0).into() },
                        ]
                        .align_y(Alignment::Center),
                    );
                }
            }
            if !extras.incompatibles.is_empty() {
                lista = lista.push(Space::with_height(espacio::S2)).push(etiqueta(p, "Incompatibles"));
                for (dep, motivo) in extras.incompatibles {
                    let presente = todos.iter().any(|m| m.id == *dep);
                    lista = lista.push(
                        row![
                            text(nombre_de(dep)).size(texto::BODY.0).color(p.text),
                            text(*motivo).size(texto::BODY_SM.0).color(p.text_muted).width(Length::Fill),
                            if presente { insignia(p, Estado::Error, "Instalado") } else { Space::with_width(0).into() },
                        ]
                        .spacing(espacio::S4)
                        .align_y(Alignment::Center),
                    );
                }
            }
            lista.into()
        }
    };

    // Columna lateral fija, con tarjetas como en Modrinth.
    let mut lateral = column![].spacing(espacio::S3);
    let accion: Element<Mensaje> = match (&instalado, catalogo) {
        (Some(m), _) => column![
            insignia(p, Estado::Exito, &format!("Instalado {}", m.version)),
            row![
                text("Activo").size(texto::BODY.0).color(p.text).width(Length::Fill),
                componentes::interruptor(p, m.activo, Mensaje::AlternarMod(m.archivo.clone())),
            ]
            .align_y(Alignment::Center),
            componentes::boton(p, if mods::es_recurso() { "Eliminar pack" } else { "Eliminar mod" }, Variante::Peligro, Some(Mensaje::EliminarMod(m.archivo.clone()))),
        ]
        .spacing(espacio::S3)
        .into(),
        (None, Some(c)) => {
            let version = c.lanzamientos[idx_para.unwrap_or(0)].version;
            componentes::boton(
                p,
                &format!("Agregar {version}"),
                Variante::Primario,
                Some(Mensaje::InstalarMod(c.id.to_owned(), version.to_owned())),
            )
        }
        _ => Space::with_height(0).into(),
    };
    lateral = lateral.push(tarjeta_lateral(p, None, accion));
    if let Some(c) = catalogo {
        let versiones: Vec<String> = c.minecraft().iter().map(|v| (*v).to_owned()).collect();
        let mut plataformas: Vec<String> = Vec::new();
        for l in c.lanzamientos {
            for x in c.cargadores_de(l) {
                if !plataformas.iter().any(|y| y == x) {
                    plataformas.push((*x).to_owned());
                }
            }
        }
        let compat = column![
            row![
                text(format!("{instancia}: {mc_inst} · {cargador_inst}")).size(texto::BODY_SM.0).color(p.text_muted).width(Length::Fill),
                if idx_para.is_some() { insignia(p, Estado::Exito, "Compatible") } else { insignia(p, Estado::Error, "No compatible") },
            ]
            .spacing(espacio::S2)
            .align_y(Alignment::Center),
            text("Minecraft: Java Edition").size(texto::BODY.0).color(p.text),
            chips_en_filas(p, &versiones),
            text(if mods::es_recurso() { "Resolución" } else { "Plataformas" }).size(texto::BODY.0).color(p.text),
            chips_en_filas(p, &plataformas),
            text("Entornos compatibles").size(texto::BODY.0).color(p.text),
            chip_dato(p, extras.entorno),
        ]
        .spacing(espacio::S2);
        lateral = lateral.push(tarjeta_lateral(p, Some("Compatibilidad"), compat.into()));
        let etiquetas: Vec<String> = c.categorias.iter().map(|k| (*k).to_owned()).collect();
        let resumen = format!(
            "{} · {} proyectos · {} descargas",
            extras.creador_tipo, extras.creador_proyectos, extras.creador_descargas
        );
        for t in tarjetas_informacion(
            p,
            &etiquetas,
            (c.autor, if extras.creador_tipo == "Organización" { "Propietario" } else { "Miembro" }, resumen, extras.creador_top.to_vec()),
            c.licencia,
            extras.publicado,
            extras.actualizado,
            vec![
                (Icono::Descargas, format!("{} descargas", c.descargas)),
                (Icono::Info, format!("{} seguidores", extras.seguidores)),
            ],
        ) {
            lateral = lateral.push(t);
        }
    }

    let principal = desplazable(
        column![barra, contenido]
            .spacing(espacio::S6)
            .padding(Padding { right: espacio::S4, ..Padding::ZERO }),
    );
    let cuerpo = row![
        container(principal).width(Length::Fill).height(Length::Fill),
        container(desplazable(lateral))
            .width(300)
            .height(Length::Fill),
    ]
    .spacing(espacio::S8)
    .height(Length::Fill);
    column![encabezado, cuerpo]
        .spacing(espacio::S6)
        .padding(Padding::from([espacio::S6, espacio::S12]))
        .into()
}

/// Publicaciones de ejemplo de un modpack: versión, fecha y cambios.
fn publicaciones_modpack(m: &modpacks::Modpack) -> [(&'static str, &'static str, [&'static str; 2]); 3] {
    let _ = m;
    [
        ("2.1.0", "2026-09-30", ["Actualiza los mods a su última versión", "Corrige un fallo al iniciar el mundo"]),
        ("2.0.0", "2026-08-12", ["Agrega mods nuevos y quita los que ya no se mantienen", "Ajusta la configuración de rendimiento"]),
        ("1.5.3", "2026-05-20", ["Corrección de errores", "Mejora el tiempo de carga"]),
    ]
}

/// Mods que trae un modpack (muestra tomada del catálogo de mods).
fn mods_incluidos(i: usize) -> Vec<&'static mods::ModCatalogo> {
    let n = mods::CATALOGO.len();
    (0..5).map(|k| &mods::CATALOGO[(i + k) % n]).collect()
}

/// Página de un modpack: mismas vistas que la de un mod, con una pestaña con
/// los mods que incluye en lugar de dependencias.
fn pagina_modpack(app: &App, i: usize) -> Element<'_, Mensaje> {
    use icaro_ui::componentes::{etiqueta, insignia, Estado};
    let p = app.modo.paleta();
    let m = &modpacks::CATALOGO[i];
    let instalado = instancias().iter().any(|d| d.nombre == m.nombre);
    let publicaciones = publicaciones_modpack(m);
    let incluidos = mods_incluidos(i);

    let encabezado = column![
        row![componentes::boton(p, "Volver a los modpacks", Variante::Fantasma, Some(Mensaje::CerrarModpack))],
        text(m.nombre.to_uppercase()).font(fuentes::DISPLAY).size(texto::DISPLAY.0).color(p.text),
        text(m.descripcion).size(texto::BODY.0).color(p.text_muted),
        text(format!("{} descargas · {} mods · {}", m.descargas, m.mods, m.tamano))
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted),
    ]
    .spacing(espacio::S2);

    let nombres = ["Descripción", "Galería", "Versiones", "Cambios", "Mods incluidos"];
    let activa = app.modpack_pestana.min(nombres.len() - 1);
    let barra = componentes::pestanas(
        p,
        &nombres.iter().map(|n| (*n, None)).collect::<Vec<_>>(),
        activa,
        Mensaje::PestanaModpack,
    );

    let contenido: Element<Mensaje> = match nombres[activa] {
        "Descripción" => column![
            text(m.descripcion).size(texto::BODY.0).color(p.text),
            text(format!(
                "Incluye {} mods para Minecraft {} con {}. Al instalarlo se crea una instancia nueva con su propia configuración de Java y memoria.",
                m.mods, m.minecraft, m.cargador
            ))
            .size(texto::BODY.0)
            .color(p.text_muted),
            row![insignia(p, Estado::Info, m.categoria), insignia(p, Estado::Info, "Modpack")].spacing(espacio::S2),
        ]
        .spacing(espacio::S4)
        .into(),
        "Galería" => {
            let laminas = [m.portada, Lamina::TivoliCiudad, Lamina::ValleRocas];
            let titulos = ["Primer vistazo", "Exploración", "Construcciones"];
            let sel = app.modpack_galeria.min(2);
            let mut miniaturas = row![].spacing(espacio::S4);
            for (k, t) in titulos.iter().enumerate() {
                miniaturas = miniaturas.push(
                    iced::widget::button(
                        column![
                            container(icaro_ui::laminas::grabado(p, laminas[k], Length::Fill, 110)).style(icaro_ui::estilo::marco(p)),
                            text(*t).size(texto::BODY_SM.0).color(if k == sel { p.text } else { p.text_muted }),
                        ]
                        .spacing(espacio::S1),
                    )
                    .width(Length::FillPortion(1))
                    .padding(0)
                    .on_press(Mensaje::GaleriaModpack(k))
                    .style(icaro_ui::estilo::sin_estilo(p.text)),
                );
            }
            column![
                container(icaro_ui::laminas::grabado(p, laminas[sel], Length::Fill, 300)).style(icaro_ui::estilo::marco(p)),
                text(titulos[sel]).size(texto::BODY.0).color(p.text),
                miniaturas,
            ]
            .spacing(espacio::S3)
            .into()
        }
        "Versiones" => {
            let mut tabla = column![container(
                row![
                    container(etiqueta(p, "Versión")).width(Length::FillPortion(2)),
                    container(etiqueta(p, "Juego")).width(Length::FillPortion(2)),
                    container(etiqueta(p, "Cargador")).width(Length::FillPortion(2)),
                    container(etiqueta(p, "Fecha")).width(Length::FillPortion(3)),
                    Space::with_width(Length::FillPortion(4)),
                ]
            )
            .padding(Padding::from([espacio::S2, espacio::S3]))]
            .spacing(espacio::S1);
            for (k, (version, fecha, _)) in publicaciones.iter().enumerate() {
                let derecha: Element<Mensaje> = if k == 0 {
                    if instalado {
                        insignia(p, Estado::Exito, "Instalado")
                    } else {
                        componentes::boton(p, "Instalar", Variante::Primario, Some(Mensaje::InstalarModpack(i)))
                    }
                } else {
                    Space::with_width(0).into()
                };
                tabla = tabla.push(
                    container(
                        row![
                            text(*version).font(fuentes::MONO).size(texto::MONO.0).color(p.text).width(Length::FillPortion(2)),
                            text(m.minecraft).font(fuentes::MONO).size(texto::MONO.0).color(p.text).width(Length::FillPortion(2)),
                            text(m.cargador).size(texto::BODY_SM.0).color(p.text_muted).width(Length::FillPortion(2)),
                            text(*fecha).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted).width(Length::FillPortion(3)),
                            container(derecha).width(Length::FillPortion(4)),
                        ]
                        .align_y(Alignment::Center),
                    )
                    .padding(Padding::from([espacio::S2, espacio::S3]))
                    .style(icaro_ui::estilo::tarjeta(p, k == 0)),
                );
            }
            tabla.into()
        }
        "Cambios" => {
            let mut lista = column![].spacing(espacio::S5);
            for (k, (version, fecha, cambios)) in publicaciones.iter().enumerate() {
                let mut cabeza = row![text(format!("{version} · {fecha}")).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text)]
                    .spacing(espacio::S2)
                    .align_y(Alignment::Center);
                if k == 0 {
                    cabeza = cabeza.push(insignia(p, Estado::Info, "Última"));
                }
                let mut bloque = column![cabeza].spacing(espacio::S1);
                for c in cambios {
                    bloque = bloque.push(text(format!("— {c}")).size(texto::BODY.0).color(p.text_muted));
                }
                lista = lista.push(bloque);
            }
            lista.into()
        }
        _ => {
            let mut lista = column![text(format!("Muestra de los {} mods que incluye.", m.mods)).size(texto::BODY.0).color(p.text_muted)]
                .spacing(espacio::S2);
            for c in incluidos {
                lista = lista.push(
                    container(
                        row![
                            column![
                                text(c.titulo).font(fuentes::TITULO).size(texto::BODY.0).color(p.text),
                                text(c.descripcion).size(texto::BODY_SM.0).color(p.text_muted),
                            ]
                            .spacing(espacio::S1)
                            .width(Length::Fill),
                            text(c.lanzamientos.iter().find(|l| l.canal == "release").map_or("", |l| l.version)).font(fuentes::MONO).size(texto::MONO_SM.0).color(p.text_muted),
                        ]
                        .spacing(espacio::S4)
                        .align_y(Alignment::Center),
                    )
                    .padding(espacio::S3)
                    .width(Length::Fill)
                    .style(icaro_ui::estilo::tarjeta(p, false)),
                );
            }
            lista.into()
        }
    };

    let accion: Element<Mensaje> = if instalado {
        insignia(p, Estado::Exito, "Instalado como instancia")
    } else {
        componentes::boton(p, "Instalar como instancia", Variante::Primario, Some(Mensaje::InstalarModpack(i)))
    };
    let compat = column![
        text("Minecraft: Java Edition").size(texto::BODY.0).color(p.text),
        chips_en_filas(p, &[m.minecraft.to_owned()]),
        text("Plataformas").size(texto::BODY.0).color(p.text),
        chips_en_filas(p, &[m.cargador.to_owned()]),
        text("Entornos compatibles").size(texto::BODY.0).color(p.text),
        chip_dato(p, "Cliente y servidor"),
    ]
    .spacing(espacio::S2);
    let mut lateral = column![
        tarjeta_lateral(p, None, accion),
        tarjeta_lateral(p, Some("Compatibilidad"), compat.into()),
    ]
    .spacing(espacio::S3);
    let top: Vec<&str> = modpacks::CATALOGO.iter().filter(|o| o.autor == m.autor).map(|o| o.nombre).take(3).collect();
    for t in tarjetas_informacion(
        p,
        &[m.categoria.to_owned(), "Modpack".to_owned()],
        (m.autor, "Equipo", format!("{} modpacks", top.len().max(1)), top),
        "Código abierto",
        "2025-04-02",
        publicaciones[0].1,
        vec![
            (Icono::Descargas, format!("{} descargas", m.descargas)),
            (Icono::Instancias, format!("{} mods · {}", m.mods, m.tamano)),
        ],
    ) {
        lateral = lateral.push(t);
    }

    let cuerpo = row![
        container(desplazable(column![barra, contenido].spacing(espacio::S6).padding(Padding { right: espacio::S4, ..Padding::ZERO })))
            .width(Length::Fill)
            .height(Length::Fill),
        container(desplazable(lateral)).width(300).height(Length::Fill),
    ]
    .spacing(espacio::S8)
    .height(Length::Fill);
    column![encabezado, cuerpo]
        .spacing(espacio::S6)
        .padding(Padding::from([espacio::S6, espacio::S12]))
        .into()
}

/// Catálogo de modpacks: cada uno se instala como una instancia nueva.
fn pantalla_modpacks(app: &App) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    if let Some(i) = app.modpack_pagina {
        return con_entrada(app, pagina_modpack(app, i));
    }
    let consulta = app.busqueda_modpacks.trim().to_lowercase();
    let categoria = modpacks::CATEGORIAS[app.modpack_filtro.min(modpacks::CATEGORIAS.len() - 1)];
    let existentes: Vec<String> = instancias().into_iter().map(|d| d.nombre).collect();

    let mut chips = row![].spacing(espacio::S2);
    for (i, c) in modpacks::CATEGORIAS.iter().enumerate() {
        chips = chips.push(componentes::chip(p, c, app.modpack_filtro == i, Mensaje::FiltroModpacks(i)));
    }
    let herramientas = row![
        chips,
        Space::with_width(Length::Fill),
        container(componentes::campo(p, "Buscar modpacks", &app.busqueda_modpacks, Mensaje::BuscarModpacks)).width(300),
    ]
    .spacing(espacio::S4)
    .align_y(Alignment::Center);

    let mut tarjetas: Vec<Element<Mensaje>> = Vec::new();
    for (i, m) in modpacks::CATALOGO.iter().enumerate() {
        let coincide = (categoria == "Todos" || m.categoria == categoria)
            && (consulta.is_empty()
                || m.nombre.to_lowercase().contains(&consulta)
                || m.descripcion.to_lowercase().contains(&consulta));
        if !coincide {
            continue;
        }
        let instalado = existentes.iter().any(|n| n == m.nombre);
        let instalando = app.instalando.iter().any(|(n, _)| n == m.nombre);
        let accion = if instalado {
            componentes::boton(
                p,
                if instalando { "Instalando" } else { "Instalado" },
                Variante::Secundario,
                None,
            )
        } else {
            componentes::boton(p, "Instalar", Variante::Primario, Some(Mensaje::InstalarModpack(i)))
        };
        let cuerpo = column![
            iced::widget::button(text(m.nombre).font(fuentes::TITULO).size(texto::HEADING.0).color(p.text))
                .padding(0)
                .on_press(Mensaje::AbrirModpack(i))
                .style(icaro_ui::estilo::sin_estilo(p.text)),
            text(format!("{} · {}", m.minecraft, m.cargador).to_uppercase())
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted),
            text(m.descripcion).size(texto::BODY_SM.0).color(p.text),
            text(format!("{} · {} descargas
{} mods · {}", m.autor, m.descargas, m.mods, m.tamano))
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .color(p.text_muted),
            Space::with_height(Length::Fill),
            row![
                icaro_ui::componentes::insignia(p, icaro_ui::componentes::Estado::Info, m.categoria),
                Space::with_width(Length::Fill),
                accion,
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(espacio::S2)
        .height(Length::Fill);
        tarjetas.push(
            container(
                column![
                    icaro_ui::laminas::grabado(p, m.portada, Length::Fill, 110),
                    container(cuerpo).padding(espacio::S4).height(Length::Fill),
                ],
            )
            .width(icaro_ui::instancias::ANCHO_TARJETA)
            .height(330)
            .style(icaro_ui::estilo::tarjeta(p, false))
            .into(),
        );
    }

    let disponible = app.ventana.width - 232.0 - 2.0 * espacio::S12 + espacio::S6;
    let columnas = ((disponible / (icaro_ui::instancias::ANCHO_TARJETA + espacio::S6)).floor() as usize).max(1);
    let mut rejilla = column![].spacing(espacio::S6);
    let mut it = tarjetas.into_iter();
    let mut hay = false;
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
        hay = true;
        rejilla = rejilla.push(fila);
    }
    if !hay {
        rejilla = rejilla.push(text("Ningún modpack coincide con la búsqueda.").size(texto::BODY.0).color(p.text_muted));
    }
    let cuerpo = column![
        text("Cada modpack se instala como una instancia nueva, con sus mods y su versión del juego.")
            .size(texto::BODY.0)
            .color(p.text_muted),
        herramientas,
        rejilla,
    ]
    .spacing(espacio::S6)
    .padding(Padding::from([espacio::S6, espacio::S12]));
    column![
        banda(p, Lamina::HidraCabezas, "Modpacks", Some(modpacks::CATALOGO.len())),
        suave(app, cuerpo)
    ]
    .into()
}

fn pantalla_editor(app: &App, pestana: usize) -> Element<'_, Mensaje> {
    let p = app.modo.paleta();
    if pestana == 2 || pestana == 3 {
        if let Some(id) = &app.mod_pagina {
            return con_entrada(app, pagina_mod(app, id));
        }
    }
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
        2 => pestana_mods(app),
        3 => pestana_mods(app),
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
        _ => container(Space::new(0, 0)).into(),
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
        con_entrada(
            app,
            container(consola(p, &lineas_consola(), app.filtro_consola, &app.busqueda_consola, MensajesConsola { filtro: Mensaje::FiltroConsola, buscar: Mensaje::BuscarConsola, copiar: Mensaje::Nada, limpiar: Mensaje::Nada }))
                .padding(Padding::from([0.0, espacio::S12]))
                .height(Length::Fill)
                .into(),
        ),
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

fn vista(app: &App) -> Element<'_, Mensaje> {
    mods::fijar_recurso(app.editor == Some(3));
    let p = app.modo.paleta();
    let contenido = match app.seccion {
        Seccion::Instancias => pantalla_instancias(app),
        Seccion::Servidores => pantalla_servidores(app),
        Seccion::Descargas => pantalla_descargas(app),
        Seccion::Ajustes => pantalla_ajustes(app),
        Seccion::Modpacks => pantalla_modpacks(app),
        Seccion::Consola => pantalla_consola(app),
        Seccion::Capturas => pantalla_capturas(app),
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
            if !app.instalando.is_empty()
                || progreso(app.anim, app.ahora, MS_ENTRADA) < 1.0
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
        let guardado = estado::Guardado::cargar();
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
                cerrados: guardado.as_ref().map_or_else(Vec::new, |g| g.cerrados.clone()),
                busqueda_mods: String::new(),
                mods_vista: 0,
                modpack_filtro: 0,
                modpack_pagina: std::env::var("ICARO_PACK").ok().and_then(|v| v.parse().ok()),
                modpack_pestana: std::env::var("ICARO_PACKTAB").ok().and_then(|v| v.parse().ok()).unwrap_or(0),
                modpack_galeria: 0,
                busqueda_modpacks: String::new(),
                instalando: Vec::new(),
                mod_pagina: std::env::var("ICARO_MOD").ok(),
                mod_filtro_mc: "Todas".into(),
                mod_filtro_loader: "Todos".into(),
                mod_betas: false,
                mod_galeria: 0,
                mod_opcionales: Vec::new(),
                mod_pestana: std::env::var("ICARO_MODTAB").ok().and_then(|v| v.parse().ok()).unwrap_or(0),
                ultimo_guardado: String::new(),
                anim: None,
                anim_menu: None,
                ahora: std::time::Instant::now(),
                ultimo_clic: None,
                menu_grupos: false,
                lista_grupos: guardado.as_ref().map_or_else(|| vec!["Con amigos".into(), "Técnicos".into()], |g| g.lista_grupos.clone()),
                edicion_grupo: None,
                grupos: guardado.as_ref().filter(|g| g.grupos.len() >= 5).map_or_else(
                    || vec![Some("Con amigos".into()), Some("Técnicos".into()), Some("Con amigos".into()), None, None],
                    |g| g.grupos[..5].to_vec(),
                ),
                configs: guardado.as_ref().map(|g| g.configs()).filter(|c| c.len() >= 5).map(|c| c[..5].to_vec()).unwrap_or_else(|| {
                    (0..5)
                        .map(|_| ConfigJava {
                            java: None,
                            memoria_min: 2048,
                            memoria_max: 6144,
                            metaspace: 512,
                            argumentos: "-XX:+UseG1GC".into(),
                        })
                        .collect()
                }),
                editor_inst: 0,
                exp: Explorador::abrir("Supervivencia"),
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
