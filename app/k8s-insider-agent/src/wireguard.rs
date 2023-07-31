use k8s_insider_core::wireguard::keys::WgKey;
use wireguard_control::Key;

// because how many newtypes can you define in one project :/
pub trait ConvertKey<K> {
    fn convert(self) -> K;
}

impl ConvertKey<Key> for WgKey {
    fn convert(self) -> Key {
        Key(self.into())
    }
}

impl ConvertKey<WgKey> for Key {
    fn convert(self) -> WgKey {
        self.0.into()
    }
}