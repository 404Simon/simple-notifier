mod config;
mod email;
mod notifier;
mod notifiers;
mod runner;
mod storage;

use config::Config;
use notifiers::scripthookv::ScriptHookV;

fn main() {
    let config = Config::from_env();

    let notifiers: Vec<Box<dyn notifier::Notifier>> = vec![Box::new(ScriptHookV)];

    runner::run(config, notifiers);
}
