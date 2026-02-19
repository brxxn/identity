use std::{collections::HashMap, error::Error};

use lettre::{
  AsyncTransport, Message,
  message::{MultiPart, SinglePart, header},
};

use crate::{AppState, user::User};

pub struct MailMessage {
  pub to: String,
  pub subject: String,
  pub body: String,
  pub body_html: Option<String>,
}

struct MessageTemplate {
  pub text: String,
  pub html: Option<String>,
}

macro_rules! html_template {
  ($name:expr) => {
    MessageTemplate {
      text: include_str!(concat!("../mail-templates/", $name, ".txt")).to_string(),
      html: Some(include_str!(concat!("../mail-templates/", $name, ".html")).to_string())
    }
  };
}

fn replace_body(
  body: String,
  variable_name: &str,
  variable_value: &String
) -> String {
  body.replace(&format!("{{{{{}}}}}", variable_name), variable_value.as_str())
}

fn complete_template(
  template: &mut MessageTemplate,
  variables: &HashMap<&str, String>
) {
  for (name, val) in variables {
    template.text = replace_body(template.text.clone(), *name, val);
    match template.html.clone() {
      Some(html_body) => {
        template.html = Some(replace_body(html_body, *name, val));
      },
      None => {}
    };
  }
}

pub fn new_registration_message(
  user: &User,
  registration_link: String,
  origin: String
) -> MailMessage {
  let mut variables = HashMap::new();
  variables.insert("name", user.name.clone());
  variables.insert("username", user.username.clone());
  variables.insert("email", user.email.clone());
  variables.insert("origin", origin.clone());
  variables.insert("registration_link",registration_link);

  let mut template = html_template!("register-account");
  complete_template(&mut template, &variables);

  MailMessage {
    to: user.email.clone(),
    subject: format!("Setup your account on {}", origin),
    body: template.text,
    body_html: template.html
  }
}

pub async fn send_mail(state: &AppState, message: MailMessage) -> Result<(), Box<dyn Error>> {
  let Some(mailer) = &state.mailer else {
    tracing::info!("Mailing skipped due to SMTP being disabled!");
    return Ok(());
  };
  let cloned_mailer = mailer.clone();

  let email = match message.body_html {
    Some(html_body) => Message::builder()
      .from(mailer.sender.parse()?)
      .to(message.to.parse()?)
      .subject(message.subject)
      .multipart(
        MultiPart::alternative()
          .singlepart(
            SinglePart::builder()
              .header(header::ContentType::TEXT_PLAIN)
              .body(message.body),
          )
          .singlepart(
            SinglePart::builder()
              .header(header::ContentType::TEXT_HTML)
              .body(html_body),
          ),
      )?,
    None => Message::builder()
      .from(mailer.sender.parse()?)
      .to(message.to.parse()?)
      .subject(message.subject)
      .body(message.body)?,
  };

  tokio::spawn(async move {
    tracing::info!("Sending mail to {}", message.to);
    match cloned_mailer.transport.send(email).await {
      Ok(_) => {
        tracing::info!("Mail sent successfully!");
      }
      Err(e) => {
        tracing::error!("Error while sending email: {}", e.to_string())
      }
    }
  });
  Ok(())
}
