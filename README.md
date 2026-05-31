# simple-notifier

Simple notifier that monitors sources for updates and sends email notifications.

Currently supports:
- **ScriptHookV** — checks [dev-c.com/gtav/scripthookv](http://www.dev-c.com/gtav/scripthookv/) for new versions

## Configuration

Create a `.env` file or set environment variables:

```
SMTP_HOST=localhost
SMTP_PORT=587
SMTP_USERNAME=
SMTP_PASSWORD=
FROM_EMAIL=
TO_EMAIL=
STATE_FILE=./state
CHECK_INTERVAL_MINUTES=60
RANDOM_DELAY_MIN_MINUTES=5
RANDOM_DELAY_MAX_MINUTES=15
SEND_TEST_MAIL_ON_STARTUP=false
```

## Build

```sh
cargo build --release
```

## Run

```sh
./target/release/simple-notifier
```
