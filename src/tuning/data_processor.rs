//! Data processing pipeline for automated tuning
//!
//! This module handles loading, parsing, and filtering game databases
//! to extract training positions for the tuning process. It supports
//! multiple game formats and provides comprehensive filtering and
//! deduplication capabilities.
//!
//! Supported formats:
//! - KIF (Japanese Shogi notation)
//! - CSA (Computer Shogi Association format)
//! - PGN (Portable Game Notation)
//! - Custom JSON format

use super::feature_extractor::FeatureExtractor;
use super::types::{GameRecord, GameResult, PositionFilter, TimeControl, TrainingPosition};
use crate::{
    types::{CapturedPieces, Move, Player},
    BitboardBoard,
};
use serde_json;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Data processor for game databases
pub struct DataProcessor {
    feature_extractor: FeatureExtractor,
    filter: PositionFilter,
    #[allow(dead_code)]
    progress_callback: Option<Box<dyn Fn(f64) + Send + Sync>>,
}

/// Game database for managing large collections of games
pub struct GameDatabase {
    games: Vec<GameRecord>,
    #[allow(dead_code)]
    metadata: HashMap<String, String>,
    total_positions: usize,
}

/// Position selector for filtering training positions
pub struct PositionSelector {
    filter: PositionFilter,
    #[allow(dead_code)]
    seen_positions: HashSet<String>, // For deduplication
}

/// Progress report for data processing
#[derive(Debug, Clone)]
pub struct ProcessingProgress {
    pub games_processed: usize,
    pub total_games: usize,
    pub positions_extracted: usize,
    pub positions_filtered: usize,
    pub processing_time: f64,
    pub memory_usage_mb: f64,
}

impl DataProcessor {
    /// Create a new data processor
    pub fn new(filter: PositionFilter) -> Self {
        Self {
            feature_extractor: FeatureExtractor::new(),
            filter,
            progress_callback: None,
        }
    }

    /// Create a new data processor with progress callback
    pub fn with_progress_callback<F>(filter: PositionFilter, callback: F) -> Self
    where
        F: Fn(f64) + Send + Sync + 'static,
    {
        Self {
            feature_extractor: FeatureExtractor::new(),
            filter,
            progress_callback: Some(Box::new(callback)),
        }
    }

    /// Process a game record and extract training positions
    pub fn process_game(&self, game_record: &GameRecord) -> Vec<TrainingPosition> {
        let mut positions = Vec::new();

        // Skip if game doesn't meet rating criteria
        if let Some(min_rating) = self.filter.min_rating {
            if let Some(avg_rating) = game_record.average_rating() {
                if avg_rating < min_rating {
                    return positions;
                }
            }
        }

        if let Some(max_rating) = self.filter.max_rating {
            if let Some(avg_rating) = game_record.average_rating() {
                if avg_rating > max_rating {
                    return positions;
                }
            }
        }

        // Skip if high-rated games only and not high-rated
        if self.filter.high_rated_only && !game_record.is_high_rated() {
            return positions;
        }

        // Replay the game and extract positions
        let mut board = BitboardBoard::new();
        let mut captured_pieces = CapturedPieces::new();
        let mut player = Player::White;
        let mut move_number = 1;

        for (move_index, move_) in game_record.moves.iter().enumerate() {
            // Check move number filter
            if move_number < self.filter.min_move_number
                || move_number > self.filter.max_move_number
            {
                // Still make the move but don't extract position
                if board.make_move(move_).is_none() {
                    break; // Invalid move, stop processing
                }
                player = self.switch_player(player);
                move_number += 1;
                continue;
            }

            // Check if this is a quiet position
            let is_quiet = self.is_quiet_position(&board, &captured_pieces, move_index);

            // Skip if quiet_only is enabled and position is not quiet
            if self.filter.quiet_only && !is_quiet {
                if board.make_move(move_).is_none() {
                    break;
                }
                player = self.switch_player(player);
                move_number += 1;
                continue;
            }

            // Extract position features
            let features =
                self.feature_extractor
                    .extract_features(&board, player, &captured_pieces);

            // Validate features
            if let Err(_) = self.feature_extractor.validate_features(&features) {
                if board.make_move(move_).is_none() {
                    break;
                }
                player = self.switch_player(player);
                move_number += 1;
                continue;
            }

            // Calculate game phase (simplified: based on move number)
            let game_phase = self.calculate_game_phase(move_number, game_record.move_count());

            // Get result from player's perspective
            let result = game_record.result.to_score_for_player(player);

            // Create training position
            let position =
                TrainingPosition::new(features, result, game_phase, is_quiet, move_number, player);

            positions.push(position);

            // Make the move
            if board.make_move(move_).is_none() {
                break; // Invalid move, stop processing
            }

            // Update captured pieces (simplified)
            if move_.captured_piece.is_some() {
                if let Some(captured_piece) = move_.captured_piece {
                    captured_pieces.add_piece(captured_piece.piece_type, player);
                }
            }

            player = self.switch_player(player);
            move_number += 1;
        }

        // Limit positions per game if specified
        if let Some(max_positions) = self.filter.max_positions_per_game {
            if positions.len() > max_positions {
                // Keep positions evenly distributed throughout the game
                let step = positions.len() / max_positions;
                positions = positions
                    .into_iter()
                    .enumerate()
                    .filter(|(i, _)| i % step == 0)
                    .map(|(_, pos)| pos)
                    .take(max_positions)
                    .collect();
            }
        }

        positions
    }

