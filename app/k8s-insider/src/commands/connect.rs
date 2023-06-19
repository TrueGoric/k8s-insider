use kube::Client;

use crate::cli::{ConnectArgs, GlobalArgs};

pub async fn connect(
    global_args: &GlobalArgs,
    args: &ConnectArgs,
    client: &Client,
) -> anyhow::Result<()> {
    todo!();
}
