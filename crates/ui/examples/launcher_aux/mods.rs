// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Mods de una instancia de demostración: los instalados viven como archivos
//! `.jar` en su carpeta `mods` y el catálogo es una lista de ejemplo, ya que
//! todavía no hay conexión con un servicio de descargas.

use std::fs;
use std::path::PathBuf;

static RECURSO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Elige si las funciones de este módulo trabajan con mods (`false`) o con
/// resourcepacks (`true`): cambia la carpeta, la extensión y el catálogo.
pub fn fijar_recurso(recurso: bool) {
    RECURSO.store(recurso, std::sync::atomic::Ordering::Relaxed);
}

pub fn es_recurso() -> bool {
    RECURSO.load(std::sync::atomic::Ordering::Relaxed)
}

fn ext() -> &'static str {
    if es_recurso() { ".zip" } else { ".jar" }
}

/// Catálogo del tipo actual.
pub fn catalogo_actual() -> &'static [ModCatalogo] {
    if es_recurso() { RECURSOS } else { CATALOGO }
}

/// Mod instalado, leído del nombre del archivo (`nombre-version.jar`).
#[derive(Debug, Clone)]
pub struct ModInstalado {
    /// Archivo sin la marca de desactivado.
    pub archivo: String,
    /// Identificador en minúsculas, la parte previa a la versión.
    pub id: String,
    pub version: String,
    pub activo: bool,
}

/// Versión publicada de un mod.
pub struct Lanzamiento {
    pub version: &'static str,
    pub minecraft: &'static str,
    pub fecha: &'static str,
    pub cambios: &'static [&'static str],
    /// `release`, `beta` o `alpha`.
    pub canal: &'static str,
    /// Cargadores propios de esta publicación; vacío usa los del mod.
    pub cargadores: &'static [&'static str],
}

/// Mod del catálogo de ejemplo.
pub struct ModCatalogo {
    pub id: &'static str,
    pub titulo: &'static str,
    pub autor: &'static str,
    pub descargas: &'static str,
    pub descripcion: &'static str,
    /// Texto largo de la pestaña Descripción.
    pub detalle: &'static str,
    pub licencia: &'static str,
    pub cargadores: &'static [&'static str],
    pub categorias: &'static [&'static str],
    /// Identificadores de los mods que necesita.
    pub dependencias: &'static [&'static str],
    /// Versiones publicadas, de la más nueva a la más antigua.
    pub lanzamientos: &'static [Lanzamiento],
}

impl ModCatalogo {
    /// Cargadores de una publicación concreta.
    pub fn cargadores_de(&self, l: &Lanzamiento) -> &'static [&'static str] {
        if l.cargadores.is_empty() { self.cargadores } else { l.cargadores }
    }

    /// Versiones del juego para las que hay alguna publicación.
    pub fn minecraft(&self) -> Vec<&'static str> {
        let mut v: Vec<&'static str> = Vec::new();
        for l in self.lanzamientos {
            if !v.contains(&l.minecraft) {
                v.push(l.minecraft);
            }
        }
        v
    }
}

const fn l(
    version: &'static str,
    minecraft: &'static str,
    fecha: &'static str,
    cambios: &'static [&'static str],
) -> Lanzamiento {
    Lanzamiento { version, minecraft, fecha, cambios, canal: "release", cargadores: &[] }
}

const fn lb(
    version: &'static str,
    minecraft: &'static str,
    fecha: &'static str,
    cambios: &'static [&'static str],
) -> Lanzamiento {
    Lanzamiento { version, minecraft, fecha, cambios, canal: "beta", cargadores: &[] }
}

const fn lc(
    version: &'static str,
    minecraft: &'static str,
    fecha: &'static str,
    cambios: &'static [&'static str],
    cargadores: &'static [&'static str],
) -> Lanzamiento {
    Lanzamiento { version, minecraft, fecha, cambios, canal: "release", cargadores }
}

