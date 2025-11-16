//! Principal Variation Search (PVS) Module
//!
//! This module handles PVS helper functions for bounds validation, score conversion, and
//! search result validation. The main `negamax_with_context()` function remains in
//! `search_engine.rs` due to tight coupling and will be extracted as part of Task 1.8
//! (coordinator refactoring).
//!
//! Extracted from `search_engine.rs` as part of Task 1.0: File Modularization and Structure Improvements.

use crate::types::core::Move;

/// Constants for score bounds
pub const MIN_SCORE: i32 = i32::MIN + 1;
pub const MAX_SCORE: i32 = i32::MAX - 1;

/// PVS helper functions for bounds and validation
pub struct PVSHelper;

impl PVSHelper {
    /// Validate window bounds (alpha < beta)
    pub fn validate_bounds(alpha: i32, beta: i32) -> bool {
        alpha < beta
    }

    /// Check if score causes beta cutoff
    pub fn is_beta_cutoff(score: i32, beta: i32) -> bool {
        score >= beta
    }

    /// Check if score improves alpha
    pub fn improves_alpha(score: i32, alpha: i32) -> bool {
        score > alpha
    }

    /// Check if score is within window bounds
    pub fn is_in_window(score: i32, alpha: i32, beta: i32) -> bool {
        score > alpha && score < beta
    }

    /// Clamp score to valid range
    pub fn clamp_score(score: i32) -> i32 {
        score.max(MIN_SCORE).min(MAX_SCORE)
    }

    /// Check if score is within reasonable bounds (not mate scores)
    pub fn is_reasonable_score(score: i32) -> bool {
        score >= -50000 && score <= 50000
    }

    /// Convert tablebase result to search score
    pub fn convert_tablebase_score(result: &crate::tablebase::TablebaseResult) -> i32 {
        match result.outcome {
            crate::tablebase::TablebaseOutcome::Win => {
                // Winning position: score based on distance to mate
                if let Some(distance) = result.distance_to_mate {
                    10000 - distance as i32
                } else {
                    10000
                }
            }
            crate::tablebase::TablebaseOutcome::Loss => {
                // Losing position: negative score based on distance to mate
                if let Some(distance) = result.distance_to_mate {
                    -10000 - distance as i32
                } else {
                    -10000
                }
            }
            crate::tablebase::TablebaseOutcome::Draw => {
                // Draw position
                0
            }
            crate::tablebase::TablebaseOutcome::Unknown => {
                // Unknown position: use confidence to scale the score
                if let Some(distance) = result.distance_to_mate {
                    ((10000 - distance as i32) as f32 * result.confidence) as i32
                } else {
                    0
                }
            }
        }
    }

    /// Validate search result consistency
    pub fn validate_search_result(
        result: Option<(Move, i32)>,
        _depth: u8,
        alpha: i32,
        beta: i32,
    ) -> bool {
        match result {
            Some((ref move_, score)) => {
                // Validate score is within reasonable bounds
                if !Self::is_reasonable_score(score) {
                    crate::utils::telemetry::trace_log(
                        "SEARCH_VALIDATION",
                        &format!("WARNING: Score {} is outside reasonable bounds", score),
                    );
                    return false;
                }

                // Validate move is not empty
                if move_.to_usi_string().is_empty() {
                    crate::utils::telemetry::trace_log(
                        "SEARCH_VALIDATION",
                        "WARNING: Empty move string in search result",
                    );
                    return false;
                }

                // Safe arithmetic to prevent integer overflow
                let alpha_threshold = alpha.saturating_sub(1000);
                let beta_threshold = beta.saturating_add(1000);
                if score < alpha_threshold || score > beta_threshold {
                    crate::utils::telemetry::trace_log(
                        "SEARCH_VALIDATION",
                        &format!(
                            "WARNING: Score {} significantly outside window [{}, {}]",
                            score, alpha, beta
                        ),
                    );
                    // This is not necessarily an error, but worth logging
                }

                // Validate move format (basic USI format check)
                let move_str = move_.to_usi_string();
                if move_str.len() < 4 || move_str.len() > 6 {
                    crate::utils::telemetry::trace_log(
                        "SEARCH_VALIDATION",
                        &format!("WARNING: Move string length {} is unusual", move_str.len()),
                    );
                    // Not necessarily an error, but worth logging
                }

                true
            }
            None => {
                // None is valid if there are no legal moves
                true
            }
        }
    }

