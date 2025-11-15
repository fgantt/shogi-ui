//! Advanced Interpolation Module
//!
//! This module provides advanced interpolation methods for the tapered evaluation system.
//! Goes beyond basic linear interpolation to provide:
//! - Cubic spline interpolation
//! - Multi-phase evaluation (opening/middlegame/endgame)
//! - Position-type specific phases
//! - Dynamic phase boundaries
//! - Adaptive interpolation
//! - Custom interpolation curves
//!
//! # Overview
//!
//! Advanced interpolation techniques:
//! - Smooth transitions with splines
//! - Multiple control points for fine-grained control
//! - Position-aware phase calculation
//! - Dynamic boundary adjustment
//! - Adaptive methods based on position characteristics
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::advanced_interpolation::AdvancedInterpolator;
//!
//! let interpolator = AdvancedInterpolator::new();
//! let score = interpolator.interpolate_multi_phase(mg, eg, phase, position_type);
//! ```

use crate::types::evaluation::TaperedScore;
use serde::{Deserialize, Serialize};

/// Advanced interpolator with multiple methods
#[derive(Debug, Clone)]
pub struct AdvancedInterpolator {
    /// Configuration
    config: AdvancedInterpolationConfig,
    /// Spline coefficients (cached)
    spline_cache: Option<SplineCoefficients>,
}

