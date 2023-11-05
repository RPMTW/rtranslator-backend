use anyhow::Context;

const GAME_VERSION_REGEX: &str = r"^\d+\.\d+(\.\d+)?$";

pub fn is_stable(version: &str) -> bool {
    let re = regex::Regex::new(GAME_VERSION_REGEX).unwrap();
    re.is_match(version)
}

pub fn to_semver(version: &str) -> anyhow::Result<semver::Version> {
    let re = regex::Regex::new(GAME_VERSION_REGEX).unwrap();
    if re.is_match(version) {
        let mut parts = version.split('.');

        let major = parts
            .next()
            .context("Failed to get major version")?
            .parse::<u64>()
            .context("Failed to parse major version")?;
        let minor = parts
            .next()
            .context("Failed to get minor version")?
            .parse::<u64>()
            .context("Failed to parse minor version")?;
        let patch = if let Some(patch) = parts.next() {
            patch
                .parse::<u64>()
                .context("Failed to parse patch version")?
        } else {
            0
        };

        let semver = semver::Version::new(major, minor, patch);
        Ok(semver)
    } else {
        Err(anyhow::anyhow!("Invalid version"))
    }
}
