import type { ShortcutMap } from '../types';

export type ShortcutAction = keyof ShortcutMap;
type Handler = () => void;

/**
 * Converts a KeyboardEvent to the canonical key-combo string stored in ShortcutMap.
 * Format: [Ctrl+][Shift+][Alt+]<code>
 */
export function eventToKeyCombo(event: KeyboardEvent): string {
  const parts: string[] = [];
  if (event.ctrlKey || event.metaKey) parts.push('Ctrl');
  if (event.shiftKey) parts.push('Shift');
  if (event.altKey) parts.push('Alt');

  // Normalize key codes
  if (event.code === 'Space') parts.push('Space');
  else if (event.code === 'Comma') parts.push('Comma');
  else parts.push(event.code);

  return parts.join('+');
}

class ShortcutManager {
  private handlers = new Map<ShortcutAction, Handler>();
  private map: ShortcutMap | null = null;

  setMap(map: ShortcutMap) {
    this.map = map;
  }

  on(action: ShortcutAction, handler: Handler) {
    this.handlers.set(action, handler);
  }

  off(action: ShortcutAction) {
    this.handlers.delete(action);
  }

  clear() {
    this.handlers.clear();
  }

  handle(event: KeyboardEvent): boolean {
    if (!this.map) return false;

    // Ignore when focus is in an input element
    const target = event.target as HTMLElement;
    if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)) {
      return false;
    }

    const combo = eventToKeyCombo(event);

    for (const [action, bound] of Object.entries(this.map) as [ShortcutAction, string][]) {
      if (bound === combo) {
        const handler = this.handlers.get(action);
        if (handler) {
          event.preventDefault();
          handler();
          return true;
        }
      }
    }

    return false;
  }
}

export const shortcuts = new ShortcutManager();
