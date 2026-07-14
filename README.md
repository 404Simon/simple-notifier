# simple-notifier

Monitors sources for updates and sends email notifications.

**Notifiers:**

- **ScriptHookV** - checks [dev-c.com](http://www.dev-c.com/gtav/scripthookv/) for new versions
- **GitHub** - monitors repos for commits, PRs, and releases
- **Weather** - checks Open-Meteo 3-day forecast for temperature thresholds and extreme weather (storms, heavy rain/snow, strong wind)

## Config

Edit `~/.config/simple-notifier.yml`:

```yaml
state_file: ~/.local/state/simple-notifier/state
check_interval_minutes: 60
random_delay_min_minutes: 5
random_delay_max_minutes: 15
send_test_mail_on_startup: false

# Optional GitHub personal access token.
# No scopes needed for public repos (5000 req/hr instead of 60).
# Add `repo` scope to monitor private repos.
# Generate at: https://github.com/settings/tokens (classic)
# github_token: ghp_abc123def456

email:
  smtp_host: smtp.gmail.com
  smtp_port: 587
  smtp_username: me
  smtp_password: secret
  from: me@gmail.com
  to: you@example.com

repos:
  - owner: 404Simon
    repo: simple-notifier
    watch:
      commits: true
      prs: true
      releases: false
      ignore_authors: [404Simon]     # optional: filter out commits by these GitHub usernames
    branches: default

weather:
  latitude: 48.8566
  longitude: 2.3522
  min_night_temp: 5        # alert if night low below this °C (optional)
  max_day_temp: 35         # alert if day high above this °C (optional)
  extreme_weather: true    # alert on storms, heavy rain/snow, strong wind (optional)
```

## Build & Run

```sh
cargo build --release
./target/release/simple-notifier
```
