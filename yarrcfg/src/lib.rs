use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde_derive::Deserialize;
use std::collections::BTreeMap;
use yarrpasslib::{password, SaltAndCipher};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub twitch: Twitch,
}

#[derive(Deserialize, Debug)]
pub struct Twitch {
    pub username: String,
    pub channels: Vec<String>,
    pub oauth_token: SecString,
}

#[derive(Deserialize, Debug)]
pub struct SecConfig {
    pub secrets: BTreeMap<String, Secrets>,
}

#[derive(Deserialize, Debug)]
pub struct Secrets {
    pub placeholder: String,
    pub secret: SecString,
    pub version: i64,
}

pub struct SecReplace {
    pub name: String,
    pub placeholder: String,
    pub secret: SecString,
    pub use_count: usize,
}

/// SecString is basically a string that doesn't have debug output by default.
pub struct SecString(String);

impl<'de> serde::Deserialize<'de> for SecString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Self)
    }
}

impl std::fmt::Debug for SecString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use sha2::{Digest, Sha256};
        let hash = hex::encode(Sha256::digest(&self.0));
        let hash8 = &hash[..8].to_owned();
        f.debug_tuple("SecString").field(&hash8).finish()
    }
}

pub fn parse_config() -> Result<()> {
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::Path;
    let path = Path::new("yarrsecrets.toml");
    let display = path.display();
    let mut file = File::open(&path).with_context(|| format!("couldn't open {}", display))?;
    let mut s = String::new();
    file.read_to_string(&mut s)
        .with_context(|| format!("couldn't read {}", display))?;

    let cfg: SecConfig =
        toml::from_str(&s).with_context(|| format!("couldn't parse {}", display))?;
    debug!("SecConfig: {:?}", &cfg);
    // -- replace all secrets --
    let mut secrets: Vec<SecReplace> = vec![];
    let pass = if cfg.secrets.is_empty() {
        vec![]
    } else {
        password()?
    };
    // password, SaltAndCipher
    for (name, v) in cfg.secrets.iter() {
        // v.secret --> decode this!
        let sac = SaltAndCipher::deserialize(&v.secret.0).with_context(|| {
            format!(
                "while processing secret for {:?}, placeholder {:?}",
                name, v.placeholder
            )
        })?;
        let secret = sac.decrypt(&pass)?;
        let s = SecReplace {
            name: name.to_string(),
            placeholder: v.placeholder.clone(),
            secret: SecString(secret),
            use_count: 0,
        };
        secrets.push(s);
    }

    let path = Path::new("yarrosco.toml");
    let display = path.display();

    let mut file = File::open(&path).with_context(|| format!("couldn't open {}", display))?;

    let mut s = String::new();
    file.read_to_string(&mut s)
        .with_context(|| format!("couldn't read {}", display))?;

    for secret in secrets.iter_mut() {
        let count = s.matches(&secret.placeholder).count();
        if count == 0 {
            warn!(
                "secret {:?} unused in config, {:?} not found in the text",
                &secret.name, &secret.placeholder
            );
            continue;
        }
        s = s.replace(&secret.placeholder, &secret.secret.0);
        let zcount = s.matches(&secret.placeholder).count();
        if zcount != 0 {
            warn!(
                "not all occurences of {} where replaced. {} references left.",
                &secret.placeholder, zcount
            );
        }
        secret.use_count = count - zcount;
        info!("Used secret {:?} {} times", &secret.name, secret.use_count)
    }
    let cfg: Config = toml::from_str(&s).with_context(|| format!("couldn't parse {}", display))?;
    debug!("Config: {:?}", &cfg);
    Ok(())
}
