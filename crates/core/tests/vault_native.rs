// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

#![cfg(windows)]

#[test]
#[ignore = "Comprueba Credential Manager con una entrada de prueba aislada y la elimina"]
fn almacen_windows_guarda_lee_y_elimina_credencial_de_prueba() {
    let mut nonce = [0u8; 16];
    getrandom::fill(&mut nonce).unwrap();
    let name: String = nonce.iter().map(|b| format!("{b:02x}")).collect();
    let entry = keyring::Entry::new("org.icaro.launcher.tests", &name).unwrap();
    struct Cleanup(keyring::Entry);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = self.0.delete_credential();
        }
    }
    let entry = Cleanup(entry);
    entry.0.set_password("ICARO_CANARY_TEST_ONLY").unwrap();
    assert_eq!(entry.0.get_password().unwrap(), "ICARO_CANARY_TEST_ONLY");
    entry.0.delete_credential().unwrap();
    assert!(matches!(
        entry.0.get_password(),
        Err(keyring::Error::NoEntry)
    ));
}
