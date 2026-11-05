<script>
  // Noise Cancel's three settings (power, mode, and the TX-button toggle) are
  // closely related, so they render as one combined Material 3 Expressive card
  // item instead of three separate rows.
  let {
    power,
    mode,
    button,
    values = {},
    pending = {},
    lockReason,
    compact = false,
    onchange,
  } = $props();

  const MODE_DESCRIPTIONS = {
    basic: "Preserve audio fidelity, which is suitable for relatively quiet indoor environments",
    strong: "Reduce noise effectively, which is suitable for noisy outdoor environments",
  };

  const powerValue = $derived(values[power.id] ?? null);
  const modeValue = $derived(values[mode.id] ?? null);
  const buttonValue = $derived(values[button.id] ?? null);
  const modeDescription = $derived(MODE_DESCRIPTIONS[modeValue] ?? MODE_DESCRIPTIONS.basic);

  const powerReason = $derived(lockReason(power));
  const modeReason = $derived(lockReason(mode));
  const buttonReason = $derived(lockReason(button));

  const powerLocked = $derived(!!powerReason);
  const modeLocked = $derived(!!modeReason);
  const buttonLocked = $derived(!!buttonReason);

  const powerDisabled = $derived(powerLocked || !!pending[power.id]);
  const modeDisabled = $derived(modeLocked || !!pending[mode.id]);
  const buttonDisabled = $derived(buttonLocked || !!pending[button.id]);

  const powerChecked = $derived(powerValue === power.options[1]?.value);
  const buttonChecked = $derived(buttonValue === button.options[1]?.value);

  function togglePower() {
    if (powerDisabled || powerValue === null) return;
    onchange?.(power.id, powerChecked ? power.options[0].value : power.options[1].value);
  }
  function toggleButton() {
    if (buttonDisabled || buttonValue === null) return;
    onchange?.(button.id, buttonChecked ? button.options[0].value : button.options[1].value);
  }
  function chooseMode(v) {
    if (modeDisabled || v === modeValue) return;
    onchange?.(mode.id, v);
  }
</script>

