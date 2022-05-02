extern crate base64;
use anyhow::Context;
use anyhow::Result;
use log::debug;
use orion::aead;
use orion::kdf;
use std::env;
use std::time::Instant;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PassError {
    #[error("salt not found in text {0:?}")]
    MissingSalt(String),
    #[error("ciphertext not found in text, did you miss a pipe <|>? {0:?}")]
    MissingCiphertext(String),
    #[error("unknown error")]
    Unknown,
}

pub fn flush() {
    use std::io::Write;
    std::io::stdout()
        .flush()
        .expect("Error flushing STDOUT :-(");
}

pub fn password() -> Result<Vec<u8>> {
    let pass = env::var("YARROSCO_PASSPHRASE").unwrap_or_default();
    if pass.is_empty() {
        get_password("Input Password")
    } else {
        Ok(pass.into_bytes())
    }
}

pub fn get_password(ask: &str) -> Result<Vec<u8>> {
    print!("{}: ", ask);
    flush();
    let password = rpassword::read_password()?;
    Ok(password.into_bytes())
}

pub fn get_password_str(ask: &str) -> Result<String> {
    print!("{}: ", ask);
    flush();
    Ok(rpassword::read_password()?)
}

pub fn b64_enc(v: &[u8]) -> String {
    base64::encode_config(v, base64::URL_SAFE_NO_PAD)
}

pub fn b64_dec(v: &str) -> Result<Vec<u8>> {
    Ok(base64::decode_config(v, base64::URL_SAFE_NO_PAD)?)
}

pub struct SaltAndCipher {
    pub salt: kdf::Salt,
    pub ciphertext: Vec<u8>,
}

impl SaltAndCipher {
    pub fn serialize(&self) -> String {
        let salt64 = b64_enc(self.salt.as_ref());
        let cipher64 = b64_enc(&self.ciphertext);
        format!("{}|{}", salt64, cipher64)
    }
    pub fn deserialize(text: &str) -> Result<Self> {
        let mut cipherv = text.split('|');
        let salt64 = cipherv
            .next()
            .ok_or_else(|| PassError::MissingSalt(text.to_string()))?;
        let cipher64 = cipherv
            .next()
            .ok_or_else(|| PassError::MissingCiphertext(text.to_string()))?;

        let salt = b64_dec(salt64)
            .with_context(|| format!("deserializing {:?} failed to do base64 decode", text))?;
        let salt = kdf::Salt::from_slice(&salt)?;
        let ciphertext = b64_dec(cipher64)?;

        Ok(SaltAndCipher { salt, ciphertext })
    }
    pub fn derive_key(&self, pass: &[u8]) -> Result<kdf::SecretKey> {
        Self::derive_key_salt(pass, &self.salt)
    }
    pub fn derive_key_salt(pass: &[u8], salt: &kdf::Salt) -> Result<kdf::SecretKey> {
        let user_password = kdf::Password::from_slice(pass)?;
        let now = Instant::now();
        let derived_key = kdf::derive_key(&user_password, salt, 4, 1 << 12, 32)?;
        let elapsed = now.elapsed();
        debug!("Derivation took {:?}", elapsed);
        Ok(derived_key)
    }
    pub fn decrypt(&self, pass: &[u8]) -> Result<String> {
        let key = self.derive_key(pass)?;
        let decrypted_data = aead::open(&key, &self.ciphertext)
            .context("error decrypting data - is the passphrase correct?")?;
        Ok(std::str::from_utf8(&decrypted_data)?.to_string())
    }
    pub fn new(pass: &[u8], message: &str) -> Result<Self> {
        let salt = kdf::Salt::default();
        let key = Self::derive_key_salt(pass, &salt)?;
        let message = message.as_bytes();
        let ciphertext = aead::seal(&key, message)?;
        Ok(Self { salt, ciphertext })
    }
}

// As salt is in the message, it will need a key derivation each step,
// and the raw password to be held in memory for longer periods.
// Given that this is done in threads in yarrcfg, and there should be just a
// few, it should be okay. This step takes <200ms.

mod tests;
