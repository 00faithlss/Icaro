// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Editor de instancia: encabezado con datos clave y el contenido de las
//! pestañas General, Java, Recursos, Mundos y Archivos. La pestaña Mods
//! queda fuera hasta que se ordene su desarrollo.

use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Element, Length, Padding};

use crate::componentes::{
    self, boton, boton_jugar, campo, casilla, etiqueta, insignia, interruptor, migas, pestanas,
    segmentado, selector, senal, Estado, Variante,
};
use crate::estilo;
use crate::fuentes;
use crate::iconos::{icono, Icono, Tam};
use crate::laminas::{banda_con, Lamina};
use crate::tema::{borde, espacio, texto, Paleta};

/// Datos del encabezado del editor.
#[derive(Debug, Clone)]
pub struct DatosEditor {
    pub nombre: String,
    pub version: String,
    pub jugado: String,
    pub mods: String,
    pub tamano: String,
    pub memoria: String,
    pub portada: Lamina,
}

/// Mensajes del encabezado.
#[derive(Debug, Clone)]
pub struct MensajesEncabezado<M> {
    pub volver: M,
    pub abrir_carpeta: M,
    pub mas: M,
    pub jugar: M,
}

/// Pestañas del editor, en su orden. Mods lleva su conteo.
pub fn pestanas_editor<'a, M: Clone + 'a>(
    p: Paleta,
    activa: usize,
    mods: usize,
    mundos: usize,
    al_elegir: impl Fn(usize) -> M,
) -> Element<'a, M> {
    pestanas(
        p,
        &[
            ("General", None),
            ("Java", None),
            ("Mods", Some(mods)),
            ("Recursos", None),
            ("Mundos", Some(mundos)),
            ("Registros", None),
            ("Archivos", None),
        ],
        activa,
        al_elegir,
    )
}

fn dato<'a, M: 'a>(p: Paleta, nombre: &str, valor: &str) -> Element<'a, M> {
    column![
        etiqueta(p, nombre),
        text(valor.to_owned())
            .font(fuentes::MONO)
            .size(texto::MONO.0)
            .color(p.text),
    ]
    .spacing(espacio::S1)
    .into()
}

/// Encabezado: banda con la portada, migas, datos clave y acciones.
pub fn encabezado_editor<'a, M: Clone + 'a>(
    p: Paleta,
    d: &DatosEditor,
    m: MensajesEncabezado<M>,
) -> Element<'a, M> {
    let acciones = row![
        crate::descargas::boton_icono(p, Icono::Carpeta, m.abrir_carpeta),
        crate::descargas::boton_icono(p, Icono::Acciones, m.mas),
        boton_jugar(p, "Jugar", Some(&d.memoria), m.jugar),
    ]
    .spacing(espacio::S2)
    .align_y(Alignment::Center);
    let datos = row![
        dato(p, "Versión", &d.version),
        dato(p, "Jugado", &d.jugado),
        dato(p, "Mods", &d.mods),
        dato(p, "Tamaño", &d.tamano),
    ]
    .spacing(espacio::S8);
    column![
        banda_con(p, d.portada, &d.nombre, None, 150.0, texto::DISPLAY),
        container(
            column![
                migas(p, &["Instancias", &d.nombre], move |_| m.volver.clone()),
                row![
                    container(icono(Icono::Instancias, Tam::Lg, p.text))
                        .padding(espacio::S3)
                        .style(estilo::marco(p)),
                    datos,
                    Space::with_width(Length::Fill),
                    acciones,
                ]
                .spacing(espacio::S6)
                .align_y(Alignment::Center),
            ]
            .spacing(espacio::S4)
        )
        .padding(Padding::from([espacio::S4, espacio::S12])),
    ]
    .into()
}

