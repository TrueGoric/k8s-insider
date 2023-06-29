use std::ops::{Deref, DerefMut};

pub use wireguard_control::{InvalidKey, Key, KeyPair};

#[derive(Debug, Clone)]
pub struct PublicKey(Key);

#[derive(Debug, Clone)]
pub struct PrivateKey(Key);

impl Deref for PrivateKey {
    type Target = Key;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PrivateKey {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for PublicKey {
    type Target = Key;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PublicKey {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Key> for PrivateKey {
    fn from(value: Key) -> Self {
        Self(value)
    }
}

impl From<Key> for PublicKey {
    fn from(value: Key) -> Self {
        Self(value)
    }
}

impl TryFrom<&[u8]> for PrivateKey {
    type Error = InvalidKey;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Key(value.try_into().map_err(|_| InvalidKey)?).into())
    }
}

impl TryFrom<Vec<u8>> for PublicKey {
    type Error = InvalidKey;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Key(value.try_into().map_err(|_| InvalidKey)?).into())
    }
}


#[derive(Debug, Clone)]
pub enum Keys {
    Pair(PrivateKey, PublicKey),
    Public(PublicKey),
}

impl Keys {
    pub fn generate_new_pair() -> Self {
        let key_pair = KeyPair::generate();
        Self::Pair(key_pair.public.into(), key_pair.private.into())
    }

    pub fn get_private_key(&self) -> Option<&PrivateKey> {
        match self {
            Self::Pair(private, _) => Some(private),
            Self::Public(_) => None,
        }
    }

    pub fn get_public_key(&self) -> &PublicKey {
        match self {
            Self::Pair(_, public) => public,
            Self::Public(public) => public,
        }
    }
}

impl From<PrivateKey> for Keys {
    fn from(value: PrivateKey) -> Self {
        let key_pair = KeyPair::from_private(value.0);
        Self::Pair(key_pair.private.into(), key_pair.public.into())
    }
}

impl From<PublicKey> for Keys {
    fn from(value: PublicKey) -> Self {
        Self::Public(value)
    }
}

impl From<KeyPair> for Keys {
    fn from(value: KeyPair) -> Self {
        Self::Pair(value.public.into(), value.private.into())
    }
}
