import React from 'react';
import { PieceType } from 'tsshogi';
import { getSvgPathForPiece, isSvgTheme } from '../utils/pieceThemes';
import { KANJI_MAP, ENGLISH_MAP } from '../utils/pieceMaps';
import SvgPiece from './SvgPiece';

interface PiecePreviewProps {
  theme: string;
}

const PiecePreview: React.FC<PiecePreviewProps> = ({ theme }) => {
  // Define piece pairs: [base piece, promoted piece]
  const piecePairs: Array<[PieceType, PieceType]> = [
    ['pawn', 'promPawn'],
    ['lance', 'promLance'],
    ['knight', 'promKnight'],
    ['silver', 'promSilver'],
    ['gold', 'gold'], // Gold doesn't promote
    ['bishop', 'horse'],
    ['rook', 'dragon'],
    ['king', 'king'], // King doesn't promote
  ];

  const renderSvgPiece = (pieceType: PieceType) => {
    const svgPath = getSvgPathForPiece(pieceType, 'player1', theme);
    return (
      <img
        src={svgPath}
        alt={`${pieceType} piece`}
        style={{ width: '56px', height: '56px' }}
        onError={(e) => {
          console.warn(`Failed to load piece image: ${svgPath}`);
          e.currentTarget.style.display = 'none';
        }}
      />
    );
  };

  const renderSvgPieceWhite = (pieceType: PieceType) => {
    const svgPath = getSvgPathForPiece(pieceType, 'player2', theme);
    return (
      <img
        src={svgPath}
        alt={`${pieceType} piece (white)`}
        style={{ width: '56px', height: '56px' }}
        onError={(e) => {
          console.warn(`Failed to load piece image: ${svgPath}`);
          e.currentTarget.style.display = 'none';
        }}
      />
    );
  };

  const renderTextPiece = (pieceType: PieceType, isKanji: boolean) => {
    return (
      <div style={{ width: '56px', height: '56px' }}>
        <SvgPiece
          type={pieceType}
          player="player1"
          pieceThemeType={theme}
        />
      </div>
    );
  };

  const isSvg = isSvgTheme(theme);
  const isKanji = theme === 'kanji';

  return (
    <div style={{ marginTop: '16px' }}>
      <h4 style={{ marginBottom: '12px', fontSize: '14px', color: '#666' }}>
        Piece Preview
      </h4>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '4px' }}>
        {piecePairs.map(([basePiece, promotedPiece]) => (
          <div key={basePiece} style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '2px' }}>
            <div style={{ display: 'flex', gap: '2px', alignItems: 'center' }}>
              {isSvg ? (
                renderSvgPiece(basePiece)
              ) : (
                renderTextPiece(basePiece, isKanji)
              )}
              {basePiece !== promotedPiece && (
                <>
                  <span style={{ fontSize: '12px', color: '#999' }}>→</span>
                  {isSvg ? (
                    renderSvgPiece(promotedPiece)
                  ) : (
                    renderTextPiece(promotedPiece, isKanji)
                  )}
                </>
              )}
              {/* For SVG themes, show both black and white kings */}
              {isSvg && basePiece === 'king' && (
                <>
                  <span style={{ fontSize: '12px', color: '#999' }}>|</span>
                  {renderSvgPieceWhite(basePiece)}
                </>
              )}
            </div>
            <div style={{ fontSize: '8px', color: '#999', textAlign: 'center' }}>
              {basePiece === 'pawn' && 'Pawn'}
              {basePiece === 'lance' && 'Lance'}
              {basePiece === 'knight' && 'Knight'}
              {basePiece === 'silver' && 'Silver'}
              {basePiece === 'gold' && 'Gold'}
              {basePiece === 'bishop' && 'Bishop'}
              {basePiece === 'rook' && 'Rook'}
              {basePiece === 'king' && 'King'}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default PiecePreview;