impl AdvancedInterpolator {
    /// Create a new advanced interpolator
    pub fn new() -> Self {
        Self {
            config: AdvancedInterpolationConfig::default(),
            spline_cache: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: AdvancedInterpolationConfig) -> Self {
        let mut interpolator = Self {
            config,
            spline_cache: None,
        };
        interpolator.initialize_spline();
        interpolator
    }

    /// Initialize spline coefficients
    fn initialize_spline(&mut self) {
        if self.config.use_spline {
            self.spline_cache = Some(SplineCoefficients::new(&self.config.control_points));
        }
    }

    /// Interpolate using cubic spline
    pub fn interpolate_spline(&self, score: TaperedScore, phase: i32) -> i32 {
        if let Some(ref spline) = self.spline_cache {
            spline.evaluate(score, phase)
        } else {
            // Fallback to linear
            score.interpolate(phase)
        }
    }

    /// Multi-phase interpolation (opening/middlegame/endgame)
    pub fn interpolate_multi_phase(
        &self,
        score: TaperedScore,
        phase: i32,
        position_type: PositionType,
    ) -> i32 {
        let boundaries = self.get_phase_boundaries(position_type);

        if phase >= boundaries.opening_threshold {
            // Opening phase - use mostly middlegame values
            self.interpolate_segment(score, phase, boundaries.opening_threshold, 256, 0.8, 0.2)
        } else if phase >= boundaries.endgame_threshold {
            // Middlegame - blend MG and EG
            self.interpolate_segment(
                score,
                phase,
                boundaries.endgame_threshold,
                boundaries.opening_threshold,
                0.0,
                1.0,
            )
        } else {
            // Endgame - use mostly endgame values
            self.interpolate_segment(score, phase, 0, boundaries.endgame_threshold, 0.2, 0.8)
        }
    }

    /// Interpolate within a segment
    fn interpolate_segment(
        &self,
        score: TaperedScore,
        phase: i32,
        min_phase: i32,
        max_phase: i32,
        mg_bias: f64,
        eg_bias: f64,
    ) -> i32 {
        let range = (max_phase - min_phase) as f64;
        if range <= 0.0 {
            return score.interpolate(phase);
        }

        let normalized = ((phase - min_phase) as f64 / range).clamp(0.0, 1.0);
        let mg_weight = mg_bias + (1.0 - mg_bias - eg_bias) * normalized;
        let eg_weight = 1.0 - mg_weight;

        (score.mg as f64 * mg_weight + score.eg as f64 * eg_weight) as i32
    }

    /// Get phase boundaries based on position type
    fn get_phase_boundaries(&self, position_type: PositionType) -> PhaseBoundaries {
        match position_type {
            PositionType::Tactical => PhaseBoundaries {
                opening_threshold: 180,
                endgame_threshold: 50,
            },
            PositionType::Positional => PhaseBoundaries {
                opening_threshold: 200,
                endgame_threshold: 70,
            },
            PositionType::Endgame => PhaseBoundaries {
                opening_threshold: 150,
                endgame_threshold: 80,
            },
            PositionType::Standard => self.config.default_boundaries.clone(),
        }
    }

    /// Adaptive interpolation based on position characteristics
    pub fn interpolate_adaptive(
        &self,
        score: TaperedScore,
        phase: i32,
        characteristics: &PositionCharacteristics,
    ) -> i32 {
        // Adjust phase based on position characteristics
        let adjusted_phase = self.adjust_phase(phase, characteristics);

        // Select interpolation method based on characteristics
        if characteristics.complexity > 0.7 {
            // High complexity - use spline for smoother transitions
            self.interpolate_spline(score, adjusted_phase)
        } else if characteristics.material_reduction > 0.5 {
            // High material reduction - shift to endgame faster
            self.interpolate_multi_phase(score, adjusted_phase, PositionType::Tactical)
        } else {
            // Standard position - use multi-phase
            self.interpolate_multi_phase(score, adjusted_phase, PositionType::Standard)
        }
    }

    /// Adjust phase based on position characteristics
    fn adjust_phase(&self, phase: i32, characteristics: &PositionCharacteristics) -> i32 {
        let mut adjusted = phase as f64;

        // Adjust for material count
        adjusted *= 1.0 - (characteristics.material_reduction * 0.3);

        // Adjust for complexity
        if characteristics.complexity < 0.3 {
            // Low complexity - accelerate toward endgame
            adjusted *= 0.8;
        }

        // Adjust for king safety
        if characteristics.king_safety < 0.3 {
            // Unsafe king - stay in middlegame evaluation longer
            adjusted *= 1.2;
        }

        adjusted.clamp(0.0, 256.0) as i32
    }

    /// Bezier curve interpolation
    pub fn interpolate_bezier(
        &self,
        score: TaperedScore,
        phase: i32,
        control1: f64,
        control2: f64,
    ) -> i32 {
        let t = (phase as f64 / 256.0).clamp(0.0, 1.0);

        // Cubic Bezier with control points
        let p0 = 0.0;
        let p1 = control1;
        let p2 = control2;
        let p3 = 1.0;

        let one_minus_t = 1.0 - t;
        let bezier_t = one_minus_t * one_minus_t * one_minus_t * p0
            + 3.0 * one_minus_t * one_minus_t * t * p1
            + 3.0 * one_minus_t * t * t * p2
            + t * t * t * p3;

        let mg_weight = 1.0 - bezier_t;
        let eg_weight = bezier_t;

        (score.mg as f64 * mg_weight + score.eg as f64 * eg_weight) as i32
    }

    /// Custom interpolation with user-defined function
    pub fn interpolate_custom<F>(&self, score: TaperedScore, phase: i32, custom_fn: F) -> i32
    where
        F: Fn(i32, i32, f64) -> i32,
    {
        let t = (phase as f64 / 256.0).clamp(0.0, 1.0);
        custom_fn(score.mg, score.eg, t)
    }
}

impl Default for AdvancedInterpolator {
    fn default() -> Self {
        Self::new()
    }
}

/// Advanced interpolation configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdvancedInterpolationConfig {
    /// Enable spline interpolation
    pub use_spline: bool,
    /// Control points for spline (normalized 0-1)
    pub control_points: Vec<(f64, f64)>,
    /// Default phase boundaries
    pub default_boundaries: PhaseBoundaries,
    /// Enable adaptive interpolation
    pub enable_adaptive: bool,
}

impl Default for AdvancedInterpolationConfig {
    fn default() -> Self {
        Self {
            use_spline: false,
            control_points: vec![
                (0.0, 0.0),  // Endgame start
                (0.33, 0.3), // Early middlegame
                (0.66, 0.7), // Late middlegame
                (1.0, 1.0),  // Opening
            ],
            default_boundaries: PhaseBoundaries::default(),
            enable_adaptive: false,
        }
    }
}

/// Phase boundaries for multi-phase evaluation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseBoundaries {
    /// Phase threshold for opening (≥ this = opening)
    pub opening_threshold: i32,
    /// Phase threshold for endgame (< this = endgame)
    pub endgame_threshold: i32,
}

impl Default for PhaseBoundaries {
    fn default() -> Self {
        Self {
            opening_threshold: 192,
            endgame_threshold: 64,
        }
    }
}

