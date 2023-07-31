use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use thiserror::Error;
use x25519_dalek::{PublicKey, StaticSecret};

#[derive(Debug, Error)]
#[error("Couldn't parse the key from base64 string!")]
pub struct InvalidWgKey;

#[derive(Debug, Error)]
pub enum WgKeyError {
    #[error("No key was provided!")]
    EmptyInput,
    #[error("Couldn't parse the key from base64 string!")]
    InvalidKey,
    #[error("{}", .0)]
    IoError(std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WgKey([u8; 32]);

impl WgKey {
    pub fn from_base64_stdin() -> Result<Self, WgKeyError> {
        let mut buffer = String::new();

        match std::io::stdin().read_line(&mut buffer) {
            Ok(bytes_read) => match bytes_read {
                0 => Err(WgKeyError::EmptyInput),
                _ => Self::from_base64(&buffer).map_err(|_| WgKeyError::InvalidKey),
            },
            Err(error) => Err(WgKeyError::IoError(error)),
        }
    }

    pub fn generate_private_key() -> Self {
        let mut key = Self::get_random_raw();

        Self::clamp_key(&mut key);

        WgKey(key)
    }

    pub fn generate_preshared_key() -> Self {
        let key = Self::get_random_raw();

        WgKey(key)
    }

    pub fn get_public(&self) -> Self {
        let secret = StaticSecret::from(self.0);
        let key = PublicKey::from(&secret);

        WgKey(key.to_bytes())
    }

    pub fn to_dnssec_base32(&self) -> String {
        data_encoding::BASE32_DNSSEC.encode(&self.0)
    }

    pub fn from_base64(encoded_key: &str) -> Result<Self, InvalidWgKey> {
        let encoded_key_bytes = encoded_key.as_bytes();
        let key = data_encoding::BASE64
            .decode(encoded_key_bytes)
            .map_err(|_| InvalidWgKey)?;

        let key = key.try_into().map_err(|_| InvalidWgKey)?;

        Ok(WgKey(key))
    }

    pub fn to_base64(&self) -> String {
        data_encoding::BASE64.encode(&self.0)
    }

    fn get_random_raw() -> [u8; 32] {
        let mut random = ChaCha20Rng::from_entropy();
        let mut key = [0u8; 32];

        random.fill_bytes(&mut key);

        key
    }

    fn clamp_key(key: &mut [u8; 32]) {
        key[0] &= 0xF8;
        key[31] = (key[31] & 0x7F) | 0x40;
    }
}

impl Hash for WgKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl From<[u8; 32]> for WgKey {
    fn from(value: [u8; 32]) -> Self {
        WgKey(value)
    }
}

impl From<WgKey> for [u8; 32] {
    fn from(value: WgKey) -> Self {
        value.0
    }
}

impl TryFrom<&[u8]> for WgKey {
    type Error = InvalidWgKey;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        value.try_into().map_err(|_| InvalidWgKey)
    }
}

impl TryFrom<Vec<u8>> for WgKey {
    type Error = InvalidWgKey;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        value.try_into().map_err(|_| InvalidWgKey)
    }
}

impl Display for WgKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_base64().as_str())
    }
}

#[derive(Debug, Clone)]
pub enum Keys {
    Pair { private: WgKey, public: WgKey },
    Public(WgKey),
}

impl Keys {
    pub fn generate_new_pair() -> Self {
        let private = WgKey::generate_private_key();
        let public = private.get_public();

        Self::Pair { private, public }
    }

    pub fn from_private_key(key: WgKey) -> Self {
        Self::Pair {
            public: key.get_public(),
            private: key,
        }
    }

    pub fn from_public_key(key: WgKey) -> Self {
        Self::Public(key)
    }

    pub fn get_private_key(&self) -> Option<&WgKey> {
        match self {
            Self::Pair { private, .. } => Some(private),
            Self::Public(_) => None,
        }
    }

    pub fn get_public_key(&self) -> &WgKey {
        match self {
            Self::Pair { public, .. } => public,
            Self::Public(public) => public,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WgKey;

    #[test]
    fn base_parses_key() {
        WgKey::from_base64("kCuE5DRnJPlQtInX27vKR2JjXe6VgOE3LB3HCH3KI38=").unwrap();
    }

    #[test]
    fn base_preserves_key() {
        let key = WgKey::generate_private_key();
        let based = key.to_base64();
        let unbased = WgKey::from_base64(&based).unwrap();

        assert_eq!(key, unbased);
    }
}