pub const CATALOGO: &[ModCatalogo] = &[
    ModCatalogo {
        id: "sodium",
        titulo: "Sodium",
        autor: "JellySquid",
        descargas: "52,4 M",
        descripcion: "Motor de dibujo moderno que sube los cuadros por segundo sin cambiar cómo se ve el juego.",
        detalle: "Sodium reescribe el motor de dibujo de Minecraft para aprovechar mejor la tarjeta gráfica. Mejora los cuadros por segundo, reduce los tirones y corrige errores gráficos, sin tocar la jugabilidad. Es la base de otros mods de rendimiento y de Iris.",
        licencia: "LGPL-3.0",
        cargadores: &["Fabric", "Quilt"],
        categorias: &["Rendimiento", "Gráficos"],
        dependencias: &[],
        lanzamientos: &[
            lb("0.7.0-beta.1", "1.21.4", "2026-10-05", &["Prueba del nuevo motor de chunks"]),
            l("0.6.5", "1.21.4", "2026-09-28", &["Corrige un fallo al cambiar de dimensión", "Menos memoria al cargar chunks lejanos"]),
            l("0.6.4", "1.21.4", "2026-09-12", &["Mejora la compatibilidad con shaders", "Optimiza el dibujo de hojas"]),
            l("0.6.0", "1.21.1", "2026-07-30", &["Nuevo sistema de opciones", "Soporte para 1.21.1"]),
            l("0.5.11", "1.20.1", "2026-05-02", &["Corrección de errores"]),
        ],
    },
    ModCatalogo {
        id: "lithium",
        titulo: "Lithium",
        autor: "JellySquid",
        descargas: "38,1 M",
        descripcion: "Optimiza la lógica del juego: física, generación de mundo e inteligencia de las criaturas.",
        detalle: "Lithium mejora el rendimiento del servidor y del cliente optimizando la física, la generación del mundo, la inteligencia de las criaturas y las redes de bloques, sin cambiar el comportamiento del juego.",
        licencia: "LGPL-3.0",
        cargadores: &["Fabric", "Quilt", "NeoForge"],
        categorias: &["Rendimiento", "Servidor"],
        dependencias: &[],
        lanzamientos: &[
            l("0.14.3", "1.21.4", "2026-09-20", &["Optimiza la búsqueda de caminos", "Corrige una fuga de memoria"]),
            lc("0.14.2", "1.21.4", "2026-09-01", &["Compilación para Quilt"], &["Quilt"]),
            l("0.14.0", "1.21.1", "2026-08-02", &["Soporte para 1.21.1"]),
            l("0.13.1", "1.20.1", "2026-04-14", &["Corrección de errores"]),
        ],
    },
    ModCatalogo {
        id: "iris",
        titulo: "Iris Shaders",
        autor: "coderbot",
        descargas: "29,7 M",
        descripcion: "Permite usar paquetes de shaders compatibles con Sodium.",
        detalle: "Iris carga paquetes de shaders con el formato de OptiFine y es compatible con Sodium. Permite cambiar de shader en el juego y ajustar sus opciones sin reiniciar.",
        licencia: "LGPL-3.0",
        cargadores: &["Fabric", "Quilt"],
        categorias: &["Gráficos", "Shaders"],
        dependencias: &["sodium"],
        lanzamientos: &[
            lb("1.9.0-beta.2", "1.21.4", "2026-10-06", &["Primera prueba del nuevo canal de efectos"]),
            l("1.8.1", "1.21.4", "2026-09-30", &["Compatibilidad con Sodium 0.6.5", "Menos parpadeos al cargar shaders"]),
            l("1.8.0", "1.21.4", "2026-09-05", &["Nuevo menú de opciones de shaders"]),
            l("1.7.6", "1.21.1", "2026-06-18", &["Corrección de errores"]),
        ],
    },
    ModCatalogo {
        id: "modmenu",
        titulo: "Mod Menu",
        autor: "Prospector",
        descargas: "41,0 M",
        descripcion: "Agrega una lista de los mods instalados dentro del juego, con su configuración.",
        detalle: "Mod Menu añade una pantalla con todos los mods instalados, su descripción, versión y un acceso directo a su configuración desde el menú principal.",
        licencia: "MIT",
        cargadores: &["Fabric", "Quilt"],
        categorias: &["Utilidad"],
        dependencias: &["fabric-api"],
        lanzamientos: &[
            l("11.0.3", "1.21.4", "2026-09-10", &["Corrige el orden alfabético", "Actualiza las traducciones"]),
            l("11.0.0", "1.21.1", "2026-07-01", &["Soporte para 1.21.1"]),
            l("10.0.0", "1.20.1", "2026-03-20", &["Rediseño de la lista"]),
        ],
    },
    ModCatalogo {
        id: "fabric-api",
        titulo: "Fabric API",
        autor: "Fabric",
        descargas: "88,9 M",
        descripcion: "Biblioteca base que necesitan casi todos los mods de Fabric.",
        detalle: "Fabric API ofrece las funciones comunes que usan la mayoría de los mods de Fabric: eventos, registros, renderizado y red. No cambia nada por sí sola.",
        licencia: "Apache-2.0",
        cargadores: &["Fabric"],
        categorias: &["Biblioteca"],
        dependencias: &[],
        lanzamientos: &[
            l("0.112.0", "1.21.4", "2026-09-25", &["Nuevos eventos de renderizado", "Corrección de errores de red"]),
            l("0.110.5", "1.21.1", "2026-08-11", &["Soporte para 1.21.1"]),
            l("0.105.0", "1.20.1", "2026-05-30", &["Corrección de errores"]),
        ],
    },
    ModCatalogo {
        id: "xaeros-minimap",
        titulo: "Xaero's Minimap",
        autor: "xaero96",
        descargas: "21,3 M",
        descripcion: "Minimapa en pantalla con marcas de ubicación y detección de criaturas.",
        detalle: "Un minimapa que se actualiza mientras exploras, con marcas de ubicación, radar de criaturas y jugadores, y varias opciones de estilo.",
        licencia: "Todos los derechos reservados",
        cargadores: &["Fabric", "Forge", "NeoForge"],
        categorias: &["Utilidad", "Mapas"],
        dependencias: &[],
        lanzamientos: &[
            l("24.6.1", "1.21.4", "2026-09-15", &["Corrige marcas duplicadas", "Mejora el rendimiento del radar"]),
            l("24.5.0", "1.21.1", "2026-07-22", &["Soporte para 1.21.1"]),
        ],
    },
    ModCatalogo {
        id: "journeymap",
        titulo: "JourneyMap",
        autor: "techbrew",
        descargas: "17,8 M",
        descripcion: "Mapa en tiempo real del mundo, con marcas y mapa a pantalla completa.",
        detalle: "Dibuja un mapa del mundo mientras lo recorres, con minimapa, mapa completo, marcas personalizadas y varias vistas, como superficie y cuevas.",
        licencia: "MIT",
        cargadores: &["Fabric", "Forge", "NeoForge"],
        categorias: &["Utilidad", "Mapas"],
        dependencias: &[],
        lanzamientos: &[
            l("6.0.0", "1.21.4", "2026-08-30", &["Nuevo motor de mapas", "Soporte para 1.21.4"]),
            l("5.10.3", "1.20.1", "2026-05-12", &["Corrección de errores"]),
        ],
    },
    ModCatalogo {
        id: "create",
        titulo: "Create",
        autor: "simibubi",
        descargas: "34,5 M",
        descripcion: "Máquinas, engranajes y automatización con una estética mecánica.",
        detalle: "Create agrega engranajes, cintas, prensas y trenes para automatizar la fabricación y el transporte, con una estética de maquinaria visible y animada.",
        licencia: "MIT",
        cargadores: &["Fabric", "Forge", "NeoForge"],
        categorias: &["Tecnología", "Automatización"],
        dependencias: &["fabric-api"],
        lanzamientos: &[
            l("6.0.6", "1.21.1", "2026-09-02", &["Nuevos engranajes grandes", "Corrige un fallo con los trenes"]),
            lc("6.0.5", "1.21.1", "2026-08-20", &["Compilación solo para NeoForge"], &["NeoForge"]),
            l("0.5.1", "1.20.1", "2026-04-01", &["Corrección de errores"]),
        ],
    },
];

