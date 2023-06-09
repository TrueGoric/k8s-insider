use kube::{
    config::{KubeConfigOptions, Kubeconfig},
    Client, Config,
};

pub async fn create_client(
    config_path: &Option<String>,
) -> anyhow::Result<Client> {
    match config_path {
        Some(path) => {
            let kubeconfig = Kubeconfig::read_from(path)?;
            let config =
                Config::from_custom_kubeconfig(kubeconfig, &KubeConfigOptions::default()).await?;
            let client = Client::try_from(config)?;
            
            Ok(client)
        }
        None => Ok(Client::try_default().await?),
    }
}
