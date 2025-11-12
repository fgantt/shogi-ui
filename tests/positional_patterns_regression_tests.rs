use serde::Deserialize;
use shogi_engine::evaluation::integration::{
    ComponentFlags, IntegratedEvaluationConfig, IntegratedEvaluator,
};
use shogi_engine::evaluation::positional_fixtures::{
    positional_fixtures, FixtureAdvantage, FixtureTheme, PositionalFixture,
};
use shogi_engine::evaluation::positional_patterns::PositionalPatternAnalyzer;
use shogi_engine::types::{Player, TaperedScore};

const DEFAULT_THRESHOLD_CP: i32 = 20;
const INTEGRATED_WEIGHT: f32 = 0.35;

#[derive(Debug, Deserialize)]
struct FixtureDocument {
    fixtures: Vec<FixtureDocEntry>,
}

#[derive(Debug, Deserialize)]
struct FixtureDocEntry {
    name: String,
    #[allow(dead_code)]
    theme: String,
    #[allow(dead_code)]
    expected_winner: String,
}

#[test]
fn positional_fixtures_match_expected_advantage() {
    for fixture in positional_fixtures() {
        let (board, captured) = (fixture.builder)();

        let mut analyzer_black = PositionalPatternAnalyzer::new();
        let black_score = analyzer_black.evaluate_position(&board, Player::Black, &captured);

        let mut analyzer_white = PositionalPatternAnalyzer::new();
        let white_score = analyzer_white.evaluate_position(&board, Player::White, &captured);

        assert_fixture_advantage(fixture, black_score, white_score);
    }
}

#[test]
fn integrated_evaluator_respects_positional_weights_on_fixtures() {
    let fixture = positional_fixtures()
        .into_iter()
        .find(|f| matches!(f.theme, FixtureTheme::CentralFight))
        .expect("central fight fixture must exist");

    let (board, captured) = (fixture.builder)();

    let mut config = positional_only_config();
    config.weights.positional_weight = 0.0;
    let evaluator_suppressed = IntegratedEvaluator::with_config(config.clone());

    config.weights.positional_weight = 2.0;
    let evaluator_emphasised = IntegratedEvaluator::with_config(config);

    let suppressed_score = evaluator_suppressed.evaluate(&board, Player::Black, &captured);
    let emphasised_score = evaluator_emphasised.evaluate(&board, Player::Black, &captured);

    assert!(
        emphasised_score.abs() >= suppressed_score.abs(),
        "Emphasised positional weight should not produce a smaller absolute score (suppressed {suppressed_score}, emphasised {emphasised_score})"
    );
}

#[test]
fn positional_fixture_names_are_documented() {
    let parsed: FixtureDocument =
        toml::from_str(include_str!("data/positional_pattern_fixtures_index.toml"))
            .expect("fixture index document should parse");

    let documented: std::collections::HashSet<_> = parsed
        .fixtures
        .into_iter()
        .map(|entry| entry.name)
        .collect();

    for fixture in positional_fixtures() {
        assert!(
            documented.contains(fixture.name),
            "Fixture '{}' is not documented in positional_pattern_fixtures_index.toml",
            fixture.name
        );
    }
}

fn assert_fixture_advantage(
    fixture: PositionalFixture,
    black_score: TaperedScore,
    white_score: TaperedScore,
) {
    let delta = black_score.mg - white_score.mg;
    match fixture.advantage {
        FixtureAdvantage::Black { min_cp } => {
            assert!(
                delta >= min_cp.max(DEFAULT_THRESHOLD_CP),
                "Expected black advantage of at least {} cp for fixture '{}' (delta {}, black {}, white {})",
                min_cp,
                fixture.name,
                delta,
                black_score.mg,
                white_score.mg
            );
        }
        FixtureAdvantage::White { min_cp } => {
            assert!(
                -delta >= min_cp.max(DEFAULT_THRESHOLD_CP),
                "Expected white advantage of at least {} cp for fixture '{}' (delta {}, black {}, white {})",
                min_cp,
                fixture.name,
                delta,
                black_score.mg,
                white_score.mg
            );
        }
        FixtureAdvantage::Balanced { max_cp } => {
            assert!(
                delta.abs() <= max_cp,
                "Expected balance within ±{} cp for fixture '{}' (delta {}, black {}, white {})",
                max_cp,
                fixture.name,
                delta,
                black_score.mg,
                white_score.mg
            );
        }
    }
}

fn positional_only_config() -> IntegratedEvaluationConfig {
    let mut config = IntegratedEvaluationConfig::default();
    config.components = ComponentFlags::all_disabled();
    config.components.positional_patterns = true;
    config.enable_eval_cache = false;
    config.enable_phase_cache = false;
    config.use_optimized_path = false;
    config.weights.material_weight = 0.0;
    config.weights.position_weight = 0.0;
    config.weights.king_safety_weight = 0.0;
    config.weights.pawn_structure_weight = 0.0;
    config.weights.mobility_weight = 0.0;
    config.weights.center_control_weight = 0.0;
    config.weights.development_weight = 0.0;
    config.weights.tactical_weight = 0.0;
    config.weights.positional_weight = INTEGRATED_WEIGHT;
    config
}
