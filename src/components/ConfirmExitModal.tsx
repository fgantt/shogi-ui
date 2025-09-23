import React from 'react';
import './ConfirmExitModal.css';

interface ConfirmExitModalProps {
  isOpen: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

const ConfirmExitModal: React.FC<ConfirmExitModalProps> = ({ isOpen, onConfirm, onCancel }) => {
  console.log('ConfirmExitModal render - isOpen:', isOpen);
  
  if (!isOpen) return null;

  return (
    <div className="confirm-exit-modal-overlay">
      <div className="confirm-exit-modal">
        <div className="confirm-exit-modal-header">
          <h2>Exit Game</h2>
        </div>
        <div className="confirm-exit-modal-content">
          <div className="warning-icon">⚠️</div>
          <p>Are you sure you want to exit? Your current game will be lost.</p>
        </div>
        <div className="confirm-exit-modal-footer">
          <button className="cancel-button" onClick={onCancel}>
            Cancel
          </button>
          <button className="confirm-button" onClick={onConfirm}>
            Exit Game
          </button>
        </div>
      </div>
    </div>
  );
};

export default ConfirmExitModal;
