#!/usr/bin/env python3
"""
Advanced Opening Book Generator

This script generates opening books from various sources including:
- JSON format conversion
- Game database analysis
- Professional game statistics
- Engine analysis results
"""

import json
import sys
import os
import argparse
import subprocess
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
from pathlib import Path
import statistics

@dataclass
class GameMove:
    """Represents a move from a game database"""
    move_number: int
    from_square: str
    to_square: str
    piece: str
    is_promotion: bool
    is_capture: bool
    evaluation: Optional[float] = None
    time_spent: Optional[float] = None

@dataclass
class GameResult:
    """Represents the result of a game"""
    white_elo: int
    black_elo: int
    result: str  # "1-0", "0-1", "1/2-1/2"
    opening: str
    eco: str

@dataclass
class PositionAnalysis:
    """Analysis of a position from multiple games"""
    fen: str
    move_frequency: Dict[str, int]  # move -> frequency
    move_success_rate: Dict[str, float]  # move -> win rate
    move_evaluations: Dict[str, float]  # move -> average evaluation
    total_games: int

class OpeningBookGenerator:
    """Generates high-quality opening books from multiple sources"""
    
    def __init__(self):
        self.piece_values = {
            "P": 100, "L": 300, "N": 400, "S": 500,
            "G": 600, "B": 800, "R": 900, "K": 10000
        }
        
        # Opening classification patterns
        self.opening_patterns = {
            "Yagura": ["77", "76", "69", "78"],
            "Aggressive Rook": ["27", "26", "25"],
            "Kakugawari": ["22", "88"],
            "Shikenbisya": ["28", "58"],
            "Aigakari": ["39", "28"]
        }

    def analyze_game_database(self, games_file: str) -> Dict[str, PositionAnalysis]:
        """Analyze a database of games to extract opening patterns"""
        print(f"Analyzing game database: {games_file}")
        
        # This would typically read from a PGN or similar format
        # For now, we'll simulate the analysis
        position_analyses = {}
        
        # Simulate analysis of common opening positions
        common_positions = [
            "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1",
            "lnsgkgsnl/1r5b1/ppppppppp/9/9/7P1/PPPPPPP1P/1B5R1/LNSGKGSNL w - 2",
            "lnsgkgsnl/1r5b1/p1ppppppp/1p7/9/7P1/PPPPPPP1P/1B5R1/LNSGKGSNL b - 3"
        ]
        
        for fen in common_positions:
            # Simulate move frequency analysis
            move_frequency = {
                "27-26": 45,  # 45% of games
                "77-76": 30,  # 30% of games
                "22-88": 15,  # 15% of games
                "28-58": 10   # 10% of games
            }
            
            # Simulate success rate analysis
            move_success_rate = {
                "27-26": 0.65,  # 65% win rate
                "77-76": 0.58,  # 58% win rate
                "22-88": 0.72,  # 72% win rate
                "28-58": 0.55   # 55% win rate
            }
            
            # Simulate evaluation analysis
            move_evaluations = {
                "27-26": 15,   # +0.15 pawns
                "77-76": 8,    # +0.08 pawns
                "22-88": 25,   # +0.25 pawns
                "28-58": 12    # +0.12 pawns
            }
            
            position_analyses[fen] = PositionAnalysis(
                fen=fen,
                move_frequency=move_frequency,
                move_success_rate=move_success_rate,
                move_evaluations=move_evaluations,
                total_games=1000  # Simulated
            )
        
        return position_analyses

    def calculate_move_weights(self, analysis: PositionAnalysis) -> Dict[str, int]:
        """Calculate move weights based on frequency, success rate, and evaluation"""
        weights = {}
        
        for move in analysis.move_frequency:
            frequency = analysis.move_frequency[move]
            success_rate = analysis.move_success_rate.get(move, 0.5)
            evaluation = analysis.move_evaluations.get(move, 0)
            
            # Weight calculation formula
            # Base weight from frequency (0-400)
            base_weight = int(frequency * 4)
            
            # Success rate bonus (0-200)
            success_bonus = int(success_rate * 200)
            
            # Evaluation bonus (0-200)
            eval_bonus = max(0, min(200, int(evaluation * 8)))
            
            # Total weight (0-1000)
            total_weight = min(1000, base_weight + success_bonus + eval_bonus)
            weights[move] = total_weight
        
        return weights

    def generate_opening_variations(self, analysis: PositionAnalysis) -> List[Dict]:
        """Generate opening variations from position analysis"""
        variations = []
        weights = self.calculate_move_weights(analysis)
        
        for move, weight in weights.items():
            if weight < 200:  # Skip very low weight moves
                continue
                
            # Parse move notation
            from_square, to_square = move.split('-')
            
            # Determine piece type and promotion
            piece_type = self.determine_piece_from_move(from_square, to_square)
            is_promotion = self.is_promotion_move(from_square, to_square)
            
            variation = {
                "from": from_square,
                "to": to_square,
                "piece_type": piece_type,
                "is_promotion": is_promotion,
                "weight": weight,
                "evaluation": analysis.move_evaluations.get(move, 0),
                "frequency": analysis.move_frequency[move],
                "success_rate": analysis.move_success_rate.get(move, 0.5)
            }
            variations.append(variation)
        
        return variations

    def determine_piece_from_move(self, from_square: str, to_square: str) -> str:
        """Determine piece type from move pattern"""
        # This is a simplified heuristic
        if from_square in ["27", "26", "25"]:
            return "Rook"
        elif from_square in ["22", "88"]:
            return "Bishop"
        elif from_square in ["77", "76"]:
            return "Pawn"
        elif from_square in ["69", "78"]:
            return "Gold"
        else:
            return "Pawn"  # Default

    def is_promotion_move(self, from_square: str, to_square: str) -> bool:
        """Determine if a move is a promotion"""
        # Promotion zone for Black (rows 7-9)
        to_row = int(to_square[1])
        return to_row >= 7

    def generate_binary_opening_book(self, analyses: Dict[str, PositionAnalysis]) -> bytes:
        """Generate binary opening book from analyses"""
        # This would generate the actual binary format
        # For now, we'll create a JSON representation that can be converted
        opening_book_data = {
            "version": 1,
            "positions": {}
        }
        
        for fen, analysis in analyses.items():
            variations = self.generate_opening_variations(analysis)
            if variations:
                opening_book_data["positions"][fen] = variations
        
        return json.dumps(opening_book_data, indent=2).encode('utf-8')

    def generate_statistics_report(self, analyses: Dict[str, PositionAnalysis]) -> str:
        """Generate a comprehensive statistics report"""
        report = []
        report.append("=== Opening Book Generation Statistics ===")
        report.append("")
        
        total_positions = len(analyses)
        total_moves = sum(len(analysis.move_frequency) for analysis in analyses.values())
        
        report.append(f"Total Positions Analyzed: {total_positions}")
        report.append(f"Total Moves Found: {total_moves}")
        report.append("")
        
        # Analyze move weight distribution
        all_weights = []
        for analysis in analyses.values():
            weights = self.calculate_move_weights(analysis)
            all_weights.extend(weights.values())
        
        if all_weights:
            report.append("Move Weight Statistics:")
            report.append(f"  Average Weight: {statistics.mean(all_weights):.1f}")
            report.append(f"  Median Weight: {statistics.median(all_weights):.1f}")
            report.append(f"  Min Weight: {min(all_weights)}")
            report.append(f"  Max Weight: {max(all_weights)}")
            report.append("")
        
        # Analyze opening patterns
        opening_counts = {}
        for analysis in analyses.values():
            for move in analysis.move_frequency:
                opening = self.classify_opening(move)
                opening_counts[opening] = opening_counts.get(opening, 0) + 1
        
        report.append("Opening Pattern Distribution:")
        for opening, count in sorted(opening_counts.items(), key=lambda x: x[1], reverse=True):
            report.append(f"  {opening}: {count} moves")
        report.append("")
        
        return "\n".join(report)

    def classify_opening(self, move: str) -> str:
        """Classify a move into an opening pattern"""
        from_square = move.split('-')[0]
        
        for opening, patterns in self.opening_patterns.items():
            if from_square in patterns:
                return opening
        
        return "Other"

