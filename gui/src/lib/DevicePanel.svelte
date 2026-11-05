<script>
  import DevicePicto from "./DevicePicto.svelte";
  import SettingControl from "./SettingControl.svelte";
  import NoiseCancelControl from "./NoiseCancelControl.svelte";
  import BatteryIcon from "./BatteryIcon.svelte";

  let {
    device,
    status,
    settings = [],
    values = {},
    pending = {},
    pendingTx = {},
    onchange,
    onchangeTx,
  } = $props();

  // Audio levels sit roughly in the 0x14–0x4f range; scale to a full bar.
  // `v` is null when the connected firmware hasn't reported a level yet.
  const levelPct = (v) => (v == null ? 0 : Math.max(0, Math.min(100, Math.round((v / 80) * 100))));

  // Identity/level fields the model hasn't decoded yet read as null.
  const orUnknown = (v) => v ?? "Unknown";

  // Gain dial in dB: always show a sign except at exactly 0.
  const formatGain = (v) => (v > 0 ? `+${v}` : `${v}`);

  // Artwork varies by the transmitter's own reported product name — a Mic
  // Mini 2 looks different from the original. Returns null (no picture)
  // while that's still a transient unknown rather than a guess: v2 firmware
  // reports it a moment after connecting, so briefly null there means "not
  // arrived yet," not "never coming" — only v1 firmware (which has no such
  // field at all) or a disconnected slot falls back to the base picture.
  const TX_PICTOGRAMS = { "DJI Mic Mini 2": "mic-mini-2" };
  function txPictogram(tx, protocolVersion) {
    if (!tx) return null;
    if (tx.product_name) return `${TX_PICTOGRAMS[tx.product_name] ?? device.pictogram_key}-tx`;
    if (protocolVersion === 2) return null;
    return `${device.pictogram_key}-tx`;
  }

  // Noise Cancel's three settings render as one combined element (see
  // NoiseCancelControl) rather than three separate rows. Collapse them into
  // a single placeholder, at the position the first of them would have
  // occupied, before grouping/columning.
  const NC_IDS = new Set(["noise-cancel", "noise-cancel-power", "noise-cancel-button"]);
  // Settings addressed to one specific TX (rather than the receiver or a
  // broadcast to both) don't fit this shared list at all — they're rendered
  // on their own TX card instead (see the Voice Tone control below).
  const TX_TARGETED_IDS = new Set(["voice-tone"]);
  const renderSettings = $derived.by(() => {
    const out = [];
    let ncInserted = false;
    for (const s of settings) {
      if (TX_TARGETED_IDS.has(s.id)) continue;
      if (NC_IDS.has(s.id)) {
        if (!ncInserted) {
          out.push({ id: "__nc-group", group: s.group, __ncComposite: true });
          ncInserted = true;
        }
        continue;
      }
      out.push(s);
    }
    return out;
  });

  const settingsById = $derived(Object.fromEntries(settings.map((s) => [s.id, s])));

  // Cluster settings by their declared group, preserving first-seen order.
  const groups = $derived.by(() => {
    const map = new Map();
    for (const s of renderSettings) {
      if (!map.has(s.group)) map.set(s.group, []);
      map.get(s.group).push(s);
    }
    return [...map.entries()];
  });

  // JS masonry: distribute group cards into balanced columns ourselves rather
  // than using CSS multi-column, which paints card shadows across column
  // boundaries. Each card lands in the currently-shortest column.
  let groupsWidth = $state(0);
  const MIN_COL = 260;
  const COL_GAP = 16;
  const colCount = $derived(
    Math.max(1, Math.min(groups.length, Math.floor((groupsWidth + COL_GAP) / (MIN_COL + COL_GAP)))),
  );
  const columns = $derived.by(() => {
    const cols = Array.from({ length: colCount }, () => []);
    const heights = new Array(colCount).fill(0);
    for (const [name, items] of groups) {
      let min = 0;
      for (let i = 1; i < colCount; i++) if (heights[i] < heights[min]) min = i;
      cols[min].push([name, items]);
      heights[min] += items.length + 1.5; // ~header + rows, as a height estimate
    }
    return cols;
  });

  const labelById = $derived(Object.fromEntries(settings.map((s) => [s.id, s.label])));

  // Why a setting is locked, or null if it is freely settable.
  function lockReason(s) {
    if (status.protocol_version === 1 && s.v1_command == null) {
      return "Requires protocol v2, please update your firmware";
    }
    if (s.id === "noise-cancel" && status.nc_enabled === false) {
      const toggle = labelById["noise-cancel-power"];
      return toggle
        ? `Turn on "${toggle}" first`
        : "Turn on Noise Cancel using the button on the transmitter";
    }
    for (const other of s.exclusive_with ?? []) {
      if (other === "stereo") {
        if (values.stereo === "stereo") {
          return `Switch "${labelById.stereo ?? "Audio Channels"}" to Mono first`;
        }
      } else if (values[other] === "on") {
        return `Turn off ${labelById[other] ?? other} first`;
      }
    }
    return null;
  }
