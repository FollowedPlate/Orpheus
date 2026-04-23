<script lang="ts">
  import { settingsStore } from '../stores/settings';
  import { readerStore } from '../stores/reader';
  import { shortcuts, eventToKeyCombo } from '../utils/shortcuts';
  import { AVAILABLE_FONTS } from '../types';
  import type { Settings, ShortcutAction, AdjacentWordsCount } from '../types';

  const ADJACENT_COUNTS: { value: AdjacentWordsCount; label: string }[] = [
    { value: 1, label: '1 word each side' },
    { value: 2, label: '2 words each side' },
    { value: 3, label: '3 words each side' },
    { value: 'dynamic', label: 'Dynamic (fill screen)' },
  ];

  let activeTab: 'display' | 'speed' | 'behavior' | 'theme' | 'llm' | 'shortcuts' = 'display';
  let s: Settings = { ...$settingsStore };
  let capturingShortcut: ShortcutAction | null = null;

  $: if ($settingsStore) {
    s = { ...$settingsStore };
  }

  async function saveSettings() {
    await settingsStore.save(s);
  }

  function close() {
    readerStore.setShowSettings(false);
  }

  function captureShortcut(action: ShortcutAction) {
    capturingShortcut = action;
  }

  function handleShortcutKey(e: KeyboardEvent) {
    if (!capturingShortcut) return;
    e.preventDefault();
    if (e.key === 'Escape') {
      capturingShortcut = null;
      return;
    }
    const combo = eventToKeyCombo(e);
    s = { ...s, shortcuts: { ...s.shortcuts, [capturingShortcut]: combo } };
    capturingShortcut = null;
  }

  const THEME_PRESETS = ['Light', 'Dark', 'Sepia', 'HighContrast', 'Custom'] as const;
  const ORP_ALGORITHMS = ['Spritz', 'Center', 'Dynamic'] as const;
  const LLM_PROVIDERS = ['OpenAI', 'Ollama'] as const;
  const QUESTION_STYLES = ['MultipleChoice', 'OpenEnded', 'Mixed'] as const;

  const SHORTCUT_LABELS: Record<ShortcutAction, string> = {
    play_pause: 'Play / Pause',
    skip_forward_sentence: 'Skip Forward (Sentence)',
    skip_back_sentence: 'Skip Back (Sentence)',
    skip_forward_paragraph: 'Skip Forward (Paragraph)',
    skip_back_paragraph: 'Skip Back (Paragraph)',
    increase_wpm: 'Increase WPM',
    decrease_wpm: 'Decrease WPM',
    toggle_focus: 'Toggle Focus Mode',
    open_settings: 'Open Settings',
    return_to_library: 'Return to Library',
  };
</script>

<svelte:options runes={false} />

<svelte:window onkeydown={handleShortcutKey} />

<div class="settings-backdrop" onclick={close} role="presentation"></div>

