// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Actualizaciones publicadas: metadatos de GitHub, descarga acotada y SHA-256.
//! Solo admite el ejecutable portable declarado en latest.json de una release estable.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::io::AsyncWriteExt;

const RELEASE: &str = "https://api.github.com/repos/00faithlss/Icaro-launcher/releases/latest";
const MAX_ARCHIVO: u64 = 1024 * 1024 * 1024;
const PLATAFORMA: &str = "windows-x86_64";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Nota {
    pub tipo: String,
    pub texto: String,
}
#[derive(Debug, Clone, Deserialize)]
struct Manifest {
    version: String,
    #[serde(default)]
    notas: Vec<Nota>,
    plataformas: BTreeMap<String, Artefacto>,
}
#[derive(Debug, Clone, Deserialize)]
struct Artefacto {
    archivo: String,
    sha256: String,
}
#[derive(Debug, Clone, Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}
#[derive(Debug, Clone, Deserialize)]
struct Asset {
    name: String,
    size: u64,
    browser_download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disponible {
    pub version: String,
    pub notas: Vec<Nota>,
    pub bytes: u64,
    url: String,
    sha256: String,
}
#[derive(Debug, Clone)]
pub struct Preparada {
    pub version: String,
    archivo: PathBuf,
    sha256: String,
    bytes: u64,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("No se pudo comprobar la actualización. Revisa tu conexión e inténtalo de nuevo.")]
    Red,
    #[error("La versión publicada aún no incluye una actualización portable para este equipo.")]
    SinArtefacto,
    #[error("La información de la actualización no es válida.")]
    Datos,
    #[error("La descarga no pasó la verificación. No se ha cambiado Ícaro.")]
    Integridad,
    #[error("No se pudo guardar o aplicar la actualización.")]
    Disco,
    #[error("La actualización automática todavía no está disponible para este sistema.")]
    Plataforma,
}
fn cliente() -> Result<reqwest::Client, Error> {
    reqwest::Client::builder()
        .https_only(true)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(600))
        .redirect(reqwest::redirect::Policy::limited(5))
        .user_agent(crate::http::USER_AGENT)
        .build()
        .map_err(|_| Error::Red)
}
fn numeros(v: &str) -> Option<[u64; 3]> {
    let partes: Vec<_> = v.strip_prefix('v').unwrap_or(v).split('.').collect();
    if partes.len() != 3 {
        return None;
    }
    Some([
        partes[0].parse().ok()?,
        partes[1].parse().ok()?,
        partes[2].parse().ok()?,
    ])
}
fn asset_valido(url: &str, tag: &str, nombre: &str) -> bool {
    if nombre.is_empty()
        || !nombre
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return false;
    }
    url == format!("https://github.com/00faithlss/Icaro-launcher/releases/download/{tag}/{nombre}")
}
fn resolver(
    release: Release,
    manifest: Manifest,
    actual: &str,
) -> Result<Option<Disponible>, Error> {
    if release.draft || release.prerelease {
        return Ok(None);
    }
    let actual = numeros(actual).ok_or(Error::Datos)?;
    let version = numeros(&manifest.version).ok_or(Error::Datos)?;
    if numeros(&release.tag_name) != Some(version) {
        return Err(Error::Datos);
    }
    if version <= actual {
        return Ok(None);
    }
    let artefacto = manifest
        .plataformas
        .get(PLATAFORMA)
        .ok_or(Error::SinArtefacto)?;
    if !artefacto.archivo.ends_with(".exe")
        || artefacto.sha256.len() != 64
        || !artefacto.sha256.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(Error::Datos);
    }
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == artefacto.archivo)
        .ok_or(Error::SinArtefacto)?;
    if asset.size == 0
        || asset.size > MAX_ARCHIVO
        || !asset_valido(&asset.browser_download_url, &release.tag_name, &asset.name)
        || manifest.notas.len() > 200
        || manifest.notas.iter().any(|n| {
            n.texto.len() > 4096 || !matches!(n.tipo.as_str(), "nuevo" | "mejora" | "correccion")
        })
    {
        return Err(Error::Datos);
    }
    Ok(Some(Disponible {
        version: manifest.version.trim_start_matches('v').into(),
        notas: manifest.notas,
        bytes: asset.size,
        url: asset.browser_download_url.clone(),
        sha256: artefacto.sha256.to_ascii_lowercase(),
    }))
}