/// Resourcepacks de ejemplo; usan los mismos tipos que los mods y el campo
/// `cargadores` guarda la resolución.
pub const RECURSOS: &[ModCatalogo] = &[
    ModCatalogo {
        id: "faithful",
        titulo: "Faithful 32x",
        autor: "Faithful Team",
        descargas: "48,2 M",
        descripcion: "Las texturas originales con el doble de resolución, fieles al estilo del juego.",
        detalle: "Faithful duplica la resolución de las texturas de Minecraft sin cambiar su estilo: cada bloque y objeto se reconoce de inmediato, pero con más detalle. Es la opción más segura para quien quiere mejor imagen sin que el juego se vea distinto.",
        licencia: "Faithful License",
        cargadores: &["32x"],
        categorias: &["Fiel", "Bloques", "Objetos"],
        dependencias: &[],
        lanzamientos: &[
            l("2.4.1", "1.21.4", "2026-09-22", &["Texturas nuevas para los bloques de la 1.21.4", "Corrige las costuras de la lana"]),
            l("2.4.0", "1.21.1", "2026-07-14", &["Soporte para 1.21.1"]),
            l("2.3.0", "1.20.1", "2026-03-30", &["Corrección de errores"]),
        ],
    },
    ModCatalogo {
        id: "soft-fantasy",
        titulo: "Soft Fantasy",
        autor: "Aeris",
        descargas: "6,9 M",
        descripcion: "Colores suaves y bordes redondeados con un aire de cuento.",
        detalle: "Soft Fantasy redibuja el juego con una paleta pastel y formas redondeadas. Cambia el aspecto general del mundo, de la interfaz y de las criaturas.",
        licencia: "CC BY-NC 4.0",
        cargadores: &["16x"],
        categorias: &["Fantasía", "Interfaz"],
        dependencias: &[],
        lanzamientos: &[
            l("1.9.0", "1.20.1", "2026-05-18", &["Nuevas texturas de aldeanos", "Interfaz renovada"]),
            l("1.8.0", "1.20.1", "2026-02-02", &["Corrección de errores"]),
        ],
    },
    ModCatalogo {
        id: "bare-bones",
        titulo: "Bare Bones",
        autor: "Mojang Fans",
        descargas: "21,4 M",
        descripcion: "Texturas planas y limpias, con la menor carga posible.",
        detalle: "Un estilo plano de colores sólidos que se parece a un dibujo. Rinde muy bien en equipos modestos y deja el mundo muy legible.",
        licencia: "CC BY 4.0",
        cargadores: &["16x"],
        categorias: &["Simple", "Rendimiento"],
        dependencias: &[],
        lanzamientos: &[
            l("1.4.2", "1.21.4", "2026-09-05", &["Soporte para 1.21.4"]),
            l("1.4.0", "1.21.1", "2026-06-20", &["Nuevos colores para el nether"]),
        ],
    },
    ModCatalogo {
        id: "fresh-animations",
        titulo: "Fresh Animations",
        autor: "FreshLX",
        descargas: "12,6 M",
        descripcion: "Animaciones nuevas para las criaturas, sin cambiar sus texturas.",
        detalle: "Agrega movimientos fluidos a casi todas las criaturas: caminar, mirar, comer y atacar. Necesita un mod que permita modelos personalizados.",
        licencia: "Todos los derechos reservados",
        cargadores: &["16x"],
        categorias: &["Animaciones", "Criaturas"],
        dependencias: &[],
        lanzamientos: &[
            l("1.10.0", "1.21.4", "2026-09-12", &["Animaciones nuevas para el warden", "Corrige el movimiento de los perros"]),
            l("1.9.5", "1.21.1", "2026-07-02", &["Soporte para 1.21.1"]),
        ],
    },
    ModCatalogo {
        id: "stay-true",
        titulo: "Stay True",
        autor: "Stay True Team",
        descargas: "9,3 M",
        descripcion: "Mejora los detalles manteniendo el aspecto clásico de Minecraft.",
        detalle: "Un pack que respeta el estilo original pero afina bordes, sombras y detalles de cientos de bloques y objetos.",
        licencia: "CC BY-NC 4.0",
        cargadores: &["32x"],
        categorias: &["Fiel", "Detalle"],
        dependencias: &[],
        lanzamientos: &[
            l("1.8.0", "1.21.1", "2026-08-08", &["Nuevos detalles para la piedra y la tierra"]),
            l("1.7.2", "1.20.1", "2026-04-17", &["Corrección de errores"]),
        ],
    },
];

