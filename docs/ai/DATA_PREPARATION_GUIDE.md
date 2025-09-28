# Data Preparation Guide

Comprehensive guide for preparing game databases for automated tuning.

## Table of Contents

1. [Data Sources](#data-sources)
2. [Data Formats](#data-formats)
3. [Data Quality](#data-quality)
4. [Data Processing](#data-processing)
5. [Data Filtering](#data-filtering)
6. [Data Validation](#data-validation)
7. [Data Storage](#data-storage)
8. [Best Practices](#best-practices)

## Data Sources

### Game Databases

#### Professional Games
- **Professional Shogi Association**: High-quality professional games
- **ShogiDB**: Comprehensive database of professional games
- **KifuDB**: Japanese professional game database
- **81Dojo**: Online platform with rated games

#### Amateur Games
- **Shogi Club 24**: Large collection of amateur games
- **Shogi Wars**: Popular online platform
- **Shogi.net**: Community games
- **Lishogi**: Open-source shogi platform

#### Tournament Games
- **Local tournaments**: Regional and national championships
- **Online tournaments**: Rated tournament games
- **Blitz games**: Fast time control games
- **Correspondence games**: Long time control games

### Data Acquisition

#### Downloading Games
```bash
# Download from ShogiDB
curl -o games.kif "http://shogidb2.com/games/latest/1000"

# Download from 81Dojo
wget "https://81dojo.com/api/games/export?format=kif&limit=10000"

# Download from Shogi Wars
curl -H "Authorization: Bearer TOKEN" "https://api.shogiwars.com/games"
```

#### Converting Formats
```bash
# Convert KIF to JSON
./target/release/tuner prepare-data --input games.kif --output games.json --format json

# Convert CSA to JSON
./target/release/tuner prepare-data --input games.csa --output games.json --format json

# Convert PGN to JSON
./target/release/tuner prepare-data --input games.pgn --output games.json --format json
```

## Data Formats

### Supported Formats

#### KIF Format (Japanese Shogi Format)
```
# KIF format example
開始日時：2023/01/01
終了日時：2023/01/01
棋戦：王位戦
戦型：角換わり
先手：羽生善治
後手：佐藤天彦
手合割：平手
先手の持駒：なし
後手の持駒：なし
  ９ ８ ７ ６ ５ ４ ３ ２ １
+---------------------------+
|v香v桂v銀v金v王v金v銀v桂v香|一
| ・v飛 ・ ・ ・ ・ ・v角 ・|二
|v歩v歩v歩v歩v歩v歩v歩v歩v歩|三
| ・ ・ ・ ・ ・ ・ ・ ・ ・|四
| ・ ・ ・ ・ ・ ・ ・ ・ ・|五
| ・ ・ ・ ・ ・ ・ ・ ・ ・|六
| 歩 歩 歩 歩 歩 歩 歩 歩 歩|七
| ・ 角 ・ ・ ・ ・ ・ 飛 ・|八
| 香 桂 銀 金 玉 金 銀 桂 香|九
+---------------------------+
先手番
1 ７六歩(77)
2 ３四歩(33)
...
```

#### CSA Format (Computer Shogi Association)
```
V2.2
N+Player1
N-Player2
$EVENT:Tournament
$SITE:Online
$TIME:2023-01-01T10:00:00+09:00
P1-KY-KE-GI-KI-OU-KI-GI-KE-KY
P2 * -HI * * * * * -KA *
P3-FU-FU-FU-FU-FU-FU-FU-FU-FU
P4 * * * * * * * * *
P5 * * * * * * * * *
P6 * * * * * * * * *
P7+FU+FU+FU+FU+FU+FU+FU+FU+FU
P8 * +KA * * * * * +HI *
P9+KY+KE+GI+KI+OU+KI+GI+KE+KY
+
+7776FU
-3334FU
...
```

#### PGN Format (Portable Game Notation)
```
[Event "Tournament"]
[Site "Online"]
[Date "2023.01.01"]
[Round "1"]
[White "Player1"]
[Black "Player2"]
[Result "1-0"]
[TimeControl "30+0"]
[Time "10:00:00"]
[Termination "normal"]

1. P-76 P-34 2. P-26 P-84 3. P-25 P-85 4. P-24 P-86 5. P-23 P-87 6. P-22 P-88
7. P-21 P-89 8. P-17 P-93 9. P-16 P-94 10. P-15 P-95 11. P-14 P-96 12. P-13 P-97
13. P-12 P-98 14. P-11 P-99 15. P-27 P-83 16. P-28 P-82 17. P-29 P-81 18. P-37 P-73
19. P-38 P-72 20. P-39 P-71 21. P-47 P-63 22. P-48 P-62 23. P-49 P-61 24. P-57 P-53
25. P-58 P-52 26. P-59 P-51 27. P-67 P-43 28. P-68 P-42 29. P-69 P-41 30. P-77 P-33
31. P-78 P-32 32. P-79 P-31 33. P-87 P-23 34. P-88 P-22 35. P-89 P-21 36. P-93 P-17
37. P-94 P-16 38. P-95 P-15 39. P-96 P-14 40. P-97 P-13 41. P-98 P-12 42. P-99 P-11
43. P-83 P-27 44. P-82 P-28 45. P-81 P-29 46. P-73 P-37 47. P-72 P-38 48. P-71 P-39
49. P-63 P-47 50. P-62 P-48 51. P-61 P-49 52. P-53 P-57 53. P-52 P-58 54. P-51 P-59
55. P-43 P-67 56. P-42 P-68 57. P-41 P-69 58. P-33 P-77 59. P-32 P-78 60. P-31 P-79
61. P-23 P-87 62. P-22 P-88 63. P-21 P-89 64. P-17 P-93 65. P-16 P-94 66. P-15 P-95
67. P-14 P-96 68. P-13 P-97 69. P-12 P-98 70. P-11 P-99 71. P-27 P-83 72. P-28 P-82
73. P-29 P-81 74. P-37 P-73 75. P-38 P-72 76. P-39 P-71 77. P-47 P-63 78. P-48 P-62
79. P-49 P-61 80. P-57 P-53 81. P-52 P-58 82. P-51 P-59 83. P-43 P-67 84. P-42 P-68
85. P-41 P-69 86. P-33 P-77 87. P-32 P-78 88. P-31 P-79 89. P-23 P-87 90. P-22 P-88
91. P-21 P-89 92. P-17 P-93 93. P-16 P-94 94. P-15 P-95 95. P-14 P-96 96. P-13 P-97
97. P-12 P-98 98. P-11 P-99 99. P-27 P-83 100. P-28 P-82
1-0
```

#### JSON Format (Custom Structured Format)
```json
{
  "games": [
    {
      "id": "game_001",
      "white_player": "Player1",
      "black_player": "Player2",
      "white_rating": 2200,
      "black_rating": 2150,
      "result": "1-0",
      "time_control": "30+0",
      "date": "2023-01-01T10:00:00Z",
      "moves": [
        "7g7f", "3c3d", "2g2f", "8c8d", "2f2e", "8d8e", "2e2d", "8e8f", "2d2c", "8f8g"
      ],
      "positions": [
        {
          "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1",
          "move_number": 1,
          "player": "black",
          "move": "7g7f",
          "result": "1-0",
          "is_quiet": true
        }
      ]
    }
  ]
}
```

## Data Quality

### Quality Metrics

#### Game Quality Indicators
- **Player Ratings**: Higher ratings indicate better quality
- **Time Control**: Longer time controls allow for deeper thinking
- **Game Length**: Balanced game lengths (20-100 moves)
- **Result Distribution**: Balanced win/loss ratios
- **Completion Rate**: Games should be completed, not abandoned

#### Position Quality Indicators
- **Quiet Positions**: Positions without recent captures (better for evaluation)
- **Balanced Material**: Avoid extreme material imbalances
- **Game Phase**: Mix of opening, middlegame, and endgame positions
- **Tactical Complexity**: Avoid overly tactical positions

### Quality Assessment

#### Rating-Based Filtering
```bash
# Filter by player ratings
./target/release/tuner prepare-data \
  --input games.json \
  --output filtered_games.json \
  --min-rating 2000 \
  --max-rating 2800 \
  --rating-difference 200
```

#### Time Control Filtering
```bash
# Filter by time control
./target/release/tuner prepare-data \
  --input games.json \
  --output filtered_games.json \
  --min-time-control 300 \
  --max-time-control 3600 \
  --exclude-blitz
```

#### Game Length Filtering
```bash
# Filter by game length
./target/release/tuner prepare-data \
  --input games.json \
  --output filtered_games.json \
  --min-moves 20 \
  --max-moves 150 \
  --exclude-short-games \
  --exclude-long-games
```

## Data Processing

### Data Cleaning

#### Remove Invalid Games
```bash
# Remove games with invalid moves
./target/release/tuner prepare-data \
  --input games.json \
  --output clean_games.json \
  --remove-invalid-games \
  --validate-moves

# Remove games with incomplete data
./target/release/tuner prepare-data \
  --input games.json \
  --output clean_games.json \
  --remove-incomplete-games \
  --require-ratings \
  --require-time-control
```

#### Normalize Data
```bash
# Normalize player names
./target/release/tuner prepare-data \
  --input games.json \
  --output normalized_games.json \
  --normalize-names \
  --standardize-notation

# Normalize time controls
./target/release/tuner prepare-data \
  --input games.json \
  --output normalized_games.json \
  --normalize-time-controls \
  --convert-to-seconds
```

### Data Transformation

#### Convert Formats
```bash
# Convert between formats
./target/release/tuner prepare-data \
  --input games.kif \
  --output games.json \
  --format json \
  --include-metadata

# Convert to binary format for faster loading
./target/release/tuner prepare-data \
  --input games.json \
  --output games.bin \
  --format binary \
  --compress
```

#### Extract Positions
```bash
# Extract individual positions
./target/release/tuner prepare-data \
  --input games.json \
  --output positions.json \
  --extract-positions \
  --position-interval 5 \
  --include-moves
```

## Data Filtering

### Position Filtering

#### Quiet Position Filtering
```bash
# Keep only quiet positions
./target/release/tuner prepare-data \
  --input games.json \
  --output quiet_positions.json \
  --quiet-only \
  --quiet-threshold 3 \
  --exclude-tactical
```

#### Game Phase Filtering
```bash
# Filter by game phase
./target/release/tuner prepare-data \
  --input games.json \
  --output opening_positions.json \
  --phase opening \
  --min-move 1 \
  --max-move 20

./target/release/tuner prepare-data \
  --input games.json \
  --output middlegame_positions.json \
  --phase middlegame \
  --min-move 20 \
  --max-move 80

./target/release/tuner prepare-data \
  --input games.json \
  --output endgame_positions.json \
  --phase endgame \
  --min-move 80
```

#### Material Balance Filtering
```bash
# Filter by material balance
./target/release/tuner prepare-data \
  --input games.json \
  --output balanced_positions.json \
  --material-balance \
  --max-material-difference 5 \
  --exclude-extreme-positions
```

### Advanced Filtering

#### Statistical Filtering
```bash
# Remove statistical outliers
./target/release/tuner prepare-data \
  --input games.json \
  --output filtered_games.json \
  --remove-outliers \
  --outlier-threshold 3.0 \
  --statistical-filtering
```

#### Duplicate Removal
```bash
# Remove duplicate positions
./target/release/tuner prepare-data \
  --input games.json \
  --output unique_games.json \
  --deduplicate \
  --position-hashing \
  --similarity-threshold 0.95
```

#### Quality Scoring
```bash
# Score positions by quality
./target/release/tuner prepare-data \
  --input games.json \
  --output scored_games.json \
  --quality-scoring \
  --min-quality-score 0.7 \
  --weight-by-quality
```

## Data Validation

### Format Validation

#### Structure Validation
```bash
# Validate data structure
./target/release/tuner validate-data \
  --dataset games.json \
  --check-structure \
  --check-schema \
  --report-errors
```

#### Content Validation
```bash
# Validate game content
./target/release/tuner validate-data \
  --dataset games.json \
  --check-moves \
  --check-positions \
  --check-results \
  --validate-rules
```

### Quality Validation

#### Statistical Validation
```bash
# Validate data statistics
./target/release/tuner validate-data \
  --dataset games.json \
  --check-statistics \
  --rating-distribution \
  --result-distribution \
  --game-length-distribution
```

#### Consistency Validation
```bash
# Validate data consistency
./target/release/tuner validate-data \
  --dataset games.json \
  --check-consistency \
  --player-ratings \
  --time-controls \
  --game-results
```

## Data Storage

### Storage Formats

#### JSON Storage
```bash
# Store as JSON (human-readable)
./target/release/tuner prepare-data \
  --input games.kif \
  --output games.json \
  --format json \
  --pretty-print \
  --compress
```

#### Binary Storage
```bash
# Store as binary (fast loading)
./target/release/tuner prepare-data \
  --input games.json \
  --output games.bin \
  --format binary \
  --compress \
  --optimize
```

#### Compressed Storage
```bash
# Store with compression
./target/release/tuner prepare-data \
  --input games.json \
  --output games.json.gz \
  --compress gzip \
  --compression-level 9
```

### Storage Optimization

#### Indexing
```bash
# Create indices for fast access
./target/release/tuner prepare-data \
  --input games.json \
  --output indexed_games.json \
  --create-indices \
  --index-by-rating \
  --index-by-date \
  --index-by-player
```

#### Partitioning
```bash
# Partition data by criteria
./target/release/tuner prepare-data \
  --input games.json \
  --output-partition-dir games_partitions/ \
  --partition-by-rating \
  --partition-size 10000
```

## Best Practices

### Data Collection

1. **Source Diversity**: Use multiple sources for robust training
2. **Quality over Quantity**: Prefer high-quality games over large quantities
3. **Recent Data**: Include recent games for current playing styles
4. **Balanced Data**: Ensure balanced representation of different aspects

### Data Processing

1. **Incremental Processing**: Process data in chunks for large datasets
2. **Validation**: Always validate data after processing
3. **Backup**: Keep backups of original and processed data
4. **Documentation**: Document processing steps and parameters

### Data Storage

1. **Format Choice**: Use appropriate format for intended use
2. **Compression**: Use compression for storage efficiency
3. **Indexing**: Create indices for frequently accessed data
4. **Versioning**: Version control for processed datasets

### Quality Assurance

1. **Automated Validation**: Use automated validation tools
2. **Manual Inspection**: Manually inspect sample data
3. **Statistical Analysis**: Analyze data statistics for anomalies
4. **Continuous Monitoring**: Monitor data quality over time

## Examples

### Complete Data Preparation Pipeline

```bash
# 1. Download and convert data
wget "http://example.com/games.kif"
./target/release/tuner prepare-data --input games.kif --output games.json --format json

# 2. Clean and filter data
./target/release/tuner prepare-data \
  --input games.json \
  --output clean_games.json \
  --remove-invalid-games \
  --min-rating 2000 \
  --max-rating 2800 \
  --min-moves 20 \
  --max-moves 150 \
  --quiet-only \
  --quiet-threshold 3

# 3. Validate data
./target/release/tuner validate-data \
  --dataset clean_games.json \
  --check-structure \
  --check-moves \
  --check-statistics

# 4. Optimize for training
./target/release/tuner prepare-data \
  --input clean_games.json \
  --output training_data.json \
  --extract-positions \
  --position-interval 5 \
  --deduplicate \
  --compress

# 5. Create train/validation split
./target/release/tuner prepare-data \
  --input training_data.json \
  --output-train train_data.json \
  --output-validation validation_data.json \
  --validation-split 0.2 \
  --stratified-split
```

### High-Quality Dataset Creation

```bash
# Create high-quality dataset for production tuning
./target/release/tuner prepare-data \
  --input professional_games.json \
  --output production_data.json \
  --min-rating 2400 \
  --min-time-control 600 \
  --quiet-only \
  --quiet-threshold 4 \
  --material-balance \
  --max-material-difference 3 \
  --quality-scoring \
  --min-quality-score 0.8 \
  --deduplicate \
  --compress \
  --create-indices
```

## Next Steps

- [User Guide](USER_GUIDE.md) for usage instructions
- [Optimization Examples](OPTIMIZATION_EXAMPLES.md) for configuration examples
- [Performance Tuning Guide](PERFORMANCE_TUNING_GUIDE.md) for optimization tips