/// Sin releases publicadas (404) es un resultado normal, no una actualización ficticia.
pub async fn comprobar(actual: &str) -> Result<Option<Disponible>, Error> {
    let client = cliente()?;
    let respuesta = client
        .get(RELEASE)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|_| Error::Red)?;
    if respuesta.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !respuesta.status().is_success() {
        return Err(Error::Red);
    }
    let release: Release = crate::http::body_json(respuesta, "GitHub", 2 * 1024 * 1024)
        .await
        .map_err(|_| Error::Datos)?;
    if release.draft || release.prerelease {
        return Ok(None);
    }
    let version = numeros(&release.tag_name).ok_or(Error::Datos)?;
    if version <= numeros(actual).ok_or(Error::Datos)? {
        return Ok(None);
    }
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == "latest.json")
        .ok_or(Error::SinArtefacto)?;
    if !asset_valido(&asset.browser_download_url, &release.tag_name, &asset.name)
        || asset.size > 1024 * 1024
    {
        return Err(Error::Datos);
    }
    let respuesta = client
        .get(&asset.browser_download_url)
        .send()
        .await
        .map_err(|_| Error::Red)?;
    if !respuesta.status().is_success() {
        return Err(Error::Red);
    }
    let manifest = crate::http::body_json(respuesta, "GitHub", 1024 * 1024)
        .await
        .map_err(|_| Error::Datos)?;
    resolver(release, manifest, actual)
}

pub fn carpeta() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("Icaro")
        .join("updates")
}

