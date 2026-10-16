// SPDX-License-Identifier: GPL-3.0-only
//! Inventario incremental de `mods`: reutiliza hashes e identidades Fabric de
//! archivos cuyos metadatos no cambiaron desde la última lectura verificada.
//!
//! El inventario es una caché, nunca una fuente de verdad: si falta, está
//! corrupto o es dudoso, se vuelve a leer el archivo completo.
use super::*;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Junto al bloqueo de la carpeta de juego; la exportación excluye `.icaro-*`.
pub(super) const ARCHIVO: &str = ".icaro-mods-inventario.json";
const ESQUEMA: u32 = 1;
const MAX_BYTES: u64 = 8 * 1024 * 1024;
/// Cada archivo se vuelve a leer completo al menos una vez por semana.
const VIGENCIA: Duration = Duration::from_secs(7 * 24 * 3600);
/// Un archivo modificado hace menos de esto podría cambiar sin que cambie su
/// fecha (resolución del sistema de archivos); no se guarda en caché.
const MARGEN_RECIENTE: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Inventario {
    esquema: u32,
    archivos: BTreeMap<String, Entrada>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entrada {
    bytes: u64,
    modificado: u64,
    creado: Option<u64>,
    nodo: u64,
    sha512: String,
    /// Segundos desde epoch de la última lectura completa del contenido.
    verificado: u64,
    fabric_leido: bool,
    fabric_id: Option<String>,
}

fn nanos(t: Option<SystemTime>) -> Option<u64> {
    t.and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .and_then(|d| u64::try_from(d.as_nanos()).ok())
}
fn ahora() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Resultado de leer `mods` con ayuda del inventario.
pub(super) struct Lectura {
    pub huella: Vec<(String, String)>,
    pub releidos: usize,
    pub reutilizados: usize,
}

impl Inventario {
    /// Un inventario ilegible equivale a uno vacío: solo cuesta releer.
    pub(super) fn cargar(game: &Path) -> Self {
        let ruta = game.join(ARCHIVO);
        let leido = (|| {
            if regular(&ruta, false).ok()?.len() > MAX_BYTES {
                return None;
            }
            serde_json::from_slice::<Inventario>(&fs::read(&ruta).ok()?).ok()
        })();
        leido.filter(|i| i.esquema == ESQUEMA).unwrap_or_default()
    }

    pub(super) fn guardar(&self, game: &Path) -> Result<(), String> {
        let valor = serde_json::to_value(self).map_err(err)?;
        crate::almacenamiento::escribir_json(&game.join(ARCHIVO), &valor)
    }

    fn vigente(&self, estado: &EstadoArchivo, ahora: u64) -> Option<&Entrada> {
        let entrada = self.archivos.get(&estado.ruta)?;
        let coincide = entrada.bytes == estado.bytes
            && Some(entrada.modificado) == nanos(estado.modificado)
            && entrada.creado == nanos(estado.creado)
            && entrada.nodo == estado.nodo
            && entrada.sha512.len() == 128;
        let reciente = ahora.saturating_sub(entrada.verificado) < VIGENCIA.as_secs();
        (coincide && reciente).then_some(entrada)
    }

    /// Calcula la huella de `mods` releyendo solo lo nuevo, modificado o caducado.
    /// `estado` debe venir de [`estado`](super::estado) de la misma carpeta.
    pub(super) fn leer(
        &mut self,
        dir: &Path,
        estado: &[EstadoArchivo],
        cancel: &CancellationToken,
    ) -> Result<Lectura, String> {
        let momento = ahora();
        let limite_reciente = SystemTime::now().checked_sub(MARGEN_RECIENTE);
        let mut huella = Vec::with_capacity(estado.len());
        let (mut releidos, mut reutilizados, mut total) = (0, 0, 0u64);
        let mut siguiente = BTreeMap::new();
        for archivo in estado {
            if cancel.is_cancelled() {
                return Err(CANCELADA.into());
            }
            total = total
                .checked_add(archivo.bytes)
                .ok_or("La carpeta de mods es demasiado grande.")?;
            if archivo.bytes > MAX_FILE || total > 4 * 1024 * 1024 * 1024 {
                return Err("La carpeta de mods supera los límites de sincronización.".into());
            }
            let previa = self.vigente(archivo, momento).cloned();
            let sha = match &previa {
                Some(entrada) => {
                    reutilizados += 1;
                    entrada.sha512.clone()
                }
                None => {
                    releidos += 1;
                    hash_archivo(&dir.join(&archivo.ruta)).map_err(err)?
                }
            };
            let estable = archivo
                .modificado
                .zip(limite_reciente)
                .is_some_and(|(m, limite)| m < limite);
            if let (true, Some(modificado)) = (estable, nanos(archivo.modificado)) {
                let (verificado, fabric_leido, fabric_id) = match previa {
                    Some(p) => (p.verificado, p.fabric_leido, p.fabric_id),
                    None => (momento, false, None),
                };
                siguiente.insert(
                    archivo.ruta.clone(),
                    Entrada {
                        bytes: archivo.bytes,
                        modificado,
                        creado: nanos(archivo.creado),
                        nodo: archivo.nodo,
                        sha512: sha.clone(),
                        verificado,
                        fabric_leido,
                        fabric_id,
                    },
                );
            }
            huella.push((archivo.ruta.clone(), sha));
        }
        // Los archivos retirados desaparecen del inventario.
        self.esquema = ESQUEMA;
        self.archivos = siguiente;
        huella.sort();
        Ok(Lectura { huella, releidos, reutilizados })
    }

    /// Identidad Fabric guardada para un archivo cuyo hash coincide con el leído.
    pub(super) fn identidad(&self, ruta: &str, sha512: &str) -> Option<Option<String>> {
        self.archivos
            .get(ruta)
            .filter(|e| e.fabric_leido && e.sha512 == sha512)
            .map(|e| e.fabric_id.clone())
    }

    pub(super) fn recordar_identidad(&mut self, ruta: &str, id: Option<String>) {
        if let Some(entrada) = self.archivos.get_mut(ruta) {
            entrada.fabric_leido = true;
            entrada.fabric_id = id;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn carpeta() -> PathBuf {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).unwrap();
        let dir = std::env::temp_dir().join(format!("icaro-inventario-{}", hash(&bytes)));
        fs::create_dir_all(dir.join("mods")).unwrap();
        dir
    }
    fn envejecer(ruta: &Path) {
        let hace = SystemTime::now() - Duration::from_secs(3600);
        fs::File::options().write(true).open(ruta).unwrap().set_modified(hace).unwrap();
    }
    fn leer(inv: &mut Inventario, game: &Path) -> Lectura {
        let dir = game.join("mods");
        inv.leer(&dir, &estado(&dir).unwrap(), &CancellationToken::new()).unwrap()
    }

    #[test]
    fn reutiliza_solo_archivos_sin_cambios_y_relee_los_modificados() {
        let game = carpeta();
        let mods = game.join("mods");
        for (nombre, contenido) in [("a.jar", "uno"), ("b.jar", "dos"), ("c.jar", "tres")] {
            fs::write(mods.join(nombre), contenido).unwrap();
            envejecer(&mods.join(nombre));
        }
        let mut inv = Inventario::cargar(&game);
        let primera = leer(&mut inv, &game);
        assert_eq!((primera.releidos, primera.reutilizados), (3, 0));
        inv.guardar(&game).unwrap();

        let mut inv = Inventario::cargar(&game);
        let segunda = leer(&mut inv, &game);
        assert_eq!((segunda.releidos, segunda.reutilizados), (0, 3));
        assert_eq!(segunda.huella, fingerprint(&mods).unwrap());

        // Mismo tamaño y otra fecha: se vuelve a leer y el hash es el nuevo.
        fs::write(mods.join("b.jar"), "DOS").unwrap();
        envejecer(&mods.join("b.jar"));
        fs::remove_file(mods.join("c.jar")).unwrap();
        let tercera = leer(&mut inv, &game);
        assert_eq!((tercera.releidos, tercera.reutilizados), (1, 1));
        assert_eq!(tercera.huella, fingerprint(&mods).unwrap());
        assert!(!inv.archivos.contains_key("c.jar"));
        fs::remove_dir_all(game).unwrap();
    }

    #[test]
    fn archivos_recientes_caducados_o_inventario_corrupto_se_releen() {
        let game = carpeta();
        let mods = game.join("mods");
        fs::write(mods.join("reciente.jar"), "recién escrito").unwrap();
        fs::write(mods.join("viejo.jar"), "viejo").unwrap();
        envejecer(&mods.join("viejo.jar"));
        let mut inv = Inventario::default();
        leer(&mut inv, &game);
        // El archivo recién modificado no entra en la caché.
        assert!(!inv.archivos.contains_key("reciente.jar"));
        assert!(inv.archivos.contains_key("viejo.jar"));

        inv.archivos.get_mut("viejo.jar").unwrap().verificado = ahora() - VIGENCIA.as_secs() - 1;
        let caducado = leer(&mut inv, &game);
        assert_eq!(caducado.releidos, 2);
        assert!(inv.archivos["viejo.jar"].verificado >= ahora() - 5);

        fs::write(game.join(ARCHIVO), b"{no es json").unwrap();
        let mut corrupto = Inventario::cargar(&game);
        assert!(corrupto.archivos.is_empty());
        assert_eq!(leer(&mut corrupto, &game).releidos, 2);
        fs::remove_dir_all(game).unwrap();
    }

    #[test]
    fn la_identidad_fabric_solo_se_reutiliza_con_el_mismo_hash() {
        let game = carpeta();
        let mods = game.join("mods");
        fs::write(mods.join("a.jar"), "uno").unwrap();
        envejecer(&mods.join("a.jar"));
        let mut inv = Inventario::default();
        let lectura = leer(&mut inv, &game);
        let sha = &lectura.huella[0].1;
        assert_eq!(inv.identidad("a.jar", sha), None);
        inv.recordar_identidad("a.jar", Some("sodium".into()));
        assert_eq!(inv.identidad("a.jar", sha), Some(Some("sodium".into())));
        assert_eq!(inv.identidad("a.jar", "otro"), None);
        fs::remove_dir_all(game).unwrap();
    }
}
