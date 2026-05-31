use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct Storage {
    path: String,
    data: HashMap<String, String>,
    dirty: bool,
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

    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        if self.data.get(key) != Some(&value.to_string()) {
            self.data.insert(key.to_string(), value.to_string());
            self.dirty = true;
        }
    }

    pub fn save(&mut self) {
        if !self.dirty {
            return;
        }
        if let Some(parent) = Path::new(&self.path).parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut file) = fs::File::create(&self.path) {
            for (key, value) in self.data.iter() {
                let _ = writeln!(file, "{key}={value}");
            }
            self.dirty = false;
        }
    }
}
