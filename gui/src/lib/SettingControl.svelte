<script>
  // Renders one setting from its descriptor: a Material 3 Expressive switch for
  // toggles, or a connected morphing segmented button group for enums.
  let {
    setting,
    value = null,
    pending = false,
    compact = false,
    locked = false,
    lockReason = "",
    onchange,
  } = $props();

  const SETTING_DESCRIPTIONS = {
    "low-cut":
      "Remove unwanted frequencies from audio signal and add clarity",
    "safety-track":
      "Only available in Mono mode. Use post-production software and separate audio tracks later",
    "clip-limiter":
      "Automatically adjust volume limit to avoid clipping",
    "auto-off-15m":
      "When enabled, RX will automatically turn off after 15 minutes of inactivity when not connected to save power",
    "tx-auto-off-15m":
      "When enabled, the TX will automatically power off to save power if it is not connected, internal recording is not enabled, and there has been no operation for 15 min",
    "camera-power":
      "When enabled, receiver and camera will be simultaneously powered on or off",
    "plug-free":
      "When enabled, attach mobile device adapter to expansion port of receiver and then connect to mobile device for audio output",
    "mic-leds":
      "When enabled, indicator on transmitter will light up to show device status",
  };

  const OPTION_DESCRIPTIONS = {
    "voice-tone": {
      standard: "Offers balanced sound with clear and full tones",
      rich: "Provides solid sound without muddiness and boosts low frequencies",
      bright: "Enhances high frequencies, resulting in clearer and more transparent sound without harshness",
    },
  };

  const isToggle = $derived(setting.kind === "toggle");
  const onValue = $derived(setting.options[1]?.value);
  const checked = $derived(value === onValue);
  const disabled = $derived(pending || locked);
  const description = $derived(
    OPTION_DESCRIPTIONS[setting.id]?.[value] ?? SETTING_DESCRIPTIONS[setting.id] ?? null,
  );
  const isStacked = $derived(setting.id === "voice-tone");

  function optionLabel(opt) {
    if (setting.id === "voice-tone" && opt.value === "standard") {
      return "Regular";
    }
    return opt.label;
  }

  function choose(v) {
    if (disabled || v === value) return;
    onchange?.(setting.id, v);
  }

  function toggle() {
    if (disabled) return;
    const next = checked ? setting.options[0].value : setting.options[1].value;
    onchange?.(setting.id, next);
  }
</script>

<div
  class="row m3-item"
  class:compact
  class:stacked={isStacked}
  class:busy={pending}
  class:locked
  title={locked ? lockReason : null}
>
  <div class="row-top">
    <div class="label">
      <div class="title-line">
        <span class="title">{setting.label}</span>
        {#if locked}
          <span class="lock" title={lockReason} aria-label={lockReason}>
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none">
              <rect x="5" y="11" width="14" height="10" rx="3" stroke="currentColor" stroke-width="2" />
              <path d="M8 11V7a4 4 0 0 1 8 0v4" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
            </svg>
          </span>
        {/if}
      </div>
      {#if description && !isStacked}
        <p class="desc">{description}</p>
      {/if}
    </div>

    <div class="control" class:dim={locked}>
      {#if value === null}
        <span class="unknown">—</span>
      {:else if isToggle}
        <button
          class="m3-switch"
          class:on={checked}
          role="switch"
          aria-checked={checked}
          aria-label={setting.label}
          onclick={toggle}
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
      {:else}
        <div class="m3-segmented" role="group" aria-label={setting.label}>
          {#each setting.options as opt (opt.value)}
            <button
              class="m3-segment"
              class:active={value === opt.value}
              onclick={() => choose(opt.value)}
            >
              {optionLabel(opt)}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  {#if description && isStacked}
    <p class="desc stacked-desc">{description}</p>
  {/if}
</div>

<style>
  .row {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 14px 18px;
    background: var(--md-sys-color-surface-container);
    border-radius: 8px;
    transition: background 0.18s var(--m3-standard);
  }
  .row:first-child {
    border-top-left-radius: 22px;
    border-top-right-radius: 22px;
  }
  .row:last-child {
    border-bottom-left-radius: 22px;
    border-bottom-right-radius: 22px;
  }
  .row:only-child {
    border-radius: 22px;
  }
  .row.compact {
    padding: 12px 14px;
    background: var(--md-sys-color-surface-container);
    border-radius: 8px;
  }
  .row.compact:first-child {
    border-top-left-radius: 20px;
    border-top-right-radius: 20px;
  }
  .row.compact:last-child {
    border-bottom-left-radius: 20px;
    border-bottom-right-radius: 20px;
  }
  .row.compact:only-child {
    border-radius: 20px;
  }
  .row.busy {
    cursor: progress;
  }
  .row.locked .title {
    color: var(--md-sys-color-on-surface-variant);
  }

  .row-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
  }
  .row.stacked .row-top {
    flex-wrap: wrap;
    gap: 8px;
  }

  .control.dim {
    opacity: 0.38;
    pointer-events: none;
  }
  .lock {
    display: inline-flex;
    align-items: center;
    color: var(--md-sys-color-outline);
    cursor: help;
    flex: 0 0 auto;
  }
  .label {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
    flex: 1 1 auto;
  }
  .title-line {
    display: flex;
    align-items: center;
    gap: 6px;
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
  .desc {
    margin: 0;
    font-size: 12px;
    line-height: 1.4;
    color: var(--md-sys-color-on-surface-variant);
  }
  .stacked-desc {
    padding-top: 2px;
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
  .m3-switch:active .m3-thumb {
    width: 26px;
    height: 24px;
    left: 2px;
  }
  .m3-switch.on:active .m3-thumb {
    width: 26px;
    left: 20px;
  }
  .compact .m3-switch:active .m3-thumb {
    width: 22px;
    height: 21px;
    left: 2px;
  }
  .compact .m3-switch.on:active .m3-thumb {
    width: 23px;
    left: 17px;
  }
  .m3-check {
    width: 14px;
    height: 14px;
    opacity: 0;
    transform: scale(0.4);
    transition:
      opacity 0.16s var(--m3-standard),
      transform 0.24s var(--m3-spring-fast);
  }
  .compact .m3-check {
    width: 12px;
    height: 12px;
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
    padding: 6px 14px;
    font-size: 13px;
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
  .compact .m3-segment {
    padding: 5px 11px;
    font-size: 12px;
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