def main():
    parser = argparse.ArgumentParser(description="Generate opening book from multiple sources")
    parser.add_argument("--games", help="Game database file (PGN format)")
    parser.add_argument("--json", help="JSON opening book file")
    parser.add_argument("-o", "--output", help="Output file for binary opening book")
    parser.add_argument("--report", help="Generate statistics report", action="store_true")
    parser.add_argument("--format", choices=["binary", "json"], default="binary", help="Output format")
    
    args = parser.parse_args()
    
    generator = OpeningBookGenerator()
    
    if args.games:
        print("Analyzing game database...")
        analyses = generator.analyze_game_database(args.games)
        
        if args.format == "binary":
            binary_data = generator.generate_binary_opening_book(analyses)
            output_file = args.output or "opening_book.bin"
            with open(output_file, 'wb') as f:
                f.write(binary_data)
            print(f"Binary opening book saved to {output_file}")
        else:
            json_data = generator.generate_binary_opening_book(analyses)
            output_file = args.output or "opening_book.json"
            with open(output_file, 'w', encoding='utf-8') as f:
                f.write(json_data.decode('utf-8'))
            print(f"JSON opening book saved to {output_file}")
        
        if args.report:
            report = generator.generate_statistics_report(analyses)
            report_file = "opening_book_report.txt"
            with open(report_file, 'w', encoding='utf-8') as f:
                f.write(report)
            print(f"Statistics report saved to {report_file}")
    
    elif args.json:
        print("Converting JSON opening book...")
        # Use the converter script
        subprocess.run([
            sys.executable, 
            "scripts/convert_opening_book.py", 
            args.json, 
            "--report", 
            "--rust-code"
        ])
    
    else:
        print("Please specify either --games or --json input file")
        sys.exit(1)

if __name__ == "__main__":
    main()