fn rechazar_enlaces(ruta: &Path) -> Result<(), Error> {
    for parte in ruta.ancestors() {
        match std::fs::symlink_metadata(parte) {
            Ok(m) => {
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if m.file_attributes() & 0x400 != 0 {
                        return Err(Error::Datos);
                    }
                }
                if m.file_type().is_symlink() {
                    return Err(Error::Datos);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(Error::Disco),
        }
    }
    Ok(())
}
fn staging() -> Result<PathBuf, Error> {
    #[cfg(windows)]
    if std::env::var_os("LOCALAPPDATA").is_none() {
        return Err(Error::Disco);
    }
    let raiz = carpeta();
    rechazar_enlaces(&raiz)?;
    std::fs::create_dir_all(&raiz).map_err(|_| Error::Disco)?;
    rechazar_enlaces(&raiz)?;
    for _ in 0..4 {
        let mut aleatorio = [0u8; 16];
        getrandom::fill(&mut aleatorio).map_err(|_| Error::Disco)?;
        let id: String = aleatorio.iter().map(|b| format!("{b:02x}")).collect();
        let ruta = raiz.join(id);
        match std::fs::create_dir(&ruta) {
            Ok(()) => {
                rechazar_enlaces(&ruta)?;
                return Ok(ruta);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return Err(Error::Disco),
        }
    }
    Err(Error::Disco)
}
/// Descarga a un temporal y solo prepara el reemplazo cuando coinciden hash y tamaño.
pub async fn descargar<F: Fn(u64, u64)>(d: Disponible, progreso: F) -> Result<Preparada, Error> {
    validar_descarga(&d)?;
    if !cfg!(all(windows, target_arch = "x86_64")) {
        return Err(Error::Plataforma);
    }
    let ruta = staging()?;
    let temporal = ruta.join(format!("{}.part", d.sha256));
    let destino = ruta.join(format!("{}.exe", d.sha256));
    let resultado = async {
        let mut respuesta = cliente()?
            .get(&d.url)
            .send()
            .await
            .map_err(|_| Error::Red)?;
        if !respuesta.status().is_success() {
            return Err(Error::Red);
        }
        if respuesta.content_length().is_some_and(|n| n != d.bytes) {
            return Err(Error::Integridad);
        }
        rechazar_enlaces(&temporal)?;
        let mut archivo = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporal)
            .await
            .map_err(|_| Error::Disco)?;
        let mut hash = Sha256::new();
        let mut recibidos = 0u64;
        while let Some(chunk) = respuesta.chunk().await.map_err(|_| Error::Red)? {
            recibidos += chunk.len() as u64;
            if recibidos > d.bytes || recibidos > MAX_ARCHIVO {
                return Err(Error::Integridad);
            }
            hash.update(&chunk);
            archivo.write_all(&chunk).await.map_err(|_| Error::Disco)?;
            progreso(recibidos, d.bytes);
        }
        if recibidos != d.bytes || format!("{:x}", hash.finalize()) != d.sha256 {
            return Err(Error::Integridad);
        }
        archivo.sync_all().await.map_err(|_| Error::Disco)?;
        drop(archivo);
        tokio::fs::rename(&temporal, &destino)
            .await
            .map_err(|_| Error::Disco)?;
        Ok(Preparada {
            version: d.version,
            archivo: destino,
            sha256: d.sha256,
            bytes: d.bytes,
        })
    }
    .await;
    if resultado.is_err() {
        let _ = tokio::fs::remove_file(&temporal).await;
    }
    resultado
}

fn validar_descarga(d: &Disponible) -> Result<(), Error> {
    let url = reqwest::Url::parse(&d.url).map_err(|_| Error::Datos)?;
    let partes: Vec<_> = url.path_segments().ok_or(Error::Datos)?.collect();
    if url.scheme() != "https"
        || url.host_str() != Some("github.com")
        || url.port().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || partes.len() != 6
        || partes[..4] != ["00faithlss", "Icaro-launcher", "releases", "download"]
        || numeros(partes[4]) != numeros(&d.version)
        || numeros(&d.version).is_none()
        || !partes[5].ends_with(".exe")
        || !asset_valido(&d.url, partes[4], partes[5])
        || d.bytes == 0
        || d.bytes > MAX_ARCHIVO
        || d.sha256.len() != 64
        || !d.sha256.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(Error::Datos);
    }
    Ok(())
}

/// Revalida el ejecutable preparado antes de delegar el reemplazo al salir del proceso.
pub fn aplicar(preparada: &Preparada, reiniciar: bool) -> Result<(), Error> {
    if !cfg!(windows) {
        return Err(Error::Plataforma);
    }
    rechazar_enlaces(&preparada.archivo)?;
    rechazar_enlaces(&carpeta())?;
    let origen = preparada.archivo.canonicalize().map_err(|_| Error::Disco)?;
    let raiz = carpeta().canonicalize().map_err(|_| Error::Disco)?;
    if !origen.starts_with(&raiz)
        || origen.parent().and_then(|p| p.parent()) != Some(raiz.as_path())
    {
        return Err(Error::Datos);
    }
    let esperado = &preparada.sha256;
    if esperado.len() != 64 || !esperado.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Error::Datos);
    }
    let mut archivo = std::fs::File::open(&origen).map_err(|_| Error::Disco)?;
    if !archivo
        .metadata()
        .is_ok_and(|m| m.is_file() && m.len() == preparada.bytes)
    {
        return Err(Error::Integridad);
    }
    let mut hash = Sha256::new();
    std::io::copy(&mut archivo, &mut hash).map_err(|_| Error::Disco)?;
    if format!("{:x}", hash.finalize()) != *esperado {
        return Err(Error::Integridad);
    }
    let destino = std::env::current_exe().map_err(|_| Error::Disco)?;
    rechazar_enlaces(&destino)?;
    lanzar(&origen, &destino, esperado, reiniciar)
}

