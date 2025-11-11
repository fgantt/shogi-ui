use serde::Deserialize;
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::tactical_patterns::{TacticalConfig, TacticalPatternRecognizer};

#[derive(Debug, Deserialize)]
struct TacticalCorpus {
    positions: Vec<TacticalPosition>,
}

#[derive(Debug, Deserialize)]
struct TacticalPosition {
    name: String,
    fen: String,
    expectation: Expectation,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Expectation {
    Positive,
    Negative,
    Neutral,
}

const NEUTRAL_EPSILON_CP: i32 = 80;

#[test]
fn tactical_corpus_scores_match_expectations() {
    let corpus: TacticalCorpus =
        toml::from_str(include_str!("data/tactical_corpus.toml")).expect("valid corpus");

    for case in corpus.positions {
        let mut recognizer = TacticalPatternRecognizer::with_config(TacticalConfig::default());
        let (board, player, captured) = BitboardBoard::from_fen(&case.fen)
            .unwrap_or_else(|_| panic!("invalid FEN {}", case.fen));
        let score = recognizer.evaluate_tactics(&board, player, &captured);
        let mg = score.mg;
        match case.expectation {
            Expectation::Positive => assert!(
                mg > 0,
                "Expected positive score for {}, got {}",
                case.name,
                mg
            ),
            Expectation::Negative => assert!(
                mg < 0,
                "Expected negative score for {}, got {}",
                case.name,
                mg
            ),
            Expectation::Neutral => assert!(
                mg.abs() <= NEUTRAL_EPSILON_CP,
                "Expected neutral score (±{NEUTRAL_EPSILON_CP}) for {}, got {}",
                case.name,
                mg
            ),
        }

        // Ensure evaluator perspective matches expectation for the opponent as well.
        let opponent = player.opposite();
        let opponent_score = recognizer.evaluate_tactics(&board, opponent, &captured);
        let opponent_mg = opponent_score.mg;
        match case.expectation {
            Expectation::Positive => assert!(
                mg >= opponent_mg - NEUTRAL_EPSILON_CP,
                "Opponent score should not exceed attacker advantage by more than {NEUTRAL_EPSILON_CP} for {} (attacker {}, opponent {})",
                case.name,
                mg,
                opponent_mg
            ),
            Expectation::Negative => assert!(
                mg <= opponent_mg + NEUTRAL_EPSILON_CP,
                "Opponent score should not be worse than defender penalty by more than {NEUTRAL_EPSILON_CP} for {} (defender {}, opponent {})",
                case.name,
                mg,
                opponent_mg
            ),
            Expectation::Neutral => assert!(
                (opponent_mg - mg).abs() <= NEUTRAL_EPSILON_CP,
                "Neutral scenario should evaluate symmetrically within ±{NEUTRAL_EPSILON_CP} for {} (attacker {}, opponent {})",
                case.name,
                mg,
                opponent_mg
            ),
        }
    }
}
