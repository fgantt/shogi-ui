import React from 'react';

interface ClockProps {
  time: number; // in milliseconds
  isByoyomi: boolean;
}

const formatTime = (time: number) => {
  const totalSeconds = Math.floor(time / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  const pad = (num: number) => num.toString().padStart(2, '0');

  return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}`;
};

const Clock: React.FC<ClockProps> = ({ time, isByoyomi }) => {
  const clockStyle = {
    color: isByoyomi ? 'red' : 'goldenrod',
    fontFamily: 'monospace',
    fontSize: '1.5rem',
  };

  return (
    <div style={clockStyle}>
      {formatTime(time)}
    </div>
  );
};

export default Clock;
