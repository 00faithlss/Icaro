// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Lista de mods como texto: no instala, desactiva ni modifica los JAR.
use crate::{instancias::Almacen, modrinth::Catalog, mods};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModMeta {
    pub name: String,
    pub version: Option<String>,
    pub authors: Vec<String>,
    pub url: Option<String>,
    pub filename: String,
    pub activo: bool,
    #[serde(default)]
    pub project_id: String,
    #[serde(default)]
    pub sha512: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Formato {
    Html,
    Markdown,
    Texto,
    Json,
    Csv,
    Personalizado,
}
impl Formato {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Html => "html",
            Self::Markdown => "md",
            Self::Json => "json",
            Self::Csv => "csv",
            _ => "txt",
        }
    }
}
#[derive(Debug, Clone)]
pub struct Opciones {
    pub formato: Formato,
    pub campos: [bool; 4],
    pub solo_activos: bool,
    pub plantilla: String,
}
impl Default for Opciones {
    fn default() -> Self {
        Self {
            formato: Formato::Html,
            campos: [true, true, true, false],
            solo_activos: true,
            plantilla: String::new(),
        }
    }
}
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn md(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if matches!(c, '\\' | '[' | ']' | '*' | '_' | '`' | '<' | '>') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}
fn csv(s: &str) -> String {
    // CSV quoting alone does not stop spreadsheet applications from evaluating a cell.
    let seguro = if matches!(
        s.chars().next(),
        Some('=' | '+' | '-' | '@' | '\t' | '\r' | '\n')
    ) {
        format!("'{s}")
    } else {
        s.to_owned()
    };
    if s.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", seguro.replace('"', "\"\""))
    } else {
        seguro
    }
}
fn columnas(o: &Opciones) -> Vec<&'static str> {
    let mut fields = vec!["name"];
    for (n, f) in ["version", "authors", "url", "filename"].iter().enumerate() {
        if o.campos[n] {
            fields.push(*f);
        }
    }
    fields
}
fn campo(m: &ModMeta, k: &str) -> String {
    match k {
        "name" => m.name.clone(),
        "version" => m.version.clone().unwrap_or_default(),
        "authors" => m.authors.join(", "),
        "url" => m.url.clone().unwrap_or_default(),
        "filename" => m.filename.clone(),
        _ => String::new(),
    }
}
pub fn plantilla(o: &Opciones) -> String {
    if o.formato == Formato::Personalizado {
        return o.plantilla.clone();
    }
    if matches!(o.formato, Formato::Json | Formato::Csv) {
        return String::new();
    }
    let mut s = match o.formato {
        Formato::Html => {
            if o.campos[2] {
                "<li><a href=\"{url}\">{name}</a>"
            } else {
                "<li>{name}"
            }
        }
        Formato::Markdown => {
            if o.campos[2] {
                "- [{name}]({url})"
            } else {
                "- {name}"
            }
        }
        _ => "{name}",
    }
    .to_owned();
    if o.campos[0] {
        s.push_str(" [{version}]");
    }
    if o.campos[1] {
        s.push_str(" por {authors}");
    }
    if o.campos[2] && o.formato == Formato::Texto {
        s.push_str(" <{url}>");
    }
    if o.campos[3] {
        s.push_str(if o.formato == Formato::Markdown {
            " (`{filename}`)"
        } else {
            " ({filename})"
        });
    }
    if o.formato == Formato::Html {
        s.push_str("</li>");
    }
    s
}
pub fn renderizar(mods: &[ModMeta], o: &Opciones) -> String {
    let selected: Vec<_> = mods
        .iter()
        .filter(|m| !o.solo_activos || m.activo)
        .collect();
    if o.formato == Formato::Json {
        let values: Vec<_> = selected
            .into_iter()
            .map(|m| {
                let mut v = serde_json::Map::new();
                for f in columnas(o) {
                    let value = if f == "authors" {
                        if m.authors.is_empty() {
                            serde_json::Value::Null
                        } else {
                            serde_json::json!(m.authors)
                        }
                    } else {
                        let s = campo(m, f);
                        if s.is_empty() {
                            serde_json::Value::Null
                        } else {
                            serde_json::json!(s)
                        }
                    };
                    v.insert(f.into(), value);
                }
                serde_json::Value::Object(v)
            })
            .collect();
        return serde_json::to_string_pretty(&values).unwrap_or_else(|_| "[]".into());
    }
    if o.formato == Formato::Csv {
        let fields = columnas(o);
        let mut rows = vec![fields.join(",")];
        for m in selected {
            rows.push(
                fields
                    .iter()
                    .map(|f| csv(&campo(m, f)))
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }
        return rows.join("\n");
    }
    let mut lines = vec![];
    for m in selected {
        if o.formato == Formato::Personalizado {
            // Un solo recorrido: los datos del mod no se interpretan como variables.
            let mut out = String::new();
            let mut rest = o.plantilla.as_str();
            while let Some(p) = rest.find('{') {
                out.push_str(&rest[..p]);
                rest = &rest[p..];
                if let Some(end) = rest.find('}') {
                    let key = &rest[1..end];
                    if matches!(key, "name" | "version" | "authors" | "url" | "filename") {
                        out.push_str(&campo(m, key));
                        rest = &rest[end + 1..];
                        continue;
                    }
                }
                out.push('{');
                rest = &rest[1..];
            }
            out.push_str(rest);
            lines.push(out);
            continue;
        }
        let escape = |s: &str| match o.formato {
            Formato::Html => esc(s),
            Formato::Markdown => md(s),
            _ => s.into(),
        };
        let url = m.url.as_deref().filter(|s| {
            (s.starts_with("https://") || s.starts_with("http://"))
                && !s.chars().any(char::is_control)
        });
        let mut line = if o.campos[2] && url.is_some() {
            match o.formato {
                Formato::Html => format!("<a href=\"{}\">{}</a>", esc(url.unwrap()), esc(&m.name)),
                Formato::Markdown => format!(
                    "[{}]({})",
                    md(&m.name),
                    url.unwrap()
                        .replace(' ', "%20")
                        .replace(')', "%29")
                        .replace('(', "%28")
                        .replace('\\', "%5C")
                        .replace('<', "%3C")
                        .replace('>', "%3E")
                        .replace('[', "%5B")
                        .replace(']', "%5D")
                        .replace('"', "%22")
                ),
                _ => m.name.clone(),
            }
        } else {
            escape(&m.name)
        };
        if o.campos[0] {
            if let Some(v) = m.version.as_deref().filter(|s| !s.is_empty()) {
                line.push_str(&format!(" [{}]", escape(v)));
            }
        }
        if o.campos[1] && !m.authors.is_empty() {
            line.push_str(&format!(" por {}", escape(&m.authors.join(", "))));
        }
        if o.campos[2] && o.formato == Formato::Texto {
            if let Some(url) = url {
                line.push_str(&format!(" <{url}>"));
            }
        }
        if o.campos[3] && !m.filename.is_empty() {
            line.push_str(&format!(
                " ({})",
                if o.formato == Formato::Markdown {
                    format!("`{}`", m.filename.replace('`', "\\`"))
                } else {
                    escape(&m.filename)
                }
            ));
        }
        lines.push(match o.formato {
            Formato::Html => format!("  <li>{line}</li>"),
            Formato::Markdown => format!("- {line}"),
            _ => line,
        });
    }
    if o.formato == Formato::Html {
        format!("<ul>\n{}\n</ul>", lines.join("\n"))
    } else {
        lines.join("\n")
    }
}
fn normal(path: &Path) -> Result<fs::Metadata, String> {
    let m = fs::symlink_metadata(path).map_err(|_| "No se pudo leer un mod.")?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err("Los mods deben estar en archivos locales, sin enlaces.".into());
        }
    }
    if !m.is_file() || m.file_type().is_symlink() {
        return Err("No se pudo leer un mod local.".into());
    }
    Ok(m)
}
fn hash(path: &Path) -> Result<String, String> {
    let m = normal(path)?;
    if m.len() > 128 * 1024 * 1024 {
        return Err("Un mod supera los 128 MB admitidos para consultar sus metadatos.".into());
    }
    let mut file = fs::File::open(path).map_err(|_| "No se pudo leer un mod.")?;
    let mut sha = Sha512::new();
    let mut buf = [0u8; 65536];
    let mut total = 0;
    loop {
        let n = file.read(&mut buf).map_err(|_| "No se pudo leer un mod.")?;
        if n == 0 {
            break;
        }
        total += n as u64;
        if total > 128 * 1024 * 1024 {
            return Err("Un mod cambió durante la lectura.".into());
        }
        sha.update(&buf[..n]);
    }
    Ok(format!("{:x}", sha.finalize()))
}
pub fn leer(store: &Almacen, id: &str) -> Result<Vec<ModMeta>, String> {
    let game = store.juego(id).map_err(|e| e.to_string())?;
    let root = store.carpeta(id).map_err(|e| e.to_string())?;
    let cached = root.join("mod-list-meta.json");
    let cache: Vec<ModMeta> = if cached.exists() {
        let m = normal(&cached)?;
        if m.len() > 2 * 1024 * 1024 {
            return Err("El archivo de metadatos de lista es demasiado grande.".into());
        }
        serde_json::from_slice(&fs::read(cached).map_err(|_| "No se pudo leer los metadatos.")?)
            .unwrap_or_default()
    } else {
        vec![]
    };
    let installed = mods::inventario(store, id).map_err(|e| e.to_string())?;
    let mut result = vec![];
    for m in installed {
        let path = game.join("mods").join(if m.activo {
            m.archivo.clone()
        } else {
            format!("{}.disabled", m.archivo)
        });
        let digest = hash(&path)?;
        if let Some(saved) = cache
            .iter()
            .find(|c| c.filename == m.archivo && c.sha512 == digest)
        {
            let mut saved = saved.clone();
            saved.activo = m.activo;
            result.push(saved);
            continue;
        }
        // The manifest describes the installed bytes, not any replacement sharing a filename.
        let manual = m.project_id.is_empty() || !m.sha512.eq_ignore_ascii_case(&digest);
        result.push(ModMeta {
            name: if manual {
                m.archivo.trim_end_matches(".jar").into()
            } else {
                m.nombre
            },
            version: if manual { None } else { Some(m.version) },
            authors: vec![],
            url: if manual {
                None
            } else {
                Some(format!("https://modrinth.com/mod/{}", m.project_id))
            },
            filename: m.archivo,
            activo: m.activo,
            project_id: if manual { String::new() } else { m.project_id },
            sha512: digest,
        });
    }
    result.sort_by_key(|m| m.name.to_lowercase());
    Ok(result)
}
pub async fn completar(
    store: &Almacen,
    id: &str,
    mut items: Vec<ModMeta>,
) -> Result<Vec<ModMeta>, String> {
    let catalog = Catalog::new().map_err(|e| e.to_string())?;
    let mut recognized = BTreeMap::new();
    let hashes: Vec<_> = items
        .iter()
        .filter(|m| m.project_id.is_empty())
        .map(|m| m.sha512.clone())
        .collect();
    for batch in hashes.chunks(512) {
        recognized.extend(
            catalog
                .versions_from_hashes(batch)
                .await
                .map_err(|e| e.to_string())?,
        );
    }
    for m in &mut items {
        if let Some(v) = recognized.get(&m.sha512) {
            m.project_id = v.project_id.clone();
            m.version = Some(v.version_number.clone());
        }
    }
    let ids: std::collections::BTreeSet<_> = items
        .iter()
        .filter(|m| !m.project_id.is_empty() && (m.authors.is_empty() || m.url.is_none()))
        .map(|m| m.project_id.clone())
        .collect();
    let mut tasks = tokio::task::JoinSet::new();
    let mut ids = ids.into_iter();
    let mut projects = BTreeMap::new();
    loop {
        while tasks.len() < 4 {
            let Some(id) = ids.next() else {
                break;
            };
            let c = catalog.clone();
            tasks.spawn(async move {
                let p = c.project(&id).await.map_err(|e| e.to_string())?;
                let members = c.members(&id).await.map_err(|e| e.to_string())?;
                Ok::<_, String>((id, p, members))
            });
        }
        let Some(next) = tasks.join_next().await else {
            break;
        };
        let (id, p, members) = next.map_err(|_| "No se pudieron consultar los metadatos.")??;
        projects.insert(id, (p, members));
    }
    for m in &mut items {
        if let Some((p, authors)) = projects.get(&m.project_id) {
            m.name = p.title.clone();
            m.url = Some(format!("https://modrinth.com/mod/{}", p.slug));
            m.authors = authors
                .iter()
                .map(|a| {
                    a.user
                        .name
                        .clone()
                        .unwrap_or_else(|| a.user.username.clone())
                })
                .collect();
        }
    }
    // El caché se publica únicamente si el archivo instalado sigue siendo el leído.
    let store = store.clone();
    let id = id.to_owned();
    tokio::task::spawn_blocking(move || {
        let mut current = leer(&store, &id)?;
        for m in &mut items {
            let Some(saved) = current
                .iter_mut()
                .find(|c| c.filename == m.filename && c.sha512 == m.sha512)
            else {
                return Err(
                    "Los mods cambiaron durante la consulta. Vuelve a abrir la lista.".into(),
                );
            };
            // Activating/disabling during the network request must not revert the export filter.
            m.activo = saved.activo;
            *saved = m.clone();
        }
        let root = store.carpeta(&id).map_err(|e| e.to_string())?;
        let bytes = serde_json::to_vec_pretty(&current)
            .map_err(|_| "No se pudieron guardar los metadatos.")?;
        if bytes.len() > 2 * 1024 * 1024 {
            return Err("La lista de metadatos es demasiado grande.".into());
        }
        let target = root.join("mod-list-meta.json");
        if target.exists() {
            normal(&target)?;
        }
        let mut nonce = [0u8; 8];
        getrandom::fill(&mut nonce).map_err(|_| "No se pudo guardar los metadatos.")?;
        let temp = root.join(format!("mod-list-meta-{}.next", u64::from_le_bytes(nonce)));
        let result = (|| {
            let mut f = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temp)
                .map_err(|_| "No se pudo guardar los metadatos.")?;
            f.write_all(&bytes)
                .map_err(|_| "No se pudo guardar los metadatos.")?;
            f.sync_all()
                .map_err(|_| "No se pudo guardar los metadatos.")?;
            fs::rename(&temp, &target).map_err(|_| "No se pudo publicar los metadatos.")?;
            Ok(items)
        })();
        let _ = fs::remove_file(temp);
        result
    })
    .await
    .unwrap_or_else(|_| Err("No se pudieron guardar los metadatos.".into()))
}
pub fn guardar(path: &Path, contenido: &str) -> Result<PathBuf, String> {
    let parent = path
        .parent()
        .ok_or("Destino inválido.")?
        .canonicalize()
        .map_err(|_| "Elige una carpeta existente.")?;
    let path = parent.join(path.file_name().ok_or("Nombre de archivo inválido.")?);
    if path.exists() {
        return Err("Ese archivo ya existe; elige otro nombre para conservarlo.".into());
    }
    let mut nonce = [0u8; 8];
    getrandom::fill(&mut nonce).map_err(|_| "No se pudo preparar el archivo.")?;
    let temp = parent.join(format!(".icaro-lista-{}.tmp", u64::from_le_bytes(nonce)));
    let r = (|| {
        let mut f = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp)
            .map_err(|_| "No se pudo guardar la lista.")?;
        f.write_all(contenido.as_bytes())
            .map_err(|_| "No se pudo guardar la lista.")?;
        f.sync_all().map_err(|_| "No se pudo guardar la lista.")?;
        fs::hard_link(&temp, &path).map_err(|_| "No se pudo publicar la lista.")?;
        Ok(path)
    })();
    let _ = fs::remove_file(temp);
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    fn mods() -> Vec<ModMeta> {
        vec![
            ModMeta {
                name: "<Sodium & \"uno\">".into(),
                version: Some("0.6.5".into()),
                authors: vec!["CaffeineMC".into(), "IMS".into()],
                url: Some("https://modrinth.com/mod/sodium".into()),
                filename: "sodium.jar".into(),
                activo: true,
                ..Default::default()
            },
            ModMeta {
                name: "local".into(),
                filename: "local.jar".into(),
                activo: true,
                ..Default::default()
            },
            ModMeta {
                name: "desactivado".into(),
                activo: false,
                ..Default::default()
            },
        ]
    }
    #[test]
    fn formatos_escapan_y_omiten_datos_ausentes_sin_sufijos_sueltos() {
        let ms = mods();
        let mut o = Opciones::default();
        let html = renderizar(&ms, &o);
        assert!(html.contains("&lt;Sodium &amp; &quot;"));
        assert!(html.contains("<li>local</li>"));
        assert!(!html.contains("desactivado"));
        o.formato = Formato::Texto;
        let text = renderizar(&ms, &o);
        assert_eq!(text.lines().last(), Some("local"));
        o.formato = Formato::Markdown;
        assert!(renderizar(&ms, &o).ends_with("- local"));
        o.formato = Formato::Json;
        let v: serde_json::Value = serde_json::from_str(&renderizar(&ms, &o)).unwrap();
        assert!(v[1]["version"].is_null());
        assert_eq!(v[0]["authors"].as_array().unwrap().len(), 2);
        o.formato = Formato::Csv;
        assert!(renderizar(&ms, &o).contains("\"CaffeineMC, IMS\""));
    }
    #[test]
    fn plantilla_no_reinterpreta_variables_en_metadatos_y_campos_fijos() {
        let mut ms = mods();
        ms[0].name = "{version}".into();
        let mut o = Opciones {
            formato: Formato::Personalizado,
            plantilla: "{name}|{version}|{desconocido}".into(),
            ..Default::default()
        };
        assert!(renderizar(&ms, &o).starts_with("{version}|0.6.5|{desconocido}"));
        o.formato = Formato::Json;
        o.campos = [false; 4];
        let value: serde_json::Value = serde_json::from_str(&renderizar(&ms, &o)).unwrap();
        assert_eq!(value[0].as_object().unwrap().len(), 1);
    }
    #[test]
    fn csv_impide_formulas_y_preserva_metadatos_en_otros_formatos() {
        for celda in [
            "=1+1", "+SUM(A1)", "-2+3", "@SUM(A1)", "\t=1+1", "\r=1+1", "\n=1+1",
        ] {
            let encoded = csv(celda);
            assert!(encoded.starts_with('\'') || encoded.starts_with("\"'"));
            let ms = vec![ModMeta {
                name: celda.into(),
                activo: true,
                ..Default::default()
            }];
            let mut o = Opciones {
                formato: Formato::Json,
                campos: [false; 4],
                ..Default::default()
            };
            let json: serde_json::Value = serde_json::from_str(&renderizar(&ms, &o)).unwrap();
            assert_eq!(json[0]["name"], celda);
            o.formato = Formato::Texto;
            assert_eq!(renderizar(&ms, &o), celda);
        }
        assert_eq!(
            csv("=HYPERLINK(\"url\",\"name\")"),
            "\"'=HYPERLINK(\"\"url\"\",\"\"name\"\")\""
        );
        assert_eq!(csv("Minecraft 26.3"), "Minecraft 26.3");
    }
    #[test]
    fn enlaces_descartan_esquemas_activos_controles_y_escapan_delimitadores_markdown() {
        let mut ms = mods();
        for url in [
            "javascript:alert(1)",
            "https://example.test/\n[link](javascript:alert(1))",
            "https://example.test/\tpath",
        ] {
            ms[0].url = Some(url.into());
            let mut o = Opciones::default();
            assert!(!renderizar(&ms, &o).contains("<a href="));
            o.formato = Formato::Markdown;
            assert!(!renderizar(&ms, &o).contains("](https://"));
        }
        ms[0].url = Some("https://example.test/\\<file>(draft)[1]\"".into());
        let o = Opciones {
            formato: Formato::Markdown,
            ..Default::default()
        };
        assert!(
            renderizar(&ms, &o).contains("https://example.test/%5C%3Cfile%3E%28draft%29%5B1%5D%22")
        );
    }
    #[test]
    fn jar_reemplazado_no_hereda_metadatos_del_manifest_ni_cache_anterior() {
        let mut nonce = [0u8; 8];
        getrandom::fill(&mut nonce).unwrap();
        let root =
            std::env::temp_dir().join(format!("icaro-lista-hash-{}", u64::from_le_bytes(nonce)));
        let store = Almacen::nuevo(root.join("instances"));
        let i = store.crear("Prueba", "1.21.1", 0).unwrap();
        let dir = store.juego(&i.id).unwrap().join("mods");
        fs::create_dir(&dir).unwrap();
        let path = dir.join("sodium.jar");
        fs::write(&path, b"jar original").unwrap();
        let digest = hash(&path).unwrap();
        let manifest = serde_json::to_vec(&vec![mods::Instalado {
            project_id: "AANobbMI".into(),
            nombre: "Sodium".into(),
            version_id: "version-real".into(),
            version: "0.6.5".into(),
            publicada: String::new(),
            archivo: "sodium.jar".into(),
            sha512: digest.clone(),
            activo: true,
            dependencias: vec![],
        }])
        .unwrap();
        fs::write(dir.join(".icaro-mods.json"), &manifest).unwrap();
        let known = leer(&store, &i.id).unwrap();
        assert_eq!(known[0].version.as_deref(), Some("0.6.5"));
        let mut cached = known;
        cached[0].authors = vec!["CaffeineMC".into()];
        fs::write(
            store.carpeta(&i.id).unwrap().join("mod-list-meta.json"),
            serde_json::to_vec(&cached).unwrap(),
        )
        .unwrap();
        fs::write(&path, b"jar reemplazado").unwrap();
        let unknown = leer(&store, &i.id).unwrap();
        assert_eq!(unknown[0].name, "sodium");
        assert!(unknown[0].version.is_none());
        assert!(unknown[0].authors.is_empty());
        assert!(unknown[0].url.is_none());
        assert!(unknown[0].project_id.is_empty());
        assert_ne!(unknown[0].sha512, digest);
        assert_eq!(fs::read(&path).unwrap(), b"jar reemplazado");
        assert_eq!(fs::read(dir.join(".icaro-mods.json")).unwrap(), manifest);
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        fs::remove_dir_all(root).unwrap();
    }
}
