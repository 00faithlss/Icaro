// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Explorador de archivos real de una instancia de demostración: lee y
//! escribe en una carpeta del directorio temporal para que el árbol, la
//! edición de texto y las acciones de archivo funcionen de verdad.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use icaro_ui::componentes::Estado;
use icaro_ui::editor::Nodo;
use iced::widget::text_editor::{Action, Content};

/// Extensiones que se pueden editar como texto.
const EDITABLES: &[&str] = &[
    "txt", "json", "json5", "properties", "toml", "cfg", "conf", "ini", "log", "yml", "yaml",
    "mcmeta", "snbt", "md",
];

#[derive(Debug, Clone)]
struct Entrada {
    ruta: PathBuf,
    nombre: String,
    profundidad: usize,
    carpeta: bool,
    bloqueado: bool,
}

pub struct Explorador {
    raiz: PathBuf,
    abiertas: HashSet<PathBuf>,
    entradas: Vec<Entrada>,
    pub nodos: Vec<Nodo>,
    pub elegido: Option<usize>,
    pub contenido: Option<Content>,
    original: String,
    pub nota: Option<(Estado, String)>,
    pub renombrando: Option<String>,
}

impl Explorador {
    /// Abre (y crea si falta) la carpeta de demostración de una instancia.
    pub fn abrir(instancia: &str) -> Self {
        let raiz = std::env::temp_dir().join("icaro-demo").join(instancia);
        crear_demo(&raiz);
        let mut e = Self {
            abiertas: HashSet::from([raiz.join("mods"), raiz.join("config")]),
            raiz,
            entradas: Vec::new(),
            nodos: Vec::new(),
            elegido: None,
            contenido: None,
            original: String::new(),
            nota: None,
            renombrando: None,
        };
        e.refrescar();
        e
    }

    fn reconstruir_nodos(&mut self) {
        self.nodos = self
            .entradas
            .iter()
            .map(|e| Nodo {
                nombre: e.nombre.clone(),
                profundidad: e.profundidad,
                carpeta: e.carpeta,
                abierta: self.abiertas.contains(&e.ruta),
                bloqueado: e.bloqueado,
            })
            .collect();
    }

    fn refrescar(&mut self) {
        let mut entradas = Vec::new();
        listar(&self.raiz, 0, &self.abiertas, &mut entradas);
        self.entradas = entradas;
        self.reconstruir_nodos();
    }

    fn ruta_elegida(&self) -> Option<PathBuf> {
        self.elegido
            .and_then(|i| self.entradas.get(i))
            .map(|e| e.ruta.clone())
    }

    /// Elige una entrada: las carpetas se abren o se cierran y los archivos
    /// de texto se cargan en el editor.
    pub fn elegir(&mut self, i: usize) {
        let Some(entrada) = self.entradas.get(i).cloned() else {
            return;
        };
        self.renombrando = None;
        self.elegido = Some(i);
        if entrada.carpeta {
            if !self.abiertas.remove(&entrada.ruta) {
                self.abiertas.insert(entrada.ruta.clone());
            }
            self.refrescar();
            self.contenido = None;
            self.nota = None;
        } else {
            self.cargar(&entrada.ruta);
        }
    }

