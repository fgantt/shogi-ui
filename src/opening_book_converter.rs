use crate::opening_book::*;
use crate::types::*;
/// Opening Book JSON to Binary Converter
///
/// This module provides functionality to convert the existing JSON opening book
/// format to the new binary format, with enhanced move analysis and weight assignment.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JSON format structures for parsing the existing opening book
#[derive(Debug, Deserialize, Serialize)]
struct JsonMove {
    from: String,
    to: String,
    #[serde(default)]
    promote: bool,
    #[serde(rename = "pieceType")]
    #[serde(default)]
    piece_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonOpening {
    name: String,
    moves: HashMap<String, Vec<JsonMove>>,
}

/// Migration statistics
#[derive(Debug, Clone)]
pub struct MigrationStats {
    pub total_positions: usize,
    pub total_moves: usize,
    pub opening_counts: HashMap<String, usize>,
    pub piece_type_counts: HashMap<String, usize>,
    pub weight_distribution: WeightDistribution,
}

#[derive(Debug, Clone)]
pub struct WeightDistribution {
    pub high: usize,   // 800+
    pub medium: usize, // 500-799
    pub low: usize,    // <500
}

/// Opening book converter with enhanced analysis
pub struct OpeningBookConverter {
    opening_weights: HashMap<String, u32>,
    evaluation_scores: HashMap<String, i32>,
}

impl OpeningBookConverter {
    /// Create a new converter
    pub fn new() -> Self {
        let mut opening_weights = HashMap::new();
        opening_weights.insert("Aggressive Rook".to_string(), 850);
        opening_weights.insert("Yagura".to_string(), 800);
        opening_weights.insert("Kakugawari (Bishop Exchange)".to_string(), 750);
        opening_weights.insert("Shikenbisya (Fourth File Rook)".to_string(), 700);
        opening_weights.insert("Aigakari (Double Wing Attack)".to_string(), 650);
        opening_weights.insert("Side Pawn Picker (Yokofudori)".to_string(), 600);

        let mut evaluation_scores = HashMap::new();
        evaluation_scores.insert("development".to_string(), 15);
        evaluation_scores.insert("central_control".to_string(), 20);
        evaluation_scores.insert("king_safety".to_string(), 25);
        evaluation_scores.insert("tactical".to_string(), 30);
        evaluation_scores.insert("positional".to_string(), 10);
        evaluation_scores.insert("neutral".to_string(), 0);

        Self {
            opening_weights,
            evaluation_scores,
        }
    }

    /// Convert JSON opening book to binary format
    pub fn convert_from_json(
        &self,
        json_data: &str,
    ) -> Result<(OpeningBook, MigrationStats), OpeningBookError> {
        let openings: Vec<JsonOpening> = serde_json::from_str(json_data).map_err(|e| {
            OpeningBookError::JsonParseError(format!("Failed to parse JSON: {}", e))
        })?;

        let mut book = OpeningBook::new();
        let mut stats = MigrationStats {
            total_positions: 0,
            total_moves: 0,
            opening_counts: HashMap::new(),
            piece_type_counts: HashMap::new(),
            weight_distribution: WeightDistribution {
                high: 0,
                medium: 0,
                low: 0,
            },
        };

        for opening in openings {
            for (fen, moves) in opening.moves {
                let converted_moves = self.convert_moves(&moves, &opening.name, &fen)?;

                if !converted_moves.is_empty() {
                    // Update statistics before moving the moves
                    stats.total_positions += 1;
                    stats.total_moves += moves.len();
                    *stats
                        .opening_counts
                        .entry(opening.name.clone())
                        .or_insert(0) += moves.len();

                    // Update piece type and weight statistics
                    for book_move in &converted_moves {
                        let piece_type_str = format!("{:?}", book_move.piece_type);
                        *stats.piece_type_counts.entry(piece_type_str).or_insert(0) += 1;

                        match book_move.weight {
                            800..=1000 => stats.weight_distribution.high += 1,
                            500..=799 => stats.weight_distribution.medium += 1,
                            _ => stats.weight_distribution.low += 1,
                        }
                    }

                    book.add_position(fen.clone(), converted_moves);
                }
            }
        }

        // Mark as loaded and validate
        book = book.mark_loaded();
        book.validate()?;

        Ok((book, stats))
    }

    /// Convert JSON moves to BookMoves
    fn convert_moves(
        &self,
        moves: &[JsonMove],
        opening_name: &str,
        fen: &str,
    ) -> Result<Vec<BookMove>, OpeningBookError> {
        let mut converted_moves = Vec::new();

        for (i, json_move) in moves.iter().enumerate() {
            let book_move = self.convert_move(json_move, opening_name, fen, i)?;
            converted_moves.push(book_move);
        }

        Ok(converted_moves)
    }

