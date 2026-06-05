use serde::Deserialize;
use std::fs;

fn default_state_file() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    format!("{home}/.local/state/simple-notifier/state")
}

fn default_check_interval() -> u64 {
    60
}
fn default_random_delay_min() -> u64 {
    5
}
fn default_random_delay_max() -> u64 {
    15
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_state_file")]
    pub state_file: String,
    #[serde(default = "default_check_interval")]
    pub check_interval_minutes: u64,
    #[serde(default = "default_random_delay_min")]
    pub random_delay_min_minutes: u64,
    #[serde(default = "default_random_delay_max")]
    pub random_delay_max_minutes: u64,
    #[serde(default)]
    pub send_test_mail_on_startup: bool,
    pub email: EmailConfig,
    #[serde(default)]
    pub repos: Vec<RepoConfig>,
    #[serde(default)]
    pub weather: Option<WeatherConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WeatherConfig {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub min_night_temp: Option<f64>,
    #[serde(default)]
    pub max_day_temp: Option<f64>,
    #[serde(default)]
    pub extreme_weather: bool,
}

impl WeatherConfig {
    pub fn has_checks(&self) -> bool {
        self.min_night_temp.is_some() || self.max_day_temp.is_some() || self.extreme_weather
    }
}

#[derive(Debug, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct WatchConfig {
    #[serde(default)]
    pub commits: bool,
    #[serde(default)]
    pub prs: bool,
    #[serde(default)]
    pub releases: bool,
}

#[derive(Debug, Clone)]
pub enum BranchesConfig {
    Default,
    All,
    List(Vec<String>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum BranchesField {
    Keyword(String),
    List(Vec<String>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepoConfig {
    pub owner: String,
    pub repo: String,
    #[serde(default)]
    pub watch: WatchConfig,
    #[serde(
        default = "default_branches",
        deserialize_with = "deserialize_branches"
    )]
    pub branches: BranchesConfig,
}

fn default_branches() -> BranchesConfig {
    BranchesConfig::Default
}

fn deserialize_branches<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<BranchesConfig, D::Error> {
    let field = BranchesField::deserialize(d)?;
    match field {
        BranchesField::Keyword(s) => match s.as_str() {
            "default" => Ok(BranchesConfig::Default),
            "all" => Ok(BranchesConfig::All),
            other => Err(serde::de::Error::unknown_variant(
                other,
                &["default", "all"],
            )),
        },
        BranchesField::List(list) => Ok(BranchesConfig::List(list)),
    }
}

impl Config {
    pub fn from_file() -> Result<Self, String> {
        let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
        let xdg_config =
            std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{home}/.config"));
        let path = format!("{xdg_config}/simple-notifier.yml");

        let contents = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read config at {path}: {e}"))?;

        let config: Config = serde_yaml::from_str(&contents)
            .map_err(|e| format!("failed to parse config YAML: {e}"))?;

        Ok(config.expand_tilde(&home))
    }

    fn expand_tilde(mut self, home: &str) -> Self {
        if self.state_file.starts_with("~/") {
            self.state_file = format!("{home}/{}", &self.state_file[2..]);
        }
        self
    }
}
