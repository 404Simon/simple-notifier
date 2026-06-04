# simple-notifier

Monitors sources for updates and sends email notifications.

**Notifiers:**
- **ScriptHookV** - checks [dev-c.com](http://www.dev-c.com/gtav/scripthookv/) for new versions
- **GitHub** - monitors repos for commits, PRs, and releases

## Config

Edit `~/.config/simple-notifier.yml`:

```yaml
state_file: ~/.local/state/simple-notifier/state
check_interval_minutes: 60
random_delay_min_minutes: 5
random_delay_max_minutes: 15
send_test_mail_on_startup: false

email:
  smtp_host: smtp.gmail.com
  smtp_port: 587
  smtp_username: me
  smtp_password: secret
  from: me@gmail.com
  to: you@example.com

repos:
  - owner: simon
    repo: my-project
    watch:
      commits: true
      prs: true
      releases: false
    branches: default
```

## Build & Run

```sh
cargo build --release
./target/release/simple-notifier
```