fn pagina<'a, M: 'a>(contenido: Element<'a, M>) -> Element<'a, M> {
    crate::scroll::desplazable(container(contenido).padding(Padding::from([espacio::S6, espacio::S12])))
}

fn campo_rotulado<'a, M: 'a>(p: Paleta, nombre: &str, control: Element<'a, M>) -> Element<'a, M> {
    column![etiqueta(p, nombre), control]
        .spacing(espacio::S2)
        .width(Length::Fill)
        .into()
}

/// Estado de la pestaña General.
#[derive(Debug, Clone)]
pub struct EstadoGeneral {
    pub nombre: String,
    pub grupo: Option<String>,
    /// Grupos disponibles, con "Sin grupo" al final.
    pub grupos: Vec<String>,
    pub ancho: String,
    pub alto: String,
    pub pantalla_completa: bool,
    pub servidor: Option<&'static str>,
}

/// Mensajes de la pestaña General.
pub struct MensajesGeneral<M> {
    pub nombre: fn(String) -> M,
    pub grupo: fn(String) -> M,
    pub ancho: fn(String) -> M,
    pub alto: fn(String) -> M,
    pub pantalla_completa: fn(bool) -> M,
    pub servidor: fn(&'static str) -> M,
}

/// Pestaña General: nombre, grupo, ventana y servidor.
pub fn pestana_general<'a, M: Clone + 'a>(
    p: Paleta,
    e: &EstadoGeneral,
    m: &MensajesGeneral<M>,
) -> Element<'a, M> {
    let ventana = row![
        container(campo(p, "Ancho", &e.ancho, m.ancho)).width(100),
        text("×").color(p.text_muted),
        container(campo(p, "Alto", &e.alto, m.alto)).width(100),
        casilla(p, "Pantalla completa", e.pantalla_completa, m.pantalla_completa),
    ]
    .spacing(espacio::S3)
    .align_y(Alignment::Center);
    let rejilla = column![
        row![
            campo_rotulado(p, "Nombre", campo(p, "Nombre", &e.nombre, m.nombre)),
            campo_rotulado(
                p,
                "Grupo",
                selector(
                    p,
                    e.grupos.clone(),
                    Some(e.grupo.clone().unwrap_or_else(|| "Sin grupo".to_owned())),
                    m.grupo
                )
            ),
        ]
        .spacing(espacio::S8),
        row![
            campo_rotulado(p, "Ventana del juego", ventana.into()),
            campo_rotulado(
                p,
                "Al jugar, unirse a",
                selector(
                    p,
                    vec!["Amigos · 10.147.17.1", "Ninguno"],
                    e.servidor,
                    m.servidor
                )
            ),
        ]
        .spacing(espacio::S8),
    ]
    .spacing(espacio::S6);
    pagina(rejilla.into())
}

/// Instalación de Java detectada.
#[derive(Debug, Clone)]
pub struct InstalacionJava {
    pub version: String,
    pub proveedor: String,
    pub gestionada: bool,
    pub ruta: String,
    pub usado_por: String,
    pub existe: bool,
}

/// Tabla de instalaciones de Java detectadas y sus acciones.
fn tabla_instalaciones<'a, M: Clone + 'a>(
    p: Paleta,
    instalaciones: &[InstalacionJava],
) -> Element<'a, M> {
    let mut tabla = column![container(
        row![
            columna_titulo(p, "Java", 1),
            columna_titulo(p, "Proveedor", 2),
            columna_titulo(p, "Ruta", 3),
            columna_titulo(p, "Usado por", 2),
            Space::with_width(90),
        ]
    )
    .padding(Padding::from([espacio::S2, 0.0]))];
    for i in instalaciones {
        let tono = if i.existe { p.text } else { p.error };
        let tenue = if i.existe { p.text_muted } else { p.error };
        let mut proveedor = row![text(i.proveedor.clone()).size(texto::BODY.0).color(tono)]
            .spacing(espacio::S2)
            .align_y(Alignment::Center);
        if i.gestionada {
            proveedor = proveedor.push(insignia(p, Estado::Info, "Ícaro"));
        }
        let accion = if i.existe {
            boton(p, "Probar", Variante::Secundario, None::<M>)
        } else {
            boton(p, "Quitar", Variante::Peligro, None::<M>)
        };
        tabla = tabla
            .push(crate::descargas::regla(borde::FINO, p.border))
            .push(
                container(
                    row![
                        text(i.version.clone())
                            .font(fuentes::MONO)
                            .size(texto::MONO.0)
                            .color(tono)
                            .width(Length::FillPortion(1)),
                        container(proveedor).width(Length::FillPortion(2)),
                        text(i.ruta.clone())
                            .font(fuentes::MONO)
                            .size(texto::MONO_SM.0)
                            .color(tenue)
                            .width(Length::FillPortion(3)),
                        text(i.usado_por.clone())
                            .size(texto::BODY_SM.0)
                            .color(tenue)
                            .width(Length::FillPortion(2)),
                        container(accion).width(90),
                    ]
                    .align_y(Alignment::Center),
                )
                .padding(Padding::from([espacio::S2, 0.0])),
            );
    }
    column![
        tabla,
        row![
                boton(p, "Descargar Java", Variante::Secundario, None::<M>),
                boton(p, "Detectar instalaciones", Variante::Secundario, None::<M>),
                boton(p, "Agregar ruta", Variante::Fantasma, None::<M>),
            ]
        .spacing(espacio::S2),
    ]
    .spacing(espacio::S4)
    .into()
}

fn columna_titulo<'a, M: 'a>(p: Paleta, nombre: &str, partes: u16) -> Element<'a, M> {
    container(etiqueta(p, nombre))
        .width(Length::FillPortion(partes))
        .into()
}

