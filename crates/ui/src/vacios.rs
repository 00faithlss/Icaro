// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Estados vacíos: una lámina a la izquierda, el friso de meandro, un título
//! que dice qué falta y la acción que lo resuelve. Cada sección tiene su
//! lámina y su texto.

use iced::widget::{column, container, row, text};
use iced::{Alignment, Element, Length, Padding};

use crate::botones::{boton, TamBoton};
use crate::componentes::Variante;
use crate::estilo;
use crate::fuentes;
use crate::grabados::{friso, Imagen};
use crate::iconos::Icono;
use crate::laminas::Lamina;
use crate::tema::{borde, espacio, texto, Paleta};

/// Bajo este ancho de ventana la lámina pasa arriba.
pub const ANCHO_COMPACTO: f32 = 1100.0;
/// Medida de lectura del texto: unos 46 caracteres.
const ANCHO_TEXTO: f32 = 400.0;
/// Los títulos grandes caben en dos líneas de este ancho.
const ANCHO_TITULO: f32 = 580.0;
const LAMINA_ANCHO: f32 = 300.0;
const LAMINA_ALTO: f32 = 200.0;
const LAMINA_MIN: f32 = 220.0;
/// Ancho de la barra lateral abierta, para repartir el espacio.
const BARRA: f32 = 232.0;

/// Lo que cada pantalla sabe del entorno al pintar un vacío.
#[derive(Debug, Clone, Copy)]
pub struct Contexto {
    pub p: Paleta,
    /// Ancho de la ventana, para decidir dónde va la lámina.
    pub ventana: f32,
    /// Avance de la Prensa de la lámina, de 0 a 1.
    pub prensa: f32,
}

/// Bloque de estado vacío con lámina, título (hasta dos líneas separadas por
/// `\n`), explicación y acciones.
pub fn estado_vacio<'a, M: 'a>(
    c: Contexto,
    lamina: Lamina,
    titulo: &str,
    explicacion: &str,
    acciones: Vec<Element<'a, M>>,
) -> Element<'a, M> {
    let p = c.p;
    let compacto = c.ventana < ANCHO_COMPACTO;
    let marco = container(
        Imagen::nueva(p, lamina)
            .barrido(p)
            .prensa(c.prensa)
            .tamano(Length::Fill, Length::Fill),
    )
    .padding(borde::MEDIO)
    .style(estilo::marco(p));
    // La lámina cede ancho (de 300 a 220 px) para que el título siga en dos líneas.
    let ancho_lamina = (c.ventana - BARRA - 2.0 * (espacio::S12 + espacio::S12) - espacio::S12 - ANCHO_TITULO)
        .clamp(LAMINA_MIN, LAMINA_ANCHO);
    let marco = if compacto {
        marco.width(Length::Fill).height(LAMINA_ALTO)
    } else {
        marco.width(ancho_lamina).height((ancho_lamina * 2.0 / 3.0).round())
    };

    let mut fila_acciones = row![].spacing(espacio::S2).align_y(Alignment::Center);
    for a in acciones {
        fila_acciones = fila_acciones.push(a);
    }
    let cuerpo = column![
        friso(p, 192.0),
        text(titulo.to_uppercase())
            .font(fuentes::DISPLAY)
            .size(texto::DISPLAY.0)
            .line_height(iced::widget::text::LineHeight::Absolute(texto::DISPLAY.1.into()))
            .color(p.text),
        text(explicacion.to_owned())
            .size(texto::BODY.0)
            .color(p.text_muted)
            .width(ANCHO_TEXTO),
        fila_acciones.wrap(),
    ]
    .spacing(espacio::S4)
    .max_width(ANCHO_TITULO);

    let contenido: Element<M> = if compacto {
        column![marco, cuerpo].spacing(espacio::S8).into()
    } else {
        row![marco, cuerpo]
            .spacing(espacio::S12)
            .align_y(Alignment::Center)
            .into()
    };
    container(contenido)
        .padding(espacio::S12)
        .width(Length::Fill)
        .style(estilo::tarjeta(p, false))
        .into()
}

/// Margen en que se coloca un vacío dentro de su pantalla.
pub fn margen() -> Padding {
    Padding::from([espacio::S6, espacio::S12])
}

