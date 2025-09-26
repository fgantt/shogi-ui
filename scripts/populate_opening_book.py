#!/usr/bin/env python3
"""
Script to populate the opening book with popular Shogi openings.
This creates a comprehensive opening book with well-known opening strategies.
"""

import json
import sys
from typing import Dict, List, Any

def create_opening_book() -> Dict[str, Any]:
    """Create a comprehensive opening book with popular Shogi openings."""
    
    opening_book = {
        "positions": []
    }
    
    # Starting position
    starting_position = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1"
    
    # Popular opening moves from starting position
    starting_moves = [
        {
            "from": "7g",
            "to": "6g", 
            "piece": "pawn",
            "weight": 900,
            "evaluation": 30,
            "opening_name": "Yagura",
            "move_notation": "7g7f"
        },
        {
            "from": "3c",
            "to": "4c",
            "piece": "pawn", 
            "weight": 850,
            "evaluation": 25,
            "opening_name": "Yagura",
            "move_notation": "3c4c"
        },
        {
            "from": "2g",
            "to": "3g",
            "piece": "pawn",
            "weight": 800,
            "evaluation": 20,
            "opening_name": "Ranging Rook",
            "move_notation": "2g2f"
        },
        {
            "from": "8c",
            "to": "7c",
            "piece": "pawn",
            "weight": 750,
            "evaluation": 15,
            "opening_name": "Ranging Rook",
            "move_notation": "8c8d"
        },
        {
            "from": "6g",
            "to": "5g",
            "piece": "pawn",
            "weight": 700,
            "evaluation": 10,
            "opening_name": "Central Pawn",
            "move_notation": "6g6f"
        },
        {
            "from": "4c",
            "to": "5c",
            "piece": "pawn",
            "weight": 650,
            "evaluation": 5,
            "opening_name": "Central Pawn",
            "move_notation": "4c4d"
        }
    ]
    
    opening_book["positions"].append({
        "fen": starting_position,
        "moves": starting_moves
    })
    
    # Yagura Opening (Castle Building)
    yagura_positions = [
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 2",
            "moves": [
                {
                    "from": "3c",
                    "to": "4c",
                    "piece": "pawn",
                    "weight": 950,
                    "evaluation": 35,
                    "opening_name": "Yagura",
                    "move_notation": "3c4c"
                },
                {
                    "from": "2g",
                    "to": "3g",
                    "piece": "pawn",
                    "weight": 800,
                    "evaluation": 20,
                    "opening_name": "Yagura",
                    "move_notation": "2g2f"
                }
            ]
        },
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 3",
            "moves": [
                {
                    "from": "7g",
                    "to": "6g",
                    "piece": "pawn",
                    "weight": 900,
                    "evaluation": 30,
                    "opening_name": "Yagura",
                    "move_notation": "7g7f"
                },
                {
                    "from": "8g",
                    "to": "7g",
                    "piece": "silver",
                    "weight": 850,
                    "evaluation": 25,
                    "opening_name": "Yagura",
                    "move_notation": "8g7g"
                }
            ]
        }
    ]
    
    # Ranging Rook Opening
    ranging_rook_positions = [
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 2",
            "moves": [
                {
                    "from": "2g",
                    "to": "3g",
                    "piece": "pawn",
                    "weight": 900,
                    "evaluation": 30,
                    "opening_name": "Ranging Rook",
                    "move_notation": "2g2f"
                },
                {
                    "from": "8c",
                    "to": "7c",
                    "piece": "pawn",
                    "weight": 850,
                    "evaluation": 25,
                    "opening_name": "Ranging Rook",
                    "move_notation": "8c8d"
                }
            ]
        },
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 3",
            "moves": [
                {
                    "from": "2h",
                    "to": "3h",
                    "piece": "rook",
                    "weight": 950,
                    "evaluation": 40,
                    "opening_name": "Ranging Rook",
                    "move_notation": "2h3h"
                },
                {
                    "from": "8h",
                    "to": "7h",
                    "piece": "rook",
                    "weight": 900,
                    "evaluation": 35,
                    "opening_name": "Ranging Rook",
                    "move_notation": "8h7h"
                }
            ]
        }
    ]
    
    # Central Pawn Opening
    central_pawn_positions = [
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 2",
            "moves": [
                {
                    "from": "6g",
                    "to": "5g",
                    "piece": "pawn",
                    "weight": 900,
                    "evaluation": 30,
                    "opening_name": "Central Pawn",
                    "move_notation": "6g6f"
                },
                {
                    "from": "4c",
                    "to": "5c",
                    "piece": "pawn",
                    "weight": 850,
                    "evaluation": 25,
                    "opening_name": "Central Pawn",
                    "move_notation": "4c4d"
                }
            ]
        }
    ]
    
    # Bishop Exchange Opening
    bishop_exchange_positions = [
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 2",
            "moves": [
                {
                    "from": "2h",
                    "to": "3i",
                    "piece": "bishop",
                    "weight": 800,
                    "evaluation": 20,
                    "opening_name": "Bishop Exchange",
                    "move_notation": "2h3i"
                },
                {
                    "from": "8h",
                    "to": "7i",
                    "piece": "bishop",
                    "weight": 750,
                    "evaluation": 15,
                    "opening_name": "Bishop Exchange",
                    "move_notation": "8h7i"
                }
            ]
        }
    ]
    
    # Add all positions to the opening book
    opening_book["positions"].extend(yagura_positions)
    opening_book["positions"].extend(ranging_rook_positions)
    opening_book["positions"].extend(central_pawn_positions)
    opening_book["positions"].extend(bishop_exchange_positions)
    
    # Add more advanced positions for each opening
    add_advanced_yagura_positions(opening_book)
    add_advanced_ranging_rook_positions(opening_book)
    add_advanced_central_pawn_positions(opening_book)
    
    # Add additional popular openings
    add_quick_attack_positions(opening_book)
    add_anaguma_positions(opening_book)
    add_ibisha_positions(opening_book)
    add_ai_funibisha_positions(opening_book)
    add_side_pawn_positions(opening_book)
    
    return opening_book

