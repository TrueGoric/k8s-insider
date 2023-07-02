use std::{
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut}, fmt::Display,
};

pub use wireguard_control::{InvalidKey, Key, KeyPair};

pub struct InvalidWgKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WgKey(wireguard_control::Key);

impl WgKey {
    pub fn from_base64(key: &str) -> Result<Self, InvalidWgKey> {
        Ok(WgKey(Key::from_base64(key).map_err(|_| InvalidWgKey)?))
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
    Pair {
        private: WgKey,
        public: WgKey
    },
    Public(WgKey),
}

impl Keys {
    pub fn generate_new_pair() -> Self {
        let key_pair = KeyPair::generate();
        Self::Pair { private:  key_pair.private.into(), public: key_pair.public.into() }
    }

    pub fn from_private_key(key: WgKey) -> Self {
        Self::Pair { public: key.get_public().into(), private: key }
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