    /// Determine transposition flag from score and bounds
    pub fn determine_transposition_flag(score: i32, alpha: i32, beta: i32) -> crate::types::TranspositionFlag {
        if score <= alpha {
            crate::types::TranspositionFlag::UpperBound
        } else if score >= beta {
            crate::types::TranspositionFlag::LowerBound
        } else {
            crate::types::TranspositionFlag::Exact
        }
    }

    /// Check if window is full-width (no aspiration)
    pub fn is_full_width_window(alpha: i32, beta: i32) -> bool {
        alpha <= MIN_SCORE && beta >= MAX_SCORE
    }

    /// Calculate window size
    pub fn calculate_window_size(alpha: i32, beta: i32) -> i32 {
        if Self::is_full_width_window(alpha, beta) {
            i32::MAX
        } else {
            beta.saturating_sub(alpha)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_validation() {
        assert!(PVSHelper::validate_bounds(100, 200));
        assert!(!PVSHelper::validate_bounds(200, 100));
        assert!(!PVSHelper::validate_bounds(100, 100));
    }

    #[test]
    fn test_beta_cutoff() {
        assert!(PVSHelper::is_beta_cutoff(200, 150));
        assert!(!PVSHelper::is_beta_cutoff(100, 150));
    }

    #[test]
    fn test_improves_alpha() {
        assert!(PVSHelper::improves_alpha(200, 100));
        assert!(!PVSHelper::improves_alpha(100, 200));
    }

    #[test]
    fn test_is_in_window() {
        assert!(PVSHelper::is_in_window(150, 100, 200));
        assert!(!PVSHelper::is_in_window(250, 100, 200));
        assert!(!PVSHelper::is_in_window(50, 100, 200));
    }

    #[test]
    fn test_clamp_score() {
        assert_eq!(PVSHelper::clamp_score(100), 100);
        assert_eq!(PVSHelper::clamp_score(i32::MIN), MIN_SCORE);
        assert_eq!(PVSHelper::clamp_score(i32::MAX), MAX_SCORE);
    }

    #[test]
    fn test_is_reasonable_score() {
        assert!(PVSHelper::is_reasonable_score(1000));
        assert!(PVSHelper::is_reasonable_score(-1000));
        assert!(!PVSHelper::is_reasonable_score(100000));
        assert!(!PVSHelper::is_reasonable_score(-100000));
    }

    #[test]
    fn test_is_full_width_window() {
        assert!(PVSHelper::is_full_width_window(MIN_SCORE, MAX_SCORE));
        assert!(!PVSHelper::is_full_width_window(100, 200));
    }

    #[test]
    fn test_calculate_window_size() {
        assert_eq!(PVSHelper::calculate_window_size(MIN_SCORE, MAX_SCORE), i32::MAX);
        assert_eq!(PVSHelper::calculate_window_size(100, 200), 100);
    }

    #[test]
    fn test_determine_transposition_flag() {
        let flag1 = PVSHelper::determine_transposition_flag(50, 100, 200);
        assert!(matches!(flag1, crate::types::TranspositionFlag::UpperBound));

        let flag2 = PVSHelper::determine_transposition_flag(250, 100, 200);
        assert!(matches!(flag2, crate::types::TranspositionFlag::LowerBound));

        let flag3 = PVSHelper::determine_transposition_flag(150, 100, 200);
        assert!(matches!(flag3, crate::types::TranspositionFlag::Exact));
    }
}