/// Datos del catálogo de un mod por su identificador.
pub fn en_catalogo(id: &str) -> Option<&'static ModCatalogo> {
    CATALOGO.iter().chain(RECURSOS.iter()).find(|m| m.id == id)
}

fn carpeta(instancia: &str) -> PathBuf {
    let raiz = std::env::temp_dir().join("icaro-demo").join(instancia);
    let mods = raiz.join(if es_recurso() { "resourcepacks" } else { "mods" });
    let _ = fs::create_dir_all(&mods);
    mods
}

/// Separa `nombre-version.jar` en identificador y versión.
fn separar(archivo: &str) -> (String, String) {
    let base = archivo.trim_end_matches(ext());
    let corte = base
        .char_indices()
        .find(|(i, c)| *c == '-' && base[i + 1..].starts_with(|d: char| d.is_ascii_digit()))
        .map(|(i, _)| i);
    match corte {
        Some(i) => (base[..i].to_lowercase(), base[i + 1..].to_owned()),
        None => (base.to_lowercase(), String::new()),
    }
}

pub fn listar(instancia: &str) -> Vec<ModInstalado> {
    let mut lista: Vec<ModInstalado> = fs::read_dir(carpeta(instancia))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let nombre = e.file_name().to_string_lossy().into_owned();
            let (archivo, activo) = if let Some(b) = nombre.strip_suffix(&format!("{}.disabled", ext())) {
                (format!("{b}{}", ext()), false)
            } else if nombre.ends_with(ext()) {
                (nombre, true)
            } else {
                return None;
            };
            let (id, version) = separar(&archivo);
            Some(ModInstalado { archivo, id, version, activo })
        })
        .collect();
    lista.sort_by(|a, b| a.id.cmp(&b.id));
    lista
}

