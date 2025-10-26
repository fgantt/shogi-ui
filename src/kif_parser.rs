//! KIF Format Parser
//! 
//! Parser for Japanese Shogi KIF (棋譜) format game files
//! Supports parsing game metadata, moves, and positions

use std::fs::File;
use std::io::{BufRead, BufReader};
// Note: Move and Player types are available but not directly imported here

/// Parsed move from KIF file
#[derive(Debug, Clone)]
pub struct KifMove {
    pub move_number: usize,
    pub move_text: String,
    pub usi_move: Option<String>,
    pub comment: Option<String>,
}

/// Game metadata from KIF header
#[derive(Debug, Clone)]
pub struct KifMetadata {
    pub date: Option<String>,
    pub time_control: Option<String>,
    pub player1_name: Option<String>,
    pub player2_name: Option<String>,
    pub game_type: Option<String>,
}

/// Complete parsed KIF game
#[derive(Debug, Clone)]
pub struct KifGame {
    pub metadata: KifMetadata,
    pub moves: Vec<KifMove>,
}

impl KifGame {
    /// Load a KIF game from a file
    pub fn from_file(path: &str) -> Result<Self, String> {
        let file = File::open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let reader = BufReader::new(file);
        let lines: Result<Vec<String>, _> = reader.lines().collect();
        let lines = lines.map_err(|e| format!("Failed to read file: {}", e))?;
        
        let content = lines.join("\n");
        Self::from_string(&content)
    }
    
    /// Parse KIF content from a string
    pub fn from_string(content: &str) -> Result<Self, String> {
        let lines: Vec<&str> = content.lines().collect();
        
        let mut metadata = KifMetadata {
            date: None,
            time_control: None,
            player1_name: None,
            player2_name: None,
            game_type: None,
        };
        
        let mut moves = Vec::new();
        let mut in_move_section = false;
        
        for line in lines {
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                continue;
            }
            
            // Parse metadata using substring to avoid UTF-8 boundary issues
            if trimmed.starts_with("開始日時：") {
                metadata.date = Some(trimmed.split_once("開始日時：").map(|(_, v)| v).unwrap_or("").to_string());
            } else if trimmed.starts_with("終了日時：") {
                // End date - could be used for game duration
            } else if trimmed.starts_with("持ち時間：") {
                metadata.time_control = Some(trimmed.split_once("持ち時間：").map(|(_, v)| v).unwrap_or("").to_string());
            } else if trimmed.starts_with("先手：") {
                metadata.player1_name = Some(trimmed.split_once("先手：").map(|(_, v)| v).unwrap_or("").to_string());
            } else if trimmed.starts_with("後手：") {
                metadata.player2_name = Some(trimmed.split_once("後手：").map(|(_, v)| v).unwrap_or("").to_string());
            } else if trimmed.starts_with("手合割：") {
                metadata.game_type = Some(trimmed.split_once("手合割：").map(|(_, v)| v).unwrap_or("").to_string());
            } else if trimmed.starts_with("手数") || trimmed.starts_with("手-----") {
                // Move header - start of move section
                in_move_section = true;
                continue;
            } else if in_move_section && trimmed.starts_with(char::is_numeric) {
                // Parse move line
                if let Some(kif_move) = Self::parse_move_line(trimmed) {
                    moves.push(kif_move);
                }
            }
        }
        
        Ok(KifGame {
            metadata,
            moves,
        })
    }
    
    /// Parse a single move line from KIF format
    fn parse_move_line(line: &str) -> Option<KifMove> {
        // Parse format: "   1 ７六歩(77)"
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        if parts.len() < 2 {
            return None;
        }
        
        let move_number: usize = parts[0].parse().ok()?;
        let move_text = parts[1].to_string();
        
        // Try to extract comment if present
        let comment = if line.contains('(') {
            let start = line.find('(')?;
            let end = line.find(')')?;
            Some(line[start+1..end].to_string())
        } else {
            None
        };
        
        // Convert to USI format (simplified for now)
        let usi_move = Self::kif_to_usi(&move_text);
        
        Some(KifMove {
            move_number,
            move_text,
            usi_move,
            comment,
        })
    }
    
    /// Convert KIF notation to USI format (simplified)
    fn kif_to_usi(kif_text: &str) -> Option<String> {
        // This is a simplified converter
        // Real implementation would need full Japanese notation parsing
        
        // For now, skip conversion and return None
        // This avoids UTF-8 boundary issues with Japanese characters
        // A full implementation would use proper character-based indexing
        
        // Return None to indicate no conversion available
        None
    }
    
    /// Parse Japanese number to integer
    fn parse_japanese_number(s: &str) -> Option<u32> {
        match s {
            "一" => Some(1),
            "二" => Some(2),
            "三" => Some(3),
            "四" => Some(4),
            "五" => Some(5),
            "六" => Some(6),
            "七" => Some(7),
            "八" => Some(8),
            "九" => Some(9),
            _ => s.parse().ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_move_line() {
        let line = "   1 ７六歩(77)";
        let kif_move = KifGame::parse_move_line(line);
        
        assert!(kif_move.is_some());
        let kif_move = kif_move.unwrap();
        assert_eq!(kif_move.move_number, 1);
        assert_eq!(kif_move.move_text, "７六歩");
    }
    
    #[test]
    fn test_kif_to_usi() {
        // Test basic pawn move conversion
        let result = KifGame::kif_to_usi("７六歩");
        assert!(result.is_some());
    }
}