/// Paquete de recursos o shader.
#[derive(Debug, Clone)]
pub struct Paquete {
    pub nombre: String,
    pub autor: String,
    pub resolucion: String,
    pub activo: bool,
    pub compat: CompatPaquete,
    /// Fijo al pie, sin asa (paquetes del juego).
    pub fijo: bool,
}

/// Compatibilidad de un paquete con la versión de la instancia.
#[derive(Debug, Clone, PartialEq)]
pub enum CompatPaquete {
    Compatible,
    /// Hecho para otra versión; el juego lo carga pero puede verse mal.
    HechoPara(String),
    Incompatible,
}

/// Pestaña Recursos: paquetes ordenados por prioridad.
pub fn pestana_recursos<'a, M: Clone + 'a>(
    p: Paleta,
    vista: usize,
    al_elegir_vista: impl Fn(usize) -> M + 'a,
    paquetes: &[Paquete],
    alternar: impl Fn(usize) -> M,
) -> Element<'a, M> {
    let mut lista = column![crate::descargas::regla(borde::FINO, p.border)];
    for (n, q) in paquetes.iter().enumerate() {
        let (estado, palabra) = match &q.compat {
            CompatPaquete::Compatible => (Estado::Exito, "Compatible".to_owned()),
            CompatPaquete::HechoPara(v) => (Estado::Aviso, format!("Hecho para {v}")),
            CompatPaquete::Incompatible => (Estado::Error, "Incompatible".to_owned()),
        };
        let asa: Element<M> = if q.fijo {
            Space::with_width(Tam::Base.px()).into()
        } else {
            icono(Icono::Arrastrar, Tam::Base, p.text_muted).into()
        };
        lista = lista
            .push(
                container(
                    row![
                        asa,
                        interruptor(p, q.activo, alternar(n)),
                        column![
                            text(q.nombre.clone())
                                .font(fuentes::TITULO)
                                .size(texto::BODY.0)
                                .color(p.text),
                            text(q.autor.clone())
                                .size(texto::BODY_SM.0)
                                .color(p.text_muted),
                        ]
                        .width(Length::Fill),
                        text(q.resolucion.clone())
                            .font(fuentes::MONO)
                            .size(texto::MONO_SM.0)
                            .color(p.text_muted),
                        insignia(p, estado, &palabra),
                    ]
                    .spacing(espacio::S4)
                    .align_y(Alignment::Center),
                )
                .padding(Padding::from([espacio::S3, 0.0])),
            )
            .push(crate::descargas::regla(borde::FINO, p.border));
    }
    pagina(
        column![
            row![
                segmentado(p, &["Recursos", "Shaders"], vista, al_elegir_vista),
                Space::with_width(Length::Fill),
                text("Arriba tiene prioridad")
                    .size(texto::BODY_SM.0)
                    .color(p.text_muted),
            ]
            .align_y(Alignment::Center),
            lista,
        ]
        .spacing(espacio::S4)
        .into(),
    )
}