    fn cargar(&mut self, ruta: &Path) {
        self.nota = None;
        self.contenido = None;
        let editable = ruta
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| EDITABLES.contains(&e.to_lowercase().as_str()));
        if !editable {
            self.nota = Some((Estado::Info, "Este archivo no es de texto y no se puede editar.".into()));
            return;
        }
        match fs::read_to_string(ruta) {
            Ok(texto) => {
                let contenido = Content::with_text(&texto);
                self.original = contenido.text();
                self.contenido = Some(contenido);
                self.validar();
            }
            Err(e) => {
                self.nota = Some((Estado::Error, format!("No se pudo leer: {e}")));
            }
        }
    }

    pub fn modificado(&self) -> bool {
        self.contenido
            .as_ref()
            .is_some_and(|c| c.text() != self.original)
    }

    pub fn editar(&mut self, accion: Action) {
        if let Some(c) = self.contenido.as_mut() {
            c.perform(accion);
        }
        self.validar();
    }

    /// Avisa de un JSON mal formado mientras se edita.
    fn validar(&mut self) {
        self.nota = None;
        let (Some(ruta), Some(c)) = (self.ruta_elegida(), self.contenido.as_ref()) else {
            return;
        };
        if ruta.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Err(e) = serde_json::from_str::<serde_json::Value>(&c.text()) {
                self.nota = Some((
                    Estado::Error,
                    format!("JSON inválido: línea {}, columna {}", e.line(), e.column()),
                ));
            }
        }
    }

    pub fn guardar(&mut self) {
        let (Some(ruta), Some(c)) = (self.ruta_elegida(), self.contenido.as_ref()) else {
            return;
        };
        let texto = c.text();
        match fs::write(&ruta, &texto) {
            Ok(()) => {
                self.original = texto;
                self.validar();
            }
            Err(e) => self.nota = Some((Estado::Error, format!("No se pudo guardar: {e}"))),
        }
    }

    pub fn descartar(&mut self) {
        if let Some(ruta) = self.ruta_elegida() {
            self.cargar(&ruta);
        }
    }

    pub fn ruta_texto(&self) -> Option<String> {
        self.ruta_elegida().map(|r| r.display().to_string())
    }

    /// Abre la carpeta o el archivo con el sistema.
    pub fn abrir_en_sistema(&self) {
        let Some(ruta) = self.ruta_elegida() else {
            return;
        };
        let programa = if cfg!(target_os = "windows") {
            "explorer"
        } else if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        let _ = std::process::Command::new(programa).arg(ruta).spawn();
    }

    pub fn iniciar_renombrar(&mut self) {
        if let Some(e) = self.elegido.and_then(|i| self.entradas.get(i)) {
            if !e.bloqueado {
                self.renombrando = Some(e.nombre.clone());
            }
        }
    }

    pub fn escribir_nombre(&mut self, nombre: String) {
        if self.renombrando.is_some() {
            self.renombrando = Some(nombre);
        }
    }

    pub fn confirmar_nombre(&mut self) {
        let (Some(ruta), Some(nuevo)) = (self.ruta_elegida(), self.renombrando.take()) else {
            return;
        };
        let nuevo = nuevo.trim();
        if nuevo.is_empty() || nuevo.contains(['/', '\\']) {
            self.nota = Some((Estado::Error, "El nombre no es válido.".into()));
            return;
        }
        let destino = ruta.with_file_name(nuevo);
        if destino.exists() {
            self.nota = Some((Estado::Error, "Ya existe un elemento con ese nombre.".into()));
            return;
        }
        match fs::rename(&ruta, &destino) {
            Ok(()) => {
                if self.abiertas.remove(&ruta) {
                    self.abiertas.insert(destino.clone());
                }
                self.refrescar();
                self.elegido = self.entradas.iter().position(|e| e.ruta == destino);
                self.contenido = None;
                if let Some(i) = self.elegido {
                    if !self.entradas[i].carpeta {
                        let r = self.entradas[i].ruta.clone();
                        self.cargar(&r);
                    }
                }
            }
            Err(e) => self.nota = Some((Estado::Error, format!("No se pudo renombrar: {e}"))),
        }
    }

    pub fn cancelar_nombre(&mut self) {
        self.renombrando = None;
    }

    pub fn duplicar(&mut self) {
        let Some(ruta) = self.ruta_elegida() else {
            return;
        };
        let destino = nombre_libre(&ruta);
        let resultado = if ruta.is_dir() {
            copiar_carpeta(&ruta, &destino)
        } else {
            fs::copy(&ruta, &destino).map(|_| ())
        };
        match resultado {
            Ok(()) => {
                self.refrescar();
                self.elegido = self.entradas.iter().position(|e| e.ruta == destino);
            }
            Err(e) => self.nota = Some((Estado::Error, format!("No se pudo duplicar: {e}"))),
        }
    }

    /// Mueve el elemento a la papelera interna de la instancia.
    pub fn a_la_papelera(&mut self) {
        let Some(ruta) = self.ruta_elegida() else {
            return;
        };
        let papelera = self.raiz.join(".papelera");
        let destino = nombre_libre(&papelera.join(ruta.file_name().unwrap_or_default()));
        let resultado = fs::create_dir_all(&papelera).and_then(|_| fs::rename(&ruta, &destino));
        match resultado {
            Ok(()) => {
                self.elegido = None;
                self.contenido = None;
                self.nota = None;
                self.refrescar();
            }
            Err(e) => self.nota = Some((Estado::Error, format!("No se pudo mover: {e}"))),
        }
    }
}

