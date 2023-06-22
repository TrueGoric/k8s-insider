pub fn generate_wireguard_private_key() -> String{
    wireguard_control::Key::generate_private().to_base64()
}