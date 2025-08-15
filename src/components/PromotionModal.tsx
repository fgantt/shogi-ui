import React from 'react';
import '../styles/shogi.css';

interface PromotionModalProps {
  onPromote: (promote: boolean) => void;
}

const PromotionModal: React.FC<PromotionModalProps> = ({ onPromote }) => {
  return (
    <div className="settings-overlay">
      <div className="settings-panel">
        <h2>Promote Piece?</h2>
        <p>Do you want to promote this piece?</p>
        <div className="promotion-modal-buttons">
          <button onClick={() => onPromote(true)}>Promote</button>
          <button onClick={() => onPromote(false)}>Don't Promote</button>
        </div>
      </div>
    </div>
  );
};

export default PromotionModal;