<div class="settings-panel" role="dialog" aria-label="Settings">
  <div class="settings-header">
    <h2>Settings</h2>
    <button class="close-btn" onclick={close} aria-label="Close settings">×</button>
  </div>

  <!-- Tab bar -->
  <div class="tab-bar" role="tablist">
    {#each (['display', 'speed', 'behavior', 'theme', 'llm', 'shortcuts'] as const) as tab (tab)}
      <button
        class="tab-btn"
        class:active={activeTab === tab}
        role="tab"
        aria-selected={activeTab === tab}
        onclick={() => (activeTab = tab)}
      >
        {tab === 'llm' ? 'LLM' : tab.charAt(0).toUpperCase() + tab.slice(1)}
      </button>
    {/each}
  </div>

  <div class="settings-body">
    <!-- ─── DISPLAY ─────────────────────────────────── -->
    {#if activeTab === 'display'}
      <div class="section">
        <label class="field">
          <span class="field-label">Font</span>
          <select bind:value={s.font} onchange={saveSettings}>
            {#each AVAILABLE_FONTS as font (font)}
              <option value={font} style="font-family: '{font}'">{font}</option>
            {/each}
          </select>
        </label>

        <label class="field">
          <span class="field-label">Font size: {s.font_size}px</span>
          <input
            type="range"
            min="14"
            max="100"
            step="1"
            bind:value={s.font_size}
            onchange={saveSettings}
          />
        </label>

        <label class="field">
          <span class="field-label">ORP Algorithm</span>
          <div class="radio-group">
            {#each ORP_ALGORITHMS as alg (alg)}
              <label class="radio-label">
                <input
                  type="radio"
                  name="orp_algorithm"
                  value={alg}
                  bind:group={s.orp_algorithm}
                  onchange={saveSettings}
                />
                <span>
                  {alg === 'Spritz'
                    ? 'Spritz (by length)'
                    : alg === 'Center'
                    ? 'Center'
                    : 'Dynamic (Spritz -> Center for long words)'}
                </span>
              </label>
            {/each}
          </div>
        </label>

        <label class="field">
          <span class="field-label">ORP Color</span>
          <div class="color-row">
            <input type="color" bind:value={s.orp_color} onchange={saveSettings} />
            <span class="color-value">{s.orp_color}</span>
          </div>
        </label>

        <label class="field checkbox-field">
          <input type="checkbox" bind:checked={s.orp_bold} onchange={saveSettings} />
          <span class="field-label">ORP Bold</span>
        </label>

        <label class="field checkbox-field">
          <input type="checkbox" bind:checked={s.orp_underline} onchange={saveSettings} />
          <span class="field-label">ORP Underline</span>
        </label>

        <label class="field">
          <span class="field-label">ORP Size Multiplier: {s.orp_size_multiplier.toFixed(2)}×</span>
          <input
            type="range"
            min="1.0"
            max="1.5"
            step="0.05"
            bind:value={s.orp_size_multiplier}
            onchange={saveSettings}
          />
        </label>
      </div>

    <!-- ─── SPEED ──────────────────────────────────── -->
    {:else if activeTab === 'speed'}
      <div class="section">
        <label class="field">
          <span class="field-label">Target WPM: {s.wpm}</span>
          <input
            type="range"
            min="50"
            max="1000"
            step="5"
            bind:value={s.wpm}
            onchange={saveSettings}
          />
        </label>

        <label class="field">
          <span class="field-label">WPM Step (arrow keys): {s.wpm_step}</span>
          <input
            type="range"
            min="5"
            max="100"
            step="5"
            bind:value={s.wpm_step}
            onchange={saveSettings}
          />
        </label>

        <label class="field">
          <span class="field-label">Sentence-end pause: {s.sentence_end_multiplier.toFixed(1)}×</span>
          <input
            type="range"
            min="1.0"
            max="5.0"
            step="0.1"
            bind:value={s.sentence_end_multiplier}
            onchange={saveSettings}
          />
        </label>

        <label class="field">
          <span class="field-label">Clause pause (,;:): {s.clause_multiplier.toFixed(1)}×</span>
          <input
            type="range"
            min="1.0"
            max="3.0"
            step="0.1"
            bind:value={s.clause_multiplier}
            onchange={saveSettings}
          />
        </label>

        <label class="field">
          <span class="field-label">Paragraph break: {s.paragraph_multiplier.toFixed(1)}×</span>
          <input
            type="range"
            min="1.0"
            max="8.0"
            step="0.5"
            bind:value={s.paragraph_multiplier}
            onchange={saveSettings}
          />
        </label>

        <label class="field">
          <span class="field-label">Long word threshold: {s.long_word_threshold} chars</span>
          <input
            type="range"
            min="4"
            max="20"
            step="1"
            bind:value={s.long_word_threshold}
            onchange={saveSettings}
          />
        </label>

        <label class="field">
          <span class="field-label">Long word multiplier: {s.long_word_multiplier.toFixed(2)}× per extra char</span>
          <input
            type="range"
            min="1.0"
            max="3.0"
            step="0.1"
            bind:value={s.long_word_multiplier}
            onchange={saveSettings}
          />
        </label>

        <div class="field-group">
          <label class="field checkbox-field">
            <input type="checkbox" bind:checked={s.speed_ramp_enabled} onchange={saveSettings} />
            <span class="field-label">Speed Ramp-Up</span>
          </label>

          {#if s.speed_ramp_enabled}
            <label class="field">
              <span class="field-label">Start WPM: {s.speed_ramp_start_wpm}</span>
              <input
                type="range"
                min="50"
                max={s.wpm}
                step="5"
                bind:value={s.speed_ramp_start_wpm}
                onchange={saveSettings}
              />
            </label>
            <label class="field">
              <span class="field-label">Ramp duration: {s.speed_ramp_duration_secs}s</span>
              <input
                type="range"
                min="5"
                max="120"
                step="5"
                bind:value={s.speed_ramp_duration_secs}
                onchange={saveSettings}
              />
            </label>
          {/if}
        </div>
      </div>

    <!-- ─── BEHAVIOR ──────────────────────────────── -->
    {:else if activeTab === 'behavior'}
      <div class="section">
        <label class="field checkbox-field">
          <input type="checkbox" bind:checked={s.word_grouping_enabled} onchange={saveSettings} />
          <span class="field-label">Smart word grouping (merge short words)</span>
        </label>

        <div class="divider"></div>

        <label class="field checkbox-field">
          <input type="checkbox" bind:checked={s.adjacent_words_enabled} onchange={saveSettings} />
          <span class="field-label">Show adjacent words during reading</span>
        </label>

        {#if s.adjacent_words_enabled}
          <label class="field">
            <span class="field-label">Adjacent word count</span>
            <div class="radio-group">
              {#each ADJACENT_COUNTS as opt (opt.value)}
                <label class="radio-label">
                  <input
                    type="radio"
                    name="adjacent_count"
                    value={opt.value}
                    bind:group={s.adjacent_words_count}
                    onchange={saveSettings}
                  />
                  <span>{opt.label}</span>
                </label>
              {/each}
            </div>
          </label>
        {/if}

        <label class="field checkbox-field">
          <input type="checkbox" bind:checked={s.skip_context_enabled} onchange={saveSettings} />
          <span class="field-label">Show surrounding words when skipping (full-width, 1.5 s)</span>
        </label>

        <div class="divider"></div>

        <label class="field">
          <span class="field-label">Break interval (minutes, 0 = off)</span>
          <div class="number-row">
            <input
              type="range"
              min="0"
              max="120"
              step="1"
              bind:value={s.break_interval_minutes}
              onchange={saveSettings}
            />
            <span class="num-val">{s.break_interval_minutes === 0 ? 'Off' : s.break_interval_minutes + ' min'}</span>
          </div>
        </label>

        {#if s.break_interval_minutes > 0}
          <label class="field checkbox-field">
            <input
              type="checkbox"
              bind:checked={s.comprehension_questions_enabled}
              onchange={saveSettings}
            />
            <span class="field-label">Comprehension questions during breaks</span>
          </label>
        {/if}
      </div>

    <!-- ─── THEME ──────────────────────────────────── -->
    {:else if activeTab === 'theme'}
      <div class="section">
        <label class="field">
          <span class="field-label">Theme Preset</span>
          <div class="preset-grid">
            {#each THEME_PRESETS as preset (preset)}
              <button
                class="preset-btn"
                class:active={s.theme_preset === preset}
                onclick={() => {
                  s = { ...s, theme_preset: preset };
                  saveSettings();
                }}
              >{preset === 'HighContrast' ? 'High Contrast' : preset}</button>
            {/each}
          </div>
        </label>

        <div class="divider"></div>
        <p class="section-subtitle">Custom color overrides (leave empty to use preset)</p>

        <label class="field">
          <span class="field-label">Background</span>
          <div class="color-row">
            <input
              type="color"
              value={s.custom_bg_color ?? '#121212'}
              oninput={(e) => {
                s = { ...s, custom_bg_color: (e.target as HTMLInputElement).value };
              }}
              onchange={saveSettings}
            />
            <button class="clear-btn" onclick={() => { s = { ...s, custom_bg_color: null }; saveSettings(); }}>Clear</button>
          </div>
        </label>

        <label class="field">
          <span class="field-label">Text Color</span>
          <div class="color-row">
            <input
              type="color"
              value={s.custom_text_color ?? '#e8e8e8'}
              oninput={(e) => {
                s = { ...s, custom_text_color: (e.target as HTMLInputElement).value };
              }}
              onchange={saveSettings}
            />
            <button class="clear-btn" onclick={() => { s = { ...s, custom_text_color: null }; saveSettings(); }}>Clear</button>
          </div>
        </label>

        <label class="field">
          <span class="field-label">ORP / Guide Color</span>
          <div class="color-row">
            <input
              type="color"
              value={s.custom_orp_color ?? s.orp_color}
              oninput={(e) => {
                s = { ...s, custom_orp_color: (e.target as HTMLInputElement).value };
              }}
              onchange={saveSettings}
            />
            <button class="clear-btn" onclick={() => { s = { ...s, custom_orp_color: null }; saveSettings(); }}>Clear</button>
          </div>
        </label>

        <label class="field">
          <span class="field-label">Accent Color</span>
          <div class="color-row">
            <input
              type="color"
              value={s.custom_accent_color ?? '#82b1ff'}
              oninput={(e) => {
                s = { ...s, custom_accent_color: (e.target as HTMLInputElement).value };
              }}
              onchange={saveSettings}
            />
            <button class="clear-btn" onclick={() => { s = { ...s, custom_accent_color: null }; saveSettings(); }}>Clear</button>
          </div>
        </label>
      </div>

    <!-- ─── LLM ───────────────────────────────────── -->
    {:else if activeTab === 'llm'}
      <div class="section">
        <label class="field">
          <span class="field-label">Provider</span>
          <div class="radio-group">
            {#each LLM_PROVIDERS as provider (provider)}
              <label class="radio-label">
                <input
                  type="radio"
                  name="llm_provider"
                  value={provider}
                  bind:group={s.llm_provider}
                  onchange={saveSettings}
                />
                <span>{provider === 'OpenAI' ? 'OpenAI API' : 'Ollama (local)'}</span>
              </label>
            {/each}
          </div>
        </label>

        <label class="field">
          <span class="field-label">API Endpoint</span>
          <input
            type="url"
            class="text-input"
            bind:value={s.llm_endpoint}
            placeholder={s.llm_provider === 'Ollama' ? 'http://localhost:11434' : 'https://api.openai.com/v1'}
            onchange={saveSettings}
          />
          {#if s.llm_provider === 'Ollama'}
            <p class="section-subtitle" style="margin-top: 6px;">
              Base URL only (e.g. port 11434). Requests use OpenAI-compatible
              <code>/v1/chat/completions</code>; you may include <code>/v1</code> in the URL or omit it.
            </p>
          {/if}
        </label>

        {#if s.llm_provider === 'OpenAI'}
          <label class="field">
            <span class="field-label">API Key</span>
            <input
              type="password"
              class="text-input"
              bind:value={s.llm_api_key}
              placeholder="sk-..."
              onchange={saveSettings}
            />
          </label>
        {/if}

        <label class="field">
          <span class="field-label">Model</span>
          <input
            type="text"
            class="text-input"
            bind:value={s.llm_model}
            placeholder={s.llm_provider === 'Ollama' ? 'llama3' : 'gpt-4o-mini'}
            onchange={saveSettings}
          />
        </label>

        <label class="field">
          <span class="field-label">Quiz context max words (since last break)</span>
          <div class="number-row">
            <input
              type="range"
              min="50"
              max="2000"
              step="10"
              bind:value={s.quiz_context_max_words}
              onchange={saveSettings}
            />
            <span class="num-val">{s.quiz_context_max_words}</span>
          </div>
          <p class="section-subtitle" style="margin-top: 6px;">
            Only text read since the last break is sent to the LLM, up to this many words.
          </p>
        </label>

        <label class="field">
          <span class="field-label">Metadata context max words (start of book)</span>
          <div class="number-row">
            <input
              type="range"
              min="500"
              max="5000"
              step="100"
              bind:value={s.metadata_context_max_words}
              onchange={saveSettings}
            />
            <span class="num-val">{s.metadata_context_max_words}</span>
          </div>
          <p class="section-subtitle" style="margin-top: 6px;">
            Number of initial words sent to the LLM to infer book metadata.
          </p>
        </label>

        <label class="field">
          <span class="field-label">Question Style</span>
          <div class="radio-group">
            {#each QUESTION_STYLES as style (style)}
              <label class="radio-label">
                <input
                  type="radio"
                  name="question_style"
                  value={style}
                  bind:group={s.question_style}
                  onchange={saveSettings}
                />
                <span>
                  {style === 'MultipleChoice'
                    ? 'Multiple choice'
                    : style === 'OpenEnded'
                    ? 'Open-ended'
                    : 'Mixed'}
                </span>
              </label>
            {/each}
          </div>
        </label>
      </div>

    <!-- ─── SHORTCUTS ─────────────────────────────── -->
    {:else if activeTab === 'shortcuts'}
      <div class="section">
        <p class="section-subtitle">Click a shortcut and press a key combination to rebind.</p>
        {#each Object.entries(SHORTCUT_LABELS) as [action, label] (action)}
          <div class="shortcut-row">
            <span class="shortcut-label">{label}</span>
            <button
              class="shortcut-key"
              class:capturing={capturingShortcut === action}
              onclick={() => captureShortcut(action as ShortcutAction)}
            >
              {capturingShortcut === action
                ? 'Press key…'
                : s.shortcuts[action as ShortcutAction]}
            </button>
          </div>
        {/each}

        <button
          class="reset-btn"
          onclick={() => {
            s = { ...s, shortcuts: { ...{
              play_pause: 'Space',
              skip_forward_sentence: 'ArrowRight',
              skip_back_sentence: 'ArrowLeft',
              skip_forward_paragraph: 'Ctrl+ArrowRight',
              skip_back_paragraph: 'Ctrl+ArrowLeft',
              increase_wpm: 'ArrowUp',
              decrease_wpm: 'ArrowDown',
              toggle_focus: 'KeyF',
              open_settings: 'Ctrl+Comma',
              return_to_library: 'Escape',
            }}};
            saveSettings();
          }}
        >Reset to Defaults</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .settings-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    z-index: 300;
  }

  .settings-panel {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: 420px;
    max-width: 100vw;
    background: var(--surface);
    border-left: 1px solid var(--border);
    z-index: 400;
    display: flex;
    flex-direction: column;
    font-family: var(--font-family);
    animation: slide-in 0.2s ease;
  }

  @keyframes slide-in {
    from { transform: translateX(100%); }
    to { transform: translateX(0); }
  }

  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 24px;
    border-bottom: 1px solid var(--border);
  }

  .settings-header h2 {
    font-size: 18px;
    font-weight: 700;
    color: var(--text);
    margin: 0;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--muted);
    cursor: pointer;
    font-size: 22px;
    line-height: 1;
    padding: 0;
  }

  .tab-bar {
    display: flex;
    overflow-x: auto;
    border-bottom: 1px solid var(--border);
    scrollbar-width: none;
  }

  .tab-bar::-webkit-scrollbar { display: none; }

  .tab-btn {
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--muted);
    cursor: pointer;
    font-size: 12px;
    font-family: var(--font-family);
    font-weight: 500;
    letter-spacing: 0.04em;
    padding: 10px 16px;
    text-transform: uppercase;
    white-space: nowrap;
    transition: color 0.15s;
  }

  .tab-btn.active {
    color: var(--accent);
    border-bottom-color: var(--accent);
  }

  .settings-body {
    flex: 1;
    overflow-y: auto;
    padding: 20px 24px;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .section-subtitle {
    font-size: 12px;
    color: var(--muted);
    margin: 0;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .field-label {
    font-size: 13px;
    font-weight: 500;
    color: var(--text);
  }

  .checkbox-field {
    flex-direction: row;
    align-items: center;
    gap: 10px;
  }

  .checkbox-field input[type="checkbox"] {
    accent-color: var(--accent);
    width: 16px;
    height: 16px;
  }

  select {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    font-family: var(--font-family);
    font-size: 13px;
    padding: 7px 10px;
  }

  input[type="range"] {
    width: 100%;
    accent-color: var(--accent);
  }

  .radio-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .radio-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--text);
    cursor: pointer;
  }

  .radio-label input {
    accent-color: var(--accent);
  }

  .color-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  input[type="color"] {
    width: 40px;
    height: 32px;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 2px;
    cursor: pointer;
    background: var(--bg);
  }

  .color-value {
    font-size: 12px;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }

  .clear-btn {
    background: none;
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--muted);
    cursor: pointer;
    font-size: 11px;
    font-family: var(--font-family);
    padding: 3px 8px;
  }

  .clear-btn:hover {
    background: var(--border);
    color: var(--text);
  }

  .field-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  .divider {
    height: 1px;
    background: var(--border);
    margin: 4px 0;
  }

  .number-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .number-row input {
    flex: 1;
  }

  .num-val {
    font-size: 13px;
    font-weight: 600;
    color: var(--accent);
    min-width: 48px;
    text-align: right;
  }

  .preset-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
  }

  .preset-btn {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    cursor: pointer;
    font-family: var(--font-family);
    font-size: 12px;
    padding: 8px 4px;
    transition: background 0.12s;
    text-align: center;
  }

  .preset-btn:hover {
    background: var(--border);
  }

  .preset-btn.active {
    border-color: var(--accent);
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 10%, transparent);
  }

  .text-input {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    font-family: var(--font-family);
    font-size: 13px;
    padding: 8px 10px;
    width: 100%;
    box-sizing: border-box;
  }

  .shortcut-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
    gap: 12px;
  }

  .shortcut-label {
    font-size: 13px;
    color: var(--text);
    flex: 1;
  }

  .shortcut-key {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 5px;
    color: var(--text);
    cursor: pointer;
    font-family: monospace;
    font-size: 12px;
    padding: 4px 10px;
    min-width: 100px;
    text-align: center;
    white-space: nowrap;
    transition: border-color 0.15s;
  }

  .shortcut-key.capturing {
    border-color: var(--accent);
    color: var(--accent);
    animation: pulse 1s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .reset-btn {
    background: none;
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--muted);
    cursor: pointer;
    font-family: var(--font-family);
    font-size: 12px;
    padding: 8px 14px;
    margin-top: 8px;
    align-self: flex-start;
  }

  .reset-btn:hover {
    background: var(--border);
    color: var(--text);
  }
</style>
