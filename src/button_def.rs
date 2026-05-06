use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct ButtonFile {
    pub label: String,
    pub command: String,
    pub order: Option<i32>,
}

impl ButtonFile {
    pub fn validate(&self) -> Result<(), String> {
        if self.label.trim().is_empty() {
            return Err("label is empty".to_string());
        }
        if self.command.trim().is_empty() {
            return Err("command is empty".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Button {
    pub index: usize,
    pub label: String,
    pub command: String,
    pub order: Option<i32>,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ButtonParseError {
    pub file_path: PathBuf,
    pub error: String,
}

pub fn parse_button_file(path: &PathBuf) -> Result<ButtonFile, ButtonParseError> {
    let contents = std::fs::read_to_string(path).map_err(|e| ButtonParseError {
        file_path: path.clone(),
        error: format!("Failed to read file: {}", e),
    })?;

    let button: ButtonFile = toml::from_str(&contents).map_err(|e| ButtonParseError {
        file_path: path.clone(),
        error: format!("Failed to parse TOML: {}", e),
    })?;

    button.validate().map_err(|e| ButtonParseError {
        file_path: path.clone(),
        error: e,
    })?;

    Ok(button)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_button() {
        let button = ButtonFile {
            label: "Build".to_string(),
            command: "cargo build".to_string(),
            order: Some(10),
        };
        assert!(button.validate().is_ok());
    }

    #[test]
    fn test_empty_label() {
        let button = ButtonFile {
            label: "".to_string(),
            command: "cargo build".to_string(),
            order: None,
        };
        assert!(button.validate().is_err());
    }

    #[test]
    fn test_empty_command() {
        let button = ButtonFile {
            label: "Build".to_string(),
            command: "".to_string(),
            order: None,
        };
        assert!(button.validate().is_err());
    }
}
