use kube::{
    config::{KubeConfigOptions, Kubeconfig},
    Client, Config,
};

pub async fn create_client(
    config_path: &Option<String>,
    context_name: &Option<String>
) -> anyhow::Result<Client> {
    let config_options = KubeConfigOptions {
        context: context_name.to_owned(),
        ..Default::default()
    };

    let config = match config_path {
        Some(path) => {
            let kubeconfig = Kubeconfig::read_from(path)?;
            Config::from_custom_kubeconfig(kubeconfig, &config_options).await?
        }
        None => Config::from_kubeconfig(&config_options).await?,
    };

    let client = Client::try_from(config)?;
            
    Ok(client)
}
