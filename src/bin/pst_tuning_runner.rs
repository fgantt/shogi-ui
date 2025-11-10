use clap::Parser;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const PIECE_NAMES: &[&str] = &[
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
];

#[derive(Parser, Debug)]
#[command(
    name = "pst-tuning-runner",
    about = "Convert tuner CSV outputs into PST loader JSON files"
)]
struct Args {
    /// Directory containing <piece>_mg.csv and <piece>_eg.csv files
    #[arg(long)]
    input: PathBuf,

    /// Output JSON file compatible with the PST loader
    #[arg(long)]
    output: PathBuf,

    /// Semantic version attached to the exported PST set
    #[arg(long, default_value = "0.1.0")]
    version: String,

    /// Optional description embedded in the JSON header
    #[arg(long)]
    description: Option<String>,
}

#[derive(Serialize)]
struct PieceTable {
    mg: Vec<Vec<i32>>,
    eg: Vec<Vec<i32>>,
}

#[derive(Serialize)]
struct ExportDocument {
    version: String,
    description: Option<String>,
    tables: BTreeMap<String, PieceTable>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder().format_timestamp(None).init();
    let args = Args::parse();

    let mut tables = BTreeMap::new();
    for piece in PIECE_NAMES {
        let mg_path = args.input.join(format!("{}_mg.csv", piece));
        let eg_path = args.input.join(format!("{}_eg.csv", piece));

        let mg =
            parse_grid(&mg_path).map_err(|err| format!("{} (file: {})", err, mg_path.display()))?;
        let eg =
            parse_grid(&eg_path).map_err(|err| format!("{} (file: {})", err, eg_path.display()))?;

        tables.insert(piece.to_string(), PieceTable { mg, eg });
    }

    let export = ExportDocument {
        version: args.version,
        description: args.description,
        tables,
    };

    if let Some(parent) = args.output.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&export)?;
    fs::write(&args.output, json)?;

    println!("Wrote PST loader file to {}", args.output.display());
    Ok(())
}

fn parse_grid(path: &Path) -> Result<Vec<Vec<i32>>, String> {
    let contents = fs::read_to_string(path).map_err(|err| format!("failed to read file: {err}"))?;
    let mut rows = Vec::new();

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let cells: Vec<i32> = trimmed
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|token| !token.is_empty())
            .map(|token| {
                token
                    .parse::<i32>()
                    .map_err(|err| format!("failed to parse '{token}': {err}"))
            })
            .collect::<Result<_, _>>()?;

        if cells.len() != 9 {
            return Err(format!(
                "expected 9 numeric entries per row, found {}",
                cells.len()
            ));
        }

        rows.push(cells);
    }

    if rows.len() != 9 {
        return Err(format!(
            "expected 9 rows but found {} (hint: ensure the CSV is 9x9)",
            rows.len()
        ));
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn parse_grid_reads_basic_csv() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("pawn_mg.csv");
        let mut file = fs::File::create(&file_path).unwrap();

        for _ in 0..9 {
            writeln!(file, "0,1,2,3,4,5,6,7,8").unwrap();
        }

        let grid = parse_grid(&file_path).unwrap();
        assert_eq!(grid.len(), 9);
        assert_eq!(grid[0][0], 0);
        assert_eq!(grid[0][8], 8);
    }

    #[test]
    fn parse_grid_rejects_bad_dimensions() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("rook_mg.csv");
        let mut file = fs::File::create(&file_path).unwrap();

        for _ in 0..8 {
            writeln!(file, "0,0,0,0,0,0,0,0,0").unwrap();
        }

        let err = parse_grid(&file_path).unwrap_err();
        assert!(err.contains("expected 9 rows"), "{}", err);
    }
}