fn ruta_real(instancia: &str, archivo: &str) -> Option<PathBuf> {
    let c = carpeta(instancia);
    let activo = c.join(archivo);
    let apagado = c.join(format!("{archivo}.disabled"));
    if activo.exists() {
        Some(activo)
    } else if apagado.exists() {
        Some(apagado)
    } else {
        None
    }
}

/// Activa o desactiva renombrando el archivo, como hacen otros launchers.
pub fn alternar(instancia: &str, archivo: &str) {
    let c = carpeta(instancia);
    let activo = c.join(archivo);
    let apagado = c.join(format!("{archivo}.disabled"));
    let _ = if activo.exists() {
        fs::rename(activo, apagado)
    } else {
        fs::rename(apagado, activo)
    };
}

/// Instala una versión del catálogo (aquí solo crea el archivo).
pub fn instalar(instancia: &str, id: &str, version: &str) {
    let _ = fs::write(carpeta(instancia).join(format!("{id}-{version}{}", ext())), b"PK");
}

/// Cambia un mod instalado a otra versión; conserva si estaba desactivado.
/// Devuelve el nombre del archivo nuevo.
pub fn cambiar_version(instancia: &str, archivo: &str, id: &str, version: &str) -> Option<String> {
    let actual = ruta_real(instancia, archivo)?;
    let activo = actual.extension().and_then(|e| e.to_str()) == Some(ext().trim_start_matches('.'));
    let nuevo = format!("{id}-{version}{}", ext());
    let destino = carpeta(instancia).join(if activo { nuevo.clone() } else { format!("{nuevo}.disabled") });
    fs::rename(actual, destino).ok()?;
    Some(nuevo)
}

/// Mueve el mod a la papelera interna de la instancia.
pub fn eliminar(instancia: &str, archivo: &str) {
    let Some(actual) = ruta_real(instancia, archivo) else {
        return;
    };
    let papelera = carpeta(instancia).join("..").join(".papelera");
    let _ = fs::create_dir_all(&papelera);
    let _ = fs::rename(&actual, papelera.join(actual.file_name().unwrap_or_default()));
}

