export interface ThemeAttribution {
  name: string;
  url: string;
  license: string;
  licenseUrl: string;
}

export interface ThemeConfig {
  id: string;
  displayName: string;
  description?: string;
  attribution?: ThemeAttribution;
}

export interface ThemesConfig {
  themes: ThemeConfig[];
}

let themesConfig: ThemesConfig | null = null;

export async function loadThemesConfig(): Promise<ThemesConfig> {
  if (themesConfig) {
    return themesConfig;
  }

  try {
    const response = await fetch('/piece-themes/themes.json');
    if (!response.ok) {
      throw new Error(`Failed to load themes config: ${response.statusText}`);
    }
    themesConfig = await response.json();
    return themesConfig;
  } catch (error) {
    console.error('Error loading themes config:', error);
    // Return empty config as fallback
    return { themes: [] };
  }
}

export async function getAvailableThemes(): Promise<ThemeConfig[]> {
  const config = await loadThemesConfig();
  return config.themes;
}

export async function getThemeById(id: string): Promise<ThemeConfig | undefined> {
  const themes = await getAvailableThemes();
  return themes.find(theme => theme.id === id);
}
