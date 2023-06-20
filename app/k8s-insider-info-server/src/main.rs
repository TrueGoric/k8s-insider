use handshakes::{get_handshakes, HandshakeResponse};
use warp::Filter;

mod handshakes;

const WIREGUARD_INTERFACE: &str = "wg0";

#[tokio::main]
async fn main() {
    let handshakes = warp::get()
        .and(warp::path("handshakes"))
        .map(|| get_handshakes(WIREGUARD_INTERFACE).response());

    warp::serve(handshakes).run(([0, 0, 0, 0], 54444)).await;
}
