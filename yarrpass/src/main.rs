extern crate base64;
use orion::aead;
use orion::kdf;
use std::time::Instant;

fn flush() {
    use std::io::Write;
    std::io::stdout()
        .flush()
        .expect("Error flushing STDOUT :-(");
}

fn get_password(ask: &str) -> Vec<u8> {
    print!("{}: ", ask);
    flush();
    let password = rpassword::read_password().unwrap();
    password.into_bytes()
}

fn main() {
    match std::env::args().nth(1) {
        Some(c) => decode(c),
        None => encode(),
    }
}

// TODO: If salt is in the message, it will need a key derivation each step,
//       and the raw password to be held in memory for longer periods.

fn decode(ciphertext: String) {
    println!("**** DECODE ****");
    let mut cipherv = ciphertext.split('|');
    let salt64 = cipherv.next().expect("missing salt part from ciphertext");
    let cipher64 = cipherv
        .next()
        .expect("missing cipher part - did you forget <|> ?");
    let salt = base64::decode_config(salt64, base64::URL_SAFE_NO_PAD).unwrap();
    let ciphertext = base64::decode_config(cipher64, base64::URL_SAFE_NO_PAD).unwrap();

    let pass = get_password("Input Password");
    let user_password = kdf::Password::from_slice(&pass).unwrap();

    let salt = kdf::Salt::from_slice(&salt).unwrap();

    let now = Instant::now();
    let derived_key = kdf::derive_key(&user_password, &salt, 4, 1 << 12, 32).unwrap();
    let elapsed = now.elapsed();
    println!("Derivation took {:?}", elapsed);

    let decrypted_data = aead::open(&derived_key, &ciphertext).unwrap();
    println!("decrypted_data: {:?}", std::str::from_utf8(&decrypted_data));
}

fn encode() {
    println!("**** ENCODE ****");
    let pass = get_password("Input Password");
    let secret_message = get_password("Input Secret Message");

    let user_password = kdf::Password::from_slice(&pass).unwrap();
    let salt = kdf::Salt::default();

    let now = Instant::now();
    let derived_key = kdf::derive_key(&user_password, &salt, 4, 1 << 12, 32).unwrap();
    let elapsed = now.elapsed();
    println!("Derivation took {:?}", elapsed);

    let ciphertext = aead::seal(&derived_key, &secret_message).unwrap();

    let decrypted_data = aead::open(&derived_key, &ciphertext).unwrap();
    if secret_message != decrypted_data {
        panic!("Secret message and decrypted message do not match. Unknown error!")
    }
    let salt64 = base64::encode_config(&salt, base64::URL_SAFE_NO_PAD);
    let cipher64 = base64::encode_config(&ciphertext, base64::URL_SAFE_NO_PAD);

    println!("{}|{}", &salt64, &cipher64);
    println!("secret: {:?}", std::str::from_utf8(&decrypted_data));
}
