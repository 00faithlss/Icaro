// SPDX-License-Identifier: GPL-3.0-only
//! API de texturas de solo lectura, limitada al loopback y a esta partida.
use super::*;
use base64::{engine::general_purpose::STANDARD, Engine};
use rsa::{
    pkcs8::{EncodePublicKey, LineEnding},
    RsaPrivateKey,
};
use std::{sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task::{JoinHandle, JoinSet},
};
use tokio_util::sync::CancellationToken;

const URL: &str = "https://authlib-injector.yushi.moe/artifact/56/authlib-injector-1.2.8.jar";
const SHA: &str = "9c7f4343e6c82034958ffb48c14a2cb0c85928be7283103ce17da00c6d5a7b10";
const MAX_AGENTE: usize = 4 * 1024 * 1024;

pub struct Preparada {
    pub argumentos: Vec<String>,
    tarea: JoinHandle<()>,
}
impl Drop for Preparada {
    fn drop(&mut self) {
        self.tarea.abort();
    }
}

async fn agente(raiz: &Path, cancel: &CancellationToken) -> Result<PathBuf, String> {
    let dir = raiz.join("cache/authlib-injector");
    let path = dir.join(format!("{SHA}.jar"));
    let verificar = |p: &Path| -> bool {
        let Ok(meta) = almacenamiento::regular(p) else {
            return false;
        };
        if !meta.is_file() || meta.len() > MAX_AGENTE as u64 {
            return false;
        }
        std::fs::read(p).is_ok_and(|b| format!("{:x}", Sha256::digest(b)) == SHA)
    };
    if path.exists() {
        almacenamiento::regular(&dir)?;
        if verificar(&path) {
            return path
                .canonicalize()
                .map_err(|_| "No se pudo abrir el soporte de skins.".into());
        }
        return Err(
            "El soporte de skins cambió; elimina su copia dañada de la caché y reintenta.".into(),
        );
    }
    let cliente = crate::http::client().map_err(|e| e.to_string())?;
    let mut respuesta = tokio::select! {_ = cancel.cancelled()=>return Err("Preparación de skin cancelada.".into()),r=cliente.get(URL).send()=>r.map_err(|_|"No se pudo descargar el soporte de skins; vuelve a intentarlo con conexión.")?};
    if !respuesta.status().is_success() {
        return Err("No se pudo descargar el soporte de skins.".into());
    }
    let mut bytes = Vec::new();
    loop {
        let trozo = tokio::select! {_ = cancel.cancelled()=>return Err("Preparación de skin cancelada.".into()),r=respuesta.chunk()=>r.map_err(|_|"Se interrumpió la descarga del soporte de skins.")?};
        let Some(trozo) = trozo else { break };
        if bytes.len() + trozo.len() > MAX_AGENTE {
            return Err("El soporte de skins supera el tamaño permitido.".into());
        }
        bytes.extend_from_slice(&trozo);
    }
    if format!("{:x}", Sha256::digest(&bytes)) != SHA {
        return Err("El soporte de skins no superó la verificación SHA-256.".into());
    }
    std::fs::create_dir_all(&dir).map_err(|_| "No se pudo preparar la caché de skins.")?;
    almacenamiento::regular(&dir)?;
    let mut nonce = [0; 8];
    getrandom::fill(&mut nonce).map_err(|_| "No se pudo preparar la caché.")?;
    let temp = dir.join(format!(".descarga-{}.part", u64::from_le_bytes(nonce)));
    let result = (|| {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp)
            .map_err(|_| "No se pudo guardar el soporte de skins.")?;
        f.write_all(&bytes)
            .and_then(|_| f.sync_all())
            .map_err(|_| "No se pudo guardar el soporte de skins.")?;
        drop(f);
        std::fs::rename(&temp, &path).map_err(|_| "No se pudo publicar el soporte de skins.")
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result?;
    path.canonicalize()
        .map_err(|_| "No se pudo abrir el soporte de skins.".into())
}

struct Datos {
    prefijo: String,
    metadatos: Vec<u8>,
    perfil: Vec<u8>,
    simple: Vec<u8>,
    id: String,
    nombre: String,
    hash: String,
    png: Arc<[u8]>,
}

async fn iniciar(
    nombre: &str,
    imagen: &Imagen,
    delgada: bool,
) -> Result<(String, Vec<u8>, JoinHandle<()>), String> {
    static CLAVE: tokio::sync::OnceCell<Arc<RsaPrivateKey>> = tokio::sync::OnceCell::const_new();
    let clave = CLAVE
        .get_or_try_init(|| async {
            tokio::task::spawn_blocking(|| {
                RsaPrivateKey::new(&mut rand::rngs::OsRng, 2048)
                    .map(Arc::new)
                    .map_err(|_| "No se pudo preparar la firma de la skin.".to_string())
            })
            .await
            .map_err(|_| "No se pudo preparar la firma.".to_string())?
        })
        .await?;
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .map_err(|_| "No se pudo abrir el servicio local de skins.")?;
    let mut nonce = [0; 16];
    getrandom::fill(&mut nonce).map_err(|_| "No se pudo preparar la skin.")?;
    let prefijo = format!("/{:x}/", md5::Md5::digest(nonce));
    let url = format!(
        "http://127.0.0.1:{}{prefijo}",
        listener
            .local_addr()
            .map_err(|_| "No se pudo abrir el servicio local.")?
            .port()
    );
    let hash = format!("{:x}", Sha256::digest(&imagen.textura.png));
    let id = Session::local(nombre)?.profile.id;
    let valor=STANDARD.encode(serde_json::to_vec(&serde_json::json!({"timestamp":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64,"profileId":id,"profileName":nombre,"textures":{"SKIN":{"url":format!("{url}textures/{hash}"),"metadata":{"model":if delgada {"slim"}else{"default"}}}}})).map_err(|e|e.to_string())?);
    let firma = clave
        .sign(
            rsa::Pkcs1v15Sign::new::<sha1::Sha1>(),
            &sha1::Sha1::digest(valor.as_bytes()),
        )
        .map_err(|_| "No se pudo firmar la skin.")?;
    let metadatos=serde_json::to_vec(&serde_json::json!({"meta":{"serverName":"Ícaro · Skin local","implementationName":"Icaro Launcher","feature.enable_mojang_anti_features":true,"feature.enable_profile_key":false},"skinDomains":["127.0.0.1"],"signaturePublickey":clave.to_public_key().to_public_key_pem(LineEnding::LF).map_err(|_|"No se pudo preparar la firma pública.")?})).map_err(|e|e.to_string())?;
    let simple = serde_json::to_vec(&serde_json::json!({"id":id,"name":nombre}))
        .map_err(|e| e.to_string())?;
    let perfil=serde_json::to_vec(&serde_json::json!({"id":id,"name":nombre,"properties":[{"name":"textures","value":valor,"signature":STANDARD.encode(firma)}]})).map_err(|e|e.to_string())?;
    let datos = Arc::new(Datos {
        prefijo,
        metadatos: metadatos.clone(),
        perfil,
        simple,
        id,
        nombre: nombre.into(),
        hash,
        png: imagen.textura.png.clone(),
    });
    let tarea = tokio::spawn(async move {
        let slots = Arc::new(tokio::sync::Semaphore::new(8));
        let mut hijos = JoinSet::new();
        loop {
            tokio::select! {
                r=listener.accept()=>{let Ok((stream,_))=r else{break};let Ok(permiso)=slots.clone().try_acquire_owned()else{continue};let datos=datos.clone();hijos.spawn(async move {let _permiso=permiso;let _=tokio::time::timeout(Duration::from_secs(5),atender(stream,datos)).await;});},
                _=hijos.join_next(),if !hijos.is_empty()=>{},
            }
        }
    });
    Ok((url, metadatos, tarea))
}

async fn atender(mut stream: TcpStream, datos: Arc<Datos>) -> std::io::Result<()> {
    let mut bytes = Vec::new();
    let fin;
    loop {
        let mut b = [0; 1024];
        let n = stream.read(&mut b).await?;
        if n == 0 {
            return Ok(());
        }
        bytes.extend_from_slice(&b[..n]);
        if bytes.len() > 8192 {
            return Ok(());
        }
        if let Some(i) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            fin = i + 4;
            break;
        }
    }
    let cab = String::from_utf8_lossy(&bytes[..fin]).into_owned();
    let mut linea = cab.lines().next().unwrap_or("").split_whitespace();
    let metodo = linea.next().unwrap_or("");
    let ruta = linea.next().unwrap_or("").split('?').next().unwrap_or("");
    let Some(ruta) = ruta.strip_prefix(&datos.prefijo) else {
        return responder(&mut stream, "404 Not Found", "application/json", b"{}").await;
    };
    let cuerpo = if metodo == "GET" && ruta.is_empty() {
        Some(("application/json", datos.metadatos.as_slice()))
    } else if metodo == "GET" && ruta == format!("textures/{}", datos.hash) {
        Some(("image/png", datos.png.as_ref()))
    } else if metodo == "GET"
        && ruta == format!("sessionserver/session/minecraft/profile/{}", datos.id)
    {
        Some(("application/json", datos.perfil.as_slice()))
    } else if metodo == "GET"
        && (ruta == format!("api/users/profiles/minecraft/{}", datos.nombre)
            || ruta == format!("users/profiles/minecraft/{}", datos.nombre))
    {
        Some(("application/json", datos.simple.as_slice()))
    } else {
        None
    };
    if let Some((mime, cuerpo)) = cuerpo {
        return responder(&mut stream, "200 OK", mime, cuerpo).await;
    }
    if metodo == "POST" && ruta == "api/profiles/minecraft" {
        let tam = cab
            .lines()
            .filter_map(|l| l.split_once(':'))
            .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
            .and_then(|(_, v)| v.trim().parse::<usize>().ok())
            .unwrap_or(0);
        if tam > 4096 {
            return responder(
                &mut stream,
                "413 Payload Too Large",
                "application/json",
                b"{}",
            )
            .await;
        }
        while bytes.len() < fin + tam {
            let mut b = [0; 1024];
            let n = stream.read(&mut b).await?;
            if n == 0 {
                return Ok(());
            }
            bytes.extend_from_slice(&b[..n]);
        }
        let nombres =
            serde_json::from_slice::<Vec<String>>(&bytes[fin..fin + tam]).unwrap_or_default();
        let respuesta = if nombres
            .iter()
            .any(|n| n.eq_ignore_ascii_case(&datos.nombre))
        {
            format!("[{}]", String::from_utf8_lossy(&datos.simple))
        } else {
            "[]".into()
        };
        return responder(
            &mut stream,
            "200 OK",
            "application/json",
            respuesta.as_bytes(),
        )
        .await;
    }
    // Nunca autenticar ni aceptar join/refresh con una cuenta local.
    responder(
        &mut stream,
        if ruta.starts_with("sessionserver/session/minecraft/profile/") {
            "204 No Content"
        } else {
            "403 Forbidden"
        },
        "application/json",
        b"",
    )
    .await
}
async fn responder(
    stream: &mut TcpStream,
    estado: &str,
    mime: &str,
    cuerpo: &[u8],
) -> std::io::Result<()> {
    stream.write_all(format!("HTTP/1.1 {estado}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n",cuerpo.len()).as_bytes()).await?;
    stream.write_all(cuerpo).await?;
    stream.shutdown().await
}

