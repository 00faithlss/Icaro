// SPDX-License-Identifier: GPL-3.0-only
//! Reorganiza ajustes y papeleras antiguas sin iniciar el juego.
use icaro_core::almacenamiento;
use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let root = PathBuf::from(args.next().ok_or("Falta la raíz de datos")?);
    if args.next().is_some() {
        return Err("Argumentos adicionales no admitidos".into());
    }
    almacenamiento::regular(&root)?;
    almacenamiento::migrar_configuraciones(&root)?;
    let recovered = almacenamiento::recuperar_operaciones(&root)?;
    let moved = almacenamiento::trasladar_papeleras(&root)?;
    almacenamiento::registrar(&root, "Reorganización de datos completada")?;
    let report = almacenamiento::medir(&root)?;
    println!("Ajustes migrados; {recovered} operaciones recuperadas; {moved} elementos enviados a la papelera del sistema.");
    for (label, size) in report.categorias {
        println!("{label}: {size} bytes");
    }
    Ok(())
}