def add_advanced_yagura_positions(opening_book: Dict[str, Any]) -> None:
    """Add advanced Yagura positions."""
    
    # Yagura with Silver advancement
    yagura_silver_positions = [
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 5",
            "moves": [
                {
                    "from": "8g",
                    "to": "7g",
                    "piece": "silver",
                    "weight": 950,
                    "evaluation": 40,
                    "opening_name": "Yagura",
                    "move_notation": "8g7g"
                },
                {
                    "from": "6g",
                    "to": "5g",
                    "piece": "pawn",
                    "weight": 900,
                    "evaluation": 35,
                    "opening_name": "Yagura",
                    "move_notation": "6g6f"
                }
            ]
        },
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 6",
            "moves": [
                {
                    "from": "7g",
                    "to": "6g",
                    "piece": "silver",
                    "weight": 900,
                    "evaluation": 35,
                    "opening_name": "Yagura",
                    "move_notation": "7g6g"
                },
                {
                    "from": "8g",
                    "to": "7g",
                    "piece": "gold",
                    "weight": 850,
                    "evaluation": 30,
                    "opening_name": "Yagura",
                    "move_notation": "8g7g"
                }
            ]
        }
    ]
    
    opening_book["positions"].extend(yagura_silver_positions)

def add_advanced_ranging_rook_positions(opening_book: Dict[str, Any]) -> None:
    """Add advanced Ranging Rook positions."""
    
    # Ranging Rook with Rook advancement
    ranging_rook_advanced = [
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 5",
            "moves": [
                {
                    "from": "2h",
                    "to": "3h",
                    "piece": "rook",
                    "weight": 950,
                    "evaluation": 40,
                    "opening_name": "Ranging Rook",
                    "move_notation": "2h3h"
                },
                {
                    "from": "8h",
                    "to": "7h",
                    "piece": "rook",
                    "weight": 900,
                    "evaluation": 35,
                    "opening_name": "Ranging Rook",
                    "move_notation": "8h7h"
                }
            ]
        },
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 6",
            "moves": [
                {
                    "from": "3h",
                    "to": "4h",
                    "piece": "rook",
                    "weight": 900,
                    "evaluation": 35,
                    "opening_name": "Ranging Rook",
                    "move_notation": "3h4h"
                },
                {
                    "from": "7h",
                    "to": "6h",
                    "piece": "rook",
                    "weight": 850,
                    "evaluation": 30,
                    "opening_name": "Ranging Rook",
                    "move_notation": "7h6h"
                }
            ]
        }
    ]
    
    opening_book["positions"].extend(ranging_rook_advanced)

