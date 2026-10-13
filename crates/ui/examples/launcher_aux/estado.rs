// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Persistencia en disco de lo que el usuario organiza: grupos, a qué grupo
//! pertenece cada instancia y la configuración de Java de cada una.

use std::fs;
use std::path::PathBuf;

use icaro_ui::editor::ConfigJava;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct JavaGuardado {
    java: Option<String>,
    memoria_min: u32,
    memoria_max: u32,
    metaspace: u32,
    argumentos: String,
}

#[derive(Serialize, Deserialize)]
pub struct Guardado {
    #[serde(default)]
    pub apariencia: AparienciaGuardada,
    #[serde(default)]
    pub ids: Vec<String>,
    pub lista_grupos: Vec<String>,
    pub grupos: Vec<Option<String>>,
    pub cerrados: Vec<Option<String>>,
    /// Modpacks instalados como instancias, por nombre.
    #[serde(default)]
    pub modpacks: Vec<String>,
    configs: Vec<JavaGuardado>,
}

/// Preferencias de Apariencia compatibles con estados anteriores.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AparienciaGuardada {
    pub tema: usize,
    pub metal: usize,
    pub escala: i32,
    pub reducir: bool,
}

impl Default for AparienciaGuardada {
    fn default() -> Self { Self { tema: 0, metal: 0, escala: 100, reducir: false } }
}

impl AparienciaGuardada {
    pub fn normalizar(&mut self) {
        self.tema = self.tema.min(2);
        self.metal = self.metal.min(2);
        self.escala = self.escala.clamp(80, 150);
    }
}

pub fn directorio(demo: bool) -> PathBuf {
    if demo {
        return std::env::temp_dir().join("icaro-demo");
    }
    #[cfg(windows)]
    let base = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(not(windows))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")));
    base.unwrap_or_else(std::env::temp_dir).join("Icaro")
}

fn ruta(demo: bool) -> PathBuf {
    directorio(demo).join("estado.json")
}

impl Guardado {
    pub fn nuevo(
        lista_grupos: &[String],
        grupos: &[Option<String>],
        cerrados: &[Option<String>],
        configs: &[ConfigJava],
        modpacks: &[String],
    ) -> Self {
        Self {
            apariencia: AparienciaGuardada::default(),
            ids: Vec::new(),
            lista_grupos: lista_grupos.to_vec(),
            grupos: grupos.to_vec(),
            cerrados: cerrados.to_vec(),
            modpacks: modpacks.to_vec(),
            configs: configs
                .iter()
                .map(|c| JavaGuardado {
                    java: c.java.clone(),
                    memoria_min: c.memoria_min,
                    memoria_max: c.memoria_max,
                    metaspace: c.metaspace,
                    argumentos: c.argumentos.clone(),
                })
                .collect(),
        }
    }

    /// Asocia ajustes a la identidad de la instancia, no a su posición en la lista.
    pub fn asociar(&mut self, ids: Vec<String>) {
        self.ids = ids;
    }

    pub fn restaurar_instancias(&mut self, ids: Vec<String>) {
        if self.ids.is_empty() {
            self.ids = ids;
            return;
        }
        let indices: Vec<_> = ids.iter().map(|id| self.ids.iter().position(|i| i == id)).collect();
        self.grupos = indices.iter().map(|i| i.and_then(|i| self.grupos.get(i).cloned()).flatten()).collect();
        self.configs = indices.iter().map(|i| i.and_then(|i| self.configs.get(i).cloned()).unwrap_or_else(|| {
            let c = crate::config_por_defecto();
            JavaGuardado { java: c.java, memoria_min: c.memoria_min, memoria_max: c.memoria_max, metaspace: c.metaspace, argumentos: c.argumentos }
        })).collect();
        self.ids = ids;
    }

    pub fn configs(&self) -> Vec<ConfigJava> {
        self.configs
            .iter()
            .map(|c| ConfigJava {
                java: c.java.clone(),
                memoria_min: c.memoria_min,
                memoria_max: c.memoria_max,
                metaspace: c.metaspace,
                argumentos: c.argumentos.clone(),
            })
            .collect()
    }

    pub fn cargar(demo: bool) -> Option<Self> {
        serde_json::from_str(&fs::read_to_string(ruta(demo)).ok()?).ok()
    }

    /// Serializa el estado; el llamador decide si escribirlo.
    pub fn texto(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn escribir(texto: &str, demo: bool) {
        let r = ruta(demo);
        if let Some(padre) = r.parent() {
            let _ = fs::create_dir_all(padre);
        }
        let _ = fs::write(r, texto);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn estados_anteriores_restauran_apariencia_por_defecto() {
        let a: Guardado = serde_json::from_str(r#"{"lista_grupos":[],"grupos":[],"cerrados":[],"configs":[]}"#).unwrap();
        assert_eq!(a.apariencia.escala, 100);
        assert_eq!(a.apariencia.tema, 0);
        assert_eq!(a.apariencia.metal, 0);
        let mut b: AparienciaGuardada = serde_json::from_str(r#"{"tema":44,"escala":300}"#).unwrap();
        b.normalizar();
        assert_eq!((b.tema, b.metal, b.escala), (2, 0, 150));
    }
}
