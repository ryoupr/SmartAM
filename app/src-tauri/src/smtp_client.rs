use lettre::{
    message::{header::ContentType, Mailbox, MessageBuilder},
    transport::smtp::authentication::{Credentials, Mechanism},
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use crate::SmtpConfig;

fn build_mailer(config: &SmtpConfig) -> Result<AsyncSmtpTransport<Tokio1Executor>, String> {
    let creds = if config.auth_type == "oauth" {
        Credentials::new(config.email.clone(), config.access_token.clone())
    } else {
        Credentials::new(config.email.clone(), config.password.clone())
    };
    let mut builder = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)
        .map_err(|e| format!("SMTP接続失敗: {e}"))?
        .credentials(creds)
        .port(config.smtp_port);
    if config.auth_type == "oauth" {
        builder = builder.authentication(vec![Mechanism::Xoauth2]);
    }
    Ok(builder.build())
}

pub async fn send_mail(
    config: &SmtpConfig,
    to: &[String],
    cc: &[String],
    bcc: &[String],
    subject: &str,
    body: &str,
) -> Result<String, String> {
    let from: Mailbox = config.email.parse().map_err(|e| format!("From解析失敗: {e}"))?;
    let mut builder = MessageBuilder::new()
        .from(from)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN);

    for addr in to {
        let mb: Mailbox = addr.parse().map_err(|e| format!("To解析失敗: {e}"))?;
        builder = builder.to(mb);
    }
    for addr in cc {
        let mb: Mailbox = addr.parse().map_err(|e| format!("CC解析失敗: {e}"))?;
        builder = builder.cc(mb);
    }
    for addr in bcc {
        let mb: Mailbox = addr.parse().map_err(|e| format!("BCC解析失敗: {e}"))?;
        builder = builder.bcc(mb);
    }

    let email = builder.body(body.to_string()).map_err(|e| format!("メール構築失敗: {e}"))?;

    let mailer = build_mailer(config)?;
    mailer.send(email).await.map_err(|e| format!("送信失敗: {e}"))?;
    Ok("送信成功".into())
}

pub async fn send_mail_with_attachments(
    config: &SmtpConfig,
    to: &[String], cc: &[String], bcc: &[String],
    subject: &str, body: &str,
    attachment_paths: &[String],
) -> Result<String, String> {
    use lettre::message::{MultiPart, SinglePart, Attachment as LAttachment, header::ContentType as CT};

    let from: Mailbox = config.email.parse().map_err(|e| format!("{e}"))?;
    let mut builder = MessageBuilder::new().from(from).subject(subject);
    for addr in to { builder = builder.to(addr.parse::<Mailbox>().map_err(|e| format!("{e}"))?); }
    for addr in cc { builder = builder.cc(addr.parse::<Mailbox>().map_err(|e| format!("{e}"))?); }
    for addr in bcc { builder = builder.bcc(addr.parse::<Mailbox>().map_err(|e| format!("{e}"))?); }

    let mut multipart = MultiPart::mixed().singlepart(
        SinglePart::builder().header(CT::TEXT_PLAIN).body(body.to_string())
    );

    for path in attachment_paths {
        let data = std::fs::read(path).map_err(|e| format!("ファイル読み込み失敗: {e}"))?;
        let filename = std::path::Path::new(path).file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "attachment".into());
        let content_type = CT::parse("application/octet-stream").unwrap();
        let attachment = LAttachment::new(filename).body(data, content_type);
        multipart = multipart.singlepart(attachment);
    }

    let email = builder.multipart(multipart).map_err(|e| format!("{e}"))?;
    let mailer = build_mailer(config)?;
    mailer.send(email).await.map_err(|e| format!("送信失敗: {e}"))?;
    Ok("送信成功".into())
}
