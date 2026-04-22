import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { DEFAULT_SETTINGS } from '../types';
import type { Settings } from '../types';

function createSettingsStore() {
  const { subscribe, set, update } = writable<Settings>(DEFAULT_SETTINGS);
  let loaded = false;

  return {
    subscribe,

    async load() {
      if (loaded) return;
      try {
        const s = await invoke<Settings>('load_settings');
        set(s);
        loaded = true;
      } catch (e) {
        console.error('Failed to load settings:', e);
      }
    },

    async save(settings: Settings) {
      set(settings);
      try {
        await invoke('save_settings', { settings });
      } catch (e) {
        console.error('Failed to save settings:', e);
      }
    },

    async patch(partial: Partial<Settings>) {
      let merged!: Settings;
      update((s) => {
        merged = { ...s, ...partial };
        return merged;
      });
      try {
        await invoke('save_settings', { settings: merged });
      } catch (e) {
        console.error('Failed to save settings:', e);
      }
    },
  };
}

export const settingsStore = createSettingsStore();