/// Mundo guardado.
#[derive(Debug, Clone)]
pub struct Mundo {
    pub nombre: String,
    pub modo: String,
    pub hardcore: bool,
    pub ultima_vez: String,
    pub tamano: String,
    pub copias: usize,
}

/// Copia de seguridad de un mundo.
#[derive(Debug, Clone)]
pub struct Copia {
    pub fecha: String,
    pub tamano: String,
    pub motivo: String,
}

/// Pestaña Mundos: lista a la izquierda y copias del mundo elegido.
pub fn pestana_mundos<'a, M: Clone + 'a>(
    p: Paleta,
    mundos: &[Mundo],
    elegido: usize,
    al_elegir: impl Fn(usize) -> M + 'a,
    copias: &[Copia],
) -> Element<'a, M> {
    let mut lista = column![].spacing(espacio::S3).width(Length::FillPortion(3));
    for (i, w) in mundos.iter().enumerate() {
        let modo = if w.hardcore {
            text(w.modo.to_uppercase())
                .font(fuentes::ETIQUETA)
                .size(texto::LABEL.0)
                .color(p.error)
        } else {
            text(w.modo.to_uppercase())
                .font(fuentes::ETIQUETA)
                .size(texto::LABEL.0)
                .color(p.text_muted)
        };
        let contenido = row![
            icono(Icono::Instancias, Tam::Md, p.text),
            column![
                text(w.nombre.clone())
                    .font(fuentes::TITULO)
                    .size(texto::HEADING.0)
                    .color(p.text),
                row![
                    modo,
                    text(format!("{} · {}", w.ultima_vez, w.tamano))
                        .size(texto::BODY_SM.0)
                        .color(p.text_muted),
                ]
                .spacing(espacio::S3),
            ]
            .width(Length::Fill)
            .spacing(espacio::S1),
            text(if w.copias == 0 {
                "Sin copias".to_owned()
            } else {
                format!("{} copias", w.copias)
            })
            .font(fuentes::MONO)
            .size(texto::MONO_SM.0)
            .color(p.text_muted),
        ]
        .spacing(espacio::S4)
        .align_y(Alignment::Center);
        lista = lista.push(
            iced::widget::button(
                container(contenido)
                    .padding(espacio::S4)
                    .width(Length::Fill)
                    .style(estilo::tarjeta(p, i == elegido)),
            )
            .padding(0)
            .width(Length::Fill)
            .on_press(al_elegir(i))
            .style(estilo::sin_estilo(p.text)),
        );
    }
    let mut panel = column![
        text("COPIAS DE SEGURIDAD")
            .font(fuentes::TITULO)
            .size(texto::HEADING.0)
            .color(p.text),
    ]
    .spacing(espacio::S3);
    for c in copias {
        panel = panel
            .push(crate::descargas::regla(borde::FINO, p.border))
            .push(
                row![
                    column![
                        text(c.fecha.clone())
                            .font(fuentes::MONO)
                            .size(texto::MONO_SM.0)
                            .color(p.text),
                        text(c.motivo.clone())
                            .size(texto::BODY_SM.0)
                            .color(p.text_muted),
                    ]
                    .width(Length::Fill),
                    text(c.tamano.clone())
                        .font(fuentes::MONO)
                        .size(texto::MONO_SM.0)
                        .color(p.text_muted),
                    boton(p, "Restaurar", Variante::Secundario, None::<M>),
                ]
                .spacing(espacio::S3)
                .align_y(Alignment::Center),
            );
    }
    pagina(
        row![
            lista,
            container(panel)
                .padding(espacio::S6)
                .width(Length::FillPortion(2))
                .style(estilo::tarjeta(p, false)),
        ]
        .spacing(espacio::S6)
        .into(),
    )
}

