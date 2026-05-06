use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

const APP_QUALIFIER: &str = "org";
const APP_ORGANIZATION: &str = "cmd-buttons";
const APP_NAME: &str = "cmd-buttons";

#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub buttons_dir: Option<PathBuf>,
    pub shell: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            buttons_dir: None,
            shell: Some("bash".to_string()),
        }
    }
}

pub struct Paths {
    pub config_dir: PathBuf,
    pub state_dir: PathBuf,
    pub config_file: PathBuf,
    pub index_file: PathBuf,
    pub sessions_dir: PathBuf,
}

impl Paths {
    pub fn new() -> Option<Self> {
        let proj = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)?;
        let config_dir = proj.config_dir().to_path_buf();
        let state_dir = proj.state_dir()?.to_path_buf();
        let config_file = config_dir.join("config.toml");
        let index_file = state_dir.join("index.toml");
        let sessions_dir = state_dir.join("sessions");

        Some(Self {
            config_dir,
            state_dir,
            config_file,
            index_file,
            sessions_dir,
        })
    }
}

pub fn load_config(paths: &Paths) -> AppConfig {
    if paths.config_file.exists() {
        if let Ok(contents) = fs::read_to_string(&paths.config_file) {
            if let Ok(config) = toml::from_str::<AppConfig>(&contents) {
                return config;
            }
        }
    }
    AppConfig::default()
}

pub fn resolve_buttons_dir(paths: &Paths, config: &AppConfig) -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let local_dir = cwd.join("cmd-buttons");
    if local_dir.is_dir() {
        return Some(local_dir);
    }
    config.buttons_dir.clone().or_else(|| {
        let default_dir = paths.state_dir.join("buttons");
        if default_dir.is_dir() {
            Some(default_dir)
        } else {
            None
        }
    })
}

pub fn ensure_dirs(paths: &Paths) -> std::io::Result<()> {
    fs::create_dir_all(&paths.config_dir)?;
    fs::create_dir_all(&paths.state_dir)?;
    fs::create_dir_all(&paths.sessions_dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.shell, Some("bash".to_string()));
        assert!(config.buttons_dir.is_none());
    }
}
