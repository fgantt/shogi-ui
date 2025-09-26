#!/usr/bin/env python3
"""
Opening Book JSON to Binary Converter

This script converts the existing openingBook.json format to the new binary format.
It handles coordinate conversion, move weight assignment, and generates position evaluations.
"""

import json
import sys
import os
import argparse
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
from pathlib import Path

# Add the src directory to the path to import Rust types
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'src'))

@dataclass
class JsonMove:
    """Represents a move from the JSON format"""
    from_pos: str
    to_pos: str
    promote: bool = False
    piece_type: str = ""

@dataclass
class JsonOpening:
    """Represents an opening from the JSON format"""
    name: str
    moves: Dict[str, List[JsonMove]]

@dataclass
class ConvertedMove:
    """Represents a converted move with enhanced metadata"""
    from_row: int
    from_col: int
    to_row: int
    to_col: int
    piece_type: str
    is_drop: bool
    is_promotion: bool
    weight: int
    evaluation: int
    opening_name: str
    move_notation: str

@dataclass
class ConvertedPosition:
    """Represents a converted position with moves"""
    fen: str
    moves: List[ConvertedMove]

class OpeningBookConverter:
    """Converts JSON opening book to binary format"""
    
    def __init__(self):
        self.piece_type_mapping = {
            "P": "Pawn", "L": "Lance", "N": "Knight", "S": "Silver",
            "G": "Gold", "B": "Bishop", "R": "Rook", "K": "King",
            "+P": "PromotedPawn", "+L": "PromotedLance", "+N": "PromotedKnight",
            "+S": "PromotedSilver", "+B": "PromotedBishop", "+R": "PromotedRook"
        }
        
        # Move frequency weights based on common opening patterns
        self.opening_weights = {
            "Aggressive Rook": 850,
            "Yagura": 800,
            "Kakugawari (Bishop Exchange)": 750,
            "Shikenbisya (Fourth File Rook)": 700,
            "Aigakari (Double Wing Attack)": 650,
            "Side Pawn Picker (Yokofudori)": 600
        }
        
        # Base evaluation scores for different move types
        self.evaluation_scores = {
            "development": 15,      # Moving pieces to better positions
            "central_control": 20,  # Controlling center squares
            "king_safety": 25,      # Improving king safety
            "tactical": 30,         # Tactical moves (captures, threats)
            "positional": 10,       # General positional improvements
            "neutral": 0            # Neutral moves
        }

    def convert_coordinate(self, coord: str) -> Tuple[int, int]:
        """
        Convert string coordinate ("27") to (row, col) tuple.
        Format: "27" where first digit is col (1-9), second digit is row (1-9)
        Returns: (row, col) where both are 0-based indices
        """
        if len(coord) != 2:
            raise ValueError(f"Invalid coordinate format: {coord}")
        
        col = int(coord[0]) - 1  # Convert 1-9 to 0-8
        row = int(coord[1]) - 1  # Convert 1-9 to 0-8
        
        if not (0 <= row <= 8) or not (0 <= col <= 8):
            raise ValueError(f"Coordinate out of bounds: {coord}")
        
        return (row, col)

    def determine_piece_type(self, move: JsonMove, fen: str, opening_name: str) -> str:
        """
        Determine the piece type for a move based on context.
        This is a simplified heuristic - in a real implementation,
        you would analyze the board position.
        """
        # For now, use heuristics based on opening patterns
        if opening_name == "Aggressive Rook":
            if "27" in move.from_pos or "26" in move.from_pos or "25" in move.from_pos:
                return "Rook"
        elif opening_name == "Yagura":
            if "77" in move.from_pos or "76" in move.from_pos:
                return "Pawn"
            elif "69" in move.from_pos or "78" in move.to_pos:
                return "Gold"
        elif opening_name == "Kakugawari (Bishop Exchange)":
            if "22" in move.from_pos or "88" in move.to_pos:
                return "Bishop"
        
        # Default to Pawn for most moves
        return "Pawn"

    def determine_move_type(self, move: JsonMove, opening_name: str) -> str:
        """Determine the type of move for evaluation purposes"""
        # Check for drop moves
        if move.from_pos == "drop":
            return "tactical"
        
        # Check for promotion
        if move.promote:
            return "tactical"
        
        # Check for central moves
        central_squares = ["44", "45", "54", "55"]
        if move.to_pos in central_squares:
            return "central_control"
        
        # Check for king safety moves
        king_safety_moves = ["77", "78", "87", "88"]
        if move.to_pos in king_safety_moves:
            return "king_safety"
        
        # Check for development moves
        if opening_name in ["Yagura", "Aggressive Rook"]:
            return "development"
        
        return "positional"

    def assign_weight(self, move: JsonMove, opening_name: str) -> int:
        """Assign weight to a move based on opening and move characteristics"""
        base_weight = self.opening_weights.get(opening_name, 500)
        
        # Adjust weight based on move characteristics
        if move.promote:
            base_weight += 100
        elif move.from_pos == "drop":
            base_weight += 50
        
        # Adjust for move frequency in opening
        if opening_name == "Aggressive Rook":
            if move.from_pos in ["27", "26", "25"]:
                base_weight += 50
        elif opening_name == "Yagura":
            if move.from_pos in ["77", "69"]:
                base_weight += 50
        
        return min(base_weight, 1000)  # Cap at 1000

    def assign_evaluation(self, move: JsonMove, opening_name: str) -> int:
        """Assign evaluation score to a move"""
        move_type = self.determine_move_type(move, opening_name)
        base_eval = self.evaluation_scores.get(move_type, 0)
        
        # Adjust based on opening
        if opening_name in ["Aggressive Rook", "Yagura"]:
            base_eval += 5
        elif opening_name in ["Kakugawari (Bishop Exchange)"]:
            base_eval += 10
        
        # Adjust for promotion
        if move.promote:
            base_eval += 15
        
        return base_eval

    def generate_move_notation(self, move: JsonMove, opening_name: str) -> str:
        """Generate USI-style move notation"""
        if move.from_pos == "drop":
            piece = move.piece_type[0] if move.piece_type else "P"
            to_coord = f"{int(move.to_pos[0])}{chr(ord('a') + int(move.to_pos[1]) - 1)}"
            return f"{piece}*{to_coord}"
        else:
            from_coord = f"{int(move.from_pos[0])}{chr(ord('a') + int(move.from_pos[1]) - 1)}"
            to_coord = f"{int(move.to_pos[0])}{chr(ord('a') + int(move.to_pos[1]) - 1)}"
            notation = f"{from_coord}{to_coord}"
            if move.promote:
                notation += "+"
            return notation

    def convert_move(self, move: JsonMove, fen: str, opening_name: str) -> ConvertedMove:
        """Convert a JSON move to the new format"""
        # Handle drop moves
        if move.from_pos == "drop":
            to_row, to_col = self.convert_coordinate(move.to_pos)
            piece_type = self.determine_piece_type(move, fen, opening_name)
            return ConvertedMove(
                from_row=0, from_col=0,  # Will be None in final format
                to_row=to_row, to_col=to_col,
                piece_type=piece_type,
                is_drop=True,
                is_promotion=move.promote,
                weight=self.assign_weight(move, opening_name),
                evaluation=self.assign_evaluation(move, opening_name),
                opening_name=opening_name,
                move_notation=self.generate_move_notation(move, opening_name)
            )
        else:
            from_row, from_col = self.convert_coordinate(move.from_pos)
            to_row, to_col = self.convert_coordinate(move.to_pos)
            piece_type = self.determine_piece_type(move, fen, opening_name)
            return ConvertedMove(
                from_row=from_row, from_col=from_col,
                to_row=to_row, to_col=to_col,
                piece_type=piece_type,
                is_drop=False,
                is_promotion=move.promote,
                weight=self.assign_weight(move, opening_name),
                evaluation=self.assign_evaluation(move, opening_name),
                opening_name=opening_name,
                move_notation=self.generate_move_notation(move, opening_name)
            )

    def convert_opening(self, opening: JsonOpening) -> List[ConvertedPosition]:
        """Convert a JSON opening to a list of converted positions"""
        positions = []
        
        for fen, moves in opening.moves.items():
            converted_moves = []
            for move_data in moves:
                # Parse move data
                move = JsonMove(
                    from_pos=move_data.get("from", ""),
                    to_pos=move_data.get("to", ""),
                    promote=move_data.get("promote", False),
                    piece_type=move_data.get("pieceType", "")
                )
                
                converted_move = self.convert_move(move, fen, opening.name)
                converted_moves.append(converted_move)
            
            positions.append(ConvertedPosition(fen=fen, moves=converted_moves))
        
        return positions

    def convert_json_file(self, json_file: str) -> List[ConvertedPosition]:
        """Convert the entire JSON file"""
        with open(json_file, 'r', encoding='utf-8') as f:
            data = json.load(f)
        
        all_positions = []
        for opening_data in data:
            opening = JsonOpening(
                name=opening_data["name"],
                moves=opening_data["moves"]
            )
            positions = self.convert_opening(opening)
            all_positions.extend(positions)
        
        return all_positions

    def generate_rust_code(self, positions: List[ConvertedPosition]) -> str:
        """Generate Rust code to create the opening book"""
        rust_code = []
        rust_code.append("use crate::opening_book::*;")
        rust_code.append("use crate::types::*;")
        rust_code.append("")
        rust_code.append("pub fn create_opening_book() -> OpeningBook {")
        rust_code.append("    let mut book = OpeningBook::new();")
        rust_code.append("")
        
        for pos in positions:
            rust_code.append(f"    // Position: {pos.fen}")
            rust_code.append("    let moves = vec![")
            
            for move_data in pos.moves:
                from_str = "None" if move_data.is_drop else f"Some(Position::new({move_data.from_row}, {move_data.from_col}))"
                rust_code.append(f"        BookMove::new_with_metadata(")
                rust_code.append(f"            {from_str},")
                rust_code.append(f"            Position::new({move_data.to_row}, {move_data.to_col}),")
                rust_code.append(f"            PieceType::{move_data.piece_type},")
                rust_code.append(f"            {str(move_data.is_drop).lower()},")
                rust_code.append(f"            {str(move_data.is_promotion).lower()},")
                rust_code.append(f"            {move_data.weight},")
                rust_code.append(f"            {move_data.evaluation},")
                rust_code.append(f"            Some(\"{move_data.opening_name}\".to_string()),")
                rust_code.append(f"            Some(\"{move_data.move_notation}\".to_string()),")
                rust_code.append(f"        ),")
            
            rust_code.append("    ];")
            rust_code.append(f"    book.add_position(\"{pos.fen}\".to_string(), moves);")
            rust_code.append("")
        
        rust_code.append("    book.mark_loaded()")
        rust_code.append("}")
        
        return "\n".join(rust_code)

    def generate_migration_report(self, positions: List[ConvertedPosition]) -> str:
        """Generate a migration report"""
        total_positions = len(positions)
        total_moves = sum(len(pos.moves) for pos in positions)
        
        opening_counts = {}
        piece_type_counts = {}
        weight_distribution = {"high": 0, "medium": 0, "low": 0}
        
        for pos in positions:
            for move_data in pos.moves:
                # Count openings
                opening_counts[move_data.opening_name] = opening_counts.get(move_data.opening_name, 0) + 1
                
                # Count piece types
                piece_type_counts[move_data.piece_type] = piece_type_counts.get(move_data.piece_type, 0) + 1
                
                # Count weight distribution
                if move_data.weight >= 800:
                    weight_distribution["high"] += 1
                elif move_data.weight >= 500:
                    weight_distribution["medium"] += 1
                else:
                    weight_distribution["low"] += 1
        
        report = []
        report.append("=== Opening Book Migration Report ===")
        report.append(f"Total Positions: {total_positions}")
        report.append(f"Total Moves: {total_moves}")
        report.append("")
        report.append("Opening Distribution:")
        for opening, count in sorted(opening_counts.items(), key=lambda x: x[1], reverse=True):
            report.append(f"  {opening}: {count} moves")
        report.append("")
        report.append("Piece Type Distribution:")
        for piece_type, count in sorted(piece_type_counts.items(), key=lambda x: x[1], reverse=True):
            report.append(f"  {piece_type}: {count} moves")
        report.append("")
        report.append("Weight Distribution:")
        report.append(f"  High (800+): {weight_distribution['high']} moves")
        report.append(f"  Medium (500-799): {weight_distribution['medium']} moves")
        report.append(f"  Low (<500): {weight_distribution['low']} moves")
        
        return "\n".join(report)