/// Position type for adaptive interpolation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionType {
    /// Tactical position (sharp, concrete)
    Tactical,
    /// Positional (strategic, long-term)
    Positional,
    /// Endgame position
    Endgame,
    /// Standard position
    Standard,
}

/// Position characteristics for adaptive interpolation
#[derive(Debug, Clone)]
pub struct PositionCharacteristics {
    /// Material reduction from starting position (0.0-1.0)
    pub material_reduction: f64,
    /// Position complexity (0.0-1.0)
    pub complexity: f64,
    /// King safety (0.0-1.0, 1.0 = safe)
    pub king_safety: f64,
}

impl Default for PositionCharacteristics {
    fn default() -> Self {
        Self {
            material_reduction: 0.0,
            complexity: 0.5,
            king_safety: 1.0,
        }
    }
}

/// Cubic spline coefficients for smooth interpolation
#[derive(Debug, Clone)]
struct SplineCoefficients {
    segments: Vec<SplineSegment>,
}

impl SplineCoefficients {
    fn new(control_points: &[(f64, f64)]) -> Self {
        let segments = Self::compute_segments(control_points);
        Self { segments }
    }

    fn compute_segments(points: &[(f64, f64)]) -> Vec<SplineSegment> {
        if points.len() < 2 {
            return vec![];
        }

        let mut segments = Vec::new();

        for i in 0..points.len() - 1 {
            let (x0, y0) = points[i];
            let (x1, y1) = points[i + 1];

            // Simple cubic segment (can be enhanced with natural splines)
            segments.push(SplineSegment {
                x_start: x0,
                x_end: x1,
                a: y0,
                b: 3.0 * (y1 - y0),
                c: -2.0 * (y1 - y0),
                d: 0.0,
            });
        }

        segments
    }

    fn evaluate(&self, score: TaperedScore, phase: i32) -> i32 {
        let t = (phase as f64 / 256.0).clamp(0.0, 1.0);

        // Find appropriate segment
        for segment in &self.segments {
            if t >= segment.x_start && t <= segment.x_end {
                let local_t = (t - segment.x_start) / (segment.x_end - segment.x_start);
                let interpolation_factor = segment.evaluate(local_t);

                let mg_weight = 1.0 - interpolation_factor;
                let eg_weight = interpolation_factor;

                return (score.mg as f64 * mg_weight + score.eg as f64 * eg_weight) as i32;
            }
        }

        // Fallback
        score.interpolate(phase)
    }
}

/// Single cubic spline segment
#[derive(Debug, Clone)]
struct SplineSegment {
    x_start: f64,
    x_end: f64,
    a: f64,
    b: f64,
    c: f64,
    d: f64,
}

