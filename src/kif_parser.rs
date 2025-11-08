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
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

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
                metadata.date = Some(
                    trimmed
                        .split_once("開始日時：")
                        .map(|(_, v)| v)
                        .unwrap_or("")
                        .to_string(),
                );
            } else if trimmed.starts_with("終了日時：") {
                // End date - could be used for game duration
            } else if trimmed.starts_with("持ち時間：") {
                metadata.time_control = Some(
                    trimmed
                        .split_once("持ち時間：")
                        .map(|(_, v)| v)
                        .unwrap_or("")
                        .to_string(),
                );
            } else if trimmed.starts_with("先手：") {
                metadata.player1_name = Some(
                    trimmed
                        .split_once("先手：")
                        .map(|(_, v)| v)
                        .unwrap_or("")
                        .to_string(),
                );
            } else if trimmed.starts_with("後手：") {
                metadata.player2_name = Some(
                    trimmed
                        .split_once("後手：")
                        .map(|(_, v)| v)
                        .unwrap_or("")
                        .to_string(),
                );
            } else if trimmed.starts_with("手合割：") {
                metadata.game_type = Some(
                    trimmed
                        .split_once("手合割：")
                        .map(|(_, v)| v)
                        .unwrap_or("")
                        .to_string(),
                );
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

        Ok(KifGame { metadata, moves })
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
            Some(line[start + 1..end].to_string())
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
        // Strip any trailing annotation such as elapsed time or comments after spaces
        let token = kif_text.split_whitespace().next().unwrap_or(kif_text);

        // Separate the base move (e.g. "７六歩") from optional origin information "(77)"
        let (base, origin_hint) = match token.split_once('(') {
            Some((head, tail)) => (head, Some(tail.trim_end_matches(')'))),
            None => (token, None),
        };

        // Drop moves contain the Japanese character "打"
        let is_drop = base.ends_with('打');
        let promotion = base.contains('成');

        // The destination square is described by the first two characters (file, rank)
        let mut base_chars = base.chars();
        let file_char = base_chars.next()?;
        let rank_char = base_chars.next()?;

        let to_file = Self::char_to_digit(file_char)?;
        let to_rank = Self::char_to_digit(rank_char)?;
        let to_rank_letter = Self::rank_to_letter(to_rank)?;

        if is_drop {
            // Determine piece initial for drop (e.g. "歩" -> "P")
            let piece_char = base_chars.find(|&c| c != '打' && c != '成')?;
            let piece_code = Self::piece_to_usi_letter(piece_char)?;
            return Some(format!("{}*{}{}", piece_code, to_file, to_rank_letter));
        }

        // Determine origin square from the hint (e.g. "77")
        let origin = origin_hint.and_then(|hint| {
            let mut chars = hint.chars();
            let file = chars.next().and_then(Self::char_to_digit)?;
            let rank = chars.next().and_then(Self::char_to_digit)?;
            Some((file, rank))
        })?;

        let from_rank_letter = Self::rank_to_letter(origin.1)?;
        let mut usi = format!(
            "{}{}{}{}",
            origin.0, from_rank_letter, to_file, to_rank_letter
        );

        if promotion {
            usi.push('+');
        }

        Some(usi)
    }

    fn char_to_digit(c: char) -> Option<u8> {
        match c {
            '1' | '１' | '一' => Some(1),
            '2' | '２' | '二' => Some(2),
            '3' | '３' | '三' => Some(3),
            '4' | '４' | '四' => Some(4),
            '5' | '５' | '五' => Some(5),
            '6' | '６' | '六' => Some(6),
            '7' | '７' | '七' => Some(7),
            '8' | '８' | '八' => Some(8),
            '9' | '９' | '九' => Some(9),
            _ => None,
        }
    }

    fn rank_to_letter(rank: u8) -> Option<char> {
        match rank {
            1 => Some('a'),
            2 => Some('b'),
            3 => Some('c'),
            4 => Some('d'),
            5 => Some('e'),
            6 => Some('f'),
            7 => Some('g'),
            8 => Some('h'),
            9 => Some('i'),
            _ => None,
        }
    }

    fn piece_to_usi_letter(piece: char) -> Option<&'static str> {
        match piece {
            '歩' | '香' | '桂' | '銀' | '金' | '角' | '飛' | '玉' => Some(match piece {
                '歩' => "P",
                '香' => "L",
                '桂' => "N",
                '銀' => "S",
                '金' => "G",
                '角' => "B",
                '飛' => "R",
                '玉' => "K",
                _ => unreachable!(),
            }),
            _ => None,
        }
    }

    /// Parse Japanese number to integer
    #[allow(dead_code)]
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
        assert_eq!(kif_move.move_text, "７六歩(77)");
    }

    #[test]
    fn test_kif_to_usi() {
        // Test basic pawn move conversion
        let result = KifGame::kif_to_usi("７六歩(77)");
        assert_eq!(result.as_deref(), Some("7g7f"));
    }
}
