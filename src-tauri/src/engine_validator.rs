use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

/// Engine metadata extracted during validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMetadata {
    pub name: String,
    pub author: Option<String>,
    pub options: Vec<EngineOption>,
}

/// USI engine option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineOption {
    pub name: String,
    pub option_type: String,
    pub default: Option<String>,
    pub min: Option<String>,
    pub max: Option<String>,
    pub var: Vec<String>,
}

impl EngineOption {
    /// Parse an option line from USI protocol
    /// Format: option name <name> type <type> [default <value>] [min <value>] [max <value>] [var <value>]*
    pub fn parse(line: &str) -> Option<Self> {
        if !line.starts_with("option name ") {
            return None;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut name = String::new();
        let mut option_type = String::new();
        let mut default = None;
        let mut min = None;
        let mut max = None;
        let mut var = Vec::new();

        let mut i = 2; // Skip "option name"
        
        // Parse name (until "type")
        while i < parts.len() && parts[i] != "type" {
            if !name.is_empty() {
                name.push(' ');
            }
            name.push_str(parts[i]);
            i += 1;
        }

        // Skip "type"
        i += 1;
        
        // Parse type (until next keyword)
        if i < parts.len() {
            option_type = parts[i].to_string();
            i += 1;
        }

        // Parse remaining fields
        while i < parts.len() {
            match parts[i] {
                "default" => {
                    i += 1;
                    if i < parts.len() {
                        default = Some(parts[i].to_string());
                        i += 1;
                    }
                }
                "min" => {
                    i += 1;
                    if i < parts.len() {
                        min = Some(parts[i].to_string());
                        i += 1;
                    }
                }
                "max" => {
                    i += 1;
                    if i < parts.len() {
                        max = Some(parts[i].to_string());
                        i += 1;
                    }
                }
                "var" => {
                    i += 1;
                    if i < parts.len() {
                        var.push(parts[i].to_string());
                        i += 1;
                    }
                }
                _ => i += 1,
            }
        }

        if name.is_empty() || option_type.is_empty() {
            return None;
        }

        Some(Self {
            name,
            option_type,
            default,
            min,
            max,
            var,
        })
    }
}

/// Validate a USI engine and extract its metadata
pub async fn validate_engine(path: &str) -> Result<EngineMetadata> {
    log::info!("Validating engine at path: {}", path);

    // Check if the file exists
    if !std::path::Path::new(path).exists() {
        return Err(anyhow!("Engine executable not found at path: {}", path));
    }

    // Spawn the engine process
    let mut child = Command::new(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| anyhow!("Failed to spawn engine process: {}", e))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("Failed to get stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("Failed to get stdout"))?;

    // Send "usi" command
    stdin
        .write_all(b"usi\n")
        .await
        .map_err(|e| anyhow!("Failed to write to engine: {}", e))?;
    stdin.flush().await?;

    // Read and parse the response with timeout
    let result = timeout(Duration::from_secs(5), async {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let mut name = String::from("Unknown Engine");
        let mut author = None;
        let mut options = Vec::new();
        let mut got_usiok = false;

        while let Some(line) = lines.next_line().await? {
            log::debug!("Engine validation output: {}", line);

            if line.starts_with("id name ") {
                name = line[8..].trim().to_string();
            } else if line.starts_with("id author ") {
                author = Some(line[10..].trim().to_string());
            } else if line.starts_with("option name ") {
                if let Some(option) = EngineOption::parse(&line) {
                    options.push(option);
                }
            } else if line == "usiok" {
                got_usiok = true;
                break;
            }
        }

        if !got_usiok {
            return Err(anyhow!("Engine did not respond with 'usiok'"));
        }

        Ok::<EngineMetadata, anyhow::Error>(EngineMetadata {
            name,
            author,
            options,
        })
    })
    .await;

    // Try to kill the process gracefully
    let _ = stdin.write_all(b"quit\n").await;
    let _ = stdin.flush().await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    let _ = child.kill().await;

    match result {
        Ok(Ok(metadata)) => {
            log::info!("Engine validation successful: {}", metadata.name);
            Ok(metadata)
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err(anyhow!(
            "Timeout waiting for engine response (5 seconds)"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_option_spin() {
        let line = "option name USI_Hash type spin default 16 min 1 max 1024";
        let option = EngineOption::parse(line).unwrap();
        assert_eq!(option.name, "USI_Hash");
        assert_eq!(option.option_type, "spin");
        assert_eq!(option.default, Some("16".to_string()));
        assert_eq!(option.min, Some("1".to_string()));
        assert_eq!(option.max, Some("1024".to_string()));
    }

    #[test]
    fn test_parse_option_check() {
        let line = "option name Ponder type check default false";
        let option = EngineOption::parse(line).unwrap();
        assert_eq!(option.name, "Ponder");
        assert_eq!(option.option_type, "check");
        assert_eq!(option.default, Some("false".to_string()));
    }

    #[test]
    fn test_parse_option_string() {
        let line = "option name BookFile type string default book.bin";
        let option = EngineOption::parse(line).unwrap();
        assert_eq!(option.name, "BookFile");
        assert_eq!(option.option_type, "string");
        assert_eq!(option.default, Some("book.bin".to_string()));
    }
}

