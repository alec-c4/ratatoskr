use ratatoskr_core::key_vault::KeyVault;
use tempfile::tempdir;

#[test]
fn test_key_generation_and_persistence() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_identity.key");

    // 1. Generate and Save
    let (vault, _mnemonic) = KeyVault::generate_with_mnemonic();
    vault.save_to_file(&file_path).expect("Save failed");

    // 2. Load
    let loaded_vault = KeyVault::load_from_file(&file_path).expect("Load failed");

    // 3. Verify
    assert_eq!(vault.public_key_hex(), loaded_vault.public_key_hex());
}

#[test]
fn test_mnemonic_recovery() {
    // 1. Generate
    let (vault_original, mnemonic) = KeyVault::generate_with_mnemonic();
    println!("Mnemonic: {}", mnemonic);

    // 2. Recover from string
    let vault_recovered = KeyVault::recover(&mnemonic).expect("Recovery failed");

    // 3. Verify keys match
    assert_eq!(
        vault_original.public_key_hex(),
        vault_recovered.public_key_hex()
    );
}

#[test]
fn test_stable_mnemonic_recovery() {
    // Known test vector (though we generate random usually, checking deterministic property here)
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let vault1 = KeyVault::recover(phrase).unwrap();
    let vault2 = KeyVault::recover(phrase).unwrap();

    assert_eq!(vault1.public_key_hex(), vault2.public_key_hex());
}
