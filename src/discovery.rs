use crate::button_def::{parse_button_file, Button, ButtonParseError};
use serde::Serialize;
use std::cmp::Ordering;
use std::fs;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

#[derive(Serialize)]
struct IndexEntry {
    index: usize,
    path: String,
    label: String,
    order: Option<i32>,
}

#[derive(Serialize)]
struct IndexFile {
    buttons: Vec<IndexEntry>,
}

pub fn scan_buttons(dir: &Path) -> (Vec<Button>, Vec<ButtonParseError>) {
    let mut buttons = Vec::new();
    let mut errors = Vec::new();

    if !dir.is_dir() {
        return (buttons, errors);
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            errors.push(ButtonParseError {
                file_path: dir.to_path_buf(),
                error: format!("Failed to read directory: {}", e),
            });
            return (buttons, errors);
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                errors.push(ButtonParseError {
                    file_path: dir.to_path_buf(),
                    error: format!("Failed to read entry: {}", e),
                });
                continue;
            }
        };

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let extension = path.extension().and_then(|e| e.to_str());
        if extension != Some("toml") {
            continue;
        }

        match parse_button_file(&path) {
            Ok(button_file) => {
                buttons.push(Button {
                    index: 0,
                    label: button_file.label,
                    command: button_file.command,
                    order: button_file.order,
                    file_path: path,
                });
            }
            Err(e) => {
                errors.push(e);
            }
        }
    }

    sort_buttons(&mut buttons);
    assign_indices(&mut buttons);

    (buttons, errors)
}

fn sort_buttons(buttons: &mut Vec<Button>) {
    buttons.sort_by(|a, b| {
        match (a.order, b.order) {
            (Some(a_order), Some(b_order)) => {
                let cmp = a_order.cmp(&b_order);
                if cmp != Ordering::Equal {
                    return cmp;
                }
            }
            (Some(_), None) => return Ordering::Less,
            (None, Some(_)) => return Ordering::Greater,
            (None, None) => {}
        }

        let label_cmp = a.label.cmp(&b.label);
        if label_cmp != Ordering::Equal {
            return label_cmp;
        }

        a.file_path.cmp(&b.file_path)
    });
}

fn assign_indices(buttons: &mut Vec<Button>) {
    for (i, button) in buttons.iter_mut().enumerate() {
        button.index = i + 1;
    }
}

pub fn write_index(index_file: &Path, buttons: &[Button]) -> std::io::Result<()> {
    if let Some(parent) = index_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let entries: Vec<IndexEntry> = buttons
        .iter()
        .map(|b| IndexEntry {
            index: b.index,
            path: b.file_path.to_string_lossy().to_string(),
            label: b.label.clone(),
            order: b.order,
        })
        .collect();

    let index = IndexFile { buttons: entries };
    let toml_str = toml::to_string_pretty(&index).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(index_file, toml_str)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_buttons_by_order() {
        let mut buttons = vec![
            Button {
                index: 0,
                label: "B".to_string(),
                command: "cmd".to_string(),
                order: Some(20),
                file_path: PathBuf::from("b.toml"),
            },
            Button {
                index: 0,
                label: "A".to_string(),
                command: "cmd".to_string(),
                order: Some(10),
                file_path: PathBuf::from("a.toml"),
            },
        ];
        sort_buttons(&mut buttons);
        assert_eq!(buttons[0].label, "A");
        assert_eq!(buttons[1].label, "B");
    }

    #[test]
    fn test_sort_buttons_by_label() {
        let mut buttons = vec![
            Button {
                index: 0,
                label: "Zebra".to_string(),
                command: "cmd".to_string(),
                order: None,
                file_path: PathBuf::from("z.toml"),
            },
            Button {
                index: 0,
                label: "Apple".to_string(),
                command: "cmd".to_string(),
                order: None,
                file_path: PathBuf::from("a.toml"),
            },
        ];
        sort_buttons(&mut buttons);
        assert_eq!(buttons[0].label, "Apple");
        assert_eq!(buttons[1].label, "Zebra");
    }

    #[test]
    fn test_assign_indices() {
        let mut buttons = vec![
            Button {
                index: 0,
                label: "A".to_string(),
                command: "cmd".to_string(),
                order: None,
                file_path: PathBuf::from("a.toml"),
            },
            Button {
                index: 0,
                label: "B".to_string(),
                command: "cmd".to_string(),
                order: None,
                file_path: PathBuf::from("b.toml"),
            },
        ];
        assign_indices(&mut buttons);
        assert_eq!(buttons[0].index, 1);
        assert_eq!(buttons[1].index, 2);
    }
}
