use std::time::Duration;

use nix::sys::time::TimeSpec;
use nix::sys::timerfd::{ClockId, Expiration, TimerFd, TimerFlags, TimerSetTimeFlags};

use rand::RngExt;

use crate::config::Config;
use crate::email::EmailSender;
use crate::notifier::Notification;
use crate::notifier::Notifier;
use crate::storage::Storage;

pub fn run(config: Config, notifiers: Vec<Box<dyn Notifier>>) {
    let email = EmailSender::new(
        &config.email.from,
        &config.email.to,
        &config.email.smtp_host,
        config.email.smtp_port,
        &config.email.smtp_username,
        &config.email.smtp_password,
    );

    let email = match email {
        Ok(e) => Some(e),
        Err(e) => {
            eprintln!("[runner] email not configured: {e}");
            None
        }
    };

    if config.send_test_mail_on_startup {
        if let Some(ref email) = email {
            let test = Notification {
                title: "simple-notifier: test email".into(),
                body: "This is a test email from simple-notifier. Email sending is configured correctly."
                    .into(),
            };
            match email.send(&test) {
                Ok(()) => println!("[runner] test email sent successfully"),
                Err(e) => eprintln!("[runner] test email failed: {e}"),
            }
        } else {
            eprintln!("[runner] SEND_TEST_MAIL_ON_STARTUP is set but email is not configured");
        }
    }

    let mut rng = rand::rng();

    loop {
        let mut storage = Storage::load(&config.state_file);

        for notifier in &notifiers {
            let name = notifier.name();
            if let Some(notification) = notifier.check(&mut storage) {
                println!("[{name}] notification: {}", notification.title);

                if let Some(ref email) = email {
                    if let Err(e) = email.send(&notification) {
                        eprintln!("[{name}] failed to send email: {e}");
                    } else {
                        println!("[{name}] email sent");
                    }
                }
            }
        }

        storage.save();

        let delay_min = config.random_delay_min_minutes
            + rng.random_range(
                0..=config.random_delay_max_minutes - config.random_delay_min_minutes,
            );
        let total = (config.check_interval_minutes + delay_min) * 60;

        println!("[runner] sleeping for {}m {}s", total / 60, total % 60);

        let timer = TimerFd::new(ClockId::CLOCK_BOOTTIME, TimerFlags::empty())
            .expect("timerfd_create failed");
        timer
            .set(
                Expiration::OneShot(TimeSpec::from_duration(Duration::from_secs(total))),
                TimerSetTimeFlags::empty(),
            )
            .expect("timerfd_settime failed");
        timer.wait().expect("timerfd wait failed");
    }
}
