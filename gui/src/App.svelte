<script>
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import DevicePanel from "./lib/DevicePanel.svelte";
  import UdevModal from "./lib/UdevModal.svelte";
  import { snapshot, setSetting, setTxSetting, udevHelp, requestUsbPermission } from "./lib/api.js";

  // Custom window chrome. macOS keeps its native traffic lights (via the
  // Overlay title bar); Windows/Linux use our own frameless controls.
  const appWindow = getCurrentWindow();
  const isMac = /Mac/i.test(navigator.userAgent || navigator.platform || "");
  const isMobileUa = /Android|iPhone|iPad/i.test(navigator.userAgent || "");
  let maximized = $state(false);
  async function refreshMax() {
    if (isMobileUa) return;
    try {
      maximized = await appWindow.isMaximized();
    } catch {
      /* ignore */
    }
  }
  const minimizeWin = () => appWindow.minimize();
  const toggleMaxWin = () => appWindow.toggleMaximize();
  const closeWin = () => appWindow.close();

  // Frameless windows lose the compositor's edge-resize borders, so we add our
  // own invisible handles (Linux/Windows; macOS keeps its native resize).
  const startResize = (dir) => (e) => {
    if (e.button === 0) appWindow.startResizeDragging(dir);
  };

  let snap = $state(null);
  let selected = $state(null);
  let pending = $state({});
  let optimistic = $state({}); // values shown immediately, before the device confirms
  // Per-TX-targeted settings (currently just Voice Tone) don't fit the flat
  // `settings`/`optimistic` maps above, since the same setting id can hold a
  // different value per transmitter slot — keyed by 0-based TX index instead.
  let pendingTx = $state({});
  let optimisticVoiceTone = $state({});
  let error = $state(null);

  let showUdev = $state(false);
  let help = $state(null);

  const isAndroid = $derived(snap?.os === "android" || isMobileUa);
  const devices = $derived(snap?.devices ?? []);
  const selectedDevice = $derived(devices.find((d) => d.id === selected) ?? devices[0] ?? null);
  const rawStatus = $derived(selectedDevice ? snap?.status ?? null : null);
  // Merge live values with any not-yet-confirmed local changes for instant feedback.
  const values = $derived({ ...(rawStatus?.settings ?? {}), ...optimistic });
  // Same, for each TX slot's Voice Tone.
  const status = $derived(
    rawStatus && {
      ...rawStatus,
      tx: rawStatus.tx.map((t, i) =>
        t && optimisticVoiceTone[i] !== undefined ? { ...t, voice_tone: optimisticVoiceTone[i] } : t,
      ),
    },
  );
  const linuxAccessIssue = $derived(
    snap?.os === "linux" && snap?.probe?.permission_issue && devices.length === 0,
  );
  const androidAccessIssue = $derived(
    snap?.os === "android" && snap?.probe?.permission_issue && devices.length === 0,
  );

  async function refresh() {
    try {
      const next = await snapshot(selected);
      snap = next;
      // Always select the connected receiver automatically.
      if (!selected && next.devices.length > 0) {
        selected = next.devices[0].id;
      }
      if (selected && !next.devices.some((d) => d.id === selected)) {
        selected = next.devices[0]?.id ?? null;
      }
      // Retire optimistic values the device has now confirmed.
      const confirmed = next.status?.settings ?? {};
      let changed = false;
      const remaining = { ...optimistic };
      for (const k of Object.keys(remaining)) {
        if (confirmed[k] === remaining[k]) {
          delete remaining[k];
          changed = true;
        }
      }
      if (changed) optimistic = remaining;

      // Same, for each TX slot's Voice Tone.
      const confirmedTx = next.status?.tx ?? [];
      let changedTx = false;
      const remainingTx = { ...optimisticVoiceTone };
      for (const k of Object.keys(remainingTx)) {
        if (confirmedTx[k]?.voice_tone === remainingTx[k]) {
          delete remainingTx[k];
          changedTx = true;
        }
      }
      if (changedTx) optimisticVoiceTone = remainingTx;
    } catch (e) {
      error = String(e);
    }
  }

  async function change(settingId, value) {
    const devId = selectedDevice?.id;
    if (!devId) return;
    optimistic = { ...optimistic, [settingId]: value }; // reflect the flip instantly
    pending = { ...pending, [settingId]: true };
    error = null;
    try {
      await setSetting(devId, settingId, value);
      await refresh();
    } catch (e) {
      error = String(e);
      const { [settingId]: _drop, ...rest } = optimistic;
      optimistic = rest;
    } finally {
      const { [settingId]: _p, ...rest } = pending;
      pending = rest;
    }
  }

  async function changeTx(tx, settingId, value) {
    const devId = selectedDevice?.id;
    if (!devId) return;
    // Only Voice Tone exists today; extend this map if a second per-TX
    // setting is added.
    if (settingId === "voice-tone") {
      optimisticVoiceTone = { ...optimisticVoiceTone, [tx]: value }; // reflect instantly
    }
    pendingTx = { ...pendingTx, [tx]: true };
    error = null;
    try {
      await setTxSetting(devId, tx, settingId, value);
      await refresh();
    } catch (e) {
      error = String(e);
      const { [tx]: _drop, ...rest } = optimisticVoiceTone;
      optimisticVoiceTone = rest;
    } finally {
      const { [tx]: _p, ...rest } = pendingTx;
      pendingTx = rest;
    }
  }

  async function openUdev() {
    if (!help) help = await udevHelp();
    showUdev = true;
  }

  async function askAndroidUsbPermission() {
    try {
      await requestUsbPermission();
    } catch (e) {
      error = String(e);
    }
  }

  $effect(() => {
    refresh();
    const timer = setInterval(refresh, 150);
    return () => clearInterval(timer);
  });

  // Keep the maximize/restore icon in sync with the window state.
  $effect(() => {
    refreshMax();
    const onResize = () => refreshMax();
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  });
</script>

