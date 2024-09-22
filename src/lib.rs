//! # passmate
//! Manage passwords with ease.
use aes_gcm::{
    aead::{self, Aead},
    AeadCore, Aes256Gcm, Key, KeyInit, Nonce,
};
use argon2::Argon2;
use rand::{rngs::OsRng, Rng};
use std::{
    collections::HashMap,
    fs::File,
    io::{ErrorKind, Read},
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PassmateError {
    #[error("An encryption error occurred: {0}")]
    Encrypt(aead::Error),
    #[error("A decryption error occurred: {0}")]
    Decrypt(aead::Error),
    #[error("Failed to make key: {0}")]
    EncryptionKey(argon2::Error),
    #[error("Failed to serialize vault: {0}")]
    Json(serde_json::Error),
    #[error("Error writing or reading vault: {0}")]
    IO(std::io::Error),
}

/// A container for passwords or other secrets.
pub struct Vault {
    path: PathBuf,
    passphrase: String,
    data: HashMap<String, String>,
}

impl Vault {
    /// Opens the vault at the given path or returns
    /// a empty vault if it doesn't already exist.
    ///
    /// # Errors
    /// May return an error if opening, decrypting, or deserializing the vault data fails.
    pub fn open(path: impl AsRef<Path>, passphrase: &str) -> Result<Self, PassmateError> {
        match File::open(&path) {
            Ok(mut file) => {
                let mut encrypted_data = Vec::new();
                file.read_to_end(&mut encrypted_data)
                    .map_err(PassmateError::IO)?;
                let (salt, encrypted_data) = encrypted_data.split_at(16);
                let key = make_key(passphrase, salt)?;
                let data = decrypt(key, encrypted_data)?;
                let data = serde_json::from_slice(&data).map_err(PassmateError::Json)?;
                Ok(Self {
                    path: PathBuf::from(path.as_ref()),
                    passphrase: passphrase.into(),
                    data,
                })
            }
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(Self {
                path: PathBuf::from(path.as_ref()),
                passphrase: passphrase.into(),
                data: HashMap::new(),
            }),
            Err(e) => Err(PassmateError::IO(e)),
        }
    }

    /// Returns a list of entry names in alphabetical order.
    #[must_use]
    pub fn entries(&self) -> Vec<String> {
        let mut entries: Vec<String> = self.data.keys().cloned().collect();
        entries.sort();
        entries
    }

    /// Looks up an entry by the given name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&String> {
        self.data.get(name)
    }

    /// Adds or updates an entry with the given name.
    pub fn set<S>(&mut self, name: S, value: S)
    where
        S: Into<String>,
    {
        self.data.insert(name.into(), value.into());
    }

    /// Removes the entry with the given name.
    pub fn remove(&mut self, name: &str) {
        self.data.remove(name);
    }

    /// Saves the vault to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if it fails to create and write
    /// to a file at the given path.
    pub fn save(&self) -> Result<(), PassmateError> {
        let salt = generate_salt();
        let key = make_key(&self.passphrase, &salt)?;

        let data = serde_json::to_vec(&self.data).map_err(PassmateError::Json)?;
        let encrypted_data = encrypt(key, &data)?;

        let mut contents = salt.to_vec();
        contents.extend_from_slice(&encrypted_data);

        std::fs::write(&self.path, &contents).map_err(PassmateError::IO)
    }
}

#[mutants::skip]
fn make_key(pwd: &str, salt: &[u8]) -> Result<[u8; 32], PassmateError> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(pwd.as_bytes(), salt, &mut key)
        .map_err(PassmateError::EncryptionKey)?;
    Ok(key)
}

#[mutants::skip]
fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill(&mut salt);
    salt
}