<div class="ncgroup m3-item" class:compact>
  <div class="ncmain" class:busy={!!pending[power.id]} title={powerLocked ? powerReason : null}>
    <div class="label">
      <span class="title">{power.label}</span>
      {#if powerLocked}
        <span class="lock" title={powerReason} aria-label={powerReason}>
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none">
            <rect x="5" y="11" width="14" height="10" rx="3" stroke="currentColor" stroke-width="2" />
            <path d="M8 11V7a4 4 0 0 1 8 0v4" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
          </svg>
        </span>
      {/if}
    </div>
    <div class="control" class:dim={powerLocked}>
      {#if powerValue === null}
        <span class="unknown">—</span>
      {:else}
        <button
          class="m3-switch"
          class:on={powerChecked}
          role="switch"
          aria-checked={powerChecked}
          aria-label={power.label}
          onclick={togglePower}
        >
          <span class="m3-thumb">
            <svg class="m3-check" viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <path
                d="M3.5 8.3 6.6 11.4 12.6 4.8"
                stroke="currentColor"
                stroke-width="2.2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </span>
        </button>
      {/if}
    </div>
  </div>

  <div class="ncsub-container">
    <div class="ncsub-block" class:busy={!!pending[mode.id]} title={modeLocked ? modeReason : null}>
      <div class="ncsub">
        <div class="label">
          <span class="subtitle">Mode</span>
          {#if modeLocked}
            <span class="lock" title={modeReason} aria-label={modeReason}>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
                <rect x="5" y="11" width="14" height="10" rx="3" stroke="currentColor" stroke-width="2" />
                <path d="M8 11V7a4 4 0 0 1 8 0v4" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
              </svg>
            </span>
          {/if}
        </div>
        <div class="control" class:dim={modeLocked}>
          {#if modeValue === null}
            <span class="unknown">—</span>
          {:else}
            <div class="m3-segmented" role="group" aria-label={mode.label}>
              {#each mode.options as opt (opt.value)}
                <button
                  class="m3-segment"
                  class:active={modeValue === opt.value}
                  onclick={() => chooseMode(opt.value)}
                >
                  {opt.label}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      </div>
      {#if modeDescription}
        <p class="mode-desc">{modeDescription}</p>
      {/if}
    </div>

    <div class="ncsub-divider"></div>

    <div class="ncsub-block" class:busy={!!pending[button.id]} title={buttonLocked ? buttonReason : null}>
      <div class="ncsub">
        <div class="label">
          <span class="subtitle">{button.label}</span>
          {#if buttonLocked}
            <span class="lock" title={buttonReason} aria-label={buttonReason}>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
                <rect x="5" y="11" width="14" height="10" rx="3" stroke="currentColor" stroke-width="2" />
                <path d="M8 11V7a4 4 0 0 1 8 0v4" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
              </svg>
            </span>
          {/if}
        </div>
        <div class="control" class:dim={buttonLocked}>
          {#if buttonValue === null}
            <span class="unknown">—</span>
          {:else}
            <button
              class="m3-switch small"
              class:on={buttonChecked}
              role="switch"
              aria-checked={buttonChecked}
              aria-label={button.label}
              onclick={toggleButton}
            >
              <span class="m3-thumb">
                <svg class="m3-check" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                  <path
                    d="M3.5 8.3 6.6 11.4 12.6 4.8"
                    stroke="currentColor"
                    stroke-width="2.2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
              </span>
            </button>
          {/if}
        </div>
      </div>
      <p class="mode-desc">When enabled, press power button on transmitter to reduce noise</p>
    </div>
  </div>
</div>

<style>
  .ncgroup {
    padding: 14px 18px;
    background: var(--md-sys-color-surface-container);
    border-radius: 8px;
  }
  .ncgroup:first-child {
    border-top-left-radius: 22px;
    border-top-right-radius: 22px;
  }
  .ncgroup:last-child {
    border-bottom-left-radius: 22px;
    border-bottom-right-radius: 22px;
  }
  .ncgroup:only-child {
    border-radius: 22px;
  }
  .ncgroup.compact {
    padding: 12px 14px;
    border-radius: 8px;
  }
  .ncgroup.compact:first-child {
    border-top-left-radius: 20px;
    border-top-right-radius: 20px;
  }
  .ncgroup.compact:last-child {
    border-bottom-left-radius: 20px;
    border-bottom-right-radius: 20px;
  }
  .ncgroup.compact:only-child {
    border-radius: 20px;
  }

  .ncmain {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    padding: 2px 0 10px;
  }
  .ncmain.busy {
    cursor: progress;
  }
  .ncsub-container {
    background: var(--md-sys-color-surface-container-low);
    border-radius: 14px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .ncsub-block {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .ncsub-block.busy {
    cursor: progress;
  }
  .ncsub-divider {
    height: 1px;
    background: color-mix(in srgb, var(--md-sys-color-outline-variant) 45%, transparent);
  }
  .ncsub {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .ncsub.busy {
    cursor: progress;
  }
  .mode-desc {
    margin: 0;
    font-size: 12px;
    line-height: 1.4;
    color: var(--md-sys-color-on-surface-variant);
  }

  .label {
    display: flex;
    align-items: center;
    gap: 7px;
    min-width: 0;
  }
  .title {
    font-weight: 600;
    font-size: 14px;
    color: var(--md-sys-color-on-surface);
  }
  .compact .title {
    font-size: 13.5px;
  }
  .subtitle {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--md-sys-color-on-surface);
  }
  .lock {
    display: inline-flex;
    align-items: center;
    color: var(--md-sys-color-outline);
    cursor: help;
    flex: 0 0 auto;
  }
  .control.dim {
    opacity: 0.38;
    pointer-events: none;
  }
  .unknown {
    color: var(--md-sys-color-outline);
  }

  /* Material 3 Expressive Switch */
  .m3-switch {
    width: 52px;
    height: 32px;
    border-radius: var(--radius-pill);
    border: 2px solid var(--md-sys-color-outline);
    background: var(--md-sys-color-surface-container-highest);
    padding: 0;
    position: relative;
    display: inline-flex;
    align-items: center;
    flex: 0 0 auto;
    transition:
      background 0.22s var(--m3-standard),
      border-color 0.22s var(--m3-standard);
  }
  .compact .m3-switch {
    width: 46px;
    height: 28px;
  }
  .m3-switch.small {
    width: 42px;
    height: 26px;
  }
  .compact .m3-switch.small {
    width: 38px;
    height: 24px;
  }
  .m3-switch.on {
    background: var(--md-sys-color-primary);
    border-color: var(--md-sys-color-primary);
  }
  .m3-thumb {
    position: absolute;
    left: 6px;
    width: 16px;
    height: 16px;
    border-radius: var(--radius-pill);
    background: var(--md-sys-color-outline);
    color: var(--md-sys-color-primary);
    display: grid;
    place-items: center;
    transition:
      transform 0.28s var(--m3-spring-spatial),
      width 0.24s var(--m3-spring-fast),
      height 0.24s var(--m3-spring-fast),
      left 0.24s var(--m3-spring-spatial),
      background 0.2s var(--m3-standard);
  }
  .compact .m3-thumb {
    left: 5px;
    width: 14px;
    height: 14px;
  }
  .m3-switch.small .m3-thumb {
    left: 4px;
    width: 13px;
    height: 13px;
  }
  .compact .m3-switch.small .m3-thumb {
    left: 4px;
    width: 12px;
    height: 12px;
  }
  .m3-switch.on .m3-thumb {
    left: 22px;
    width: 24px;
    height: 24px;
    background: var(--md-sys-color-on-primary);
  }
  .compact .m3-switch.on .m3-thumb {
    left: 19px;
    width: 21px;
    height: 21px;
  }
  .m3-switch.small.on .m3-thumb {
    left: 17px;
    width: 19px;
    height: 19px;
  }
  .compact .m3-switch.small.on .m3-thumb {
    left: 15px;
    width: 17px;
    height: 17px;
  }
  .m3-check {
    width: 13px;
    height: 13px;
    opacity: 0;
    transform: scale(0.4);
    transition:
      opacity 0.16s var(--m3-standard),
      transform 0.24s var(--m3-spring-fast);
  }
  .m3-switch.small .m3-check {
    width: 11px;
    height: 11px;
  }
  .m3-switch.on .m3-check {
    opacity: 1;
    transform: scale(1);
  }

  /* Material 3 Expressive Connected Segmented Buttons */
  .m3-segmented {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    background: var(--md-sys-color-surface-container-highest);
    padding: 3px;
    border-radius: var(--radius-pill);
    flex: 0 0 auto;
  }
  .m3-segment {
    border: none;
    background: var(--md-sys-color-surface-container-low);
    color: var(--md-sys-color-on-surface-variant);
    padding: 5px 12px;
    font-size: 12px;
    font-weight: 600;
    border-radius: 8px;
    transition:
      background 0.2s var(--m3-standard),
      color 0.2s var(--m3-standard),
      border-radius 0.28s var(--m3-spring-spatial),
      transform 0.16s var(--m3-spring-fast);
  }
  .m3-segment:first-child {
    border-top-left-radius: var(--radius-pill);
    border-bottom-left-radius: var(--radius-pill);
  }
  .m3-segment:last-child {
    border-top-right-radius: var(--radius-pill);
    border-bottom-right-radius: var(--radius-pill);
  }
  .m3-segment:hover:not(.active) {
    background: var(--md-sys-color-surface-container);
    color: var(--md-sys-color-on-surface);
  }
  .m3-segment:active {
    transform: scale(0.95);
  }
  .m3-segment.active {
    background: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    border-radius: var(--radius-pill);
    font-weight: 700;
  }
</style>
