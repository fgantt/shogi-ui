#!/usr/bin/env python3
"""
Test script to demonstrate the opening book functionality.
This script shows how the opening book provides move suggestions for popular Shogi openings.
"""

import json
import sys
from typing import Dict, List, Any

def load_opening_book() -> Dict[str, Any]:
    """Load the opening book from JSON file."""
    try:
        with open('src/ai/openingBook.json', 'r', encoding='utf-8') as f:
            return json.load(f)
    except FileNotFoundError:
        print("Opening book not found. Please run populate_opening_book.py first.")
        sys.exit(1)

def find_position_moves(opening_book: Dict[str, Any], fen: str) -> List[Dict[str, Any]]:
    """Find moves for a specific position."""
    for position in opening_book['positions']:
        if position['fen'] == fen:
            return position['moves']
    return []

def display_moves(moves: List[Dict[str, Any]], position_name: str = "Position") -> None:
    """Display moves in a formatted way."""
    if not moves:
        print(f"{position_name}: No opening book moves available")
        return
    
    print(f"\n{position_name}:")
    print(f"FEN: {moves[0] if moves else 'N/A'}")
    print("Available moves:")
    
    for i, move in enumerate(moves, 1):
        print(f"  {i}. {move['move_notation']} ({move['piece']}) - {move['opening_name']}")
        print(f"     Weight: {move['weight']}, Evaluation: {move['evaluation']}")

def main():
    """Main function to test the opening book."""
    print("=== Shogi Opening Book Test ===\n")
    
    # Load opening book
    opening_book = load_opening_book()
    
    print(f"Opening book loaded with {len(opening_book['positions'])} positions")
    
    # Test positions
    test_positions = [
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1",
            "name": "Starting Position"
        },
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 2",
            "name": "After First Move"
        },
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 3",
            "name": "After Second Move"
        }
    ]
    
    # Test each position
    for test_pos in test_positions:
        moves = find_position_moves(opening_book, test_pos['fen'])
        display_moves(moves, test_pos['name'])
    
    # Show opening distribution
    print("\n=== Opening Distribution ===")
    opening_counts = {}
    for position in opening_book['positions']:
        for move in position['moves']:
            opening_name = move.get('opening_name', 'Unknown')
            opening_counts[opening_name] = opening_counts.get(opening_name, 0) + 1
    
    for opening, count in sorted(opening_counts.items()):
        print(f"  {opening}: {count} moves")
    
    # Show sample moves for each opening
    print("\n=== Sample Moves by Opening ===")
    opening_samples = {}
    for position in opening_book['positions']:
        for move in position['moves']:
            opening_name = move.get('opening_name', 'Unknown')
            if opening_name not in opening_samples:
                opening_samples[opening_name] = []
            if len(opening_samples[opening_name]) < 3:  # Show up to 3 moves per opening
                opening_samples[opening_name].append(move)
    
    for opening, moves in opening_samples.items():
        print(f"\n{opening}:")
        for move in moves:
            print(f"  - {move['move_notation']} ({move['piece']}) - Weight: {move['weight']}")

if __name__ == "__main__":
    main()
