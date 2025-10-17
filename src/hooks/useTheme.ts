import { useState, useEffect } from 'react';

export type Theme = 'light' | 'dark' | 'traditional' | 'ocean' | 'forest' | 'midnight' | 'sunset' | 'cyberpunk' | 'cherry' | 'monochrome' | 'sepia';

const THEME_STORAGE_KEY = 'shogiVibeTheme';
const DEFAULT_THEME: Theme = 'light';

/**
 * Custom hook for managing application theme
 * 
 * Persists theme selection to localStorage and applies it to the document element
 * 
 * @returns Object containing current theme and setter function
 */
export function useTheme() {
  const [theme, setThemeState] = useState<Theme>(() => {
    // Initialize from localStorage or use default
    const savedTheme = localStorage.getItem(THEME_STORAGE_KEY) as Theme | null;
    return savedTheme && isValidTheme(savedTheme) ? savedTheme : DEFAULT_THEME;
  });

  useEffect(() => {
    // Apply theme to document element
    document.documentElement.setAttribute('data-theme', theme);
    
    // Save to localStorage
    localStorage.setItem(THEME_STORAGE_KEY, theme);
  }, [theme]);

  const setTheme = (newTheme: Theme) => {
    if (isValidTheme(newTheme)) {
      setThemeState(newTheme);
    } else {
      console.warn(`Invalid theme: ${newTheme}. Using default theme.`);
      setThemeState(DEFAULT_THEME);
    }
  };

  return { theme, setTheme };
}

/**
 * Type guard to check if a string is a valid Theme
 */
function isValidTheme(value: string): value is Theme {
  return ['light', 'dark', 'traditional', 'ocean', 'forest', 'midnight', 'sunset', 'cyberpunk', 'cherry', 'monochrome', 'sepia'].includes(value);
}

/**
 * Get theme display name for UI
 */
export function getThemeDisplayName(theme: Theme): string {
  const names: Record<Theme, string> = {
    light: 'Light',
    dark: 'Dark',
    traditional: 'Traditional Wood',
    ocean: 'Ocean Blue',
    forest: 'Forest Green',
    midnight: 'Midnight Purple',
    sunset: 'Sunset Orange',
    cyberpunk: 'Cyberpunk Neon',
    cherry: 'Cherry Blossom',
    monochrome: 'Monochrome',
    sepia: 'Sepia Vintage',
  };
  return names[theme];
}

/**
 * Get theme description
 */
export function getThemeDescription(theme: Theme): string {
  const descriptions: Record<Theme, string> = {
    light: 'Clean and modern light theme',
    dark: 'High contrast dark theme',
    traditional: 'Classic wood and gold aesthetic',
    ocean: 'Cool and calming ocean blues',
    forest: 'Natural forest greens',
    midnight: 'Dark purple theme for night gaming',
    sunset: 'Warm orange theme with energizing colors',
    cyberpunk: 'High contrast neon green on dark',
    cherry: 'Soft pink theme with Japanese aesthetics',
    monochrome: 'Clean black and white professional',
    sepia: 'Warm vintage aged paper look',
  };
  return descriptions[theme];
}