fn encrypt(key: [u8; 32], data: &[u8]) -> Result<Vec<u8>, PassmateError> {
    let key = Key::<Aes256Gcm>::from_slice(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message

    let cipher = Aes256Gcm::new(key);
    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(PassmateError::Encrypt)?;

    let mut encrypted_data = nonce.to_vec();
    encrypted_data.extend_from_slice(&ciphertext);

    Ok(encrypted_data)
}

fn decrypt(key: [u8; 32], encrypted_data: &[u8]) -> Result<Vec<u8>, PassmateError> {
    let key = Key::<Aes256Gcm>::from_slice(&key);
    let (nonce, ciphertext) = encrypted_data.split_at(12);
    Aes256Gcm::new(key)
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(PassmateError::Decrypt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_none, assert_ok};
    use std::{collections::HashMap, path::PathBuf};
    use tempfile::TempDir;

    #[test]
    fn open_returns_an_empty_vault_for_file_that_doesnt_exist() {
        let vault = Vault::open(PathBuf::from("doesnotexist"), "testpwd").unwrap();
        assert!(vault.data.is_empty());
    }

    #[test]
    fn open_opens_a_vault_with_existing_data() {
        let mut tmp = TempVault::new();
        tmp.vault.set("mypass", "test");
        assert_ok!(tmp.vault.save());

        let vault = Vault::open(tmp.vault.path, "testpwd").unwrap();
        assert_eq!(vault.data, tmp.vault.data);
    }

    #[test]
    fn set_adds_a_new_secret_to_the_vault_with_the_given_name() {
        let mut tmp = TempVault::new();

        tmp.vault.set("mypass", "test");

        assert_eq!(tmp.vault.data.get("mypass").unwrap(), "test");
    }

    #[test]
    fn entries_returns_the_names_of_the_vault_entries_in_alphabetical_order() {
        let mut tmp = TempVault::new();
        tmp.vault.set("pass1", "test");
        tmp.vault.set("pass2", "test");

        let want: Vec<String> = vec!["pass1".into(), "pass2".into()];
        let got = tmp.vault.entries();
        assert_eq!(want, got);
    }

    #[test]
    fn get_retrieves_a_secret_from_the_vault_with_the_given_name() {
        let mut tmp = TempVault::new();
        tmp.vault.set("mypass", "test");

        assert_eq!(tmp.vault.get("mypass"), Some(&"test".to_string()));
    }

    #[test]
    fn get_returns_none_if_a_secret_does_not_exist_by_the_given_name() {
        let tmp = TempVault::new();

        assert_none!(tmp.vault.get("mypass"));
    }

    #[test]
    fn set_updates_an_existing_secret_if_it_already_exists_by_the_given_name() {
        let mut tmp = TempVault::new();
        tmp.vault.set("mypass", "test");
        tmp.vault.set("mypass", "newtest");
        assert_eq!(tmp.vault.get("mypass").unwrap(), "newtest");
    }

    #[test]
    fn remove_deletes_the_secret_with_the_given_name_from_the_vault() {
        let mut tmp = TempVault::new();
        tmp.vault.set("mypass", "test");

        tmp.vault.remove("mypass");

        assert_none!(tmp.vault.get("mypass"));
    }

    #[test]
    fn save_persists_the_vaults_data_to_disk_as_json() {
        let mut temp_vault = TempVault::new();
        temp_vault.vault.set("mypass", "test");
        assert_ok!(temp_vault.vault.save());

        let got = Vault::open(&temp_vault.vault.path, &temp_vault.vault.passphrase).unwrap();
        let want = HashMap::from([("mypass".into(), "test".into())]);
        assert_eq!(got.data, want);
    }

    #[test]
    fn data_can_be_encrypted_and_decrypted() {
        let salt = generate_salt();
        let key = make_key("testpass", &salt).expect("failed to make key");
        let original_plaintext = "this is a test";
        let ciphertext =
            encrypt(key, original_plaintext.as_bytes()).expect("failed to encrypt data");
        let decrypted_plaintext = decrypt(key, &ciphertext).expect("failed to decrypted data");
        assert_eq!(
            original_plaintext,
            String::from_utf8_lossy(&decrypted_plaintext)
        );
    }

    #[test]
    fn encrypted_a_value_should_produce_different_results_each_time() {
        let salt = generate_salt();
        let key = make_key("testpass", &salt).expect("failed to make key");
        let plaintext = "this is a test";
        let ciphertext1 = encrypt(key, plaintext.as_bytes()).expect("failed to encrypt data");
        let ciphertext2 = encrypt(key, plaintext.as_bytes()).expect("failed to encrypt data");
        assert_ne!(ciphertext1, ciphertext2);
    }

    #[test]
    fn decrypting_a_tampered_with_ciphertext_should_return_an_error() {
        let salt = generate_salt();
        let key = make_key("testpass", &salt).expect("failed to make key");
        let plaintext = "this is a test";
        let mut ciphertext = encrypt(key, plaintext.as_bytes()).expect("failed to encrypt data");
        ciphertext[0] = 0;
        assert_err!(decrypt(key, &ciphertext));
    }

    struct TempVault {
        _temp_dir: TempDir,
        vault: Vault,
    }

    impl TempVault {
        fn new() -> Self {
            let temp_dir = TempDir::new().unwrap();
            let path = temp_dir.path().join("test.vault");
            Self {
                _temp_dir: temp_dir,
                vault: Vault::open(path, "testpwd").unwrap(),
            }
        }
    }
}