/// Instancias: el primer bloque.
pub fn instancias<'a, M: Clone + 'a>(c: Contexto, crear: M, importar: M) -> Element<'a, M> {
    estado_vacio(
        c,
        Lamina::Mano,
        "Aún no hay\ninstancias",
        "Una instancia es una copia de Minecraft con su versión, sus mods y sus mundos. Crea la primera o importa un modpack que ya tengas.",
        vec![
            boton(c.p, Some(Icono::Mas), "Crear instancia", Variante::Primario, TamBoton::Lg, Some(crear)),
            boton(c.p, Some(Icono::Subir), "Importar modpack", Variante::Secundario, TamBoton::Lg, Some(importar)),
        ],
    )
}

/// Mods y Modpacks: búsqueda sin resultados.
pub fn busqueda<'a, M: Clone + 'a>(c: Contexto, consulta: &str, limpiar: M) -> Element<'a, M> {
    estado_vacio(
        c,
        Lamina::Orbita,
        &format!("Nada coincide con\n\"{consulta}\""),
        "Prueba con otra palabra, quita algún filtro o cambia la versión de Minecraft.",
        vec![boton(c.p, None, "Limpiar búsqueda", Variante::Secundario, TamBoton::Lg, Some(limpiar))],
    )
}

/// Servidores: ninguno guardado.
pub fn servidores<'a, M: Clone + 'a>(c: Contexto, agregar: M) -> Element<'a, M> {
    estado_vacio(
        c,
        Lamina::Alas,
        "Ningún servidor\nguardado",
        "Guarda la dirección de un servidor para ver quién está conectado y entrar con un clic.",
        vec![boton(c.p, Some(Icono::Mas), "Agregar servidor", Variante::Primario, TamBoton::Lg, Some(agregar))],
    )
}

/// Descargas: nada en curso.
pub fn descargas<'a, M: Clone + 'a>(c: Contexto, buscar_modpacks: M) -> Element<'a, M> {
    estado_vacio(
        c,
        Lamina::Forja,
        "No hay descargas\nen curso",
        "Cuando instales un mod, una versión de Minecraft o Java, el avance aparece aquí.",
        vec![boton(c.p, Some(Icono::Buscar), "Buscar modpacks", Variante::Primario, TamBoton::Lg, Some(buscar_modpacks))],
    )
}

/// Capturas: todavía no hay ninguna.
pub fn capturas<'a, M: Clone + 'a>(c: Contexto, ir_a_instancias: M) -> Element<'a, M> {
    estado_vacio(
        c,
        Lamina::Umbral,
        "Tus capturas\naparecerán aquí",
        "Cada captura que hagas con F2 dentro de Minecraft se guarda y se ordena aquí por instancia.",
        vec![boton(c.p, None, "Ir a Instancias", Variante::Secundario, TamBoton::Lg, Some(ir_a_instancias))],
    )
}

/// Capturas de una instancia que no tiene ninguna.
pub fn capturas_para_jugar<'a, M: Clone + 'a>(c: Contexto, jugar: M) -> Element<'a, M> {
    estado_vacio(c, Lamina::Umbral, "Tus capturas\naparecerán aquí",
        "Comienza a jugar y haz capturas con F2 dentro de Minecraft. Las capturas de esta instancia aparecerán aquí.",
        vec![boton(c.p, Some(Icono::Jugar), "Jugar", Variante::Primario, TamBoton::Lg, Some(jugar))])
}

/// Capturas de una instancia filtrada desde la galería global.
pub fn capturas_de_instancia<'a, M: Clone + 'a>(c: Contexto, instancia: &str, ver_todas: M) -> Element<'a, M> {
    estado_vacio(
        c,
        Lamina::Umbral,
        "Sin capturas\naquí",
        &format!("{instancia} todavía no tiene capturas. Las que hagas en esa instancia se mostrarán en este filtro."),
        vec![boton(c.p, None, "Ver todas", Variante::Secundario, TamBoton::Lg, Some(ver_todas))],
    )
}

/// Consola sin juego abierto: el escriba en su mesa.
pub fn consola<'a, M: Clone + 'a>(c: Contexto, ir_a_instancias: M) -> Element<'a, M> {
    estado_vacio(
        c,
        Lamina::JeronimoSanto,
        "El juego no\nestá abierto",
        "Los registros aparecen cuando juegas. Abre una instancia y vuelve aquí para seguir lo que pasa.",
        vec![boton(c.p, None, "Ir a Instancias", Variante::Primario, TamBoton::Lg, Some(ir_a_instancias))],
    )
}