/// Señal de impacto de un shader: cuatro barras invertidas (más barras,
/// menos impacto).
pub fn impacto<'a, M: 'a>(p: Paleta, nivel: usize) -> Element<'a, M> {
    senal(p, 4usize.saturating_sub(nivel))
}

/// Configuración de Java de una instancia. Cada instancia tiene la suya.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigJava {
    /// `None` deja que Ícaro elija la versión según la del juego.
    pub java: Option<String>,
    /// Memoria inicial de la JVM (`-Xms`), en MB.
    pub memoria_min: u32,
    /// Memoria máxima de la JVM (`-Xmx`), en MB.
    pub memoria_max: u32,
    /// Memoria para clases y metadatos (`-XX:MaxMetaspaceSize`), en MB.
    pub metaspace: u32,
    /// Argumentos adicionales de la JVM, separados por espacios.
    pub argumentos: String,
}

impl ConfigJava {
    /// Argumentos finales que se pasarían a la JVM.
    pub fn linea_de_comando(&self) -> String {
        let mut partes = vec![
            format!("-Xms{}M", self.memoria_min),
            format!("-Xmx{}M", self.memoria_max),
            format!("-XX:MaxMetaspaceSize={}M", self.metaspace),
        ];
        if !self.argumentos.trim().is_empty() {
            partes.push(self.argumentos.trim().to_owned());
        }
        partes.join(" ")
    }
}

/// Mensajes de la pestaña Java.
pub struct MensajesJava<M> {
    pub java: fn(Option<String>) -> M,
    pub memoria_min: fn(u32) -> M,
    pub memoria_max: fn(u32) -> M,
    pub metaspace: fn(u32) -> M,
    pub argumentos: fn(String) -> M,
}

/// Texto de la opción de Java automático en el selector.
pub const JAVA_AUTOMATICO: &str = "Automático (recomendado)";

/// Pestaña Java de una instancia: versión de Java, memoria mínima, máxima y
/// de metadatos, argumentos de la JVM y las instalaciones detectadas.
pub fn pestana_java<'a, M: Clone + 'a>(
    p: Paleta,
    cfg: &ConfigJava,
    m: &MensajesJava<M>,
    ram_total: u32,
    instalaciones: &[InstalacionJava],
) -> Element<'a, M> {
    let mut opciones = vec![JAVA_AUTOMATICO.to_owned()];
    opciones.extend(
        instalaciones
            .iter()
            .filter(|i| i.existe)
            .map(|i| format!("Java {} · {}", i.version, i.proveedor)),
    );
    let elegida = cfg.java.clone().unwrap_or_else(|| JAVA_AUTOMATICO.to_owned());
    let al_elegir = m.java;
    let version = selector(p, opciones, Some(elegida), move |o| {
        al_elegir((o != JAVA_AUTOMATICO).then_some(o))
    });

    let tope = ram_total.saturating_sub(1024).max(1024);
    let memoria = column![
        campo_rotulado(
            p,
            "Memoria mínima",
            componentes::deslizador_valor(p, cfg.memoria_min, 256, tope, 256, m.memoria_min),
        ),
        campo_rotulado(
            p,
            "Memoria máxima",
            componentes::deslizador_valor(p, cfg.memoria_max, 512, tope, 256, m.memoria_max),
        ),
        campo_rotulado(
            p,
            "Memoria de metadatos (Metaspace)",
            componentes::deslizador_valor(p, cfg.metaspace, 128, 2048, 128, m.metaspace),
        ),
    ]
    .spacing(espacio::S4)
    .width(Length::Fill);

    pagina(
        column![
            campo_rotulado(p, "Versión de Java de esta instancia", version),
            text("Con Automático, Ícaro usa Java 21 desde la 1.20.5, Java 17 de la 1.18 a la 1.20.4 y Java 8 antes.")
                .size(texto::BODY_SM.0)
                .color(p.text_muted),
            memoria,
            campo_rotulado(
                p,
                "Argumentos de la JVM",
                campo(p, "-XX:+UseG1GC -XX:MaxGCPauseMillis=50", &cfg.argumentos, m.argumentos),
            ),
            campo_rotulado(
                p,
                "Línea resultante",
                container(
                    text(cfg.linea_de_comando())
                        .font(fuentes::MONO)
                        .size(texto::MONO_SM.0)
                        .color(p.text),
                )
                .padding(espacio::S3)
                .width(Length::Fill)
                .style(estilo::marco(p))
                .into(),
            ),
            campo_rotulado(p, "Instalaciones detectadas", tabla_instalaciones(p, instalaciones)),
        ]
        .spacing(espacio::S6)
        .into(),
    )
}

