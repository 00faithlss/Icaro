// SPDX-License-Identifier: GPL-3.0-only
//! Consulta de diagnóstico; no une, abandona ni modifica redes.
#[tokio::main]
async fn main() {
    use icaro_core::zerotier::Estado;
    match icaro_core::zerotier::detectar().await {
        Estado::Ausente => println!("ZeroTier no está instalado."),
        Estado::SinAcceso => {
            println!("ZeroTier detectado; servicio no consultable por este usuario.")
        }
        Estado::Disponible {
            version,
            online,
            redes,
        } => println!(
            "ZeroTier {version}; en línea: {online}; redes: {}.",
            redes.len()
        ),
    }
}