/// Datos adicionales de la página de un mod.
pub struct Extras {
    pub seguidores: &'static str,
    pub publicado: &'static str,
    pub actualizado: &'static str,
    /// Títulos de las capturas; vacío si el mod no tiene imágenes.
    pub galeria: &'static [&'static str],
    pub opcionales: &'static [&'static str],
    pub incompatibles: &'static [(&'static str, &'static str)],
    /// "Persona" u "Organización".
    pub creador_tipo: &'static str,
    pub creador_proyectos: u32,
    pub creador_descargas: &'static str,
    pub creador_top: [&'static str; 3],
    pub entorno: &'static str,
}

pub fn extras(id: &str) -> Extras {
    let base = Extras {
        seguidores: "12,4 mil",
        publicado: "2025-03-14",
        actualizado: "2026-09-20",
        galeria: &[],
        opcionales: &[],
        incompatibles: &[],
        creador_tipo: "Persona",
        creador_proyectos: 6,
        creador_descargas: "40,2 M",
        creador_top: ["Sodium", "Lithium", "Phosphor"],
        entorno: "Cliente y servidor",
    };
    match id {
        "sodium" => Extras {
            entorno: "Cliente",
            seguidores: "86,1 mil",
            publicado: "2020-09-02",
            actualizado: "2026-09-28",
            galeria: &["Comparación de cuadros por segundo", "Opciones de vídeo", "Mundo con distancia de dibujo alta"],
            opcionales: &["iris"],
            incompatibles: &[("optifine", "Reemplaza el mismo motor de dibujo")],
            ..base
        },
        "lithium" => Extras {
            seguidores: "54,7 mil",
            galeria: &["Rendimiento del servidor", "Opciones"],
            incompatibles: &[("starlight", "Modifica la misma iluminación")],
            ..base
        },
        "iris" => Extras {
            entorno: "Cliente",
            seguidores: "61,3 mil",
            actualizado: "2026-09-30",
            galeria: &["Shader de agua", "Menú de shaders", "Atardecer", "Cuevas"],
            creador_tipo: "Persona",
            creador_proyectos: 3,
            creador_descargas: "33,9 M",
            creador_top: ["Iris Shaders", "Sodium Extra", "Colorwheel"],
            ..base
        },
        "modmenu" => Extras {
            entorno: "Cliente",
            seguidores: "29,8 mil",
            galeria: &["Lista de mods"],
            opcionales: &["sodium"],
            creador_tipo: "Organización",
            creador_proyectos: 11,
            creador_descargas: "58,7 M",
            creador_top: ["Mod Menu", "Cloth Config", "Lamb Dynamic Lights"],
            ..base
        },
        "fabric-api" => Extras {
            seguidores: "112 mil",
            creador_tipo: "Organización",
            creador_proyectos: 9,
            creador_descargas: "210 M",
            creador_top: ["Fabric API", "Fabric Loader", "Fabric Language Kotlin"],
            ..base
        },
        "create" => Extras {
            seguidores: "73,4 mil",
            galeria: &["Planta de engranajes", "Tren sobre un puente", "Prensa y cintas"],
            incompatibles: &[("optifine", "Rompe el dibujo de las máquinas")],
            creador_tipo: "Organización",
            creador_proyectos: 4,
            creador_descargas: "61,0 M",
            creador_top: ["Create", "Create Fabric", "Flywheel"],
            ..base
        },
        "faithful" => Extras {
            seguidores: "34,0 mil",
            galeria: &["Bloques de construcción", "Interfaz", "Paisaje"],
            entorno: "Cliente",
            ..base
        },
        "soft-fantasy" => Extras {
            seguidores: "5,2 mil",
            galeria: &["Aldea", "Bosque encantado"],
            entorno: "Cliente",
            ..base
        },
        "bare-bones" => Extras {
            seguidores: "18,8 mil",
            galeria: &["Mundo plano", "Cuevas"],
            entorno: "Cliente",
            ..base
        },
        "fresh-animations" => Extras {
            seguidores: "11,1 mil",
            galeria: &["Caminar y mirar", "Combate"],
            incompatibles: &[("optifine", "No admite modelos personalizados")],
            entorno: "Cliente",
            ..base
        },
        "stay-true" => Extras {
            seguidores: "7,4 mil",
            entorno: "Cliente",
            ..base
        },
        _ => base,
    }
}
