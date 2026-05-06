use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

pub fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

pub fn save_session_log(
    sessions_dir: &Path,
    button_label: &str,
    button_file_name: &str,
    command: &str,
    output: &str,
    exit_code: Option<i32>,
) -> std::io::Result<PathBuf> {
    fs::create_dir_all(sessions_dir)?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let sanitized_label = sanitize_name(button_label);
    let sanitized_file = sanitize_name(button_file_name);

    let filename = format!("{}__{}__{}.log", timestamp, sanitized_label, sanitized_file);
    let log_path = sessions_dir.join(filename);

    let mut log_content = String::new();
    log_content.push_str(&format!("Timestamp: {}\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
    log_content.push_str(&format!("Button: {}\n", button_label));
    log_content.push_str(&format!("File: {}\n", button_file_name));
    log_content.push_str(&format!("Command: {}\n", command));
    log_content.push_str(&format!("Exit Code: {:?}\n", exit_code));
    log_content.push_str("--- Output ---\n");
    log_content.push_str(output);
    if !output.ends_with('\n') {
        log_content.push('\n');
    }

    fs::write(&log_path, log_content)?;

    Ok(log_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("Build Project"), "Build_Project");
        assert_eq!(sanitize_name("test@#$%name"), "test____name");
        assert_eq!(sanitize_name("valid-name_123"), "valid-name_123");
    }
}
