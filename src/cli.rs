use std::io::{self, Write};

use sqlx::types::Uuid;

use crate::{AppState, user::User};

fn read_line(
  user_input: &mut String
) {
  *user_input = String::new();
  io::stdout().flush().unwrap();
  io::stdin()
    .read_line(user_input)
    .expect("encountered invalid user input!");
  *user_input = user_input.trim().to_string();
}

pub async fn handle_email_setup(
  state: &AppState,
  user: &User
) {
  let mut user_input = String::new();

  // If the user has SMTP configured, we will let them setup SMTP.
  if state.mailer.is_some() {
    loop {
      print!("It looks like you have SMTP configured! Would you like to receive a setup link through your email or directly through the cli (type \"email\" or \"cli\"): ");
      read_line(&mut user_input);
      if user_input.eq_ignore_ascii_case("email") {
        let Ok(join_handle_opt) = user.send_registration_mail(&state).await else {
          println!("Looks like we encountered an error with that! Let's try this again...");
          continue;
        };
        println!("Sending your email...");
        join_handle_opt.unwrap().await.unwrap();
        println!("A registration email was sent to {}!", user.email.clone());
        return;
      } else if user_input.eq_ignore_ascii_case("cli") {
        // fallthrough to CLI
        break;
      } else if user_input.eq_ignore_ascii_case("EITHER \"cli\" or \"email\"") {
        // a little easter egg
        println!("\ni'm not cracked like linus torvalds yet, so i don't think i've earned the ability to swear at developers.");
        println!("just know that if i ever do get there, you will be the first i swear at.");
        println!("let's try this again...\n");
      } else {
        println!("\nWhoops! It looks like you have trouble following basic instructions...");
        println!("Let's try this again, but this time type EITHER \"cli\" or \"email\"\n");
      }
    }
  }

  let registration_link = user.get_registration_link(&state);
  println!("Here's a link to setup {}'s account: {}", user.username.clone(), registration_link);
}

pub async fn handle_setup_cli(
  state: &AppState
) {
  let mut user_input = String::new();

  println!("Welcome to the identity setup wizard!");
  println!();
  println!("Before you start, make sure that you have setup your environment (.env) correctly!");
  print!("If your environment is setup and you are ready to begin, type y: ");

  read_line(&mut user_input);

  if !user_input.eq_ignore_ascii_case("y") {
    println!("Exiting...");
    return;
  }

  // filling this in with dummy values for now
  let mut admin_user = User {
    id: 0,
    email: "email".to_string(),
    username: "username".to_string(),
    name: "whatever".to_string(),
    is_suspended: false,
    is_admin: true,
    credential_uuid: Uuid::new_v4()
  };
  println!();

  println!("Let's setup your first account!");
  print!("Enter the email you want associated with your account: ");

  read_line(&mut user_input);
  admin_user.email = user_input.clone();

  print!("Enter the username you want to use: ");
  read_line(&mut user_input);
  admin_user.username = user_input.clone();

  print!("Enter the display name you want to see across apps you sign into: ");
  read_line(&mut user_input);
  admin_user.name = user_input.clone();

  println!();
  println!("Please verify that these are the account details you would like to setup your account with.");
  println!();
  println!("- Name: {}", admin_user.name.as_str());
  println!("- Username: {}", admin_user.username.as_str());
  println!("- Email: {}", admin_user.email.as_str());
  println!();
  print!("If this looks good, type y: ");
  read_line(&mut user_input);

  if !user_input.eq_ignore_ascii_case("y") {
    println!("Exiting...");
    return;
  }

  let Ok(_) = admin_user.create(&state.pool).await else {
    println!("Whoops! An error occurred while trying to create your account. Make sure postgres is available and try again!");
    println!("Note: you may also be seeing this if you already created your account!");
    return;
  };

  println!("Your account has been created! Let's move on to getting you registered...");
  println!();

  handle_email_setup(&state, &admin_user).await
}

pub async fn handle_email_cli(
  state: &AppState
) {
  let mut user_input = String::new();

  println!("Hi there! Looks like you want to get a setup link for someone's account on your instance!");
  println!();
  print!("Please enter their username: ");
  read_line(&mut user_input);

  let Ok(user) = User::from_username(&state.pool, user_input).await else {
    println!("Whoops! We couldn't find that user.");
    return;
  };
  println!();
  handle_email_setup(&state, &user).await;
}