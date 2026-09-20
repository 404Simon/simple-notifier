use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct Storage {
    path: String,
    data: HashMap<String, String>,
    dirty: bool,
}

pub struct Transaction {
    data: HashMap<String, String>,
}

impl Storage {
    pub fn load(path: &str) -> Self {
        let data = Self::read_file(path);
        Self {
            path: path.to_string(),
            data,
            dirty: false,
        }
    }

    fn read_file(path: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Ok(file) = fs::File::open(path) {
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = trimmed.split_once('=') {
                    map.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
        map
    }

    pub fn transaction(&self) -> Transaction {
        Transaction {
            data: self.data.clone(),
        }
    }

    pub fn commit(&mut self, transaction: Transaction) {
        if self.data != transaction.data {
            self.data = transaction.data;
            self.dirty = true;
        }
    }

    pub fn save(&mut self) -> Result<(), String> {
        if !self.dirty {
            return Ok(());
        }

        let path = Path::new(&self.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create state directory: {e}"))?;
        }

        let temp_path = path.with_extension("tmp");
        {
            let mut file = fs::File::create(&temp_path)
                .map_err(|e| format!("failed to create temporary state file: {e}"))?;
            for (key, value) in self.data.iter() {
                writeln!(file, "{key}={value}")
                    .map_err(|e| format!("failed to write state: {e}"))?;
            }
            file.sync_all()
                .map_err(|e| format!("failed to sync state: {e}"))?;
        }

        fs::rename(&temp_path, path).map_err(|e| format!("failed to replace state file: {e}"))?;
        self.dirty = false;
        Ok(())
    }
}

impl Transaction {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(String::as_str)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discarded_transaction_does_not_change_storage() {
        let storage = Storage::load("/does/not/exist");
        let mut transaction = storage.transaction();
        transaction.set("key", "new");

        drop(transaction);

        assert_eq!(storage.transaction().get("key"), None);
        assert!(!storage.dirty);
    }

    #[test]
    fn committed_transaction_changes_storage() {
        let mut storage = Storage::load("/does/not/exist");
        let mut transaction = storage.transaction();
        transaction.set("key", "new");

        storage.commit(transaction);

        assert_eq!(storage.transaction().get("key"), Some("new"));
        assert!(storage.dirty);
    }
}