def add_advanced_central_pawn_positions(opening_book: Dict[str, Any]) -> None:
    """Add advanced Central Pawn positions."""
    
    # Central Pawn with piece development
    central_pawn_advanced = [
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 5",
            "moves": [
                {
                    "from": "6g",
                    "to": "5g",
                    "piece": "pawn",
                    "weight": 950,
                    "evaluation": 40,
                    "opening_name": "Central Pawn",
                    "move_notation": "6g6f"
                },
                {
                    "from": "4c",
                    "to": "5c",
                    "piece": "pawn",
                    "weight": 900,
                    "evaluation": 35,
                    "opening_name": "Central Pawn",
                    "move_notation": "4c4d"
                }
            ]
        },
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 6",
            "moves": [
                {
                    "from": "5g",
                    "to": "4g",
                    "piece": "pawn",
                    "weight": 900,
                    "evaluation": 35,
                    "opening_name": "Central Pawn",
                    "move_notation": "5g4g"
                },
                {
                    "from": "5c",
                    "to": "4c",
                    "piece": "pawn",
                    "weight": 850,
                    "evaluation": 30,
                    "opening_name": "Central Pawn",
                    "move_notation": "5c4c"
                }
            ]
        }
    ]
    
    opening_book["positions"].extend(central_pawn_advanced)

def add_quick_attack_positions(opening_book: Dict[str, Any]) -> None:
    """Add Quick Attack (速攻) positions."""
    
    quick_attack_positions = [
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 2",
            "moves": [
                {
                    "from": "2g",
                    "to": "3g",
                    "piece": "pawn",
                    "weight": 900,
                    "evaluation": 35,
                    "opening_name": "Quick Attack",
                    "move_notation": "2g2f"
                },
                {
                    "from": "8c",
                    "to": "7c",
                    "piece": "pawn",
                    "weight": 850,
                    "evaluation": 30,
                    "opening_name": "Quick Attack",
                    "move_notation": "8c8d"
                }
            ]
        },
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 3",
            "moves": [
                {
                    "from": "2h",
                    "to": "2g",
                    "piece": "rook",
                    "weight": 950,
                    "evaluation": 40,
                    "opening_name": "Quick Attack",
                    "move_notation": "2h2g"
                },
                {
                    "from": "8h",
                    "to": "8g",
                    "piece": "rook",
                    "weight": 900,
                    "evaluation": 35,
                    "opening_name": "Quick Attack",
                    "move_notation": "8h8g"
                }
            ]
        }
    ]
    
    opening_book["positions"].extend(quick_attack_positions)

def add_anaguma_positions(opening_book: Dict[str, Any]) -> None:
    """Add Anaguma (穴熊) positions."""
    
    anaguma_positions = [
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 2",
            "moves": [
                {
                    "from": "7g",
                    "to": "6g",
                    "piece": "pawn",
                    "weight": 900,
                    "evaluation": 35,
                    "opening_name": "Anaguma",
                    "move_notation": "7g7f"
                },
                {
                    "from": "3c",
                    "to": "4c",
                    "piece": "pawn",
                    "weight": 850,
                    "evaluation": 30,
                    "opening_name": "Anaguma",
                    "move_notation": "3c4c"
                }
            ]
        },
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 3",
            "moves": [
                {
                    "from": "8g",
                    "to": "7g",
                    "piece": "silver",
                    "weight": 950,
                    "evaluation": 40,
                    "opening_name": "Anaguma",
                    "move_notation": "8g7g"
                },
                {
                    "from": "6g",
                    "to": "5g",
                    "piece": "pawn",
                    "weight": 900,
                    "evaluation": 35,
                    "opening_name": "Anaguma",
                    "move_notation": "6g6f"
                }
            ]
        }
    ]
    
    opening_book["positions"].extend(anaguma_positions)

