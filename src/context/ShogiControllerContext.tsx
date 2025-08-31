
import React, { createContext, useContext } from 'react';
import { ShogiController } from '../usi/controller';

const ShogiControllerContext = createContext<ShogiController | null>(null);

export const useShogiController = () => {
  const context = useContext(ShogiControllerContext);
  if (!context) {
    throw new Error('useShogiController must be used within a ShogiControllerProvider');
  }
  return context;
};

export const ShogiControllerProvider: React.FC<{ controller: ShogiController; children: React.ReactNode }> = ({ controller, children }) => {
  return (
    <ShogiControllerContext.Provider value={controller}>
      {children}
    </ShogiControllerContext.Provider>
  );
};
