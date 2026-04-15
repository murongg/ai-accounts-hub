use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GithubReleaseSummary {
    pub tag_name: String,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub prerelease: bool,
}

pub fn select_latest_cli_release_version(
    releases: &[GithubReleaseSummary],
) -> Result<semver::Version, String> {
    releases
        .iter()
        .filter(|release| !release.draft && !release.prerelease)
        .filter_map(|release| release.tag_name.strip_prefix("cli-v"))
        .filter_map(|version| semver::Version::parse(version).ok())
        .max()
        .ok_or_else(|| "could not find a stable cli-vX.Y.Z release".to_string())
}

pub fn fetch_latest_cli_release_version(repository: &str) -> Result<semver::Version, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("aah-cli-upgrade")
        .build()
        .map_err(|error| format!("failed to build release client: {error}"))?;
    let releases = client
        .get(format!(
            "https://api.github.com/repos/{repository}/releases?per_page=100"
        ))
        .send()
        .map_err(|error| format!("failed to fetch releases: {error}"))?
        .error_for_status()
        .map_err(|error| format!("release lookup failed: {error}"))?
        .json::<Vec<GithubReleaseSummary>>()
        .map_err(|error| format!("failed to decode GitHub releases: {error}"))?;
    select_latest_cli_release_version(&releases)
}
