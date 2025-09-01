import { render, screen, fireEvent } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import EngineSettings from './EngineSettings';
import { describe, test, expect, beforeEach } from 'vitest';

const localStorageMock = (() => {
  let store: { [key: string]: string } = {};
  return {
    getItem(key: string) {
      return store[key] || null;
    },
    setItem(key: string, value: string) {
      store[key] = value.toString();
    },
    clear() {
      store = {};
    },
    getStore() {
      return store;
    }
  };
})();

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock
});

describe('EngineSettings', () => {

  beforeEach(() => {
    localStorage.clear();
  });

  test('renders the component and initializes localStorage if empty', () => {
    render(
      <BrowserRouter>
        <EngineSettings />
      </BrowserRouter>
    );

    expect(screen.getByText('Engine Settings')).toBeInTheDocument();
    expect(screen.getByLabelText('Built-in WASM (./ai/ai.worker.ts)')).toBeInTheDocument();
    expect(screen.getByLabelText('Built-in WASM (./ai/ai.worker.ts)')).toBeChecked();
    const store = (localStorage as any).getStore();
    expect(JSON.parse(store['shogi-engines'])).toEqual([{ name: 'Built-in WASM', path: './ai/ai.worker.ts' }]);
    expect(store['shogi-selected-engine']).toBe('./ai/ai.worker.ts');
  });

  test('adds a new engine and saves it to localStorage', () => {
    render(
      <BrowserRouter>
        <EngineSettings />
      </BrowserRouter>
    );

    const engineNameInput = screen.getByPlaceholderText('Engine Name');
    const enginePathInput = screen.getByPlaceholderText('Engine Path');
    const addButton = screen.getByText('Add Engine');

    fireEvent.change(engineNameInput, { target: { value: 'Test Engine' } });
    fireEvent.change(enginePathInput, { target: { value: './test-engine.js' } });
    fireEvent.click(addButton);

    expect(screen.getByLabelText('Test Engine (./test-engine.js)')).toBeInTheDocument();
    const store = (localStorage as any).getStore();
    const storedEngines = JSON.parse(store['shogi-engines']);
    expect(storedEngines).toHaveLength(2);
    expect(storedEngines[1]).toEqual({ name: 'Test Engine', path: './test-engine.js' });
  });

  test('selects a different engine and saves the selection to localStorage', () => {
    // Setup initial state with two engines
    const initialEngines = [
        { name: 'Built-in WASM', path: './ai/ai.worker.ts' },
        { name: 'Test Engine', path: './test-engine.js' }
    ];
    localStorage.setItem('shogi-engines', JSON.stringify(initialEngines));
    localStorage.setItem('shogi-selected-engine', './ai/ai.worker.ts');

    render(
      <BrowserRouter>
        <EngineSettings />
      </BrowserRouter>
    );

    const testEngineRadio = screen.getByLabelText('Test Engine (./test-engine.js)');
    fireEvent.click(testEngineRadio);

    expect(testEngineRadio).toBeChecked();
    const store = (localStorage as any).getStore();
    expect(store['shogi-selected-engine']).toBe('./test-engine.js');
  });
});
