// SPDX-License-Identifier: GPL-3.0-only
//! Migra exclusivamente .icaro-minecraft a .icaro, conservando el perfil.
use icaro_core::{instancias::Almacen, minecraft};
use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let root = PathBuf::from(args.next().ok_or("Falta la raíz de datos")?);
    if args.next().is_some() {
        return Err("Argumentos adicionales no admitidos".into());
    }
    let store = Almacen::nuevo(root.join("instances"));
    let report = store.cargar()?;
    if report.omitidas > 0 {
        return Err("Hay instancias que no se pudieron leer".into());
    }
    for instance in report.instancias {
        let path = minecraft::migrar_directorio(&store, &instance)?;
        println!(
            "{}: {} (lista: {})",
            instance.nombre,
            path.display(),
            minecraft::listo(&store, &instance)
        );
    }
    Ok(())
}