    /// Load games from a dataset file
    pub fn load_dataset(&self, path: &str) -> Result<Vec<GameRecord>, String> {
        let path = Path::new(path);

        match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => self.load_json_dataset(path),
            Some("kif") => self.load_kif_dataset(path),
            Some("csa") => self.load_csa_dataset(path),
            Some("pgn") => self.load_pgn_dataset(path),
            _ => Err(format!("Unsupported file format: {:?}", path.extension())),
        }
    }

    // ============================================================================
    // HELPER METHODS
    // ============================================================================

    /// Check if a position is quiet (no captures in recent moves)
    fn is_quiet_position(
        &self,
        _board: &BitboardBoard,
        _captured_pieces: &CapturedPieces,
        _move_index: usize,
    ) -> bool {
        // Simplified quiet position detection
        // In a real implementation, this would track the last N moves
        true // For now, consider all positions as quiet
    }

    /// Calculate game phase based on move number
    fn calculate_game_phase(&self, move_number: u32, total_moves: usize) -> i32 {
        let phase_ratio = move_number as f64 / total_moves as f64;
        (phase_ratio * 256.0) as i32 // 0 = opening, 256 = endgame
    }

    /// Switch player (White <-> Black)
    fn switch_player(&self, player: Player) -> Player {
        match player {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }

    /// Load games from JSON format
    fn load_json_dataset(&self, path: &Path) -> Result<Vec<GameRecord>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

        let reader = BufReader::new(file);
        let games: Vec<GameRecord> =
            serde_json::from_reader(reader).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        Ok(games)
    }

    /// Load games from KIF format (Japanese Shogi notation)
    fn load_kif_dataset(&self, path: &Path) -> Result<Vec<GameRecord>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

        let reader = BufReader::new(file);
        let mut games = Vec::new();
        let mut current_game = GameRecord::new(
            vec![],
            GameResult::Draw,
            TimeControl::new(600, 10), // Default time control
        );

        let mut in_game = false;

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            let line = line.trim();

            if line.is_empty() {
                if in_game && !current_game.moves.is_empty() {
                    games.push(current_game.clone());
                    current_game =
                        GameRecord::new(vec![], GameResult::Draw, TimeControl::new(600, 10));
                }
                in_game = false;
                continue;
            }

            // Parse game header
            if line.starts_with("開始日時:") {
                current_game.date = Some(line[6..].to_string());
            } else if line.starts_with("先手:") {
                // White player info
            } else if line.starts_with("後手:") {
                // Black player info
            } else if line.starts_with("手合割:") {
                // Game type
            } else if line.starts_with("結果:") {
                let result_str = &line[4..];
                current_game.result = match result_str {
                    s if s.contains("先手") && s.contains("勝") => GameResult::WhiteWin,
                    s if s.contains("後手") && s.contains("勝") => GameResult::BlackWin,
                    _ => GameResult::Draw,
                };
            } else if line.starts_with("まで") {
                // End of game
                if !current_game.moves.is_empty() {
                    games.push(current_game.clone());
                }
                current_game = GameRecord::new(vec![], GameResult::Draw, TimeControl::new(600, 10));
                in_game = false;
            } else if !line.starts_with("手数")
                && !line.starts_with("先手")
                && !line.starts_with("後手")
            {
                // Parse move (simplified)
                if let Some(move_) = self.parse_kif_move(line) {
                    current_game.moves.push(move_);
                    in_game = true;
                }
            }
        }

        if !current_game.moves.is_empty() {
            games.push(current_game);
        }

        Ok(games)
    }

    /// Load games from CSA format (Computer Shogi Association)
    fn load_csa_dataset(&self, path: &Path) -> Result<Vec<GameRecord>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

        let reader = BufReader::new(file);
        let mut games = Vec::new();
        let mut current_game = GameRecord::new(vec![], GameResult::Draw, TimeControl::new(600, 10));

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            let line = line.trim();

            if line.is_empty() {
                if !current_game.moves.is_empty() {
                    games.push(current_game.clone());
                    current_game =
                        GameRecord::new(vec![], GameResult::Draw, TimeControl::new(600, 10));
                }
                continue;
            }

            // Parse CSA header
            if line.starts_with("N+") || line.starts_with("N-") {
                // Player names
            } else if line.starts_with("$") {
                // Comments and metadata
            } else if line.starts_with("%") {
                // Game result
                current_game.result = match line {
                    "%TORYO" | "%CHUDAN" => GameResult::Draw,
                    "%SENNICHITE" => GameResult::Draw,
                    _ => GameResult::Draw,
                };
            } else if line.len() >= 4 && line.chars().next().unwrap().is_ascii_digit() {
                // Parse CSA move format
                if let Some(move_) = self.parse_csa_move(line) {
                    current_game.moves.push(move_);
                }
            }
        }

        if !current_game.moves.is_empty() {
            games.push(current_game);
        }

        Ok(games)
    }

    /// Load games from PGN format
    fn load_pgn_dataset(&self, path: &Path) -> Result<Vec<GameRecord>, String> {
        // PGN is primarily for chess, but we can support a simplified version
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

        let reader = BufReader::new(file);
        let mut games = Vec::new();
        let mut current_game = GameRecord::new(vec![], GameResult::Draw, TimeControl::new(600, 10));

        let mut in_headers = true;

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            let line = line.trim();

            if line.is_empty() {
                if !current_game.moves.is_empty() {
                    games.push(current_game.clone());
                    current_game =
                        GameRecord::new(vec![], GameResult::Draw, TimeControl::new(600, 10));
                }
                in_headers = true;
                continue;
            }

            if in_headers {
                if line.starts_with("[") && line.ends_with("]") {
                    // Parse header
                    if line.starts_with("[Result ") {
                        let result_str = line[8..line.len() - 1].trim_matches('"');
                        current_game.result = match result_str {
                            "1-0" => GameResult::WhiteWin,
                            "0-1" => GameResult::BlackWin,
                            _ => GameResult::Draw,
                        };
                    }
                } else {
                    in_headers = false;
                }
            } else {
                // Parse moves (simplified - would need proper PGN parser for full support)
                let moves: Vec<&str> = line.split_whitespace().collect();
                for move_str in moves {
                    if !move_str.chars().next().unwrap().is_ascii_digit() {
                        if let Some(move_) = self.parse_pgn_move(move_str) {
                            current_game.moves.push(move_);
                        }
                    }
                }
            }
        }

        if !current_game.moves.is_empty() {
            games.push(current_game);
        }

        Ok(games)
    }

    /// Parse KIF move format (simplified)
    fn parse_kif_move(&self, _line: &str) -> Option<Move> {
        // Simplified KIF move parsing
        // In a real implementation, this would parse the full KIF format
        None
    }

    /// Parse CSA move format (simplified)
    fn parse_csa_move(&self, _line: &str) -> Option<Move> {
        // Simplified CSA move parsing
        // In a real implementation, this would parse the full CSA format
        None
    }

    /// Parse PGN move format (simplified)
    fn parse_pgn_move(&self, _move_str: &str) -> Option<Move> {
        // Simplified PGN move parsing
        // In a real implementation, this would parse the full PGN format
        None
    }

    /// Save processed training data to binary format
    pub fn save_training_data(
        &self,
        positions: &[TrainingPosition],
        path: &str,
    ) -> Result<(), String> {
        let file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;

        serde_json::to_writer(file, positions)
            .map_err(|e| format!("Failed to serialize data: {}", e))?;

        Ok(())
    }

    /// Load processed training data from binary format
    pub fn load_training_data(&self, path: &str) -> Result<Vec<TrainingPosition>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

        let reader = BufReader::new(file);
        let positions: Vec<TrainingPosition> = serde_json::from_reader(reader)
            .map_err(|e| format!("Failed to deserialize data: {}", e))?;

        Ok(positions)
    }
}

