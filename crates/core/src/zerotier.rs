// SPDX-License-Identifier: GPL-3.0-only
//! Consulta y pertenencia a redes ZeroTier mediante su API local y CLI oficial.
use std::{path::PathBuf, process::Stdio, time::Duration};
use tokio::io::AsyncReadExt;
use tokio_util::sync::CancellationToken;
#[derive(Debug, Clone)]
pub struct Red {
    pub id: String,
    pub nombre: String,
    pub estado: String,
    pub ips: Vec<String>,
}
#[derive(Debug, Clone)]
pub enum Estado {
    Ausente,
    Disponible {
        version: String,
        online: bool,
        redes: Vec<Red>,
    },
    SinAcceso,
}
const MAX_RESPUESTA: usize = 256 * 1024;

#[cfg(any(windows, test))]
struct ApiLocal {
    cliente: reqwest::Client,
    puerto: u16,
    autorizacion: reqwest::header::HeaderValue,
}
#[cfg(any(windows, test))]
impl ApiLocal {
    fn nueva(puerto: u16, token: &[u8]) -> Result<Self, ()> {
        let token = std::str::from_utf8(token).map_err(|_| ())?.trim();
        if puerto == 0 || token.is_empty() || token.len() > 256 || !token.bytes().all(|b| b.is_ascii_graphic()) {
            return Err(());
        }
        let mut autorizacion = reqwest::header::HeaderValue::from_str(token).map_err(|_| ())?;
        autorizacion.set_sensitive(true);
        Ok(Self {
            cliente: reqwest::Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(2))
                .build()
                .map_err(|_| ())?,
            puerto,
            autorizacion,
        })
    }
    async fn peticion(&self, metodo: reqwest::Method, ruta: &str) -> Result<Vec<u8>, ()> {
        // El acceso de ZeroTier permanece en memoria y solo se envía al servicio local.
        let mut respuesta = self.cliente
            .request(metodo, format!("http://127.0.0.1:{}{ruta}", self.puerto))
            .header("X-ZT1-Auth", self.autorizacion.clone())
            .send().await.map_err(|_| ())?;
        if !respuesta.status().is_success() || respuesta.content_length().is_some_and(|n| n > MAX_RESPUESTA as u64) {
            return Err(());
        }
        let mut bytes = vec![];
        while let Some(parte) = respuesta.chunk().await.map_err(|_| ())? {
            if bytes.len() + parte.len() > MAX_RESPUESTA { return Err(()); }
            bytes.extend_from_slice(&parte);
        }
        Ok(bytes)
    }
    async fn estado(&self) -> Result<Estado, ()> {
        let (info, redes) = tokio::try_join!(
            self.peticion(reqwest::Method::GET, "/status"),
            self.peticion(reqwest::Method::GET, "/network"),
        )?;
        analizar(&info, &redes)
    }
}
#[cfg(any(windows, test))]
async fn leer_limitado(path: &std::path::Path, max: usize) -> std::io::Result<Vec<u8>> {
    let mut archivo = tokio::fs::File::open(path).await?.take(max as u64 + 1);
    let mut bytes = vec![];
    archivo.read_to_end(&mut bytes).await?;
    if bytes.len() > max { return Err(std::io::ErrorKind::InvalidData.into()); }
    Ok(bytes)
}
#[cfg(any(windows, test))]
async fn buscar_api(base: &std::path::Path, tokens: &[PathBuf]) -> Result<(ApiLocal, Estado), ()> {
    let puerto = match leer_limitado(&base.join("zerotier-one.port"), 16).await {
        Ok(bytes) => std::str::from_utf8(&bytes).map_err(|_| ())?.trim().parse::<u16>().map_err(|_| ())?,
        Err(e) if matches!(e.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied) => 9993,
        Err(_) => return Err(()),
    };
    for path in tokens {
        if let Ok(bytes) = leer_limitado(path, 1024).await {
            let bytes = zeroize::Zeroizing::new(bytes);
            if let Ok(api) = ApiLocal::nueva(puerto, &bytes) {
                if let Ok(estado) = api.estado().await { return Ok((api, estado)); }
            }
        }
    }
    Err(())
}
#[cfg(windows)]
fn configuracion_api() -> (PathBuf, Vec<PathBuf>) {
    let base = std::env::var_os("ProgramData").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData")).join("ZeroTier/One");
    let mut tokens = vec![];
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let local = PathBuf::from(local);
        // La aplicación oficial concede acceso al usuario mediante esta copia.
        tokens.push(local.join("ZeroTier/authtoken.secret"));
        tokens.push(local.join("ZeroTier/One/authtoken.secret"));
    }
    tokens.push(base.join("authtoken.secret"));
    (base, tokens)
}
fn candidatos() -> Vec<(PathBuf, bool)> {
    let mut v = vec![];
    if let Some(paths) = std::env::var_os("PATH") {
        for p in std::env::split_paths(&paths).take(128) {
            if p.is_absolute() {
                v.push((
                    p.join(if cfg!(windows) {
                        "zerotier-cli.exe"
                    } else {
                        "zerotier-cli"
                    }),
                    false,
                ));
            }
        }
    }
    #[cfg(windows)]
    for key in ["ProgramData", "ProgramFiles", "ProgramFiles(x86)"] {
        if let Some(p) = std::env::var_os(key) {
            let base = PathBuf::from(p);
            v.push((base.join("ZeroTier/One/zerotier-one_x64.exe"), true));
            v.push((base.join("ZeroTier/One/zerotier-one_x86.exe"), true));
        }
    }
    #[cfg(not(windows))]
    for p in [
        "/usr/sbin/zerotier-cli",
        "/usr/bin/zerotier-cli",
        "/usr/local/bin/zerotier-cli",
    ] {
        v.push((p.into(), false));
    }
    v
}
async fn comando(path: &PathBuf, one: bool, args: &[&str]) -> Result<Vec<u8>, ()> {
    let mut cmd = tokio::process::Command::new(path);
    if one {
        cmd.arg("-q");
    }
    cmd.arg("-j")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    let mut child = cmd.spawn().map_err(|_| ())?;
    let mut out = child.stdout.take().ok_or(())?.take(MAX_RESPUESTA as u64 + 1);
    let mut bytes = vec![];
    out.read_to_end(&mut bytes).await.map_err(|_| ())?;
    if bytes.len() > MAX_RESPUESTA || !child.wait().await.map_err(|_| ())?.success() {
        return Err(());
    }
    Ok(bytes)
}
fn analizar(info: &[u8], nets: &[u8]) -> Result<Estado, ()> {
    let v: serde_json::Value = serde_json::from_slice(info).map_err(|_| ())?;
    let version = v["version"]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() < 80 && s.chars().all(|c| !c.is_control()))
        .ok_or(())?
        .to_owned();
    let online = v["online"].as_bool().ok_or(())?;
    let array: Vec<serde_json::Value> = serde_json::from_slice(nets).map_err(|_| ())?;
    if array.len() > 256 {
        return Err(());
    }
    let mut redes = vec![];
    for n in array {
        let id = n["nwid"]
            .as_str()
            .or(n["id"].as_str())
            .filter(|s| s.len() == 16 && s.bytes().all(|b| b.is_ascii_hexdigit()))
            .ok_or(())?
            .to_owned();
        let text = |key: &str| max_text(n[key].as_str().unwrap_or(""));
        let ips = n["assignedAddresses"]
            .as_array()
            .map(|a| {
                a.iter()
                    .take(32)
                    .filter_map(|s| s.as_str())
                    .map(max_text)
                    .collect()
            })
            .unwrap_or_default();
        redes.push(Red {
            id,
            nombre: text("name"),
            estado: text("status"),
            ips,
        });
    }
    Ok(Estado::Disponible {
        version,
        online,
        redes,
    })
}
fn max_text(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).take(160).collect()
}
#[derive(Debug, Clone, Copy)]
pub enum Accion {
    Unirse,
    Salir,
}
pub fn id_red(id: &str) -> Result<String, String> {
    let id = id.trim();
    if id.len() != 16 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("El ID de red debe tener 16 caracteres hexadecimales.".into());
    }
    Ok(id.to_ascii_lowercase())
}
pub async fn cambiar_red(id: &str, a: Accion, cancel: CancellationToken) -> Result<Estado, String> {
    let id = id_red(id)?;
    if cancel.is_cancelled() {
        return Err("La solicitud fue cancelada antes de enviarse.".into());
    }
    #[cfg(windows)]
    {
        let (base, tokens) = configuracion_api();
        let api = tokio::select! {
            _=cancel.cancelled()=>return Err("La solicitud fue cancelada antes de enviarse.".into()),
            r=tokio::time::timeout(Duration::from_secs(5), buscar_api(&base, &tokens))=>r.ok().and_then(Result::ok),
        };
        if let Some((api, _)) = api {
            let metodo = match a { Accion::Unirse => reqwest::Method::POST, Accion::Salir => reqwest::Method::DELETE };
            let ruta = format!("/network/{id}");
            tokio::select! {
                _=cancel.cancelled()=>return Err("La solicitud se interrumpió. Actualiza el estado para comprobar la pertenencia a la red.".into()),
                r=api.peticion(metodo, &ruta)=>r.map_err(|_| "No se pudo modificar la red. Comprueba el servicio y los permisos de ZeroTier.".to_string())?,
            };
            return Ok(detectar().await);
        }
    }
    let Some((p, one)) = candidatos().into_iter().find(|(p, _)| p.is_file()) else {
        return Err("ZeroTier no está instalado.".into());
    };
    let command = match a {
        Accion::Unirse => "join",
        Accion::Salir => "leave",
    };
    let args = [command, id.as_str()];
    tokio::select! {
        _=cancel.cancelled()=>return Err("La solicitud se interrumpió. Actualiza el estado para comprobar la pertenencia a la red.".into()),
        r=tokio::time::timeout(Duration::from_secs(8),comando(&p,one,&args))=>{
            r.map_err(|_|"El servicio de ZeroTier no respondió. Actualiza el estado antes de reintentar.".to_string())?
                .map_err(|_|"No se pudo modificar la red. Comprueba el servicio y los permisos de ZeroTier.".to_string())?;
        }
    }
    Ok(detectar().await)
}
pub async fn detectar() -> Estado {
    let cli = candidatos().into_iter().find(|(p, _)| p.is_file());
    #[cfg(windows)]
    let (base, tokens) = configuracion_api();
    #[cfg(windows)]
    let instalado = cli.is_some() || tokens.iter().any(|p| p.is_file()) || base.join("zerotier-one.port").is_file();
    #[cfg(not(windows))]
    let instalado = cli.is_some();
    if !instalado {
        return Estado::Ausente;
    }
    tokio::time::timeout(Duration::from_secs(5), async {
        #[cfg(windows)]
        if let Ok((_, estado)) = buscar_api(&base, &tokens).await { return Ok(estado); }
        let (p, one) = cli.ok_or(())?;
        let info = comando(&p, one, &["info"]).await?;
        let redes = comando(&p, one, &["listnetworks"]).await?;
        analizar(&info, &redes)
    })
    .await
    .ok()
    .and_then(Result::ok)
    .unwrap_or(Estado::SinAcceso)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn api_local_reconoce_una_red_unida_desde_la_aplicacion_oficial() {
        use tokio::io::AsyncWriteExt;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let api = ApiLocal::nueva(listener.local_addr().unwrap().port(), b"acceso-de-prueba").unwrap();
        assert!(api.autorizacion.is_sensitive());
        let servidor = tokio::spawn(async move {
            for _ in 0..2 {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut solicitud = vec![];
                let mut buffer = [0u8; 512];
                while !solicitud.windows(4).any(|v| v == b"\r\n\r\n") {
                    let n = socket.read(&mut buffer).await.unwrap();
                    assert!(n > 0 && solicitud.len() + n <= 2048);
                    solicitud.extend_from_slice(&buffer[..n]);
                }
                let solicitud = String::from_utf8(solicitud).unwrap();
                assert!(solicitud.to_ascii_lowercase().contains("x-zt1-auth: acceso-de-prueba\r\n"));
                let cuerpo = if solicitud.starts_with("GET /status ") {
                    r#"{"version":"1.16.2","online":true}"#
                } else {
                    assert!(solicitud.starts_with("GET /network "));
                    r#"[{"id":"1234567890abcdef","name":"Amigos","status":"OK","assignedAddresses":["10.147.17.4/24"]}]"#
                };
                let respuesta = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{cuerpo}", cuerpo.len());
                socket.write_all(respuesta.as_bytes()).await.unwrap();
            }
        });
        let Estado::Disponible { online, redes, .. } = api.estado().await.unwrap() else { panic!() };
        assert!(online);
        assert_eq!(redes[0].estado, "OK");
        assert_eq!(redes[0].ips, ["10.147.17.4/24"]);
        servidor.await.unwrap();
    }
    #[test]
    fn acceso_local_rechaza_puerto_invalido_y_cabeceras_inyectadas() {
        assert!(ApiLocal::nueva(0, b"token").is_err());
        assert!(ApiLocal::nueva(9993, b"token\r\nOtra: cabecera").is_err());
        assert!(ApiLocal::nueva(9993, b"").is_err());
    }
    #[test]
    fn id_de_red_no_admite_comandos_ni_valores_ambiguos() {
        assert_eq!(id_red(" ABCDEF0123456789 ").unwrap(), "abcdef0123456789");
        for value in [
            "",
            "123",
            "abcdef012345678g",
            "abcdef0123456789 --help",
            "../../auth",
        ] {
            assert!(id_red(value).is_err());
        }
    }
    #[tokio::test]
    async fn cancelar_antes_de_unirse_no_invoca_el_servicio() {
        let c = CancellationToken::new();
        c.cancel();
        assert!(cambiar_red("abcdef0123456789", Accion::Unirse, c)
            .await
            .unwrap_err()
            .contains("antes"));
    }
    #[test]
    fn datos_oficiales_no_inventan_estado_ni_autorizacion() {
        let s=analizar(br#"{"version":"1.16.0","online":true}"#,br#"[{"nwid":"1234567890abcdef","name":"Amigos","status":"ACCESS_DENIED","assignedAddresses":[]}]"#).unwrap();
        let Estado::Disponible { redes, .. } = s else {
            panic!()
        };
        assert_eq!(redes[0].estado, "ACCESS_DENIED");
        assert!(redes[0].ips.is_empty());
        assert!(analizar(br#"{"version":"1"}"#, b"[]").is_err());
    }
}
