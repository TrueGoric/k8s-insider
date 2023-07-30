use std::{
    fmt::Display,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

use thiserror::Error;
pub use wireguard_control::{InvalidKey, Key, KeyPair};

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
pub struct WgKey(wireguard_control::Key);

impl WgKey {
    pub fn from_base64_stdin() -> Result<Self, WgKeyError> {
        let mut buffer = String::new();

        match std::io::stdin().read_line(&mut buffer) {
            Ok(bytes_read) => match bytes_read {
                0 => Err(WgKeyError::EmptyInput),
                _ => {
                    Self::from_base64(&buffer).map_err(|_| WgKeyError::InvalidKey)
                }
            },
            Err(error) => Err(WgKeyError::IoError(error)),
        }
    }

    pub fn from_base64(key: &str) -> Result<Self, InvalidWgKey> {
        Ok(WgKey(Key::from_base64(key).map_err(|_| InvalidWgKey)?))
    }

    pub fn generate_private_key() -> Self {
        WgKey(Key::generate_private())
    }

    pub fn generate_preshared_key() -> Self {
        WgKey(Key::generate_preshared())
    }

    pub fn get_public(&self) -> Self {
        WgKey(self.deref().get_public())
    }

    pub fn to_dnssec_base32(&self) -> String {
        data_encoding::BASE32_DNSSEC.encode(&self.0 .0)
    }
}

impl Deref for WgKey {
    type Target = wireguard_control::Key;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WgKey {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Hash for WgKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0 .0.hash(state)
    }
}

impl From<Key> for WgKey {
    fn from(value: Key) -> Self {
        Self(value)
    }
}

impl From<WgKey> for Key {
    fn from(value: WgKey) -> Self {
        value.0
    }
}

impl TryFrom<&[u8]> for WgKey {
    type Error = InvalidWgKey;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Key(value.try_into().map_err(|_| InvalidWgKey)?).into())
    }
}

impl TryFrom<Vec<u8>> for WgKey {
    type Error = InvalidWgKey;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Key(value.try_into().map_err(|_| InvalidWgKey)?).into())
    }
}

impl Display for WgKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.to_base64().as_str())
    }
}

#[derive(Debug, Clone)]
pub enum Keys {
    Pair { private: WgKey, public: WgKey },
    Public(WgKey),
}

impl Keys {
    pub fn generate_new_pair() -> Self {
        let key_pair = KeyPair::generate();
        Self::Pair {
            private: key_pair.private.into(),
            public: key_pair.public.into(),
        }
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
