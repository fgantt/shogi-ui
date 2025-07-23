import React from 'react';
import '../styles/shogi.css';

const PromotionModal = ({ onPromote, onCancel }) => {
  return (
    <div className="modal-overlay">
      <div className="promotion-modal">
        <h2>Promote Piece?</h2>
        <p>Do you want to promote this piece?</p>
        <button onClick={() => onPromote(true)}>Promote</button>
        <button onClick={() => onPromote(false)}>Don't Promote</button>
      </div>
    </div>
  );
};

export default PromotionModal;
