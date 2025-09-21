import React, { useEffect, useState } from 'react';
import { PieceType as TsshogiPieceType } from 'tsshogi';
import SvgPiece from './SvgPiece';
import '../styles/shogi.css';
import { BoardRef } from './Board'; // Import BoardRef

interface PromotionModalProps {
  onPromote: (promote: boolean) => void;
  pieceType: string;
  player: 'player1' | 'player2';
  pieceThemeType: string;
  modalWidth: number;
  modalHeight: number;
  pieceSize: number;
  destinationSquareUsi: string;
  boardRef: React.RefObject<BoardRef | null>; // Changed type
  boardContainerRef: React.RefObject<HTMLDivElement | null>; // Added
}

const PromotionModal: React.FC<PromotionModalProps> = ({ onPromote, pieceType, player, pieceThemeType, modalWidth, modalHeight, pieceSize, destinationSquareUsi, boardRef, boardContainerRef }) => {
  console.log('PromotionModal: boardRef', boardRef);
  console.log('PromotionModal: boardContainerRef', boardContainerRef);
  const [modalStyle, setModalStyle] = useState({});

  useEffect(() => {
    if (boardRef.current && boardContainerRef.current) {
      const squareElement = boardRef.current.getSquareRef(destinationSquareUsi);
      if (squareElement) {
        const squareRect = squareElement.getBoundingClientRect();
        const boardRect = boardContainerRef.current.getBoundingClientRect(); // Use boardContainerRef

        // Calculate modal position relative to the square
        // Centered horizontally above the square
        const modalX = modalWidth * -0.25;
        const modalY = 0;

        setModalStyle({
          position: 'absolute',
          left: modalX,
          top: modalY,
          width: modalWidth,
          height: modalHeight,
          zIndex: 1001, // Ensure it's above other elements
          border: '2px solid black',
          padding: '0px',
          maxHeight: '100%',
        });
      }
    }
  }, [destinationSquareUsi, boardRef, boardContainerRef, modalWidth, modalHeight]);

  // Helper to get promoted piece type
  const getPromotedPieceType = (unpromotedType: string): string => {
    switch (unpromotedType) {
      case TsshogiPieceType.PAWN: return TsshogiPieceType.PROM_PAWN;
      case TsshogiPieceType.LANCE: return TsshogiPieceType.PROM_LANCE;
      case TsshogiPieceType.KNIGHT: return TsshogiPieceType.PROM_KNIGHT;
      case TsshogiPieceType.SILVER: return TsshogiPieceType.PROM_SILVER;
      case TsshogiPieceType.BISHOP: return TsshogiPieceType.HORSE;
      case TsshogiPieceType.ROOK: return TsshogiPieceType.DRAGON;
      default: return unpromotedType; // Should not happen for promotable pieces
    }
  };

  const promotedPieceType = getPromotedPieceType(pieceType);

  return (
    <div className="settings-panel promotion-modal-panel" style={{ ...modalStyle, width: modalWidth, height: modalHeight }}>
      <div className="promotion-options">
          <div className="promotion-option" onClick={() => onPromote(false)}>
            <SvgPiece type={pieceType} player={player} pieceThemeType={pieceThemeType} size={pieceSize} />
          </div>
          <div className="promotion-option" onClick={() => onPromote(true)}>
            <SvgPiece type={promotedPieceType} player={player} pieceThemeType={pieceThemeType} size={pieceSize} />
          </div>
        </div>
      </div>
  );
};

export default PromotionModal;