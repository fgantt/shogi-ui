# Promotion Matching Exercise

## Overview
I've successfully implemented a new practice exercise called "Promotion Matching" that helps players learn which Shogi pieces can be promoted and what they become.

## What It Does
The exercise presents players with a Shogi piece (either unpromoted or promoted) and asks them to:
1. **For unpromoted pieces**: Identify what the piece becomes when promoted
2. **For promoted pieces**: Identify which piece promoted to create it

## Exercise Features
- **10 randomized questions** from a pool of promotion scenarios
- **Visual piece display** using the existing SvgPiece component
- **Visual answer options** showing piece diagrams at the same size as the question piece
- **Clean, uncluttered interface** with no text labels on answer options
- **Detailed explanations** after each answer explaining the promotion
- **Progressive scoring** to track performance

## Question Types

### Forward Promotion (Unpromoted → Promoted)
- Pawn (歩兵) → Promoted Pawn (と金)
- Knight (桂馬) → Promoted Knight (成桂)
- Lance (香車) → Promoted Lance (成香)
- Silver (銀将) → Promoted Silver (成銀)
- Rook (飛車) → Promoted Rook (龍王)
- Bishop (角行) → Promoted Bishop (龍馬)

### Reverse Promotion (Promoted → Unpromoted)
- Promoted Pawn (と金) ← Pawn (歩兵)
- Promoted Rook (龍王) ← Rook (飛車)
- Promoted Bishop (龍馬) ← Bishop (角行)
- Promoted Silver (成銀) ← Silver (銀将)

## How It Works

- **Question Display**: Shows the piece to be matched (either unpromoted or promoted)
- **Answer Options**: Each option displays only the piece diagram (80px size, same as question piece)
- **Pure Visual Matching**: Players match pieces by appearance only, with no text distractions

## Implementation Details

### Files Modified
1. **`src/components/PracticePage.jsx`** - Added new exercise to the list
2. **`src/components/PracticeExerciseDetail.jsx`** - Added exercise logic and questions
3. **`src/components/PracticeExerciseDetail.css`** - Added styling for explanations

### Key Features
- **Randomized question selection** from a pool of 10 promotion scenarios
- **Bidirectional learning** - both forward and reverse promotion matching
- **Pure visual matching** - answer options show only piece diagrams at consistent size
- **Educational explanations** that teach players about piece promotions
- **Consistent UI** that matches the existing practice exercises
- **Responsive design** that works on all screen sizes

## How to Use
1. Navigate to the Practice page
2. Select "Promotion Matching" from the exercise list
3. Answer 10 questions about piece promotions
4. Review explanations to learn the rules
5. Complete the exercise and see your final score

## Educational Value
This exercise helps players:
- **Memorize promotion rules** for all Shogi pieces
- **Understand the relationship** between unpromoted and promoted pieces
- **Learn Japanese terminology** for promoted pieces
- **Build visual recognition skills** by matching piece appearances
- **Build confidence** in piece recognition and promotion mechanics

## Technical Implementation
The exercise uses the existing piece constants and SvgPiece component, ensuring consistency with the rest of the application. Questions are dynamically generated and randomized, providing a fresh experience each time the exercise is taken.
