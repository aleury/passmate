//! # passmate
//! Manage passwords with ease.
#![allow(unused)]

use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader, BufWriter, ErrorKind},
    path::PathBuf,
};

struct Vault {
    path: PathBuf,
    data: HashMap<String, String>,
}

impl Vault {
    fn open(path: PathBuf) -> io::Result<Self> {
        match File::open(&path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let data = serde_json::from_reader(reader)?;
                Ok(Self { path, data })
            }
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(Self {
                path,
                data: HashMap::new(),
            }),
            Err(e) => Err(e),
        }
    }

    fn size(&self) -> usize {
        self.data.len()
    }

    fn get(&self, name: &str) -> Option<&String> {
        self.data.get(name)
    }

    fn set(&mut self, name: &str, value: &str) {
        self.data.insert(name.into(), value.into());
    }

    fn remove(&mut self, name: &str) {
        self.data.remove(name);
    }

    fn save(&self) -> io::Result<()> {
        let file = File::create(&self.path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &self.data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Vault;
    use claims::{assert_none, assert_ok};
    use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf};
    use tempfile::TempDir;

    #[test]
    fn open_returns_an_empty_vault_for_file_that_doesnt_exist() {
        let vault = Vault::open(PathBuf::from("doesnotexist")).unwrap();
        assert!(vault.data.is_empty());
    }

    #[test]
    fn open_opens_a_vault_with_existing_data() {
        let mut tmp = TempVault::new();
        tmp.vault.set("mypass", "test");
        assert_ok!(tmp.vault.save());

        let vault = Vault::open(tmp.vault.path).unwrap();
        assert_eq!(vault.data, tmp.vault.data);
    }

    #[test]
    fn size_returns_zero_for_an_empty_vault() {
        let tmp = TempVault::new();
        assert_eq!(tmp.vault.size(), 0);
    }

    #[test]
    fn size_returns_the_count_of_the_items_in_a_nonempty_vault() {
        let mut tmp = TempVault::new();
        tmp.vault.data.extend([
            ("mypass".into(), "test".into()),
            ("mypass2".into(), "test".into()),
        ]);
        assert_eq!(tmp.vault.size(), 2);
    }

    #[test]
    fn set_adds_a_new_secret_to_the_vault_with_the_given_name() {
        let mut tmp = TempVault::new();

        tmp.vault.set("mypass", "test");

        assert_eq!(tmp.vault.data.get("mypass").unwrap(), "test");
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

        let got = read_vault_data_from_file(&temp_vault.vault.path);
        let want = HashMap::from([("mypass".into(), "test".into())]);
        assert_eq!(got, want);
    }

    fn read_vault_data_from_file(path: &PathBuf) -> HashMap<String, String> {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
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
                vault: Vault::open(path).unwrap(),
            }
        }
    }
}
