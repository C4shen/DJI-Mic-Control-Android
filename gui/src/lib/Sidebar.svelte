<script>
  import DevicePicto from "./DevicePicto.svelte";
  import BatteryIcon from "./BatteryIcon.svelte";

  let { devices = [], selected = null, onselect } = $props();
</script>

<aside
  class="sidebar"
  onclick={(e) => {
    /* Clicking anywhere that isn't a device row (the header, gaps between
       rows, or empty space below the list) deselects the current device. */
    if (!e.target.closest(".device")) onselect?.(null);
  }}
>
  <div class="sidebar-head">Devices</div>

  {#if devices.length === 0}
    <div class="empty">No devices connected</div>
  {:else}
    <ul class="device-list">
      {#each devices as d (d.id)}
        <li>
          <button
            class="device"
            class:active={d.id === selected}
            onclick={() => onselect?.(d.id)}
          >
            <span class="picto-wrap">
              <DevicePicto pictogram={`${d.pictogram_key}-rx`} size={36} />
            </span>
            <span class="labels">
              <span class="name">{d.model_name}</span>
              <span class="serial">{d.rx_serial ?? d.id}</span>
              {#if d.tx?.some((t) => t)}
                <span class="tx-batteries">
                  {#each d.tx as tx, i}
                    {#if tx}
                      <span class="tx-battery">
                        <span class="tx-battery-label">TX{i + 1}</span>
                        <BatteryIcon value={tx.battery} charging={!!tx.charging} size={16} />
                      </span>
                    {/if}
                  {/each}
                </span>
              {/if}
            </span>
            <span
              class="dot"
              class:connected={d.connected}
              title={d.connected ? "Connected" : "Connecting…"}
            ></span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</aside>

<style>
  .sidebar {
    width: 272px;
    height: 100%;
    background: var(--md-sys-color-surface-container-low);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    padding: 8px 12px 16px;
  }
  .sidebar-head {
    padding: 14px 16px 10px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
    color: var(--md-sys-color-on-surface-variant);
  }
  .empty {
    padding: 12px 16px;
    color: var(--md-sys-color-outline);
    font-size: 13px;
  }
  .device-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .device {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border: none;
    border-radius: var(--radius-lg);
    background: transparent;
    color: var(--md-sys-color-on-surface);
    text-align: left;
    transition:
      background 0.2s var(--m3-standard),
      border-radius 0.25s var(--m3-spring-spatial),
      transform 0.16s var(--m3-spring-fast);
  }
  .device:hover {
    background: var(--md-sys-color-surface-container);
  }
  .device:active {
    transform: scale(0.98);
    border-radius: var(--radius-md);
  }
  .device.active {
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
    border-radius: 26px;
  }
  .picto-wrap {
    flex: 0 0 44px;
    height: 44px;
    border-radius: 14px;
    background: var(--md-sys-color-surface-container-high);
    display: grid;
    place-items: center;
    transition: background 0.2s;
  }
  .device.active .picto-wrap {
    background: var(--md-sys-color-surface-container-lowest);
  }
  .labels {
    flex: 1 1 auto;
    min-width: 0;
    display: flex;
    flex-direction: column;
    line-height: 1.3;
  }
  .name {
    font-weight: 700;
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .serial {
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .tx-batteries {
    display: flex;
    gap: 8px;
    margin-top: 6px;
  }
  .tx-battery {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 2px 7px;
    border-radius: var(--radius-pill);
    background: var(--md-sys-color-surface-container-high);
  }
  .device.active .tx-battery {
    background: var(--md-sys-color-surface-container-lowest);
  }
  .tx-battery-label {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.03em;
    color: var(--md-sys-color-on-surface-variant);
  }
  .dot {
    flex: 0 0 auto;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--md-sys-color-outline);
  }
  .dot.connected {
    background: var(--good);
    box-shadow: 0 0 0 4px color-mix(in srgb, var(--good) 22%, transparent);
  }
</style>
