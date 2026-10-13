// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

//! Prepara una instancia existente sin abrir el juego ni acceder a cuentas.
//! Uso: cargo run -p icaro-core --example preparar_minecraft -- <datos> <id>
use icaro_core::{instancias::Almacen, minecraft};
use std::{path::PathBuf, sync::Mutex};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let data = PathBuf::from(args.next().ok_or("Falta la raíz de datos")?);
    let id = args.next().ok_or("Falta el ID de instancia")?;
    if args.next().is_some() {
        return Err("Argumentos adicionales no admitidos".into());
    }
    let store = Almacen::nuevo(data.join("instances"));
    let instance = store
        .cargar()?
        .instancias
        .into_iter()
        .find(|i| std::ffi::OsStr::new(&i.id) == id)
        .ok_or("Instancia no encontrada")?;
    let last = Mutex::new(String::new());
    let installed = minecraft::instalar(&store, &instance, CancellationToken::new(), |p| {
        let mut last = last.lock().unwrap();
        if *last != p.etapa {
            println!("{} ({}/{})", p.etapa, p.completados, p.total);
            *last = p.etapa;
        }
    })
    .await?;
    if !minecraft::listo(&store, &instance) {
        return Err("El perfil no pasó la comprobación final".into());
    }
    println!(
        "Preparada: {} / {} / {} entradas de classpath",
        instance.nombre,
        installed.version,
        installed.classpath.len()
    );
    println!("Assets: {}", installed.assets.display());
    Ok(())
}
