use anyhow::anyhow;
use log::debug;

pub const LOCAL_INSIDER_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSIDER_LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/truegoric/k8s-insider/releases/latest";

pub async fn get_latest_version() -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let response = client.get(INSIDER_LATEST_RELEASE_URL)
        .header("User-Agent", "k8s-insider")
        .header("Accept", "application/json")
        .send()
        .await?;

    debug!("{response:#?}");

    let response_text = response.text().await?;

    debug!("{response_text}");

    Ok(serde_json::from_str::<serde_json::Value>(&response_text)?
        .as_object()
        .ok_or(anyhow!("Invalid GitHub API response! Expected an object!"))?
        .get("name")
        .ok_or(anyhow!(
            "Invalid GitHub API response! Expected an object to contain a 'name' property!"
        ))?
        .as_str()
        .ok_or(anyhow!(
            "Invalid GitHub API response! Expected the 'name' property to be a string!"
        ))?
        .trim_start_matches('v')
        .to_owned())
}
