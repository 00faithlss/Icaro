// SPDX-License-Identifier: GPL-3.0-only
//! Servicio explícito del anfitrión; solo publica el índice de una instancia cliente.
use icaro_core::{instancias::Almacen, modrinth::Catalog, mods::sincronizacion as sync};
use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
#[tokio::main]
async fn main() {
    if let Err(error) = ejecutar().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
async fn ejecutar() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut data = None;
    let mut id = None;
    let mut listen = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--datos" => data = args.next().map(PathBuf::from),
            "--instancia" => id = args.next(),
            "--escuchar" => listen = args.next(),
            "--help" | "-h" => {
                println!("icaro-sync-host --datos <carpeta Icaro> --instancia <ID> --escuchar <IP ZeroTier:puerto Minecraft+1>\nComparte solo versiones y hashes de los mods de una instancia cliente. Ctrl+C detiene el servicio.");
                return Ok(());
            }
            _ => return Err("Argumento desconocido. Usa --help.".into()),
        }
    }
    let data = data.ok_or("Indica --datos con la carpeta Icaro del anfitrión.")?;
    if !data.is_absolute() {
        return Err("La carpeta de datos debe ser absoluta.".into());
    }
    let id = id.ok_or("Indica --instancia con el ID guardado en instance.json.")?;
    let addr: SocketAddr = listen
        .ok_or("Indica --escuchar con la IP privada de ZeroTier y puerto Minecraft+1.")?
        .parse()
        .map_err(|_| "Dirección de escucha inválida.")?;
    // Validación común: rechaza comodines y direcciones públicas.
    if addr.port() < 2
        || sync::endpoint(&SocketAddr::new(addr.ip(), addr.port() - 1).to_string())? != addr
    {
        return Err("Dirección de escucha inválida.".into());
    }
    let store = Almacen::nuevo(data.join("instances"));
    let catalog = Catalog::new().map_err(|e| e.to_string())?;
    let published = Arc::new(RwLock::new(None));
    let cancel = CancellationToken::new();
    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|_| {
        "No se pudo abrir el puerto del servicio. Comprueba la IP de ZeroTier y el cortafuegos."
    })?;
    println!("Servicio de mods en {addr}. Preparando el índice; no compartirá archivos ni datos de cuenta.");
    let service = tokio::spawn(sync::servir(listener, published.clone(), cancel.clone()));
    let mut last = None;
    loop {
        let refresh = async {
            let instance = store
                .cargar()
                .map_err(|e| e.to_string())?
                .instancias
                .into_iter()
                .find(|i| i.id == id)
                .ok_or("La instancia elegida no existe.")?;
            let signature = (
                sync::huella(&store, &id)?,
                instance.minecraft.clone(),
                instance.loader.clone(),
                icaro_core::versiones_instancia::leer_loader(&store, &instance),
            );
            if last.as_ref() != Some(&signature) {
                // Un cambio invalida el índice anterior antes de consultar Modrinth.
                *published.write().await = None;
                let index = sync::publicar(&catalog, &store, &instance).await?;
                let count = index.mods.len();
                *published.write().await = Some(index);
                last = Some(signature);
                println!("Índice actualizado: {count} mods comunes de Modrinth.");
            }
            Ok::<_, String>(())
        };
        tokio::select! {
            _=tokio::signal::ctrl_c()=>break,
            result=tokio::time::timeout(Duration::from_secs(120),refresh)=>{
                if let Err(error)=result.unwrap_or_else(|_|Err("La preparación del índice agotó el tiempo de espera.".into())) { *published.write().await=None; last=None; eprintln!("{error}"); }
            }
        }
        tokio::select! {_=tokio::signal::ctrl_c()=>break, _=tokio::time::sleep(Duration::from_secs(10))=>{}}
        if service.is_finished() {
            break;
        }
    }
    cancel.cancel();
    service.await.map_err(|_| "El servicio no pudo cerrarse.")?
}