impl SplineSegment {
    fn evaluate(&self, t: f64) -> f64 {
        // Cubic polynomial: a + bt + ct^2 + dt^3
        self.a + self.b * t + self.c * t * t + self.d * t * t * t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolator_creation() {
        let interpolator = AdvancedInterpolator::new();
        assert!(!interpolator.config.use_spline);
    }

    #[test]
    fn test_spline_interpolation() {
        let mut config = AdvancedInterpolationConfig::default();
        config.use_spline = true;

        let interpolator = AdvancedInterpolator::with_config(config);
        let score = TaperedScore::new_tapered(100, 200);

        let result = interpolator.interpolate_spline(score, 128);

        // Should be between mg and eg
        assert!(result >= 100 && result <= 200);
    }

    #[test]
    fn test_multi_phase_interpolation() {
        let interpolator = AdvancedInterpolator::new();
        let score = TaperedScore::new_tapered(100, 200);

        // Opening
        let opening_result =
            interpolator.interpolate_multi_phase(score, 256, PositionType::Standard);
        assert!(opening_result >= 100);

        // Endgame
        let endgame_result =
            interpolator.interpolate_multi_phase(score, 32, PositionType::Standard);
        assert!(endgame_result <= 200);
    }

    #[test]
    fn test_phase_boundaries() {
        let interpolator = AdvancedInterpolator::new();

        let tactical = interpolator.get_phase_boundaries(PositionType::Tactical);
        let positional = interpolator.get_phase_boundaries(PositionType::Positional);

        // Tactical positions should have different boundaries
        assert_ne!(tactical.opening_threshold, positional.opening_threshold);
    }

    #[test]
    fn test_adaptive_interpolation() {
        let interpolator = AdvancedInterpolator::new();
        let score = TaperedScore::new_tapered(100, 200);

        let characteristics = PositionCharacteristics {
            material_reduction: 0.5,
            complexity: 0.6,
            king_safety: 0.8,
        };

        let result = interpolator.interpolate_adaptive(score, 128, &characteristics);

        assert!(result >= 100 && result <= 200);
    }

    #[test]
    fn test_phase_adjustment() {
        let interpolator = AdvancedInterpolator::new();

        let high_reduction = PositionCharacteristics {
            material_reduction: 0.8,
            complexity: 0.5,
            king_safety: 0.5,
        };

        let adjusted = interpolator.adjust_phase(128, &high_reduction);

        // Should reduce phase (closer to endgame)
        assert!(adjusted < 128);
    }

    #[test]
    fn test_bezier_interpolation() {
        let interpolator = AdvancedInterpolator::new();
        let score = TaperedScore::new_tapered(100, 200);

        let result = interpolator.interpolate_bezier(score, 128, 0.33, 0.67);

        assert!(result >= 100 && result <= 200);
    }

    #[test]
    fn test_custom_interpolation() {
        let interpolator = AdvancedInterpolator::new();
        let score = TaperedScore::new_tapered(100, 200);

        let custom_fn = |mg: i32, eg: i32, t: f64| ((mg as f64 * (1.0 - t) + eg as f64 * t) as i32);

        let result = interpolator.interpolate_custom(score, 128, custom_fn);

        assert!(result >= 100 && result <= 200);
    }

    #[test]
    fn test_spline_coefficients() {
        let points = vec![(0.0, 0.0), (0.5, 0.5), (1.0, 1.0)];
        let spline = SplineCoefficients::new(&points);

        assert_eq!(spline.segments.len(), 2);
    }

    #[test]
    fn test_position_characteristics_default() {
        let characteristics = PositionCharacteristics::default();

        assert_eq!(characteristics.material_reduction, 0.0);
        assert_eq!(characteristics.complexity, 0.5);
        assert_eq!(characteristics.king_safety, 1.0);
    }

    #[test]
    fn test_phase_boundaries_default() {
        let boundaries = PhaseBoundaries::default();

        assert_eq!(boundaries.opening_threshold, 192);
        assert_eq!(boundaries.endgame_threshold, 64);
    }

    #[test]
    fn test_config_default() {
        let config = AdvancedInterpolationConfig::default();

        assert!(!config.use_spline);
        assert_eq!(config.control_points.len(), 4);
    }

    #[test]
    fn test_multi_phase_opening() {
        let interpolator = AdvancedInterpolator::new();
        let score = TaperedScore::new_tapered(100, 200);

        let result = interpolator.interpolate_multi_phase(score, 256, PositionType::Standard);

        // In opening, should favor middlegame
        assert!(result < 150);
    }

    #[test]
    fn test_multi_phase_endgame() {
        let interpolator = AdvancedInterpolator::new();
        let score = TaperedScore::new_tapered(100, 200);

        let result = interpolator.interpolate_multi_phase(score, 10, PositionType::Standard);

        // In endgame, should favor endgame values
        assert!(result > 150);
    }

    #[test]
    fn test_bezier_endpoints() {
        let interpolator = AdvancedInterpolator::new();
        let score = TaperedScore::new_tapered(100, 200);

        // At phase 256 (opening), should be close to mg
        let opening = interpolator.interpolate_bezier(score, 256, 0.33, 0.67);
        assert!((opening - 100).abs() < 10);

        // At phase 0 (endgame), should be close to eg
        let endgame = interpolator.interpolate_bezier(score, 0, 0.33, 0.67);
        assert!((endgame - 200).abs() < 10);
    }

    #[test]
    fn test_adaptive_high_complexity() {
        let interpolator = AdvancedInterpolator::new();
        let score = TaperedScore::new_tapered(100, 200);

        let high_complexity = PositionCharacteristics {
            material_reduction: 0.3,
            complexity: 0.9,
            king_safety: 0.5,
        };

        let result = interpolator.interpolate_adaptive(score, 128, &high_complexity);

        // Should produce a valid result
        assert!(result >= 100 && result <= 200);
    }
}
