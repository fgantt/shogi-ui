use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::castle_fixtures::{
    castle_fixtures, CastleFixture, CastleFixtureTheme,
};
use shogi_engine::evaluation::castles::CastleRecognizer;
use shogi_engine::evaluation::king_safety::KingSafetyEvaluator;
use shogi_engine::types::Player;

/// Helper to assert castle evaluation quality
fn assert_castle_quality(
    fixture: &CastleFixture,
    evaluation: shogi_engine::evaluation::castles::CastleEvaluation,
) {
    match fixture.theme {
        CastleFixtureTheme::Canonical => {
            // Canonical castles should have high quality
            assert!(
                evaluation.quality >= 0.7,
                "Canonical castle {} should have quality >= 0.7, got {}",
                fixture.name,
                evaluation.quality
            );
            assert!(
                evaluation.matched_pattern.is_some(),
                "Canonical castle {} should match a pattern",
                fixture.name
            );
        }
        CastleFixtureTheme::Partial => {
            // Partial castles should have moderate quality
            assert!(
                evaluation.quality >= 0.3 && evaluation.quality < 0.8,
                "Partial castle {} should have quality between 0.3 and 0.8, got {}",
                fixture.name,
                evaluation.quality
            );
        }
        CastleFixtureTheme::Broken => {
            // Broken castles should have low quality
            assert!(
                evaluation.quality < 0.5,
                "Broken castle {} should have quality < 0.5, got {}",
                fixture.name,
                evaluation.quality
            );
        }
        CastleFixtureTheme::Attacked => {
            // Attacked castles should have infiltration detected
            assert!(
                evaluation.infiltration_ratio > 0.0,
                "Attacked castle {} should have infiltration_ratio > 0.0, got {}",
                fixture.name,
                evaluation.infiltration_ratio
            );
        }
        CastleFixtureTheme::Mirrored => {
            // Mirrored castles should be recognized (quality may vary)
            assert!(
                evaluation.quality >= 0.0,
                "Mirrored castle {} should have valid quality, got {}",
                fixture.name,
                evaluation.quality
            );
        }
    }
}

#[test]
fn test_castle_fixtures_recognition() {
    let recognizer = CastleRecognizer::new();
    let fixtures = castle_fixtures();

    for fixture in fixtures {
        let (board, king_pos) = (fixture.builder)(fixture.player);
        let evaluation = recognizer.evaluate_castle(&board, fixture.player, king_pos);

        assert_castle_quality(&fixture, evaluation);
    }
}

#[test]
fn test_castle_fixtures_quality_scores() {
    let recognizer = CastleRecognizer::new();
    let fixtures = castle_fixtures();

    for fixture in fixtures {
        let (board, king_pos) = (fixture.builder)(fixture.player);
        let evaluation = recognizer.evaluate_castle(&board, fixture.player, king_pos);

        // All evaluations should have valid quality scores
        assert!(
            evaluation.quality >= 0.0 && evaluation.quality <= 1.0,
            "Fixture {} should have quality in [0.0, 1.0], got {}",
            fixture.name,
            evaluation.quality
        );

        // Coverage ratios should be valid
        assert!(
            evaluation.coverage_ratio >= 0.0 && evaluation.coverage_ratio <= 1.0,
            "Fixture {} should have coverage_ratio in [0.0, 1.0], got {}",
            fixture.name,
            evaluation.coverage_ratio
        );
    }
}

#[test]
fn test_canonical_vs_partial_quality_ordering() {
    let recognizer = CastleRecognizer::new();
    let fixtures = castle_fixtures();

    // Find canonical and partial Mino castles
    let canonical = fixtures
        .iter()
        .find(|f| f.name == "mino_canonical_black")
        .unwrap();
    let partial = fixtures
        .iter()
        .find(|f| f.name == "mino_partial_missing_silver")
        .unwrap();

    let (canonical_board, canonical_king) = (canonical.builder)(canonical.player);
    let (partial_board, partial_king) = (partial.builder)(partial.player);

    let canonical_eval =
        recognizer.evaluate_castle(&canonical_board, canonical.player, canonical_king);
    let partial_eval = recognizer.evaluate_castle(&partial_board, partial.player, partial_king);

    // Canonical should have higher quality than partial
    assert!(
        canonical_eval.quality > partial_eval.quality,
        "Canonical castle quality {} should be > partial quality {}",
        canonical_eval.quality,
        partial_eval.quality
    );
}

