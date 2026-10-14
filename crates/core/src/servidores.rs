// SPDX-License-Identifier: GPL-3.0-only
//! Servidores guardados y consulta del protocolo de estado de Minecraft Java.
use serde::{Deserialize, Serialize};
use std::{
    io::Read,
    net::IpAddr,
    path::Path,
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Servidor {
    pub id: String,
    pub nombre: String,
    pub direccion: String,
    pub instancia: Option<String>,
    #[serde(default)]
    pub sincronizar: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Direccion {
    pub host: String,
    pub puerto: u16,
}
impl Direccion {
    pub fn texto(&self) -> String {
        if self.host.contains(':') {
            format!("[{}]:{}", self.host, self.puerto)
        } else {
            format!("{}:{}", self.host, self.puerto)
        }
    }
}
pub fn direccion(texto: &str) -> Result<Direccion, String> {
    let t = texto.trim();
    if t.is_empty()
        || t.len() > 255
        || t.chars().any(|c| c.is_control() || c.is_whitespace())
        || t.contains(['/', '\\', '@', '?', '#'])
    {
        return Err("Usa un nombre de servidor o IP, con puerto opcional; sin http://.".into());
    }
    let (host, port) = if let Some(rest) = t.strip_prefix('[') {
        let (h, end) = rest
            .split_once(']')
            .ok_or("La dirección IPv6 debe cerrar su corchete.")?;
        if !matches!(h.parse::<IpAddr>(), Ok(IpAddr::V6(_))) {
            return Err("La dirección IPv6 no es válida.".into());
        }
        (
            h,
            if end.is_empty() {
                None
            } else {
                Some(end.strip_prefix(':').ok_or("Puerto inválido.")?)
            },
        )
    } else if t.parse::<IpAddr>().is_ok() {
        (t, None)
    } else if let Some((h, p)) = t.rsplit_once(':') {
        (h, Some(p))
    } else {
        (t, None)
    };
    if host.is_empty()
        || (host.parse::<IpAddr>().is_err()
            && (!host
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-'))
                || host.trim_end_matches('.').split('.').any(|s| {
                    s.is_empty() || s.len() > 63 || s.starts_with('-') || s.ends_with('-')
                })))
    {
        return Err("El nombre o IP del servidor no es válido.".into());
    }
    let puerto = port
        .map(|p| {
            p.parse::<u16>()
                .ok()
                .filter(|p| *p > 0)
                .ok_or("Usa un puerto entre 1 y 65535.")
        })
        .transpose()?
        .unwrap_or(25565);
    Ok(Direccion {
        host: host.to_ascii_lowercase(),
        puerto,
    })
}
pub fn nuevo(nombre: String, dir: String, instancia: Option<String>) -> Result<Servidor, String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| "No se pudo crear el identificador.")?;
    let s = Servidor {
        sincronizar: false,
        id: bytes.iter().map(|b| format!("{b:02x}")).collect(),
        nombre: nombre.trim().into(),
        direccion: dir.trim().into(),
        instancia,
    };
    validar(&s)?;
    Ok(s)
}
fn validar(s: &Servidor) -> Result<(), String> {
    if s.id.len() != 32
        || !s.id.bytes().all(|b| b.is_ascii_hexdigit())
        || s.nombre.trim().is_empty()
        || s.nombre.len() > 160
        || s.nombre.chars().any(char::is_control)
        || s.instancia.as_ref().is_some_and(|i| {
            i.is_empty()
                || i.len() > 128
                || !i.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
    {
        return Err("Los datos guardados del servidor no son válidos.".into());
    }
    direccion(&s.direccion)?;
    Ok(())
}
#[derive(Serialize, Deserialize)]
struct Documento {
    schema: u32,
    servidores: Vec<Servidor>,
}
fn validar_lista(v: &[Servidor]) -> Result<(), String> {
    if v.len() > 64 {
        return Err("Puedes guardar hasta 64 servidores.".into());
    }
    let mut ids = std::collections::BTreeSet::new();
    let mut dirs = std::collections::BTreeSet::new();
    for s in v {
        validar(s)?;
        if !ids.insert(&s.id) || !dirs.insert(direccion(&s.direccion)?.texto()) {
            return Err("Ese servidor ya está guardado.".into());
        }
    }
    Ok(())
}
pub fn cargar(root: &Path) -> Result<Vec<Servidor>, String> {
    let p = root.join("servidores.json");
    if !p.exists() {
        return Ok(vec![]);
    }
    let m = std::fs::symlink_metadata(&p).map_err(|_| "No se pudo leer la lista de servidores.")?;
    if !m.is_file() || m.file_type().is_symlink() || m.len() > 256 * 1024 {
        return Err("El archivo de servidores no es válido; se conservará.".into());
    }
    let mut bytes = vec![];
    std::fs::File::open(p)
        .map_err(|_| "No se pudo leer la lista.")?
        .take(256 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "No se pudo leer la lista.")?;
    let d: Documento = serde_json::from_slice(&bytes)
        .map_err(|_| "La lista de servidores está dañada; no se sobrescribirá.")?;
    if d.schema != 1 {
        return Err("Este esquema de servidores no está soportado.".into());
    }
    validar_lista(&d.servidores)?;
    Ok(d.servidores)
}
pub fn guardar(root: &Path, v: &[Servidor]) -> Result<(), String> {
    validar_lista(v)?;
    cargar(root)?;
    crate::almacenamiento::escribir_json(
        &root.join("servidores.json"),
        &serde_json::to_value(Documento {
            schema: 1,
            servidores: v.to_vec(),
        })
        .map_err(|_| "No se pudo preparar la lista.")?,
    )
}

/// Desactiva la próxima sincronización únicamente para el vínculo completado.
pub fn finalizar_sincronizacion(root: &Path, origen: &Servidor) -> Result<Vec<Servidor>, String> {
    let mut lista = cargar(root)?;
    let servidor = lista.iter_mut().find(|s| s.id == origen.id
        && s.direccion == origen.direccion && s.instancia == origen.instancia)
        .ok_or("Los mods se sincronizaron, pero el servidor o su instancia vinculada cambiaron. No se modificó la preferencia.")?;
    servidor.sincronizar = false;
    guardar(root, &lista).map_err(|e| format!("Los mods se sincronizaron, pero no se pudo desactivar la sincronización automática: {e}"))?;
    Ok(lista)
}
#[derive(Debug, Clone)]
pub struct Estado {
    pub version: String,
    pub protocolo: i32,
    pub jugadores: u32,
    pub maximo: u32,
    pub motd: String,
    pub ms: u64,
}
fn varint(mut n: u32) -> Vec<u8> {
    let mut v = vec![];
    loop {
        let b = (n & 127) as u8;
        n >>= 7;
        v.push(b | if n > 0 { 128 } else { 0 });
        if n == 0 {
            return v;
        }
    }
}
async fn leer_varint(s: &mut TcpStream) -> Result<u32, String> {
    let mut n = 0;
    for i in 0..5 {
        let b = s
            .read_u8()
            .await
            .map_err(|_| "El servidor cerró la conexión.")?;
        if i == 4 && b > 15 {
            return Err("La longitud de respuesta no es válida.".into());
        }
        n |= ((b & 127) as u32) << (i * 7);
        if b & 128 == 0 {
            return Ok(n);
        }
    }
    Err("VarInt demasiado largo.".into())
}
async fn paquete(s: &mut TcpStream, bytes: &[u8]) -> Result<(), String> {
    s.write_all(&varint(bytes.len() as u32))
        .await
        .map_err(|_| "No se pudo enviar la consulta.")?;
    s.write_all(bytes)
        .await
        .map_err(|_| "No se pudo enviar la consulta.".into())
}
fn tomar_varint(b: &mut &[u8]) -> Result<u32, String> {
    let mut n = 0;
    for i in 0..5 {
        let (&c, rest) = b.split_first().ok_or("Respuesta incompleta.")?;
        *b = rest;
        if i == 4 && c > 15 {
            return Err("VarInt inválido.".into());
        }
        n |= ((c & 127) as u32) << (i * 7);
        if c & 128 == 0 {
            return Ok(n);
        }
    }
    Err("VarInt inválido.".into())
}
fn descripcion(v: &serde_json::Value, out: &mut String, depth: usize) {
    if depth > 16 || out.len() > 2048 {
        return;
    }
    if let Some(s) = v.as_str() {
        out.extend(s.chars().take(1024));
    } else if let Some(a) = v.as_array() {
        for x in a.iter().take(64) {
            descripcion(x, out, depth + 1);
        }
    } else {
        if let Some(t) = v.get("text") {
            descripcion(t, out, depth + 1);
        }
        if let Some(e) = v.get("extra") {
            descripcion(e, out, depth + 1);
        }
    }
}
async fn consultar_inner(d: Direccion) -> Result<Estado, String> {
    let (host, puerto) = resolver_destino(&d).await;
    let mut s = TcpStream::connect((host.as_str(), puerto))
        .await
        .map_err(|_| "No se pudo conectar; revisa la dirección, la red o ZeroTier.")?;
    // -1 solicita estado sin afirmar una versión de protocolo compatible.
    let mut handshake = vec![0];
    handshake.extend(varint(u32::MAX));
    handshake.extend(varint(d.host.len() as u32));
    handshake.extend(d.host.as_bytes());
    handshake.extend(d.puerto.to_be_bytes());
    handshake.push(1);
    paquete(&mut s, &handshake).await?;
    paquete(&mut s, &[0]).await?;
    let size = leer_varint(&mut s).await?;
    if size == 0 || size > 256 * 1024 {
        return Err("La respuesta del servidor supera el límite.".into());
    }
    let mut data = vec![0; size as usize];
    s.read_exact(&mut data)
        .await
        .map_err(|_| "Respuesta incompleta.")?;
    let mut b = data.as_slice();
    if tomar_varint(&mut b)? != 0 {
        return Err("Respuesta de estado inválida.".into());
    }
    let json_size = tomar_varint(&mut b)? as usize;
    if json_size != b.len() {
        return Err("Longitud de estado inválida.".into());
    }
    let v: serde_json::Value =
        serde_json::from_slice(b).map_err(|_| "El servidor devolvió un estado inválido.")?;
    let version = v["version"]["name"]
        .as_str()
        .filter(|s| s.len() <= 160)
        .ok_or("El servidor no publicó su versión.")?
        .to_owned();
    let protocolo = v["version"]["protocol"]
        .as_i64()
        .and_then(|p| i32::try_from(p).ok())
        .ok_or("Protocolo inválido.")?;
    let count = |key: &str| {
        v["players"][key]
            .as_u64()
            .and_then(|p| u32::try_from(p).ok())
            .ok_or("Recuento de jugadores inválido.")
    };
    let mut motd = String::new();
    descripcion(&v["description"], &mut motd, 0);
    let mut chars = motd.chars();
    let mut plain = String::new();
    while let Some(c) = chars.next() {
        if c == '§' {
            chars.next();
        } else if !c.is_control() || c == '\n' {
            plain.push(c);
        }
        if plain.len() > 2048 {
            break;
        }
    }
    let now = Instant::now();
    let ping = [1, 0, 0, 0, 0, 0, 0, 0, 1];
    paquete(&mut s, &ping).await?;
    let size = leer_varint(&mut s).await?;
    if size != 9 {
        return Err("Respuesta de latencia inválida.".into());
    }
    let mut pong = [0; 9];
    s.read_exact(&mut pong)
        .await
        .map_err(|_| "No se recibió la latencia.")?;
    if pong != ping {
        return Err("Respuesta de latencia inválida.".into());
    }
    Ok(Estado {
        version,
        protocolo,
        jugadores: count("online")?,
        maximo: count("max")?,
        motd: plain,
        ms: now.elapsed().as_millis() as u64,
    })
}
async fn resolver_destino(d: &Direccion) -> (String, u16) {
    let original = (d.host.clone(), d.puerto);
    if d.puerto != 25565 || d.host.parse::<IpAddr>().is_ok() {
        return original;
    }
    let Ok(resolver) = hickory_resolver::TokioAsyncResolver::tokio_from_system_conf() else {
        return original;
    };
    let lookup = tokio::time::timeout(
        Duration::from_millis(1200),
        resolver.srv_lookup(format!("_minecraft._tcp.{}", d.host)),
    )
    .await;
    if let Ok(Ok(records)) = lookup {
        let records: Vec<_> = records
            .iter()
            .take(64)
            .filter(|r| r.port() > 0 && !r.target().is_root())
            .map(|r| {
                (
                    r.priority(),
                    r.weight(),
                    r.target().to_utf8().trim_end_matches('.').to_owned(),
                    r.port(),
                )
            })
            .collect();
        let mut bytes = [0; 8];
        let _ = getrandom::fill(&mut bytes);
        if let Some(destino) = elegir_srv(&records, u64::from_le_bytes(bytes)) {
            return destino;
        }
    }
    original
}
fn elegir_srv(records: &[(u16, u16, String, u16)], random: u64) -> Option<(String, u16)> {
    let priority = records.iter().map(|r| r.0).min()?;
    let group: Vec<_> = records.iter().filter(|r| r.0 == priority).collect();
    let total: u64 = group.iter().map(|r| r.1 as u64).sum();
    if total == 0 {
        let r = group[(random % group.len() as u64) as usize];
        return Some((r.2.clone(), r.3));
    }
    let mut pick = random % total;
    for r in group {
        if pick < r.1 as u64 {
            return Some((r.2.clone(), r.3));
        }
        pick -= r.1 as u64;
    }
    None
}
pub async fn consultar(dir: &str, cancel: CancellationToken) -> Result<Estado, String> {
    let d = direccion(dir)?;
    tokio::select! {_ =cancel.cancelled()=>Err("Consulta cancelada.".into()),r=tokio::time::timeout(Duration::from_secs(5),consultar_inner(d))=>r.map_err(|_|"El servidor no respondió en cinco segundos.".to_string())?}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn srv_respeta_prioridad_peso_y_puerto() {
        let records = vec![
            (20, 100, "alto.example".into(), 25566),
            (10, 1, "primero.example".into(), 25567),
            (10, 3, "segundo.example".into(), 25568),
        ];
        assert_eq!(
            elegir_srv(&records, 0),
            Some(("primero.example".into(), 25567))
        );
        assert_eq!(
            elegir_srv(&records, 1),
            Some(("segundo.example".into(), 25568))
        );
        assert_eq!(
            elegir_srv(&records, 3),
            Some(("segundo.example".into(), 25568))
        );
        assert!(elegir_srv(&[], 0).is_none());
    }
    #[test]
    fn direcciones_no_admiten_argumentos_y_conservan_ipv6() {
        assert_eq!(direccion("[::1]:25566").unwrap().texto(), "[::1]:25566");
        assert_eq!(direccion("play.example.net").unwrap().puerto, 25565);
        for bad in [
            "http://host",
            "host:0",
            "host:65536",
            "host --foo",
            "a/b",
            "-bad.net",
            "[oops]",
            "a..b",
        ] {
            assert!(direccion(bad).is_err(), "{bad}");
        }
    }
    #[tokio::test]
    async fn consulta_local_valida_estado_y_pong() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let worker = tokio::spawn(async move {
            let (mut s, _) = listener.accept().await.unwrap();
            for _ in 0..2 {
                let n = leer_varint(&mut s).await.unwrap();
                let mut b = vec![0; n as usize];
                s.read_exact(&mut b).await.unwrap();
            }
            let json=br#"{"version":{"name":"26.3","protocol":999},"players":{"online":3,"max":20},"description":{"text":"Hola","extra":[{"text":" amigos"}]}}"#;
            let mut data = vec![0];
            data.extend(varint(json.len() as u32));
            data.extend(json);
            paquete(&mut s, &data).await.unwrap();
            assert_eq!(leer_varint(&mut s).await.unwrap(), 9);
            let mut ping = [0; 9];
            s.read_exact(&mut ping).await.unwrap();
            paquete(&mut s, &ping).await.unwrap();
        });
        let status = consultar(&addr.to_string(), CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(status.jugadores, 3);
        assert_eq!(status.motd, "Hola amigos");
        worker.await.unwrap();
        let c = CancellationToken::new();
        c.cancel();
        assert!(consultar(&addr.to_string(), c).await.is_err());
    }
    #[test]
    fn sincronizacion_completada_desactiva_solo_su_vinculo_y_lo_persiste() {
        let mut origen = nuevo("Amigos".into(), "localhost".into(), Some("1".repeat(32))).unwrap();
        origen.sincronizar = true;
        let root = std::env::temp_dir().join(format!("icaro-sync-preferencia-{}", origen.id));
        let mut otro = nuevo("Otro".into(), "localhost:25567".into(), None).unwrap();
        otro.sincronizar = true;
        let mut renombrado = origen.clone(); renombrado.nombre = "Nombre nuevo".into();
        guardar(&root, &[renombrado, otro.clone()]).unwrap();
        let lista = finalizar_sincronizacion(&root, &origen).unwrap();
        assert!(!lista[0].sincronizar); assert_eq!(lista[0].nombre, "Nombre nuevo");
        assert_eq!(lista[1], otro); assert_eq!(cargar(&root).unwrap(), lista);
        let mut cambiado = origen.clone(); cambiado.instancia = Some("2".repeat(32));
        guardar(&root, &[cambiado.clone(), otro]).unwrap();
        assert!(finalizar_sincronizacion(&root, &origen).is_err());
        assert_eq!(cargar(&root).unwrap()[0], cambiado);
        let root = root.canonicalize().unwrap();
        assert_eq!(root.parent(), Some(std::env::temp_dir().canonicalize().unwrap().as_path()));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn persistencia_no_confunde_identidad_ni_sobrescribe_datos_danados() {
        let root = std::env::temp_dir().join(format!(
            "icaro-servidores-{}",
            nuevo("x".into(), "localhost".into(), None).unwrap().id
        ));
        let s = nuevo("Amigos".into(), "localhost".into(), None).unwrap();
        guardar(&root, &[s.clone()]).unwrap();
        assert_eq!(cargar(&root).unwrap(), vec![s.clone()]);
        assert!(guardar(&root, &[s.clone(), s.clone()]).is_err());
        std::fs::write(root.join("servidores.json"), b"{bad").unwrap();
        assert!(cargar(&root).is_err());
        assert!(guardar(&root, &[s]).is_err());
        assert_eq!(
            std::fs::read(root.join("servidores.json")).unwrap(),
            b"{bad"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[tokio::test]
    async fn respuesta_excesiva_y_cancelacion_no_reservan_memoria_sin_limite() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let worker = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            for _ in 0..2 {
                let n = leer_varint(&mut stream).await.unwrap();
                let mut request = vec![0; n as usize];
                stream.read_exact(&mut request).await.unwrap();
            }
            stream.write_all(&varint(256 * 1024 + 1)).await.unwrap();
        });
        assert!(consultar(&addr.to_string(), CancellationToken::new())
            .await
            .unwrap_err()
            .contains("límite"));
        worker.await.unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let c = CancellationToken::new();
        let copy = c.clone();
        let worker = tokio::spawn(async move {
            let (_s, _) = listener.accept().await.unwrap();
            copy.cancel();
        });
        assert!(consultar(&addr.to_string(), c).await.is_err());
        worker.await.unwrap();
    }
}