</script>

<div class="panel">
  <header class="head">
    <span class="picto"><DevicePicto pictogram={`${device.pictogram_key}-rx`} size={46} /></span>
    <div class="titles">
      <h1>{device.model_name}</h1>
      <span class="sub">{status.rx?.serial ?? device.id}</span>
    </div>
    <span class="pill" class:on={status.connected}>
      <span class="pill-dot"></span>
      {status.connected ? "Connected" : "No signal"}
    </span>
  </header>

  <section class="cards">
    <div class="card">
      <div class="card-head">
        <span class="card-title">Receiver</span>
        <span class="card-picto">
          <DevicePicto pictogram={`${device.pictogram_key}-rx`} size={36} />
        </span>
      </div>
      {#if status.rx}
        <div class="kv"><span>Serial</span><b>{orUnknown(status.rx.serial)}</b></div>
        <div class="kv"><span>Firmware</span><b>{orUnknown(status.rx.firmware)}</b></div>
      {:else}
        <div class="muted">No data</div>
      {/if}
      {#if status.protocol_version}
        <div class="kv">
          <span>Protocol</span>
          <span class="proto-pill" class:v1={status.protocol_version === 1} class:v2={status.protocol_version === 2}>
            v{status.protocol_version}
          </span>
        </div>
      {/if}
      {#if status.gain_dial != null}
        <div class="kv">
          <span>Gain</span>
          <span class="gain-chip" class:zero={status.gain_dial === 0}>{formatGain(status.gain_dial)} dB</span>
        </div>
      {/if}
    </div>

    {#each status.tx as tx, i}
      {@const picto = txPictogram(tx, status.protocol_version)}
      <div class="card" class:disconnected={!tx}>
        <div class="card-head">
          <span class="card-title">
            <span class="tx-badge">TX{i + 1}</span>
            <span>{tx?.product_name ?? "Transmitter"}</span>
            {#if tx}
              <BatteryIcon value={tx.battery} charging={!!tx.charging} />
            {/if}
          </span>
          {#if picto}
            <span class="card-picto">
              <DevicePicto pictogram={picto} size={34} />
            </span>
          {/if}
        </div>
        {#if tx}
          <div class="kv"><span>Serial</span><b>{orUnknown(tx.serial)}</b></div>
          <div class="kv"><span>Firmware</span><b>{orUnknown(tx.firmware)}</b></div>
          <div class="level">
            <div class="lvl-row">
              <span class="lvl-label">Input Level</span>
            </div>
            <div class="meter">
              <div class="unfilled" style="left:{levelPct(tx.level)}%"></div>
            </div>
          </div>
          {#if tx.product_name === "DJI Mic Mini 2" && status.protocol_version === 2 && settingsById["voice-tone"]}
            <div class="tone">
              <SettingControl
                setting={settingsById["voice-tone"]}
                value={tx.voice_tone ?? null}
                pending={!!pendingTx[i]}
                compact
                onchange={(settingId, value) => onchangeTx?.(i, settingId, value)}
              />
            </div>
          {/if}
        {:else}
          <div class="muted-box">Disconnected</div>
        {/if}
      </div>
    {/each}
  </section>

  <section class="settings-head">
    <span class="settings-title">Settings</span>
  </section>

  <section class="groups" bind:clientWidth={groupsWidth}>
    {#each columns as col, ci (ci)}
      <div class="col">
        {#each col as [name, items] (name)}
          <div class="group">
            <div class="group-title">{name}</div>
            <div class="group-grid">
              {#each items as s (s.id)}
                {#if s.__ncComposite}
                  <NoiseCancelControl
                    power={settingsById["noise-cancel-power"]}
                    mode={settingsById["noise-cancel"]}
                    button={settingsById["noise-cancel-button"]}
                    {values}
                    {pending}
                    {lockReason}
                    compact
                    {onchange}
                  />
                {:else}
                  <SettingControl
                    setting={s}
                    value={values[s.id] ?? null}
                    pending={!!pending[s.id]}
                    locked={!!lockReason(s)}
                    lockReason={lockReason(s) ?? ""}
                    compact
                    {onchange}
                  />
                {/if}
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {/each}
  </section>
</div>

<style>
  .panel {
    flex: 1 1 auto;
    height: 100%;
    overflow-y: auto;
    padding: 12px 24px 44px;
  }
  @media (max-width: 600px) {
    .panel {
      padding: 8px 14px 36px;
    }
  }

  /* Expressive Hero Header Card */
  .head {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 18px;
    padding: 16px 20px;
    border-radius: var(--radius-xl);
    background: var(--md-sys-color-surface-container-low);
  }
  .picto {
    width: 60px;
    height: 60px;
    flex: 0 0 60px;
    border-radius: 20px;
    background: var(--md-sys-color-primary-container);
    color: var(--md-sys-color-on-primary-container);
    display: grid;
    place-items: center;
  }
  .titles {
    flex: 1 1 auto;
    min-width: 0;
  }
  h1 {
    margin: 0;
    font-size: 22px;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--md-sys-color-on-surface);
  }
  .sub {
    display: block;
    margin-top: 2px;
    color: var(--md-sys-color-on-surface-variant);
    font-size: 13px;
    font-weight: 500;
  }
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    font-size: 12.5px;
    font-weight: 700;
    padding: 7px 14px;
    border-radius: var(--radius-pill);
    background: var(--md-sys-color-surface-container-high);
    color: var(--md-sys-color-on-surface-variant);
  }
  .pill-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: currentColor;
  }
  .pill.on {
    color: var(--on-good-container);
    background: var(--good-container);
  }
  .pill.on .pill-dot {
    background: var(--good);
  }

  /* Hardware Unit Cards */
  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 12px;
    margin-bottom: 24px;
  }
  .card {
    background: var(--md-sys-color-surface-container-low);
    border-radius: var(--radius-xl);
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
  }
  .card.disconnected {
    background: color-mix(in srgb, var(--md-sys-color-surface-container-low) 65%, transparent);
  }
  .card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    margin-bottom: 12px;
    min-height: 44px;
  }
  .card-title {
    display: inline-flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
    font-size: 13px;
    font-weight: 700;
    color: var(--md-sys-color-on-surface);
  }
  .tx-badge {
    font-size: 11px;
    font-weight: 800;
    padding: 3px 9px;
    border-radius: var(--radius-pill);
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
    letter-spacing: 0.03em;
  }
  .card-picto {
    width: 44px;
    height: 44px;
    flex: 0 0 44px;
    border-radius: 14px;
    background: var(--md-sys-color-surface-container);
    display: grid;
    place-items: center;
  }

  /* Protocol & Gain Chips */
  .proto-pill {
    font-size: 11px;
    font-weight: 700;
    padding: 2px 10px;
    border-radius: var(--radius-pill);
    background: var(--md-sys-color-surface-container-high);
    color: var(--md-sys-color-on-surface-variant);
  }
  .proto-pill.v1 {
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
  }
  .proto-pill.v2 {
    background: var(--md-sys-color-tertiary-container);
    color: var(--md-sys-color-on-tertiary-container);
  }
  .gain-chip {
    font-size: 12px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    padding: 2px 10px;
    border-radius: var(--radius-pill);
    background: var(--md-sys-color-surface-container-high);
    color: var(--md-sys-color-on-surface);
  }
  .gain-chip.zero {
    background: var(--warn-container);
    color: var(--on-warn-container);
  }

  .kv {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 5px 0;
    font-size: 13.5px;
  }
  .kv span:first-child {
    color: var(--md-sys-color-on-surface-variant);
    font-weight: 500;
  }
  .kv b {
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    color: var(--md-sys-color-on-surface);
  }
  .muted {
    color: var(--md-sys-color-outline);
  }
  .muted-box {
    margin-top: 4px;
    padding: 18px 14px;
    border-radius: var(--radius-md);
    background: var(--md-sys-color-surface-container);
    color: var(--md-sys-color-outline);
    text-align: center;
    font-size: 13px;
    font-weight: 500;
  }

  /* Expressive Audio Level Meter */
  .level {
    margin-top: 10px;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    background: var(--md-sys-color-surface-container);
  }
  .lvl-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 6px;
  }
  .lvl-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--md-sys-color-on-surface-variant);
  }
  .meter {
    position: relative;
    height: 12px;
    border-radius: var(--radius-pill);
    overflow: hidden;
    background: linear-gradient(
      90deg,
      #2ea043 0%,
      #57b846 38%,
      #e0b400 70%,
      #e8863a 86%,
      #e5484d 100%
    );
  }
  .unfilled {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
    background: var(--md-sys-color-surface-container-highest);
    transition: left 0.08s linear;
  }
  .tone {
    margin-top: 8px;
  }

  /* Settings Section Header */
  .settings-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
    padding: 0 4px;
  }
  .settings-title {
    font-size: 16px;
    font-weight: 700;
    letter-spacing: -0.01em;
    color: var(--md-sys-color-on-surface);
  }

  /* Material 3 Expressive Grouped Columns */
  .groups {
    display: flex;
    align-items: flex-start;
    gap: 16px;
  }
  .col {
    flex: 1 1 0;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .group {
    display: flex;
    flex-direction: column;
  }
  .group-title {
    font-size: 12.5px;
    font-weight: 700;
    letter-spacing: 0.02em;
    color: var(--md-sys-color-primary);
    margin: 0 0 8px 10px;
  }
  .group-grid {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
</style>
