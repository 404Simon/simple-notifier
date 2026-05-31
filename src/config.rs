use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct Config {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub to_email: String,
    pub state_file: String,
    pub check_interval_secs: u64,
    pub random_delay_min_secs: u64,
    pub random_delay_max_secs: u64,
    pub send_test_mail_on_startup: bool,
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_bool(key: &str) -> bool {
    env::var(key)
        .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

fn env_parse_or<T: std::str::FromStr>(key: &str, default: T) -> T {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

impl Config {
    pub fn from_env() -> Self {
        load_dotenv();

        Self {
            smtp_host: env_or("SMTP_HOST", "localhost"),
            smtp_port: env_parse_or("SMTP_PORT", 587u16),
            smtp_username: env_or("SMTP_USERNAME", ""),
            smtp_password: env_or("SMTP_PASSWORD", ""),
            from_email: env_or("FROM_EMAIL", ""),
            to_email: env_or("TO_EMAIL", ""),
            state_file: env_or(
                "STATE_FILE",
                "/var/lib/simple-notifier/state",
            ),
            check_interval_secs: env_parse_or("CHECK_INTERVAL_MINUTES", 60u64) * 60,
            random_delay_min_secs: env_parse_or("RANDOM_DELAY_MIN_MINUTES", 5u64) * 60,
            random_delay_max_secs: env_parse_or("RANDOM_DELAY_MAX_MINUTES", 15u64) * 60,
            send_test_mail_on_startup: env_bool("SEND_TEST_MAIL_ON_STARTUP"),
        }
    }
}

fn load_dotenv() {
    let path = env::var("DOTENV_PATH").unwrap_or_else(|_| ".env".to_string());
    let file = match fs::File::open(Path::new(&path)) {
        Ok(f) => f,
        Err(_) => return,
    };

    for line in BufReader::new(file).lines().map_while(Result::ok) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (key, val) = match trimmed.split_once('=') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue,
        };
        let val = val
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .or_else(|| val.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
            .unwrap_or(val);

        if env::var(key).is_err() {
            unsafe { env::set_var(key, val) };
        }
    }
}