#[cfg(windows)]
fn lanzar(origen: &Path, destino: &Path, hash: &str, reiniciar: bool) -> Result<(), Error> {
    use std::io::Write;
    use std::os::windows::process::CommandExt;
    let mut aleatorio = [0u8; 16];
    getrandom::fill(&mut aleatorio).map_err(|_| Error::Disco)?;
    let id: String = aleatorio.iter().map(|b| format!("{b:02x}")).collect();
    let script = origen
        .parent()
        .ok_or(Error::Datos)?
        .join(format!("aplicar-{id}.ps1"));
    rechazar_enlaces(&script)?;
    let mut archivo = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&script)
        .map_err(|_| Error::Disco)?;
    archivo
        .write_all(SCRIPT.as_bytes())
        .map_err(|_| Error::Disco)?;
    archivo.sync_all().map_err(|_| Error::Disco)?;
    drop(archivo);
    let mut cmd = std::process::Command::new("powershell.exe");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
    ])
    .arg(script)
    .arg("-ProcessId")
    .arg(std::process::id().to_string())
    .arg("-Source")
    .arg(origen)
    .arg("-Target")
    .arg(destino)
    .arg("-Hash")
    .arg(hash);
    if reiniciar {
        cmd.arg("-Restart");
    }
    cmd.creation_flags(0x08000000)
        .spawn()
        .map_err(|_| Error::Disco)?;
    Ok(())
}
#[cfg(not(windows))]
fn lanzar(_: &Path, _: &Path, _: &str, _: bool) -> Result<(), Error> {
    Err(Error::Plataforma)
}

