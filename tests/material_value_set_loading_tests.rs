use shogi_engine::evaluation::material::MaterialEvaluator;
use shogi_engine::evaluation::material::{MaterialEvaluationConfig, MaterialValueSet};
use shogi_engine::evaluation::material_value_loader::MaterialValueLoader;
use shogi_engine::types::PieceType;
use tempfile::NamedTempFile;

fn resources_dir() -> std::path::PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    std::path::Path::new(manifest_dir).join("resources/material")
}

#[test]
fn loads_builtin_value_set_from_path() {
    let path = resources_dir().join("research.json");
    let config = MaterialEvaluationConfig {
        include_hand_pieces: true,
        use_research_values: false,
        values_path: Some(path.to_string_lossy().to_string()),
        ..MaterialEvaluationConfig::default()
    };

    let evaluator = MaterialEvaluator::with_config(config);
    assert_eq!(evaluator.value_set().id, "research");
}

#[test]
fn loads_custom_json_value_set() {
    let mut custom_set = MaterialValueSet::classic();
    custom_set.id = "custom-json".to_string();
    custom_set.display_name = "Custom JSON".to_string();
    custom_set.board_values[PieceType::Rook.as_index()].mg = 1234;

    let mut temp_file = NamedTempFile::new().expect("create temp file");
    custom_set
        .to_writer(&mut temp_file)
        .expect("write custom set");
    let temp_path = temp_file.into_temp_path();

    let config = MaterialEvaluationConfig {
        include_hand_pieces: true,
        values_path: Some(temp_path.to_string_lossy().to_string()),
        ..MaterialEvaluationConfig::default()
    };

    let evaluator = MaterialEvaluator::with_config(config);
    assert_eq!(evaluator.value_set().id, "custom-json");
    assert_eq!(evaluator.get_piece_value(PieceType::Rook).mg, 1234);
}

#[test]
fn fallback_to_builtin_on_missing_file() {
    let config = MaterialEvaluationConfig {
        include_hand_pieces: true,
        values_path: Some("missing/path/value_set.json".into()),
        ..MaterialEvaluationConfig::default()
    };

    let evaluator = MaterialEvaluator::with_config(config);
    assert_eq!(evaluator.value_set().id, "research");
}

#[test]
fn material_value_loader_save_and_load_round_trip() {
    let mut value_set = MaterialValueSet::classic();
    value_set.id = "round-trip".to_string();
    value_set.board_values[PieceType::Silver.as_index()].mg = 777;

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let path = temp_dir.path().join("round_trip.json");
    MaterialValueLoader::save(&value_set, &path).expect("save value set");

    let loaded = MaterialValueSet::from_path(&path).expect("load value set");
    assert_eq!(loaded.id, "round-trip");
    assert_eq!(loaded.board_values[PieceType::Silver.as_index()].mg, 777);
}
