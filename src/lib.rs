//! # passmate
//! Manage passwords with ease.
#![allow(unused)]

use std::collections::HashMap;

struct Vault {
    data: HashMap<String, String>,
}

impl Vault {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
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
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::Vault;

    #[test]
    fn new_returns_an_empty_vault() {
        let vault = Vault::new();
        assert!(vault.data.is_empty());
    }

    #[test]
    fn size_returns_zero_for_an_empty_vault() {
        let vault = Vault::new();
        assert_eq!(vault.size(), 0);
    }

    #[test]
    fn size_returns_the_count_of_the_items_in_a_nonempty_vault() {
        let mut vault = Vault {
            data: HashMap::from([
                ("mypass".into(), "test".into()),
                ("mypass2".into(), "test".into()),
            ]),
        };
        assert_eq!(vault.size(), 2);
    }

    #[test]
    fn set_adds_a_new_secret_to_the_vault_with_the_given_name() {
        let mut vault = Vault::new();

        vault.set("mypass", "test");

        assert_eq!(vault.data.get("mypass").unwrap(), "test");
    }

    #[test]
    fn get_retrieves_a_secret_from_the_vault_with_the_given_name() {
        let mut vault = Vault::new();

        vault.set("mypass", "test");

        assert_eq!(vault.get("mypass"), Some(&"test".to_string()));
    }

    #[test]
    fn get_returns_none_if_a_secret_does_not_exist_by_the_given_name() {
        let vault = Vault::new();

        assert_eq!(vault.get("mypass"), None);
    }

    #[test]
    fn set_updates_an_existing_secret_if_it_already_exists_by_the_given_name() {
        let mut vault = Vault::new();
        vault.set("mypass", "test");
        vault.set("mypass", "newtest");
        assert_eq!(vault.get("mypass").unwrap(), "newtest");
    }

    #[test]
    fn remove_deletes_the_secret_with_the_given_name_from_the_vault() {
        let mut vault = Vault::new();
        vault.set("mypass", "test");

        vault.remove("mypass");

        assert_eq!(vault.get("mypass"), None);
    }
}