const SCRIPT: &str = r#"param([int]$ProcessId, [string]$Source, [string]$Target, [string]$Hash, [switch]$Restart)
$ErrorActionPreference = 'Stop'
$taskBackup = $Target + '.icaro-anterior-' + [Guid]::NewGuid().ToString('N')
$taskNew = $Target + '.icaro-nuevo-' + [Guid]::NewGuid().ToString('N')
$taskSourceStream = $null
$taskOutputStream = $null
$taskReplaced = $false
function Assert-RegularPath([string]$taskPath) {
    $taskCheck = [IO.Path]::GetFullPath($taskPath)
    while ($taskCheck) {
        if (Test-Path -LiteralPath $taskCheck) {
            $taskItem = Get-Item -LiteralPath $taskCheck -Force
            if ($taskItem.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Ruta no segura' }
        }
        $taskParent = [IO.Directory]::GetParent($taskCheck)
        if (-not $taskParent) { break }
        $taskCheck = $taskParent.FullName
    }
}
try {
    $taskProcess = Get-Process -Id $ProcessId -ErrorAction SilentlyContinue
    if ($taskProcess) { $taskProcess.WaitForExit() }
    Assert-RegularPath $Source
    Assert-RegularPath $Target
    $taskSourceStream = [IO.File]::Open($Source, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    $taskAlgorithm = [Security.Cryptography.SHA256]::Create()
    $taskActualHash = [BitConverter]::ToString($taskAlgorithm.ComputeHash($taskSourceStream)).Replace('-', '')
    $taskAlgorithm.Dispose()
    if ($taskActualHash -ne $Hash) { throw 'Integridad' }
    $taskSourceStream.Position = 0
    $taskOutputStream = [IO.File]::Open($taskNew, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    $taskSourceStream.CopyTo($taskOutputStream)
    $taskOutputStream.Flush($true)
    $taskOutputStream.Dispose()
    $taskOutputStream = $null
    $taskSourceStream.Dispose()
    $taskSourceStream = $null
    for ($taskAttempt = 0; $taskAttempt -lt 40; $taskAttempt++) {
        try {
            Assert-RegularPath $Target
            [IO.File]::Replace($taskNew, $Target, $taskBackup)
            $taskReplaced = $true
            break
        } catch { Start-Sleep -Milliseconds 250 }
    }
    if (-not $taskReplaced) { throw 'No se pudo reemplazar el launcher' }
    if ($Restart) { Start-Process -FilePath $Target -WindowStyle Hidden }
} catch {
    $taskFailure = $_.Exception.Message
    if ($taskReplaced -and (Test-Path -LiteralPath $taskBackup)) {
        try {
            Assert-RegularPath $taskBackup
            Assert-RegularPath $Target
            [IO.File]::Replace($taskBackup, $Target, $null)
            if ($Restart) { Start-Process -FilePath $Target -WindowStyle Hidden }
        } catch { }
    }
    try {
        $taskErrorFile = $Source + '.' + [Guid]::NewGuid().ToString('N') + '.error.txt'
        Assert-RegularPath $taskErrorFile
        $taskErrorStream = [IO.File]::Open($taskErrorFile, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
        $taskErrorWriter = [IO.StreamWriter]::new($taskErrorStream, [Text.UTF8Encoding]::new($false))
        $taskErrorWriter.WriteLine($taskFailure)
        $taskErrorWriter.Dispose()
    } catch { }
} finally {
    if ($taskOutputStream) { $taskOutputStream.Dispose() }
    if ($taskSourceStream) { $taskSourceStream.Dispose() }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    fn datos() -> (Release, Manifest) {
        let release = Release { tag_name: "v0.2.0".into(), draft: false, prerelease: false,
            assets: vec![Asset { name: "icaro-windows-x86_64.exe".into(), size: 200,
                browser_download_url: "https://github.com/00faithlss/Icaro-launcher/releases/download/v0.2.0/icaro-windows-x86_64.exe".into() }] };
        let manifest = Manifest {
            version: "0.2.0".into(),
            notas: vec![Nota {
                tipo: "mejora".into(),
                texto: "Consola más rápida.".into(),
            }],
            plataformas: BTreeMap::from([(
                PLATAFORMA.into(),
                Artefacto {
                    archivo: "icaro-windows-x86_64.exe".into(),
                    sha256: "ab".repeat(32),
                },
            )]),
        };
        (release, manifest)
    }
    #[test]
    fn compara_versiones_numericamente() {
        assert!(numeros("0.10.0") > numeros("0.9.9"));
        assert_eq!(numeros("v0.1.0"), Some([0, 1, 0]));
        assert_eq!(numeros("0.2.0-beta"), None);
    }
    #[test]
    fn solo_ofrece_release_mas_nueva() {
        let (r, m) = datos();
        assert!(resolver(r, m, "0.1.0").unwrap().is_some());
        let (r, m) = datos();
        assert!(resolver(r, m, "0.2.0").unwrap().is_none());
    }
    #[test]
    fn rechaza_artefacto_externo_o_tag_distinto() {
        let (mut r, m) = datos();
        r.assets[0].browser_download_url = "https://otro.example/launcher.exe".into();
        assert!(resolver(r, m, "0.1.0").is_err());
        let (mut r, m) = datos();
        r.tag_name = "v0.3.0".into();
        assert!(resolver(r, m, "0.1.0").is_err());
    }
    #[test]
    fn requiere_hash_y_plataforma() {
        let (r, mut m) = datos();
        m.plataformas.get_mut(PLATAFORMA).unwrap().sha256 = "ab".into();
        assert!(resolver(r, m, "0.1.0").is_err());
        let (r, mut m) = datos();
        m.plataformas.clear();
        assert!(matches!(resolver(r, m, "0.1.0"), Err(Error::SinArtefacto)));
    }
    #[test]
    fn revalida_destino_antes_de_descargar() {
        let (r, m) = datos();
        let mut d = resolver(r, m, "0.1.0").unwrap().unwrap();
        assert!(validar_descarga(&d).is_ok());
        d.url = "https://github.com/otro/Icaro/releases/download/v0.2.0/icaro.exe".into();
        assert!(validar_descarga(&d).is_err());
        d.url = "https://github.com/00faithlss/Icaro-launcher/releases/download/v0.3.0/icaro.exe".into();
        assert!(validar_descarga(&d).is_err());
    }
}