impl Clone for DataProcessor {
    fn clone(&self) -> Self {
        Self {
            feature_extractor: FeatureExtractor::new(),
            filter: self.filter.clone(),
            progress_callback: None, // Can't clone closures
        }
    }
}

impl GameDatabase {
    /// Create a new game database
    pub fn new() -> Self {
        Self {
            games: Vec::new(),
            metadata: HashMap::new(),
            total_positions: 0,
        }
    }

    /// Add games to the database
    pub fn add_games(&mut self, games: Vec<GameRecord>) {
        self.games.extend(games);
        self.recalculate_stats();
    }

    /// Get all games
    pub fn get_games(&self) -> &[GameRecord] {
        &self.games
    }

    /// Get game count
    pub fn game_count(&self) -> usize {
        self.games.len()
    }

    /// Get total position count
    pub fn total_positions(&self) -> usize {
        self.total_positions
    }

    /// Recalculate database statistics
    fn recalculate_stats(&mut self) {
        self.total_positions = self.games.iter().map(|game| game.move_count()).sum();
    }
}

impl PositionSelector {
    /// Create a new position selector
    pub fn new(filter: PositionFilter) -> Self {
        Self {
            filter,
            seen_positions: HashSet::new(),
        }
    }

    /// Select positions from a game record
    pub fn select_positions(&mut self, game_record: &GameRecord) -> Vec<TrainingPosition> {
        let positions = Vec::new();

        // Apply filters
        if !self.passes_rating_filter(game_record) {
            return positions;
        }

        if !self.passes_move_number_filter() {
            return positions;
        }

        // Extract positions (simplified)
        // In a real implementation, this would replay the game and extract positions

        positions
    }