fn listar(dir: &Path, profundidad: usize, abiertas: &HashSet<PathBuf>, salida: &mut Vec<Entrada>) {
    let Ok(lectura) = fs::read_dir(dir) else {
        return;
    };
    let mut hijos: Vec<(PathBuf, bool)> = lectura
        .flatten()
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .map(|e| {
            let ruta = e.path();
            let carpeta = ruta.is_dir();
            (ruta, carpeta)
        })
        .collect();
    // Carpetas primero y luego por nombre.
    hijos.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| a.0.file_name().cmp(&b.0.file_name()))
    });
    for (ruta, carpeta) in hijos {
        let nombre = ruta
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let bloqueado = !carpeta && nombre.ends_with(".jar");
        salida.push(Entrada {
            ruta: ruta.clone(),
            nombre,
            profundidad,
            carpeta,
            bloqueado,
        });
        if carpeta && abiertas.contains(&ruta) {
            listar(&ruta, profundidad + 1, abiertas, salida);
        }
    }
}

/// `nombre copia.ext`, `nombre copia 2.ext`… hasta encontrar uno libre.
fn nombre_libre(ruta: &Path) -> PathBuf {
    if !ruta.exists() {
        return ruta.to_path_buf();
    }
    let base = ruta.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let ext = ruta.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    let mut n = 1;
    loop {
        let sufijo = if n == 1 { " copia".to_owned() } else { format!(" copia {n}") };
        let candidato = ruta.with_file_name(format!("{base}{sufijo}{ext}"));
        if !candidato.exists() {
            return candidato;
        }
        n += 1;
    }
}

fn copiar_carpeta(origen: &Path, destino: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destino)?;
    for e in fs::read_dir(origen)?.flatten() {
        let hacia = destino.join(e.file_name());
        if e.path().is_dir() {
            copiar_carpeta(&e.path(), &hacia)?;
        } else {
            fs::copy(e.path(), hacia)?;
        }
    }
    Ok(())
}

fn crear_demo(raiz: &Path) {
    let archivos: &[(&str, &[u8])] = &[
        ("options.txt", b"version:3955\nautoJump:false\nrenderDistance:16\nresourcePacks:[\"file/Faithful 32x.zip\"]\n"),
        ("mods/fabric-api-0.112.0.jar", b"PK\x03\x04"),
        ("mods/sodium-0.6.5.jar", b"PK\x03\x04"),
        (
            "config/sodium-options.json",
            b"{\n  \"quality\": {\n    \"weather_quality\": \"DEFAULT\",\n    \"leaves_quality\": \"DEFAULT\"\n  },\n  \"advanced\": {\n    \"enable_memory_tracing\": false\n  }\n}\n",
        ),
        ("config/fabric_loader_dependencies.json", b"{\n  \"version\": 1,\n  \"overrides\": {}\n}\n"),
        ("config/servers.properties", b"motd=Servidor de pruebas\nmax-players=10\n"),
        ("logs/latest.log", b"[14:02:11] [main/INFO]: Loading Minecraft 1.21.4 with Fabric Loader 0.16.9\n"),
    ];
    for (rel, contenido) in archivos {
        let ruta = raiz.join(rel);
        if ruta.exists() {
            continue;
        }
        if let Some(padre) = ruta.parent() {
            let _ = fs::create_dir_all(padre);
        }
        let _ = fs::write(&ruta, contenido);
    }
}