def add_ibisha_positions(opening_book: Dict[str, Any]) -> None:
    """Add Ibisha (居飛車) positions."""
    
    ibisha_positions = [
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 2",
            "moves": [
                {
                    "from": "2g",
                    "to": "3g",
                    "piece": "pawn",
                    "weight": 900,
                    "evaluation": 35,
                    "opening_name": "Ibisha",
                    "move_notation": "2g2f"
                },
                {
                    "from": "8c",
                    "to": "7c",
                    "piece": "pawn",
                    "weight": 850,
                    "evaluation": 30,
                    "opening_name": "Ibisha",
                    "move_notation": "8c8d"
                }
            ]
        },
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 3",
            "moves": [
                {
                    "from": "2h",
                    "to": "2g",
                    "piece": "rook",
                    "weight": 950,
                    "evaluation": 40,
                    "opening_name": "Ibisha",
                    "move_notation": "2h2g"
                },
                {
                    "from": "8h",
                    "to": "8g",
                    "piece": "rook",
                    "weight": 900,
                    "evaluation": 35,
                    "opening_name": "Ibisha",
                    "move_notation": "8h8g"
                }
            ]
        }
    ]
    
    opening_book["positions"].extend(ibisha_positions)

def add_ai_funibisha_positions(opening_book: Dict[str, Any]) -> None:
    """Add Ai Funibisha (相振り飛車) positions."""
    
    ai_funibisha_positions = [
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 2",
            "moves": [
                {
                    "from": "2g",
                    "to": "3g",
                    "piece": "pawn",
                    "weight": 900,
                    "evaluation": 35,
                    "opening_name": "Ai Funibisha",
                    "move_notation": "2g2f"
                },
                {
                    "from": "8c",
                    "to": "7c",
                    "piece": "pawn",
                    "weight": 850,
                    "evaluation": 30,
                    "opening_name": "Ai Funibisha",
                    "move_notation": "8c8d"
                }
            ]
        },
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 3",
            "moves": [
                {
                    "from": "2h",
                    "to": "3h",
                    "piece": "rook",
                    "weight": 950,
                    "evaluation": 40,
                    "opening_name": "Ai Funibisha",
                    "move_notation": "2h3h"
                },
                {
                    "from": "8h",
                    "to": "7h",
                    "piece": "rook",
                    "weight": 900,
                    "evaluation": 35,
                    "opening_name": "Ai Funibisha",
                    "move_notation": "8h7h"
                }
            ]
        }
    ]
    
    opening_book["positions"].extend(ai_funibisha_positions)

def add_side_pawn_positions(opening_book: Dict[str, Any]) -> None:
    """Add Side Pawn (横歩取り) positions."""
    
    side_pawn_positions = [
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 2",
            "moves": [
                {
                    "from": "2g",
                    "to": "3g",
                    "piece": "pawn",
                    "weight": 900,
                    "evaluation": 35,
                    "opening_name": "Side Pawn",
                    "move_notation": "2g2f"
                },
                {
                    "from": "8c",
                    "to": "7c",
                    "piece": "pawn",
                    "weight": 850,
                    "evaluation": 30,
                    "opening_name": "Side Pawn",
                    "move_notation": "8c8d"
                }
            ]
        },
        {
            "fen": "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 3",
            "moves": [
                {
                    "from": "2h",
                    "to": "2g",
                    "piece": "rook",
                    "weight": 950,
                    "evaluation": 40,
                    "opening_name": "Side Pawn",
                    "move_notation": "2h2g"
                },
                {
                    "from": "8h",
                    "to": "8g",
                    "piece": "rook",
                    "weight": 900,
                    "evaluation": 35,
                    "opening_name": "Side Pawn",
                    "move_notation": "8h8g"
                }
            ]
        }
    ]
    
    opening_book["positions"].extend(side_pawn_positions)

def main():
    """Main function to generate the opening book."""
    print("Generating comprehensive Shogi opening book...")
    
    # Create the opening book
    opening_book = create_opening_book()
    
    # Write to file
    output_file = "src/ai/openingBook.json"
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(opening_book, f, indent=2, ensure_ascii=False)
    
    print(f"Opening book generated with {len(opening_book['positions'])} positions")
    print(f"Written to {output_file}")
    
    # Print summary
    total_moves = sum(len(pos['moves']) for pos in opening_book['positions'])
    print(f"Total moves: {total_moves}")
    
    # Count openings by type
    opening_counts = {}
    for pos in opening_book['positions']:
        for move in pos['moves']:
            opening_name = move.get('opening_name', 'Unknown')
            opening_counts[opening_name] = opening_counts.get(opening_name, 0) + 1
    
    print("\nOpening distribution:")
    for opening, count in sorted(opening_counts.items()):
        print(f"  {opening}: {count} moves")

if __name__ == "__main__":
    main()
