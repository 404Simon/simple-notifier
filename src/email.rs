use lettre::message::header::{ContentTransferEncoding, ContentType};
use lettre::message::{Body, Mailbox, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Address, Message, SmtpTransport, Transport};

use crate::notifier::Notification;

pub struct EmailSender {
    from: Mailbox,
    to: Mailbox,
    creds: Credentials,
    smtp_host: String,
    smtp_port: u16,
}

fn parse_mailbox(raw: &str, label: &str) -> Result<Mailbox, String> {
    let parse_addr = |s: &str| -> Result<Address, String> {
        let (user, domain) = s
            .split_once('@')
            .ok_or_else(|| format!("invalid {label}: missing '@' in '{s}'"))?;
        Address::new(user, domain).map_err(|e| format!("invalid {label} address: {e}"))
    };

    if let Some((name, addr)) = raw
        .split_once('<')
        .and_then(|(n, a)| a.strip_suffix('>').map(|a| (n.trim(), a.trim())))
    {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
        let addr = parse_addr(addr)?;
        Ok(Mailbox::new(name, addr))
    } else {
        let addr = parse_addr(raw)?;
        Ok(Mailbox::new(None, addr))
    }
}

impl EmailSender {
    pub fn new(
        from_email: &str,
        to_email: &str,
        smtp_host: &str,
        smtp_port: u16,
        smtp_username: &str,
        smtp_password: &str,
    ) -> Result<Self, String> {
        let from = parse_mailbox(from_email, "FROM_EMAIL")?;
        let to = parse_mailbox(to_email, "TO_EMAIL")?;

        Ok(Self {
            from,
            to,
            creds: Credentials::new(smtp_username.to_string(), smtp_password.to_string()),
            smtp_host: smtp_host.to_string(),
            smtp_port,
        })
    }

    pub fn send(&self, notification: &Notification) -> Result<(), String> {
        let body =
            Body::new_with_encoding(notification.body.clone(), ContentTransferEncoding::EightBit)
                .map_err(|_| "failed to encode email body".to_string())?;

        let email = Message::builder()
            .from(self.from.clone())
            .to(self.to.clone())
            .subject(&notification.title)
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_PLAIN)
                    .header(ContentTransferEncoding::EightBit)
                    .body(body),
            )
            .map_err(|e| format!("failed to build email: {e}"))?;

        let mailer = SmtpTransport::relay(&self.smtp_host)
            .map_err(|e| format!("invalid SMTP host: {e}"))?
            .port(self.smtp_port)
            .credentials(self.creds.clone())
            .build();

        mailer
            .send(&email)
            .map_err(|e| format!("failed to send email: {e}"))?;

        Ok(())
    }
}