/// Entrada visible del árbol de archivos.
#[derive(Debug, Clone)]
pub struct Nodo {
    pub nombre: String,
    pub profundidad: usize,
    pub carpeta: bool,
    pub abierta: bool,
    /// Gestionado por Ícaro: se muestra con candado y no se modifica aquí.
    pub bloqueado: bool,
}

/// Acciones de la pestaña Archivos.
#[derive(Debug, Clone, PartialEq)]
pub enum AccionArchivo {
    Abrir,
    Renombrar,
    ConfirmarNombre,
    CancelarNombre,
    Duplicar,
    Papelera,
    CopiarRuta,
    Guardar,
    Descartar,
}

/// Estado que la pestaña Archivos necesita para dibujarse.
pub struct VistaArchivos<'a> {
    pub arbol: &'a [Nodo],
    pub elegido: Option<usize>,
    /// Contenido del archivo de texto abierto; `None` si no hay o no es texto.
    pub contenido: Option<&'a iced::widget::text_editor::Content>,
    pub modificado: bool,
    /// Mensaje junto al editor: error de lectura o de formato.
    pub nota: Option<(Estado, String)>,
    /// Nombre en edición al renombrar.
    pub renombrando: Option<&'a str>,
}

/// Mensajes de la pestaña Archivos.
pub struct MensajesArchivos<M> {
    pub elegir: fn(usize) -> M,
    pub accion: fn(AccionArchivo) -> M,
    pub editar: fn(iced::widget::text_editor::Action) -> M,
    pub nombre: fn(String) -> M,
}

