use crate::evaluation::piece_square_tables::{PieceSquareTableRaw, PieceSquareTables};
use crate::types::core::PieceType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Deserialize)]
struct SerializedPieceTable {
    mg: [[i32; 9]; 9],
    eg: [[i32; 9]; 9],
}

#[derive(Debug, Deserialize)]
struct SerializedPieceSquareTables {
    version: Option<String>,
    description: Option<String>,
    tables: HashMap<String, SerializedPieceTable>,
}

#[derive(Debug)]
pub struct PieceSquareTableLoadResult {
    pub tables: PieceSquareTables,
    pub version: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Error)]
pub enum PieceSquareTableLoadError {
    #[error("failed to read PST file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse PST file: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("unexpected piece identifier '{0}' in PST file")]
    UnknownPiece(String),
    #[error("duplicate piece entry for {0:?}")]
    DuplicatePiece(PieceType),
    #[error("missing piece-square table for {0:?}")]
    MissingPiece(PieceType),
    #[error("king positional tables must be zero in both phases")]
    NonZeroKingTables,
    #[error("custom preset selected but no PST file path was provided")]
    MissingCustomPath,
}

pub struct PieceSquareTableLoader;

impl PieceSquareTableLoader {
    pub fn from_path(
        path: impl AsRef<Path>,
    ) -> Result<PieceSquareTableLoadResult, PieceSquareTableLoadError> {
        let mut file = File::open(path)?;
        Self::from_reader(&mut file)
    }

    pub fn from_reader<R>(
        reader: &mut R,
    ) -> Result<PieceSquareTableLoadResult, PieceSquareTableLoadError>
    where
        R: Read,
    {
        let mut data = String::new();
        reader.read_to_string(&mut data)?;
        let serialized: SerializedPieceSquareTables = serde_json::from_str(&data)?;
        Self::from_serialized(serialized)
    }

    fn from_serialized(
        serialized: SerializedPieceSquareTables,
    ) -> Result<PieceSquareTableLoadResult, PieceSquareTableLoadError> {
        let mut builder = PieceTableBuilder::default();
        for (name, entry) in serialized.tables.into_iter() {
            let piece = parse_piece_type(&name)
                .ok_or_else(|| PieceSquareTableLoadError::UnknownPiece(name.clone()))?;
            builder.insert(piece, entry.mg, entry.eg)?;
        }
        let raw = builder.build()?;
        Ok(PieceSquareTableLoadResult {
            tables: PieceSquareTables::from_raw(raw),
            version: serialized.version,
            description: serialized.description,
        })
    }

    pub fn load(
        config: &PieceSquareTableConfig,
    ) -> Result<PieceSquareTables, PieceSquareTableLoadError> {
        if let Some(path) =
            config
                .values_path
                .as_ref()
                .and_then(|p| if p.trim().is_empty() { None } else { Some(p) })
        {
            return Ok(Self::from_path(path)?.tables);
        }

        match config.preset {
            PieceSquareTablePreset::Builtin => Ok(PieceSquareTables::new()),
            PieceSquareTablePreset::Default => Ok(Self::from_path(DEFAULT_PRESET_PATH)?.tables),
            PieceSquareTablePreset::Custom => Err(PieceSquareTableLoadError::MissingCustomPath),
        }
    }
}

#[derive(Default)]
struct PieceTableBuilder {
    entries: HashMap<PieceType, ([[i32; 9]; 9], [[i32; 9]; 9])>,
}

impl PieceTableBuilder {
    fn insert(
        &mut self,
        piece: PieceType,
        mg: [[i32; 9]; 9],
        eg: [[i32; 9]; 9],
    ) -> Result<(), PieceSquareTableLoadError> {
        if self.entries.contains_key(&piece) {
            return Err(PieceSquareTableLoadError::DuplicatePiece(piece));
        }
        self.entries.insert(piece, (mg, eg));
        Ok(())
    }

