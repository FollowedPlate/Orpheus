import type { Settings, ThemePreset } from '../types';

interface ThemeColors {
  bg: string;
  text: string;
  orp: string;
  accent: string;
  guide: string;
  surface: string;
  border: string;
  muted: string;
}

const PRESETS: Record<ThemePreset, ThemeColors> = {
  Light: {
    bg: '#ffffff',
    text: '#1a1a1a',
    orp: '#e53935',
    accent: '#1565c0',
    guide: '#e53935',
    surface: '#f5f5f5',
    border: '#e0e0e0',
    muted: '#757575',
  },
  Dark: {
    bg: '#121212',
    text: '#e8e8e8',
    orp: '#ff5252',
    accent: '#82b1ff',
    guide: '#ff5252',
    surface: '#1e1e1e',
    border: '#333333',
    muted: '#9e9e9e',
  },
  Sepia: {
    bg: '#f4ecd8',
    text: '#3b2e1e',
    orp: '#c0392b',
    accent: '#8b5e3c',
    guide: '#c0392b',
    surface: '#ede0c8',
    border: '#c8b89a',
    muted: '#7a6652',
  },
  HighContrast: {
    bg: '#000000',
    text: '#ffffff',
    orp: '#ffff00',
    accent: '#00ffff',
    guide: '#ffff00',
    surface: '#111111',
    border: '#555555',
    muted: '#aaaaaa',
  },
  Custom: {
    bg: '#121212',
    text: '#e8e8e8',
    orp: '#ff5252',
    accent: '#82b1ff',
    guide: '#ff5252',
    surface: '#1e1e1e',
    border: '#333333',
    muted: '#9e9e9e',
  },
};

function resolveColors(settings: Settings): ThemeColors {
  const preset = PRESETS[settings.theme_preset] ?? PRESETS.Dark;
  return {
    bg: settings.custom_bg_color ?? preset.bg,
    text: settings.custom_text_color ?? preset.text,
    orp: settings.custom_orp_color ?? settings.orp_color ?? preset.orp,
    accent: settings.custom_accent_color ?? preset.accent,
    guide: settings.custom_guide_color ?? settings.orp_color ?? preset.guide,
    surface: preset.surface,
    border: preset.border,
    muted: preset.muted,
  };
}

function applyTheme(settings: Settings) {
  const colors = resolveColors(settings);
  const root = document.documentElement;

  root.style.setProperty('--bg', colors.bg);
  root.style.setProperty('--text', colors.text);
  root.style.setProperty('--orp', colors.orp);
  root.style.setProperty('--accent', colors.accent);
  root.style.setProperty('--guide', colors.guide);
  root.style.setProperty('--surface', colors.surface);
  root.style.setProperty('--border', colors.border);
  root.style.setProperty('--muted', colors.muted);
  root.style.setProperty('--font-family', `'${settings.font}', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif`);
  root.style.setProperty('--font-size', `${settings.font_size}px`);
}

export { applyTheme };
