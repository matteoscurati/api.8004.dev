use std::io::{self, Write};

fn main() {
    print!("Enter password to hash: ");
    io::stdout().flush().unwrap();

    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();
    let password = password.trim();

    match bcrypt::hash(password, bcrypt::DEFAULT_COST) {
        Ok(hash) => {
            println!("\nPassword hash:");
            println!("{}", hash);
            println!("\nAdd this to your .env file:");
            println!("AUTH_PASSWORD_HASH={}", hash);
        }
        Err(e) => {
            eprintln!("Error generating hash: {}", e);
            std::process::exit(1);
        }
    }
}
