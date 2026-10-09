// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Editor de instancia: encabezado con datos clave y el contenido de las
//! pestañas General, Java, Recursos, Mundos y Archivos. La pestaña Mods
//! queda fuera hasta que se ordene su desarrollo.

use iced::widget::{column, container, row, scrollable, text, Space};
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
    scrollable(container(contenido).padding(Padding::from([espacio::S6, espacio::S12])))
        .height(Length::Fill)
        .into()
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
    pub grupo: Option<&'static str>,
    pub ancho: String,
    pub alto: String,
    pub pantalla_completa: bool,
    pub servidor: Option<&'static str>,
    pub memoria: u32,
}

/// Mensajes de la pestaña General.
pub struct MensajesGeneral<M> {
    pub nombre: fn(String) -> M,
    pub grupo: fn(&'static str) -> M,
    pub ancho: fn(String) -> M,
    pub alto: fn(String) -> M,
    pub pantalla_completa: fn(bool) -> M,
    pub servidor: fn(&'static str) -> M,
    pub memoria: fn(u32) -> M,
}

/// Pestaña General: nombre, grupo, ventana, servidor y memoria.
pub fn pestana_general<'a, M: Clone + 'a>(
    p: Paleta,
    e: &EstadoGeneral,
    m: &MensajesGeneral<M>,
    ram_total: u32,
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
                selector(p, vec!["Con amigos", "Técnicos", "Sin grupo"], e.grupo, m.grupo)
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
        campo_rotulado(
            p,
            "Memoria",
            componentes::deslizador_memoria(
                p,
                e.memoria,
                ram_total,
                (ram_total as f32 * 0.35) as u32,
                (4096, 8192),
                m.memoria
            )
        ),
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

/// Pestaña Java: gestión automática y tabla de instalaciones.
pub fn pestana_java<'a, M: Clone + 'a>(
    p: Paleta,
    automatica: bool,
    alternar: M,
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
    pagina(
        column![
            crate::ajustes::fila_ajuste(
                p,
                "Gestión automática",
                "Ícaro descarga y elige la versión correcta de Java para cada instancia. Recomendado.",
                interruptor(p, automatica, alternar),
            ),
            tabla,
            row![
                boton(p, "Descargar Java", Variante::Secundario, None::<M>),
                boton(p, "Detectar instalaciones", Variante::Secundario, None::<M>),
                boton(p, "Agregar ruta", Variante::Fantasma, None::<M>),
            ]
            .spacing(espacio::S2),
        ]
        .spacing(espacio::S4)
        .into(),
    )
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

/// Nodo del árbol de archivos.
#[derive(Debug, Clone)]
pub struct Nodo {
    pub nombre: String,
    pub profundidad: usize,
    pub carpeta: bool,
    pub bloqueado: bool,
}

/// Pestaña Archivos: árbol a la izquierda y vista previa a la derecha.
pub fn pestana_archivos<'a, M: Clone + 'a>(
    p: Paleta,
    arbol: &[Nodo],
    elegido: usize,
    al_elegir: impl Fn(usize) -> M + 'a,
    previa: &str,
) -> Element<'a, M> {
    let barra = row![
        boton(p, "Abrir", Variante::Secundario, None::<M>),
        boton(p, "Renombrar", Variante::Secundario, None::<M>),
        boton(p, "Duplicar", Variante::Secundario, None::<M>),
        boton(p, "Papelera", Variante::Secundario, None::<M>),
        boton(p, "Copiar ruta", Variante::Fantasma, None::<M>),
    ]
    .spacing(espacio::S2);
    let mut nodos = column![];
    for (i, n) in arbol.iter().enumerate() {
        let glifo = if n.carpeta { Icono::Carpeta } else { Icono::Copiar };
        let actual = i == elegido;
        let tinta = if actual { p.bg } else { p.text };
        let mut fila = row![
            Space::with_width(n.profundidad as f32 * espacio::S4),
            icono(glifo, Tam::Base, tinta),
            text(n.nombre.clone())
                .size(texto::BODY_SM.0)
                .color(tinta)
                .width(Length::Fill),
        ]
        .spacing(espacio::S2)
        .align_y(Alignment::Center);
        if n.bloqueado {
            fila = fila.push(icono(Icono::Candado, Tam::Sm, tinta));
        }
        nodos = nodos.push(
            iced::widget::button(fila)
                .width(Length::Fill)
                .height(30)
                .padding(Padding::from([0.0, espacio::S3]))
                .on_press(al_elegir(i))
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
    pagina(
        column![
            barra,
            row![
                container(scrollable(nodos))
                    .width(Length::FillPortion(2))
                    .height(360)
                    .style(estilo::marco(p)),
                container(
                    text(previa.to_owned())
                        .font(fuentes::MONO)
                        .size(texto::MONO_SM.0)
                        .color(p.text)
                )
                .padding(espacio::S4)
                .width(Length::FillPortion(3))
                .height(360)
                .style(estilo::marco(p)),
            ]
            .spacing(espacio::S4),
        ]
        .spacing(espacio::S4)
        .into(),
    )
}

/// Señal de impacto de un shader: cuatro barras invertidas (más barras,
/// menos impacto).
pub fn impacto<'a, M: 'a>(p: Paleta, nivel: usize) -> Element<'a, M> {
    senal(p, 4usize.saturating_sub(nivel))
}