/// Solo devuelve un agente para una cuenta local que tenga una skin guardada.
pub async fn preparar(
    raiz: &Path,
    cuenta: &Session,
    cancel: CancellationToken,
) -> Result<Option<Preparada>, String> {
    if !cuenta.es_local() {
        return Ok(None);
    }
    let root = raiz.to_owned();
    let nombre = cuenta.profile.name.clone();
    let Some((imagen, delgada)) = tokio::task::spawn_blocking(move || leer(&root, &nombre))
        .await
        .map_err(|_| "No se pudo leer la skin.")??
    else {
        return Ok(None);
    };
    let jar = agente(raiz, &cancel).await?;
    let (url, metadata, tarea) = tokio::select! {
        _ = cancel.cancelled() => return Err("Preparación de skin cancelada.".into()),
        r = iniciar(&cuenta.profile.name, &imagen, delgada) => r?,
    };
    if cancel.is_cancelled() {
        tarea.abort();
        return Err("Preparación de skin cancelada.".into());
    }
    let ruta = jar.to_string_lossy();
    let ruta = crate::lanzamiento::ruta_java(&ruta);
    let argumentos = vec![
        format!("-javaagent:{ruta}={url}"),
        format!(
            "-Dauthlibinjector.yggdrasil.prefetched={}",
            STANDARD.encode(metadata)
        ),
        "-Dauthlibinjector.mojangNamespace=disabled".into(),
        "-Dauthlibinjector.mojangAntiFeatures=enabled".into(),
        "-Dauthlibinjector.noShowServerName".into(),
    ];
    Ok(Some(Preparada { argumentos, tarea }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Descarga el agente oficial y necesita Java en PATH"]
    async fn agente_oficial_verificado_arranca_en_java() {
        let raiz = std::env::temp_dir().join(format!(
            "icaro-agente-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let cuenta = Session::local("Prueba_skin").unwrap();
        Cuentas::activar(&raiz, &cuenta.profile.name).unwrap();
        let imagen = decodificar(&super::super::tests::ejemplo(64)).unwrap();
        guardar(&raiz, &cuenta.profile.name, &imagen, false).unwrap();
        let preparada = preparar(&raiz, &cuenta, CancellationToken::new())
            .await
            .unwrap()
            .unwrap();
        let salida = tokio::time::timeout(
            Duration::from_secs(30),
            tokio::process::Command::new("java")
                .current_dir(&raiz)
                .args(&preparada.argumentos)
                .arg("-version")
                .kill_on_drop(true)
                .output(),
        )
        .await
        .unwrap()
        .unwrap();
        assert!(
            salida.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&salida.stdout),
            String::from_utf8_lossy(&salida.stderr)
        );
        let registro = format!(
            "{}{}",
            String::from_utf8_lossy(&salida.stdout),
            String::from_utf8_lossy(&salida.stderr)
        );
        assert!(registro.contains("authlib-injector"), "{registro}");
        drop(preparada);
        let archivo = raiz.join(format!("cache/authlib-injector/{SHA}.jar"));
        std::fs::write(&archivo, b"alterado").unwrap();
        assert!(agente(&raiz, &CancellationToken::new()).await.is_err());
        std::fs::remove_file(archivo).unwrap();
        let cuentas = Cuentas::cargar(&raiz).unwrap();
        for skin in cuentas.skins.values() {
            std::fs::remove_file(ruta(&raiz, &skin.sha256).unwrap()).unwrap();
        }
        std::fs::remove_file(raiz.join("accounts/locales.json")).unwrap();
        std::fs::remove_file(raiz.join("authlib-injector.log")).unwrap();
        for dir in ["cache/authlib-injector", "cache", "skins", "accounts", ""] {
            std::fs::remove_dir(raiz.join(dir)).unwrap();
        }
    }

    #[tokio::test]
    async fn perfil_firmado_textura_y_rechazo_de_autenticacion() {
        let imagen = decodificar(&super::super::tests::ejemplo(64)).unwrap();
        let (url, metadata, tarea) = iniciar("Jugador_1", &imagen, true).await.unwrap();
        let cliente = reqwest::Client::builder().no_proxy().build().unwrap();
        let id = Session::local("Jugador_1").unwrap().profile.id;
        let perfil: serde_json::Value = cliente
            .get(format!(
                "{url}sessionserver/session/minecraft/profile/{id}?unsigned=false"
            ))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let valor = perfil["properties"][0]["value"].as_str().unwrap();
        let firma = STANDARD
            .decode(perfil["properties"][0]["signature"].as_str().unwrap())
            .unwrap();
        use rsa::pkcs8::DecodePublicKey;
        let meta: serde_json::Value = serde_json::from_slice(&metadata).unwrap();
        let clave =
            rsa::RsaPublicKey::from_public_key_pem(meta["signaturePublickey"].as_str().unwrap())
                .unwrap();
        clave
            .verify(
                rsa::Pkcs1v15Sign::new::<sha1::Sha1>(),
                &sha1::Sha1::digest(valor.as_bytes()),
                &firma,
            )
            .unwrap();
        let texturas: serde_json::Value =
            serde_json::from_slice(&STANDARD.decode(valor).unwrap()).unwrap();
        assert_eq!(texturas["textures"]["SKIN"]["metadata"]["model"], "slim");
        let png = cliente
            .get(texturas["textures"]["SKIN"]["url"].as_str().unwrap())
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();
        assert_eq!(png.as_ref(), imagen.textura.png.as_ref());
        assert_eq!(
            cliente
                .post(format!("{url}sessionserver/session/minecraft/join"))
                .send()
                .await
                .unwrap()
                .status(),
            403
        );
        assert_eq!(
            cliente
                .get(format!("{url}../accounts/locales.json"))
                .send()
                .await
                .unwrap()
                .status(),
            404
        );
        tarea.abort();
    }
}