/// Pestaña Archivos: árbol de carpetas que se abren y cierran, acciones sobre
/// el elemento elegido y editor de texto para los archivos de texto.
pub fn pestana_archivos<'a, M: Clone + 'a>(
    p: Paleta,
    v: &VistaArchivos<'a>,
    m: &MensajesArchivos<M>,
) -> Element<'a, M> {
    let nodo = v.elegido.and_then(|i| v.arbol.get(i));
    let hay = nodo.is_some();
    let libre = nodo.is_some_and(|n| !n.bloqueado);
    let accion = |texto: &str, variante: Variante, activo: bool, a: AccionArchivo| {
        boton(p, texto, variante, activo.then(|| (m.accion)(a)))
    };

    let barra: Element<'a, M> = if let Some(nombre) = v.renombrando {
        row![
            container(campo(p, "Nuevo nombre", nombre, m.nombre)).width(320),
            accion("Guardar nombre", Variante::Primario, true, AccionArchivo::ConfirmarNombre),
            accion("Cancelar", Variante::Fantasma, true, AccionArchivo::CancelarNombre),
        ]
        .spacing(espacio::S2)
        .align_y(Alignment::Center)
        .into()
    } else {
        row![
            accion("Abrir", Variante::Secundario, hay, AccionArchivo::Abrir),
            accion("Renombrar", Variante::Secundario, libre, AccionArchivo::Renombrar),
            accion("Duplicar", Variante::Secundario, libre, AccionArchivo::Duplicar),
            accion("Papelera", Variante::Secundario, libre, AccionArchivo::Papelera),
            accion("Copiar ruta", Variante::Fantasma, hay, AccionArchivo::CopiarRuta),
        ]
        .spacing(espacio::S2)
        .into()
    };

    let mut nodos = column![];
    for (i, n) in v.arbol.iter().enumerate() {
        let actual = v.elegido == Some(i);
        let tinta = if actual { p.bg } else { p.text };
        let expansor: Element<'a, M> = if n.carpeta {
            icono(if n.abierta { Icono::ChevronAbajo } else { Icono::ChevronDerecha }, Tam::Base, tinta).into()
        } else {
            Space::with_width(Tam::Base.px()).into()
        };
        let glifo = if n.carpeta { Icono::Carpeta } else { Icono::Copiar };
        let mut fila = row![
            Space::with_width(n.profundidad as f32 * espacio::S5),
            expansor,
            icono(glifo, Tam::Base, tinta),
            text(n.nombre.clone())
                .font(if actual { fuentes::CUERPO_NEGRITA } else { fuentes::CUERPO })
                .size(texto::BODY.0)
                .color(tinta)
                .width(Length::Fill),
        ]
        .spacing(espacio::S3)
        .align_y(Alignment::Center);
        if n.bloqueado {
            fila = fila.push(icono(Icono::Candado, Tam::Base, tinta));
        }
        nodos = nodos.push(
            iced::widget::button(container(fila).center_y(Length::Fill))
                .width(Length::Fill)
                .height(40)
                .padding(Padding::from([0.0, espacio::S4]))
                .on_press((m.elegir)(i))
                .style(move |_, estado| iced::widget::button::Style {
                    background: (actual
                        || matches!(estado, iced::widget::button::Status::Hovered))
                    .then_some(iced::Background::Color(if actual {
                        p.text
                    } else {
                        p.surface_hover
                    })),
                    text_color: tinta,
                    ..Default::default()
                }),
        );
    }

    let derecha: Element<'a, M> = match v.contenido {
        Some(contenido) => {
            let editor = iced::widget::text_editor(contenido)
                .on_action(m.editar)
                .font(fuentes::MONO)
                .size(texto::MONO_SM.0)
                .height(Length::Fill)
                .padding(espacio::S3)
                .style(move |_: &iced::Theme, _| iced::widget::text_editor::Style {
                    background: iced::Background::Color(p.surface_sunken),
                    border: iced::Border {
                        color: p.border_strong,
                        width: borde::MEDIO,
                        radius: 0.0.into(),
                    },
                    icon: p.text_muted,
                    placeholder: p.text_muted,
                    value: p.text,
                    selection: p.accent_soft,
                });
            let mut pie = row![].spacing(espacio::S2).align_y(Alignment::Center);
            pie = pie.push(accion("Guardar", Variante::Primario, v.modificado, AccionArchivo::Guardar));
            pie = pie.push(accion("Descartar", Variante::Secundario, v.modificado, AccionArchivo::Descartar));
            if v.modificado {
                pie = pie.push(insignia(p, Estado::Aviso, "Sin guardar"));
            }
            if let Some((estado, nota)) = &v.nota {
                pie = pie.push(insignia(p, estado.clone(), nota));
            }
            column![editor, pie].spacing(espacio::S2).into()
        }
        None => container(
            text(match &v.nota {
                Some((_, nota)) => nota.clone(),
                None => "Elige un archivo de texto para verlo y editarlo.".to_owned(),
            })
            .size(texto::BODY.0)
            .color(p.text_muted),
        )
        .padding(espacio::S4)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(estilo::marco(p))
        .into(),
    };

    pagina(
        column![
            barra,
            row![
                container(crate::scroll::desplazable(nodos))
                    .padding(borde::MEDIO)
                    .width(Length::FillPortion(2))
                    .height(420)
                    .style(estilo::marco(p)),
                container(derecha)
                    .width(Length::FillPortion(3))
                    .height(420),
            ]
            .spacing(espacio::S4),
        ]
        .spacing(espacio::S4)
        .into(),
    )
}