    /// Convert a single JSON move to BookMove
    fn convert_move(
        &self,
        json_move: &JsonMove,
        opening_name: &str,
        fen: &str,
        _move_index: usize,
    ) -> Result<BookMove, OpeningBookError> {
        // Handle drop moves
        let (from, is_drop) = if json_move.from == "drop" {
            (None, true)
        } else {
            let from_pos = coordinate_utils::string_to_position(&json_move.from)?;
            (Some(from_pos), false)
        };

        let to = coordinate_utils::string_to_position(&json_move.to)?;

        // Determine piece type
        let piece_type = if !json_move.piece_type.is_empty() {
            coordinate_utils::parse_piece_type(&json_move.piece_type)?
        } else {
            self.determine_piece_type(json_move, opening_name, fen)?
        };

        // Calculate weight and evaluation
        let weight = self.calculate_weight(json_move, opening_name);
        let evaluation = self.calculate_evaluation(json_move, opening_name);

        // Generate move notation
        let move_notation = self.generate_move_notation(json_move, &piece_type);

        Ok(BookMove::new_with_metadata(
            from,
            to,
            piece_type,
            is_drop,
            json_move.promote,
            weight,
            evaluation,
            Some(opening_name.to_string()),
            Some(move_notation),
        ))
    }

    /// Determine piece type from move context
    fn determine_piece_type(
        &self,
        json_move: &JsonMove,
        opening_name: &str,
        _fen: &str,
    ) -> Result<PieceType, OpeningBookError> {
        // Use heuristics based on opening patterns and move characteristics
        match opening_name {
            "Aggressive Rook" => {
                if json_move.from.starts_with("2") {
                    Ok(PieceType::Rook)
                } else {
                    Ok(PieceType::Pawn)
                }
            }
            "Yagura" => {
                if json_move.from == "69" || json_move.to == "78" {
                    Ok(PieceType::Gold)
                } else if json_move.from.starts_with("7") {
                    Ok(PieceType::Pawn)
                } else {
                    Ok(PieceType::Pawn)
                }
            }
            "Kakugawari (Bishop Exchange)" => {
                if json_move.from == "22" || json_move.to == "88" {
                    Ok(PieceType::Bishop)
                } else {
                    Ok(PieceType::Pawn)
                }
            }
            "Shikenbisya (Fourth File Rook)" => {
                if json_move.from == "28" || json_move.to == "58" {
                    Ok(PieceType::Rook)
                } else {
                    Ok(PieceType::Pawn)
                }
            }
            _ => {
                // Default heuristic based on move pattern
                if json_move.from.starts_with("2") || json_move.from.starts_with("8") {
                    Ok(PieceType::Rook)
                } else if json_move.from.starts_with("7") {
                    Ok(PieceType::Pawn)
                } else {
                    Ok(PieceType::Pawn)
                }
            }
        }
    }

    /// Calculate move weight based on opening and move characteristics
    fn calculate_weight(&self, json_move: &JsonMove, opening_name: &str) -> u32 {
        let base_weight = self
            .opening_weights
            .get(opening_name)
            .copied()
            .unwrap_or(500);

        let mut weight = base_weight;

        // Adjust for promotion
        if json_move.promote {
            weight += 100;
        }

        // Adjust for drop moves
        if json_move.from == "drop" {
            weight += 50;
        }

        // Adjust for specific opening patterns
        match opening_name {
            "Aggressive Rook" => {
                if json_move.from.starts_with("2") {
                    weight += 50;
                }
            }
            "Yagura" => {
                if json_move.from == "77" || json_move.from == "69" {
                    weight += 50;
                }
            }
            _ => {}
        }

        weight.min(1000)
    }

    /// Calculate position evaluation
    fn calculate_evaluation(&self, json_move: &JsonMove, opening_name: &str) -> i32 {
        let move_type = self.classify_move_type(json_move, opening_name);
        let base_eval = self.evaluation_scores.get(&move_type).copied().unwrap_or(0);

        let mut evaluation = base_eval;

        // Adjust for opening
        match opening_name {
            "Aggressive Rook" | "Yagura" => evaluation += 5,
            "Kakugawari (Bishop Exchange)" => evaluation += 10,
            _ => {}
        }

        // Adjust for promotion
        if json_move.promote {
            evaluation += 15;
        }

        evaluation
    }