    fn build(mut self) -> Result<PieceSquareTableRaw, PieceSquareTableLoadError> {
        let king_tables = self
            .entries
            .remove(&PieceType::King)
            .ok_or(PieceSquareTableLoadError::MissingPiece(PieceType::King))?;
        let zero = [[0; 9]; 9];
        if king_tables.0 != zero || king_tables.1 != zero {
            return Err(PieceSquareTableLoadError::NonZeroKingTables);
        }

        let mut take = |piece: PieceType| -> Result<([[i32; 9]; 9], [[i32; 9]; 9]), PieceSquareTableLoadError> {
            self.entries
                .remove(&piece)
                .ok_or(PieceSquareTableLoadError::MissingPiece(piece))
        };

        let (pawn_table_mg, pawn_table_eg) = take(PieceType::Pawn)?;
        let (lance_table_mg, lance_table_eg) = take(PieceType::Lance)?;
        let (knight_table_mg, knight_table_eg) = take(PieceType::Knight)?;
        let (silver_table_mg, silver_table_eg) = take(PieceType::Silver)?;
        let (gold_table_mg, gold_table_eg) = take(PieceType::Gold)?;
        let (bishop_table_mg, bishop_table_eg) = take(PieceType::Bishop)?;
        let (rook_table_mg, rook_table_eg) = take(PieceType::Rook)?;
        let (promoted_pawn_table_mg, promoted_pawn_table_eg) = take(PieceType::PromotedPawn)?;
        let (promoted_lance_table_mg, promoted_lance_table_eg) = take(PieceType::PromotedLance)?;
        let (promoted_knight_table_mg, promoted_knight_table_eg) = take(PieceType::PromotedKnight)?;
        let (promoted_silver_table_mg, promoted_silver_table_eg) = take(PieceType::PromotedSilver)?;
        let (promoted_bishop_table_mg, promoted_bishop_table_eg) = take(PieceType::PromotedBishop)?;
        let (promoted_rook_table_mg, promoted_rook_table_eg) = take(PieceType::PromotedRook)?;

        Ok(PieceSquareTableRaw {
            pawn_table_mg,
            pawn_table_eg,
            lance_table_mg,
            lance_table_eg,
            knight_table_mg,
            knight_table_eg,
            silver_table_mg,
            silver_table_eg,
            gold_table_mg,
            gold_table_eg,
            bishop_table_mg,
            bishop_table_eg,
            rook_table_mg,
            rook_table_eg,
            promoted_pawn_table_mg,
            promoted_pawn_table_eg,
            promoted_lance_table_mg,
            promoted_lance_table_eg,
            promoted_knight_table_mg,
            promoted_knight_table_eg,
            promoted_silver_table_mg,
            promoted_silver_table_eg,
            promoted_bishop_table_mg,
            promoted_bishop_table_eg,
            promoted_rook_table_mg,
            promoted_rook_table_eg,
        })
    }
}

fn parse_piece_type(name: &str) -> Option<PieceType> {
    match name.to_ascii_lowercase().as_str() {
        "pawn" => Some(PieceType::Pawn),
        "lance" => Some(PieceType::Lance),
        "knight" => Some(PieceType::Knight),
        "silver" => Some(PieceType::Silver),
        "gold" => Some(PieceType::Gold),
        "bishop" => Some(PieceType::Bishop),
        "rook" => Some(PieceType::Rook),
        "king" => Some(PieceType::King),
        "promoted_pawn" => Some(PieceType::PromotedPawn),
        "promoted_lance" => Some(PieceType::PromotedLance),
        "promoted_knight" => Some(PieceType::PromotedKnight),
        "promoted_silver" => Some(PieceType::PromotedSilver),
        "promoted_bishop" => Some(PieceType::PromotedBishop),
        "promoted_rook" => Some(PieceType::PromotedRook),
        _ => None,
    }
}

const DEFAULT_PRESET_PATH: &str = "config/pst/default.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PieceSquareTablePreset {
    Builtin,
    Default,
    Custom,
}