#[test]
fn test_broken_castle_penalties() {
    let evaluator = KingSafetyEvaluator::new();
    let fixtures = castle_fixtures();

    // Find broken and canonical castles
    let broken = fixtures
        .iter()
        .find(|f| f.name == "mino_broken_breached_wall")
        .unwrap();
    let canonical = fixtures
        .iter()
        .find(|f| f.name == "mino_canonical_black")
        .unwrap();

    let (broken_board, _) = (broken.builder)(broken.player);
    let (canonical_board, _) = (canonical.builder)(canonical.player);

    let broken_score = evaluator.evaluate(&broken_board, broken.player);
    let canonical_score = evaluator.evaluate(&canonical_board, canonical.player);

    // Broken castle should have lower (more negative) score
    assert!(
        broken_score.mg < canonical_score.mg || broken_score.eg < canonical_score.eg,
        "Broken castle should have lower score than canonical"
    );
}

#[test]
fn test_attacked_castle_infiltration() {
    let recognizer = CastleRecognizer::new();
    let fixtures = castle_fixtures();

    for fixture in fixtures {
        if fixture.theme == CastleFixtureTheme::Attacked {
            let (board, king_pos) = (fixture.builder)(fixture.player);
            let evaluation = recognizer.evaluate_castle(&board, fixture.player, king_pos);

            // Attacked castles should show infiltration
            assert!(
                evaluation.infiltration_ratio > 0.0,
                "Attacked castle {} should have infiltration_ratio > 0.0, got {}",
                fixture.name,
                evaluation.infiltration_ratio
            );
        }
    }
}

#[test]
fn test_mirrored_castle_symmetry() {
    let recognizer = CastleRecognizer::new();
    let fixtures = castle_fixtures();

    // Find left and right mirrored Mino castles
    let left = fixtures
        .iter()
        .find(|f| f.name == "mino_mirrored_left_black")
        .unwrap();
    let right = fixtures
        .iter()
        .find(|f| f.name == "mino_mirrored_right_black")
        .unwrap();

    let (left_board, left_king) = (left.builder)(left.player);
    let (right_board, right_king) = (right.builder)(right.player);

    let left_eval = recognizer.evaluate_castle(&left_board, left.player, left_king);
    let right_eval = recognizer.evaluate_castle(&right_board, right.player, right_king);

    // Mirrored castles should have similar quality (allowing for small differences)
    let quality_diff = (left_eval.quality - right_eval.quality).abs();
    assert!(
        quality_diff < 0.2,
        "Mirrored castles should have similar quality, left: {}, right: {}",
        left_eval.quality,
        right_eval.quality
    );
}

#[test]
fn test_castle_telemetry_tracking() {
    let evaluator = KingSafetyEvaluator::new();
    let fixtures = castle_fixtures();

    // Evaluate multiple castles
    for fixture in fixtures.iter().take(5) {
        let (board, _) = (fixture.builder)(fixture.player);
        evaluator.evaluate(&board, fixture.player);
    }

    let stats = evaluator.stats();
    assert!(stats.evaluations > 0, "Should have tracked evaluations");
    assert!(
        stats.castle_matches > 0 || stats.partial_castles > 0,
        "Should have tracked castle matches or partial castles"
    );
}

#[test]
fn test_castle_cache_effectiveness() {
    let recognizer = CastleRecognizer::new();
    let fixtures = castle_fixtures();

    // Evaluate same position multiple times
    let fixture = fixtures
        .iter()
        .find(|f| f.name == "mino_canonical_black")
        .unwrap();
    let (board, king_pos) = (fixture.builder)(fixture.player);

    // First evaluation - cache miss
    recognizer.evaluate_castle(&board, fixture.player, king_pos);
    let stats_after_first = recognizer.get_cache_stats();
    assert_eq!(stats_after_first.misses, 1);

    // Second evaluation - cache hit
    recognizer.evaluate_castle(&board, fixture.player, king_pos);
    let stats_after_second = recognizer.get_cache_stats();
    assert_eq!(stats_after_second.hits, 1);
    assert_eq!(stats_after_second.misses, 1);
}