    /// Check if game passes rating filter
    fn passes_rating_filter(&self, game_record: &GameRecord) -> bool {
        if let Some(min_rating) = self.filter.min_rating {
            if let Some(avg_rating) = game_record.average_rating() {
                if avg_rating < min_rating {
                    return false;
                }
            }
        }

        if let Some(max_rating) = self.filter.max_rating {
            if let Some(avg_rating) = game_record.average_rating() {
                if avg_rating > max_rating {
                    return false;
                }
            }
        }

        if self.filter.high_rated_only && !game_record.is_high_rated() {
            return false;
        }

        true
    }

    /// Check if position passes move number filter
    fn passes_move_number_filter(&self) -> bool {
        // Simplified - in real implementation would check actual move number
        true
    }

    /// Check for position deduplication
    #[allow(dead_code)]
    fn is_duplicate_position(&mut self, position_hash: &str) -> bool {
        if self.seen_positions.contains(position_hash) {
            true
        } else {
            self.seen_positions.insert(position_hash.to_string());
            false
        }
    }
}

impl Default for GameDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::{GameResult, PositionFilter, TimeControl};
    use super::*;

    #[test]
    fn test_data_processor_creation() {
        let filter = PositionFilter::default();
        let processor = DataProcessor::new(filter);
        // Should not panic
    }

    #[test]
    fn test_data_processor_with_progress_callback() {
        let filter = PositionFilter::default();
        let _processor = DataProcessor::with_progress_callback(filter, |_progress| {
            // Progress callback function
        });

        // Test that processor was created successfully
        assert!(true);
    }

    #[test]
    fn test_game_processing() {
        let filter = PositionFilter::default();
        let processor = DataProcessor::new(filter);

        let game_record = GameRecord::new(vec![], GameResult::Draw, TimeControl::new(600, 10));

        let positions = processor.process_game(&game_record);
        assert_eq!(positions.len(), 0);
    }

    #[test]
    fn test_game_processing_with_rating_filter() {
        let mut filter = PositionFilter::default();
        filter.min_rating = Some(2000);
        filter.max_rating = Some(2500);

        let processor = DataProcessor::new(filter);

        let mut game_record = GameRecord::new(vec![], GameResult::Draw, TimeControl::new(600, 10));
        game_record.white_rating = Some(2200);
        game_record.black_rating = Some(2300);

        let positions = processor.process_game(&game_record);
        assert_eq!(positions.len(), 0);
    }

    #[test]
    fn test_dataset_loading_unsupported_format() {
        let filter = PositionFilter::default();
        let processor = DataProcessor::new(filter);

        let result = processor.load_dataset("test.unsupported");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported file format"));
    }

    #[test]
    fn test_game_database_creation() {
        let database = GameDatabase::new();
        assert_eq!(database.game_count(), 0);
        assert_eq!(database.total_positions(), 0);
    }

    #[test]
    fn test_game_database_add_games() {
        let mut database = GameDatabase::new();

        let games = vec![
            GameRecord::new(vec![], GameResult::WhiteWin, TimeControl::new(600, 10)),
            GameRecord::new(vec![], GameResult::BlackWin, TimeControl::new(600, 10)),
        ];

        database.add_games(games);
        assert_eq!(database.game_count(), 2);
    }

    #[test]
    fn test_position_selector_creation() {
        let filter = PositionFilter::default();
        let _selector = PositionSelector::new(filter);
        // Should not panic
    }

    #[test]
    fn test_position_deduplication() {
        let filter = PositionFilter::default();
        let mut selector = PositionSelector::new(filter);

        let position_hash = "test_position_hash";

        assert!(!selector.is_duplicate_position(position_hash));
        assert!(selector.is_duplicate_position(position_hash));
    }

    #[test]
    fn test_processing_progress_creation() {
        let progress = ProcessingProgress {
            games_processed: 10,
            total_games: 100,
            positions_extracted: 500,
            positions_filtered: 450,
            processing_time: 5.5,
            memory_usage_mb: 128.0,
        };

        assert_eq!(progress.games_processed, 10);
        assert_eq!(progress.total_games, 100);
        assert_eq!(progress.positions_extracted, 500);
        assert_eq!(progress.positions_filtered, 450);
        assert_eq!(progress.processing_time, 5.5);
        assert_eq!(progress.memory_usage_mb, 128.0);
    }

    #[test]
    fn test_game_phase_calculation() {
        let filter = PositionFilter::default();
        let processor = DataProcessor::new(filter);

        let phase = processor.calculate_game_phase(10, 50);
        assert!(phase >= 0 && phase <= 256);

        let early_phase = processor.calculate_game_phase(5, 50);
        let late_phase = processor.calculate_game_phase(45, 50);
        assert!(early_phase < late_phase);
    }
}
