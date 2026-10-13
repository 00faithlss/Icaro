// SPDX-License-Identifier: GPL-3.0-only
//! Papelera del sistema. Nunca degrada un fallo a borrado permanente.
use std::path::{Path, PathBuf};
#[derive(Debug, Clone)]
pub struct Ticket {
    pub original: PathBuf,
    item: trash::TrashItem,
}
fn comparable(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{unc}"));
    }
    PathBuf::from(text.strip_prefix(r"\\?\").unwrap_or(&text))
}
pub fn enviar(path: &Path) -> Result<Ticket, String> {
    crate::almacenamiento::regular(path)?;
    let original = comparable(&path.canonicalize().map_err(|e| e.to_string())?);
    let previous = trash::os_limited::list().map_err(|e| e.to_string())?;
    enviar_nativo(&original)?;
    let candidates = trash::os_limited::list()
        .map_err(|e| {
            format!("El archivo está en la papelera; no se pudo leer la recuperación: {e}")
        })?
        .into_iter()
        .filter(|i| !previous.iter().any(|p| p.id == i.id))
        .collect::<Vec<_>>();
    let item = candidates
        .into_iter()
        .find(|i| comparable(&i.original_path()) == original)
        .ok_or("El archivo está en la papelera del sistema; recupéralo desde ella.")?;
    Ok(Ticket { original, item })
}
#[cfg(not(windows))]
fn enviar_nativo(path: &Path) -> Result<(), String> {
    trash::delete(path).map_err(|e| e.to_string())
}
#[cfg(windows)]
fn enviar_nativo(path: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::{
        core::PCWSTR,
        Win32::{System::Com::*, UI::Shell::*},
    };
    // La consulta anterior inicializa COM en este hilo mediante trash.
    let name: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    if name.len() >= 260 {
        return Err("La ruta es demasiado larga para la papelera de Windows; acorta el nombre o la ubicación antes de eliminarla.".into());
    }
    unsafe {
        let operation: IFileOperation =
            CoCreateInstance(&FileOperation, None, CLSCTX_ALL).map_err(|e| e.to_string())?;
        // Recycle-on-delete y early-failure: no aceptar un borrado permanente.
        operation
            .SetOperationFlags(
                FILEOPERATION_FLAGS(0x00080000 | 0x00100000 | 0x20000000)
                    | FOF_NO_UI
                    | FOF_WANTNUKEWARNING,
            )
            .map_err(|e| e.to_string())?;
        let item: IShellItem =
            SHCreateItemFromParsingName(PCWSTR(name.as_ptr()), None).map_err(|e| e.to_string())?;
        operation
            .DeleteItem(&item, None)
            .map_err(|e| e.to_string())?;
        operation.PerformOperations().map_err(|e| e.to_string())?;
        if operation
            .GetAnyOperationsAborted()
            .map_err(|e| e.to_string())?
            .as_bool()
        {
            return Err(
                "Windows no pudo enviar el elemento a la papelera; la operación se canceló.".into(),
            );
        }
    }
    Ok(())
}
pub fn restaurar(ticket: &Ticket) -> Result<(), String> {
    if ticket.original.exists() {
        return Err("Ya existe un archivo o carpeta en el destino de recuperación.".into());
    }
    trash::os_limited::restore_all([ticket.item.clone()]).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "Prueba nativa de papelera: crea y recupera un archivo temporal propio"]
    fn papelera_nativa_recupera_sin_sobrescribir() {
        let root = std::env::temp_dir().join(format!("icaro-trash-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("original.txt");
        std::fs::write(&path, b"original").unwrap();
        let ticket = super::enviar(&path).unwrap();
        assert!(!path.exists());
        std::fs::write(&path, b"colision").unwrap();
        assert!(super::restaurar(&ticket).is_err());
        std::fs::remove_file(&path).unwrap();
        super::restaurar(&ticket).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"original");
        std::fs::remove_dir_all(root).unwrap();
    }
}
