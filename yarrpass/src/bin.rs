use anyhow::{Context, Result};

use yarrpass::SaltAndCipher;
use yarrpass::{get_password_str, password};

fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    env_logger::init();
    match std::env::args().nth(1) {
        Some(c) => decode(c).context("Decode step failed")?,
        None => encode().context("Encode step failed")?,
    }
    Ok(())
}

fn decode(ciphertext: String) -> Result<()> {
    println!("**** DECODE ****");
    let sc = SaltAndCipher::deserialize(&ciphertext)?;

    let pass = password()?;
    let dec = sc.decrypt(&pass)?;
    println!("decrypted_data: {:?}", dec);
    Ok(())
}

fn encode() -> Result<()> {
    println!("**** ENCODE ****");
    let pass = password()?;
    let secret_message = get_password_str("Input Secret Message")?;
    let sc = SaltAndCipher::new(&pass, &secret_message)?;
    let token = sc.serialize();

    let decrypted_data = SaltAndCipher::deserialize(&token)?.decrypt(&pass)?;
    if secret_message != decrypted_data {
        panic!("Secret message and decrypted message do not match. Unknown error!")
    }
    println!("Token: {}", &token);
    Ok(())
}