impl Default for PieceSquareTablePreset {
    fn default() -> Self {
        PieceSquareTablePreset::Builtin
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PieceSquareTableConfig {
    #[serde(default)]
    pub preset: PieceSquareTablePreset,
    #[serde(default)]
    pub values_path: Option<String>,
}

impl Default for PieceSquareTableConfig {
    fn default() -> Self {
        Self {
            preset: PieceSquareTablePreset::Builtin,
            values_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::piece_square_tables::PieceSquareTables;
    use crate::types::{PieceType, Player, Position};
    use serde_json::json;
    use std::io::Cursor;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn zero() -> [[i32; 9]; 9] {
        [[0; 9]; 9]
    }

    fn sample_document() -> serde_json::Value {
        let mut tables = serde_json::Map::new();
        for name in [
            "pawn",
            "lance",
            "knight",
            "silver",
            "gold",
            "bishop",
            "rook",
            "king",
            "promoted_pawn",
            "promoted_lance",
            "promoted_knight",
            "promoted_silver",
            "promoted_bishop",
            "promoted_rook",
        ] {
            tables.insert(
                name.to_string(),
                json!({
                    "mg": zero(),
                    "eg": zero(),
                }),
            );
        }

        json!({
            "version": "0.1",
            "description": "test document",
            "tables": tables,
        })
    }

    #[test]
    fn parse_piece_names() {
        assert_eq!(parse_piece_type("pawn"), Some(PieceType::Pawn));
        assert_eq!(
            parse_piece_type("Promoted_Rook"),
            Some(PieceType::PromotedRook)
        );
        assert_eq!(parse_piece_type("unknown"), None);
    }

    #[test]
    fn load_basic_tables_from_json() {
        let mut cursor = Cursor::new(sample_document().to_string().into_bytes());
        let result = PieceSquareTableLoader::from_reader(&mut cursor).unwrap();
        assert_eq!(result.version.as_deref(), Some("0.1"));
        assert_eq!(result.description.as_deref(), Some("test document"));

        let zero = zero();
        for piece in [
            PieceType::Pawn,
            PieceType::Lance,
            PieceType::Knight,
            PieceType::Silver,
            PieceType::Gold,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::PromotedPawn,
            PieceType::PromotedLance,
            PieceType::PromotedKnight,
            PieceType::PromotedSilver,
            PieceType::PromotedBishop,
            PieceType::PromotedRook,
        ] {
            let (mg, eg) = result.tables.get_tables(piece);
            assert_eq!(mg, &zero);
            assert_eq!(eg, &zero);
        }
    }

    #[test]
    fn load_custom_tables_from_path() {
        let mut file = NamedTempFile::new().expect("temp file");
        write!(file, "{}", sample_document().to_string()).expect("write sample document");

        let mut config = PieceSquareTableConfig::default();
        config.preset = PieceSquareTablePreset::Custom;
        config.values_path = Some(file.path().to_string_lossy().to_string());

        let tables = PieceSquareTableLoader::load(&config).expect("load custom tables");
        let origin = Position::new(0, 0);
        let value = tables.get_value(PieceType::Pawn, origin, Player::Black);
        assert_eq!(value.mg, 0);
        assert_eq!(value.eg, 0);
    }

    #[test]
    fn custom_preset_without_path_errors() {
        let mut config = PieceSquareTableConfig::default();
        config.preset = PieceSquareTablePreset::Custom;
        config.values_path = None;

        let err = PieceSquareTableLoader::load(&config).unwrap_err();
        assert!(matches!(err, PieceSquareTableLoadError::MissingCustomPath));
    }

    #[test]
    fn missing_piece_errors() {
        let mut doc = sample_document();
        doc.get_mut("tables")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove("lance");

        let mut cursor = Cursor::new(doc.to_string().into_bytes());
        let err = PieceSquareTableLoader::from_reader(&mut cursor).unwrap_err();
        match err {
            PieceSquareTableLoadError::MissingPiece(piece) => assert_eq!(piece, PieceType::Lance),
            other => panic!("expected missing piece error, got {other:?}"),
        }
    }

    #[test]
    fn king_tables_must_be_zero() {
        let mut doc = sample_document();
        let table = doc
            .get_mut("tables")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .get_mut("king")
            .unwrap()
            .as_object_mut()
            .unwrap();
        table.insert("mg".into(), json!(vec![vec![1; 9]; 9]));

        let mut cursor = Cursor::new(doc.to_string().into_bytes());
        let err = PieceSquareTableLoader::from_reader(&mut cursor).unwrap_err();
        assert!(matches!(err, PieceSquareTableLoadError::NonZeroKingTables));
    }

    #[test]
    #[ignore]
    fn print_default_document() {
        let raw = PieceSquareTables::new().to_raw();
        let mut map = serde_json::Map::new();

        let mut insert = |name: &str, mg: &[[i32; 9]; 9], eg: &[[i32; 9]; 9]| {
            map.insert(name.to_string(), json!({ "mg": mg, "eg": eg }));
        };

        insert("pawn", &raw.pawn_table_mg, &raw.pawn_table_eg);
        insert("lance", &raw.lance_table_mg, &raw.lance_table_eg);
        insert("knight", &raw.knight_table_mg, &raw.knight_table_eg);
        insert("silver", &raw.silver_table_mg, &raw.silver_table_eg);
        insert("gold", &raw.gold_table_mg, &raw.gold_table_eg);
        insert("bishop", &raw.bishop_table_mg, &raw.bishop_table_eg);
        insert("rook", &raw.rook_table_mg, &raw.rook_table_eg);
        insert("king", &zero(), &zero());
        insert(
            "promoted_pawn",
            &raw.promoted_pawn_table_mg,
            &raw.promoted_pawn_table_eg,
        );
        insert(
            "promoted_lance",
            &raw.promoted_lance_table_mg,
            &raw.promoted_lance_table_eg,
        );
        insert(
            "promoted_knight",
            &raw.promoted_knight_table_mg,
            &raw.promoted_knight_table_eg,
        );
        insert(
            "promoted_silver",
            &raw.promoted_silver_table_mg,
            &raw.promoted_silver_table_eg,
        );
        insert(
            "promoted_bishop",
            &raw.promoted_bishop_table_mg,
            &raw.promoted_bishop_table_eg,
        );
        insert(
            "promoted_rook",
            &raw.promoted_rook_table_mg,
            &raw.promoted_rook_table_eg,
        );

        let document = json!({
            "version": "1.0.0",
            "description": "Default PST set matching in-tree constants",
            "tables": map,
        });

        println!("{}", serde_json::to_string_pretty(&document).unwrap());
    }
}
