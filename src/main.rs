mod config;
mod email;
mod notifier;
mod notifiers;
mod runner;
mod storage;

use config::Config;
use notifiers::github::GitHub;
use notifiers::scripthookv::ScriptHookV;
use notifiers::weather::Weather;

fn main() {
    let config = Config::from_file().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });

    let mut notifiers: Vec<Box<dyn notifier::Notifier>> = vec![Box::new(ScriptHookV)];

    if !config.repos.is_empty() {
        notifiers.push(Box::new(GitHub::new(
            config.repos.clone(),
            config.github_token.clone(),
        )));
    }

    if let Some(ref weather_config) = config.weather
        && weather_config.has_checks()
    {
        notifiers.push(Box::new(Weather::new(weather_config.clone())));
    }

    runner::run(config, notifiers);
}
