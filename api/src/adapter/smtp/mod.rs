//! SMTP mail adapter.
//!
//! When SMTP is not configured the adapter logs the link and returns success,
//! so a missing local mail server never blocks registration or reset.

use anyhow::{Context, Result};
use async_trait::async_trait;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, MultiPart},
    transport::smtp::authentication::Credentials,
};

use crate::app::port::{Mailer, PortError, mailer::AuthEmail};

/// SMTP settings. The mailer logs links instead of failing when SMTP is not
/// configured.
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub user: Option<String>,
    pub password: Option<String>,
    pub secure: bool,
}

/// Sends the two transactional auth messages.
pub struct SmtpMailer {
    transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
    from: Mailbox,
    app_name: String,
}

impl std::fmt::Debug for SmtpMailer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SmtpMailer")
            .field("configured", &self.transport.is_some())
            .field("from", &self.from)
            .finish_non_exhaustive()
    }
}

impl SmtpMailer {
    /// Build the adapter from configuration.
    ///
    /// # Errors
    ///
    /// Returns an error when `MAIL_FROM` is not a valid mailbox.
    pub fn new(smtp: Option<&SmtpConfig>, from: &str, app_name: &str) -> Result<Self> {
        let from = from
            .parse::<Mailbox>()
            .with_context(|| format!("MAIL_FROM is not a valid mailbox: {from}"))?;

        let transport = smtp.map(|config| {
            let builder = if config.secure {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
                    .expect("SMTP relay builder")
                    .port(config.port)
            } else {
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
                    .port(config.port)
            };

            match (&config.user, &config.password) {
                (Some(user), Some(password)) => builder
                    .credentials(Credentials::new(user.clone(), password.clone()))
                    .build(),
                _ => builder.build(),
            }
        });

        Ok(Self {
            transport,
            from,
            app_name: app_name.to_owned(),
        })
    }
}

#[async_trait]
impl Mailer for SmtpMailer {
    async fn send_auth_email(
        &self,
        kind: AuthEmail,
        to: &str,
        name: &str,
        url: &str,
    ) -> Result<(), PortError> {
        let is_verification = kind == AuthEmail::VerifyEmail;
        let subject = if is_verification {
            "Verify your email address"
        } else {
            "Reset your password"
        };
        let text = if is_verification {
            format!("Welcome, {name}. Verify your email address: {url}")
        } else {
            format!("Reset your password, {name}: {url}")
        };
        let html = if is_verification {
            verification_html(name, url, &self.app_name)
        } else {
            reset_html(name, url, &self.app_name)
        };

        let Some(transport) = &self.transport else {
            tracing::info!(mail_kind = ?kind, to, url, "SMTP not configured; logging auth link");
            return Ok(());
        };

        let message = Message::builder()
            .from(self.from.clone())
            .to(to.parse::<Mailbox>().map_err(PortError::backend)?)
            .subject(subject)
            .multipart(MultiPart::alternative_plain_html(text, html))
            .map_err(PortError::backend)?;

        transport.send(message).await.map_err(PortError::backend)?;

        tracing::debug!(mail_kind = ?kind, to, "auth email sent");
        Ok(())
    }
}

fn verification_html(name: &str, url: &str, app_name: &str) -> String {
    layout(
        "Verify your email address",
        &format!("Welcome, {}", escape(name)),
        "Confirm your email address to finish setting up your account.",
        "Verify email",
        url,
        "If you did not create this account, you can ignore this email.",
        app_name,
    )
}

fn reset_html(name: &str, url: &str, app_name: &str) -> String {
    layout(
        "Reset your password",
        &format!("Reset your password, {}", escape(name)),
        "We received a request to reset your password. This link expires in one hour.",
        "Choose a new password",
        url,
        "If you did not request a reset, no action is needed.",
        app_name,
    )
}

fn layout(
    preview: &str,
    heading: &str,
    body: &str,
    action: &str,
    url: &str,
    footer: &str,
    app_name: &str,
) -> String {
    let url = escape(url);
    let app_name = escape(app_name);

    format!(
        r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{preview}</title>
  </head>
  <body style="background:#f6f9fc;font-family:Inter,-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;margin:0;padding:40px 12px;">
    <div style="background:#ffffff;border:1px solid #e3e8ee;border-radius:12px;margin:0 auto;max-width:520px;padding:32px;">
      <p style="color:#533afd;font-size:13px;font-weight:600;letter-spacing:.08em;margin:0 0 24px;text-transform:uppercase;">{app_name}</p>
      <h1 style="color:#0d253d;font-size:26px;font-weight:300;letter-spacing:-.26px;line-height:1.2;margin:0 0 12px;">{heading}</h1>
      <p style="color:#273951;font-size:15px;font-weight:300;line-height:1.6;margin:0 0 24px;">{body}</p>
      <p style="margin:0 0 24px;">
        <a href="{url}" style="background:#533afd;border-radius:9999px;color:#ffffff;display:inline-block;font-size:15px;font-weight:500;padding:10px 20px;text-decoration:none;">{action}</a>
      </p>
      <p style="color:#64748d;font-size:12px;line-height:1.6;margin:0 0 24px;">If the button does not work, copy this link into your browser:<br><span style="color:#533afd;word-break:break-all;">{url}</span></p>
      <hr style="border-color:#e3e8ee;margin:0 0 16px;">
      <p style="color:#64748d;font-size:12px;line-height:1.6;margin:0;">{footer}</p>
    </div>
  </body>
</html>"#
    )
}

fn escape(value: &str) -> String {
    html_escape::encode_text(value).into_owned()
}