def main():
    parser = argparse.ArgumentParser(description="Convert opening book JSON to binary format")
    parser.add_argument("input_file", help="Input JSON file")
    parser.add_argument("-o", "--output", help="Output directory for generated files")
    parser.add_argument("-r", "--report", help="Generate migration report", action="store_true")
    parser.add_argument("--rust-code", help="Generate Rust code file", action="store_true")
    
    args = parser.parse_args()
    
    if not os.path.exists(args.input_file):
        print(f"Error: Input file {args.input_file} not found")
        sys.exit(1)
    
    output_dir = args.output or "."
    os.makedirs(output_dir, exist_ok=True)
    
    converter = OpeningBookConverter()
    
    try:
        print("Converting opening book...")
        positions = converter.convert_json_file(args.input_file)
        print(f"Converted {len(positions)} positions")
        
        if args.report:
            report = converter.generate_migration_report(positions)
            report_file = os.path.join(output_dir, "migration_report.txt")
            with open(report_file, 'w', encoding='utf-8') as f:
                f.write(report)
            print(f"Migration report saved to {report_file}")
        
        if args.rust_code:
            rust_code = converter.generate_rust_code(positions)
            rust_file = os.path.join(output_dir, "opening_book_generated.rs")
            with open(rust_file, 'w', encoding='utf-8') as f:
                f.write(rust_code)
            print(f"Rust code saved to {rust_file}")
        
        print("Conversion completed successfully!")
        
    except Exception as e:
        print(f"Error during conversion: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()