    /// Classify move type for evaluation
    fn classify_move_type(&self, json_move: &JsonMove, opening_name: &str) -> String {
        // Check for drop moves
        if json_move.from == "drop" {
            return "tactical".to_string();
        }

        // Check for promotion
        if json_move.promote {
            return "tactical".to_string();
        }

        // Check for central moves
        let central_squares = ["44", "45", "54", "55"];
        if central_squares.contains(&json_move.to.as_str()) {
            return "central_control".to_string();
        }

        // Check for king safety moves
        let king_safety_moves = ["77", "78", "87", "88"];
        if king_safety_moves.contains(&json_move.to.as_str()) {
            return "king_safety".to_string();
        }

        // Check for development moves
        if opening_name == "Yagura" || opening_name == "Aggressive Rook" {
            return "development".to_string();
        }

        "positional".to_string()
    }

    /// Generate USI-style move notation
    fn generate_move_notation(&self, json_move: &JsonMove, piece_type: &PieceType) -> String {
        if json_move.from == "drop" {
            let piece_char = match piece_type {
                PieceType::Pawn => "P",
                PieceType::Lance => "L",
                PieceType::Knight => "N",
                PieceType::Silver => "S",
                PieceType::Gold => "G",
                PieceType::Bishop => "B",
                PieceType::Rook => "R",
                _ => "P",
            };
            let to_coord = self.position_to_usi(&json_move.to);
            format!("{}*{}", piece_char, to_coord)
        } else {
            let from_coord = self.position_to_usi(&json_move.from);
            let to_coord = self.position_to_usi(&json_move.to);
            let mut notation = format!("{}{}", from_coord, to_coord);
            if json_move.promote {
                notation.push('+');
            }
            notation
        }
    }

    /// Convert position string to USI format
    fn position_to_usi(&self, pos: &str) -> String {
        if pos.len() != 2 {
            return "".to_string();
        }

        let col = pos.chars().nth(0).unwrap_or('1');
        let row = pos.chars().nth(1).unwrap_or('1');

        format!("{}{}", col, (b'a' + (row as u8 - b'1')) as char)
    }

    /// Generate migration report
    pub fn generate_report(&self, stats: &MigrationStats) -> String {
        let mut report = Vec::new();

        report.push("=== Opening Book Migration Report ===".to_string());
        report.push(format!("Total Positions: {}", stats.total_positions));
        report.push(format!("Total Moves: {}", stats.total_moves));
        report.push("".to_string());

        report.push("Opening Distribution:".to_string());
        let mut opening_vec: Vec<_> = stats.opening_counts.iter().collect();
        opening_vec.sort_by(|a, b| b.1.cmp(a.1));
        for (opening, count) in opening_vec {
            report.push(format!("  {}: {} moves", opening, count));
        }
        report.push("".to_string());

        report.push("Piece Type Distribution:".to_string());
        let mut piece_vec: Vec<_> = stats.piece_type_counts.iter().collect();
        piece_vec.sort_by(|a, b| b.1.cmp(a.1));
        for (piece_type, count) in piece_vec {
            report.push(format!("  {}: {} moves", piece_type, count));
        }
        report.push("".to_string());

        report.push("Weight Distribution:".to_string());
        report.push(format!(
            "  High (800+): {} moves",
            stats.weight_distribution.high
        ));
        report.push(format!(
            "  Medium (500-799): {} moves",
            stats.weight_distribution.medium
        ));
        report.push(format!(
            "  Low (<500): {} moves",
            stats.weight_distribution.low
        ));

        report.join("\n")
    }
}

impl Default for OpeningBookConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_conversion() {
        let converter = OpeningBookConverter::new();

        // Test valid coordinates
        assert_eq!(converter.position_to_usi("27"), "2g");
        assert_eq!(converter.position_to_usi("11"), "1a");
        assert_eq!(converter.position_to_usi("99"), "9i");
    }

    #[test]
    fn test_move_classification() {
        let converter = OpeningBookConverter::new();

        let json_move = JsonMove {
            from: "27".to_string(),
            to: "26".to_string(),
            promote: false,
            piece_type: "".to_string(),
        };

        let move_type = converter.classify_move_type(&json_move, "Aggressive Rook");
        assert_eq!(move_type, "development");
    }

    #[test]
    fn test_weight_calculation() {
        let converter = OpeningBookConverter::new();

        let json_move = JsonMove {
            from: "27".to_string(),
            to: "26".to_string(),
            promote: false,
            piece_type: "".to_string(),
        };

        let weight = converter.calculate_weight(&json_move, "Aggressive Rook");
        assert!(weight >= 850); // Base weight for Aggressive Rook
    }
}
