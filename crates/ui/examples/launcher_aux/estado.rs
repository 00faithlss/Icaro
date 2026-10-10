// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Persistencia en disco de lo que el usuario organiza: grupos, a qué grupo
//! pertenece cada instancia y la configuración de Java de cada una.

use std::fs;
use std::path::PathBuf;

use icaro_ui::editor::ConfigJava;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct JavaGuardado {
    java: Option<String>,
    memoria_min: u32,
    memoria_max: u32,
    metaspace: u32,
    argumentos: String,
}

#[derive(Serialize, Deserialize)]
pub struct Guardado {
    pub lista_grupos: Vec<String>,
    pub grupos: Vec<Option<String>>,
    pub cerrados: Vec<Option<String>>,
    configs: Vec<JavaGuardado>,
}

fn ruta() -> PathBuf {
    std::env::temp_dir().join("icaro-demo").join("estado.json")
}

impl Guardado {
    pub fn nuevo(
        lista_grupos: &[String],
        grupos: &[Option<String>],
        cerrados: &[Option<String>],
        configs: &[ConfigJava],
    ) -> Self {
        Self {
            lista_grupos: lista_grupos.to_vec(),
            grupos: grupos.to_vec(),
            cerrados: cerrados.to_vec(),
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

    pub fn cargar() -> Option<Self> {
        serde_json::from_str(&fs::read_to_string(ruta()).ok()?).ok()
    }

    /// Serializa el estado; el llamador decide si escribirlo.
    pub fn texto(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn escribir(texto: &str) {
        let r = ruta();
        if let Some(padre) = r.parent() {
            let _ = fs::create_dir_all(padre);
        }
        let _ = fs::write(r, texto);
    }
}
