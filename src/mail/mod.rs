use chrono::Local;
use lettre::{
    Message, SmtpTransport, Transport,
    message::header::ContentType,
    transport::smtp::authentication::{Credentials, Mechanism},
};

fn send() {
    let time = Local::now();
    let send_body = format!(
        "{}\n* This is a test email",
        time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    );
    let email = Message::builder()
        .from("orange@orgvoid.top".parse().unwrap())
        .to("voidwind@qq.com".parse().unwrap())
        .to("mzx@orgvoid.top".parse().unwrap())
        .to("voidmuzixi@proton.me".parse().unwrap())
        .subject("Test email")
        .header(ContentType::TEXT_PLAIN)
        .body(send_body.to_owned())
        .unwrap();

    // Custom TLS configuration - Use a self signed certificate
    // let cert = std::fs::read("self-signed.crt").unwrap();
    // let cert = Certificate::from_pem(&cert).unwrap();
    // let tls = TlsParameters::builder(/* TLS SNI value */ "smtp.qq.com".to_owned())
    //     .add_root_certificate(cert)
    //     .build()
    //     .unwrap();

    let user_orange = Credentials::new("orange@orgvoid.top".to_owned(), "bhx369hzy".to_owned());
    let user_qq = Credentials::new("violetmuz@qq.com".to_owned(), "nghnplnkyemrjece".to_owned());

    let sender = SmtpTransport::relay("mail.orgvoid.top")
        .unwrap()
        .port(465)
        .credentials(user_orange)
        .authentication(vec![Mechanism::Plain])
        .build();

    let response = sender.send(&email).unwrap();
    response.message();
}

#[test]
fn test1() {
    send();
}