<div class="app" class:mac={isMac} class:android={isAndroid}>
  {#if !isMac && !isAndroid}
    <div class="rz rz-n" onmousedown={startResize("North")}></div>
    <div class="rz rz-s" onmousedown={startResize("South")}></div>
    <div class="rz rz-e" onmousedown={startResize("East")}></div>
    <div class="rz rz-w" onmousedown={startResize("West")}></div>
    <div class="rz rz-ne" onmousedown={startResize("NorthEast")}></div>
    <div class="rz rz-nw" onmousedown={startResize("NorthWest")}></div>
    <div class="rz rz-se" onmousedown={startResize("SouthEast")}></div>
    <div class="rz rz-sw" onmousedown={startResize("SouthWest")}></div>
  {/if}

  <div class="topbar" class:mac={isMac} class:android={isAndroid} data-tauri-drag-region>
    <span class="brand" data-tauri-drag-region>DJI Mic Control</span>
    <div class="drag-fill" data-tauri-drag-region></div>

    {#if !isMac && !isAndroid}
      <div class="win-controls">
        <button class="win-btn" onclick={minimizeWin} aria-label="Minimize">
          <svg width="11" height="11" viewBox="0 0 12 12"><path d="M2 6h8" stroke="currentColor" stroke-width="1.2" /></svg>
        </button>
        <button class="win-btn" onclick={toggleMaxWin} aria-label={maximized ? "Restore" : "Maximize"}>
          {#if maximized}
            <svg width="11" height="11" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.1">
              <rect x="3.2" y="1.6" width="6.2" height="6.2" rx="1" /><rect x="1.6" y="3.6" width="6.2" height="6.8" rx="1" fill="var(--bg-panel)" />
            </svg>
          {:else}
            <svg width="11" height="11" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.1"><rect x="2" y="2" width="8" height="8" rx="1" /></svg>
          {/if}
        </button>
        <button class="win-btn close" onclick={closeWin} aria-label="Close">
          <svg width="11" height="11" viewBox="0 0 12 12"><path d="M2.5 2.5l7 7M9.5 2.5l-7 7" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" /></svg>
        </button>
      </div>
    {/if}
  </div>

  {#if linuxAccessIssue}
    <button class="banner" onclick={openUdev}>
      <span class="banner-dot"></span>
      <span>A microphone is connected but not accessible — click to fix USB permissions.</span>
    </button>
  {/if}

  {#if androidAccessIssue}
    <button class="banner" onclick={askAndroidUsbPermission}>
      <span class="banner-dot"></span>
      <span>A microphone is connected — tap here to grant USB access permission.</span>
    </button>
  {/if}

  {#if error}
    <div class="banner err" role="alert">{error}</div>
  {/if}

  <div class="body">
    <main class="main">
      {#if selectedDevice && status}
        <DevicePanel
          device={selectedDevice}
          {status}
          settings={snap.settings}
          {values}
          {pending}
          {pendingTx}
          onchange={change}
          onchangeTx={changeTx}
        />
      {:else}
        <div class="placeholder">
          <div class="ph-card">
            <div class="ph-squircle" aria-hidden="true">
              <svg width="34" height="34" viewBox="0 0 24 24" fill="none">
                <rect x="8.5" y="2.5" width="7" height="12" rx="3.5" stroke="currentColor" stroke-width="2" />
                <path d="M5 10.5a7 7 0 0 0 14 0M12 17.5v4M8.5 21.5h7" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
              </svg>
            </div>
            {#if linuxAccessIssue}
              <h2>Permission needed</h2>
              <p>A supported microphone is connected but can't be accessed.</p>
              <button class="cta" onclick={openUdev}>Show setup instructions</button>
            {:else if androidAccessIssue}
              <h2>USB permission needed</h2>
              <p>A DJI Mic Mini receiver is connected to your phone, but USB access has not been granted yet.</p>
              <button class="cta" onclick={askAndroidUsbPermission}>Grant USB Permission</button>
            {:else}
              <h2>No microphone connected</h2>
              <p>Connect a supported DJI microphone receiver over USB-C to get started.</p>
            {/if}
          </div>
        </div>
      {/if}
    </main>
  </div>
</div>

{#if showUdev && help}
  <UdevModal {help} onclose={() => (showUdev = false)} />
{/if}

<style>
  .app {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--md-sys-color-surface);
    border: 1px solid var(--md-sys-color-outline-variant);
  }
  .app.mac,
  .app.android {
    border: none;
  }
  .app.android {
    padding-bottom: env(safe-area-inset-bottom, 0px);
  }

  /* Invisible resize handles pinned to the window edges/corners. */
  .rz {
    position: fixed;
    z-index: 100;
  }
  .rz-n {
    top: 0;
    left: 8px;
    right: 8px;
    height: 5px;
    cursor: ns-resize;
  }
  .rz-s {
    bottom: 0;
    left: 8px;
    right: 8px;
    height: 5px;
    cursor: ns-resize;
  }
  .rz-e {
    top: 8px;
    bottom: 8px;
    right: 0;
    width: 5px;
    cursor: ew-resize;
  }
  .rz-w {
    top: 8px;
    bottom: 8px;
    left: 0;
    width: 5px;
    cursor: ew-resize;
  }
  .rz-ne {
    top: 0;
    right: 0;
    width: 11px;
    height: 11px;
    cursor: nesw-resize;
  }
  .rz-nw {
    top: 0;
    left: 0;
    width: 11px;
    height: 11px;
    cursor: nwse-resize;
  }
  .rz-se {
    bottom: 0;
    right: 0;
    width: 11px;
    height: 11px;
    cursor: nwse-resize;
  }
  .rz-sw {
    bottom: 0;
    left: 0;
    width: 11px;
    height: 11px;
    cursor: nesw-resize;
  }

  /* Material 3 Expressive Top App Bar */
  .topbar {
    position: relative;
    flex: 0 0 auto;
    height: 44px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 4px 0 8px;
    background: var(--md-sys-color-surface);
    user-select: none;
    z-index: 30;
  }
  .topbar.mac {
    padding-left: 78px;
  }
  .topbar.android {
    height: calc(56px + env(safe-area-inset-top, 0px));
    padding: env(safe-area-inset-top, 0px) 12px 0;
  }
  .topbar.android .brand {
    top: calc(50% + env(safe-area-inset-top, 0px) / 2);
    font-size: 18px;
    font-weight: 700;
  }
  .drag-fill {
    flex: 1 1 auto;
    align-self: stretch;
  }
  .topbar button svg {
    display: block;
  }
  .brand {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    font-weight: 700;
    font-size: 15px;
    letter-spacing: -0.01em;
    color: var(--md-sys-color-on-surface);
    white-space: nowrap;
  }

  .win-controls {
    display: flex;
    align-self: stretch;
    margin-left: 4px;
  }
  .win-btn {
    width: 42px;
    align-self: stretch;
    display: grid;
    place-items: center;
    border: none;
    background: transparent;
    color: var(--md-sys-color-on-surface-variant);
    transition: background 0.14s, color 0.14s;
  }
  .win-btn:hover {
    background: var(--md-sys-color-surface-container-high);
    color: var(--md-sys-color-on-surface);
  }
  .win-btn.close:hover {
    background: #e5484d;
    color: #fff;
  }

  /* Expressive Tonal Alert Banner */
  .banner {
    flex: 0 0 auto;
    margin: 4px 14px 8px;
    display: flex;
    align-items: center;
    gap: 10px;
    text-align: left;
    border: none;
    border-radius: var(--radius-md);
    padding: 12px 16px;
    background: var(--warn-container);
    color: var(--on-warn-container);
    font-size: 13px;
    font-weight: 600;
  }
  .banner-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--warn);
    flex: 0 0 auto;
  }
  .banner.err {
    background: var(--danger-container);
    color: var(--on-danger-container);
  }

  .body {
    flex: 1 1 auto;
    display: flex;
    min-height: 0;
    position: relative;
  }

  .main {
    flex: 1 1 auto;
    min-width: 0;
    display: flex;
  }

  /* Expressive Placeholder Card */
  .placeholder {
    flex: 1 1 auto;
    display: grid;
    place-items: center;
    padding: 24px;
  }
  .ph-card {
    text-align: center;
    max-width: 380px;
    background: var(--md-sys-color-surface-container-low);
    border-radius: var(--radius-xl);
    padding: 32px 28px;
    color: var(--md-sys-color-on-surface-variant);
    display: flex;
    flex-direction: column;
    align-items: center;
  }
  .ph-squircle {
    width: 72px;
    height: 72px;
    border-radius: 22px;
    background: var(--md-sys-color-primary-container);
    color: var(--md-sys-color-on-primary-container);
    display: grid;
    place-items: center;
    margin-bottom: 18px;
  }
  .ph-card h2 {
    margin: 0 0 8px;
    color: var(--md-sys-color-on-surface);
    font-size: 22px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .ph-card p {
    margin: 0 0 22px;
    line-height: 1.5;
    font-size: 14px;
  }
  .cta {
    border: none;
    background: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    padding: 12px 24px;
    border-radius: var(--radius-pill);
    font-size: 14px;
    font-weight: 600;
    letter-spacing: 0.01em;
    transition:
      transform 0.18s var(--m3-spring-fast),
      border-radius 0.22s var(--m3-spring-spatial),
      filter 0.15s;
  }
  .cta:hover {
    filter: brightness(1.06);
  }
  .cta:active {
    transform: scale(0.96);
    border-radius: var(--radius-md);
  }
</style>
