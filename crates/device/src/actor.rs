//! Per-device background task.
//!
//! Each connected device is driven by one thread that owns its USB interface.
//! The thread continuously reads bulk-IN frames (decoding heartbeats into a
//! shared [`DeviceStatus`]) and, interleaved, writes command packets requested
//! over a channel. Because `nusb` transfers are queued rather than blocking,
//! reads and writes coexist without the global lock a synchronous USB API needs.

use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(not(target_os = "android"))]
use futures_lite::future;
#[cfg(not(target_os = "android"))]
use nusb::transfer::RequestBuffer;
#[cfg(not(target_os = "android"))]
use nusb::Interface;
use protocol::packet::Dialect;
use protocol::{packet, DeviceModel, DeviceStatus};

use crate::error::{DeviceError, Result};

fn hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x} ")).collect()
}

/// A request to change one setting, with a channel to report the outcome.
pub struct Command {
    pub setting_id: String,
    pub value: String,
    /// The 0-based transmitter slot to address, for settings that target one
    /// specific TX rather than broadcasting (see `V2Target::Tx`). `None` for
    /// every other setting.
    pub tx: Option<usize>,
    pub resp: std::sync::mpsc::Sender<Result<()>>,
}

/// The channels and shared state a running device exposes to the manager.
pub struct Actor {
    pub cmd_tx: async_channel::Sender<Command>,
    pub status: Arc<Mutex<DeviceStatus>>,
    pub alive: Arc<AtomicBool>,
    /// Unix-millis timestamp of the last decoded heartbeat (0 = none yet).
    pub last_seen: Arc<AtomicU64>,
}

/// Milliseconds since the Unix epoch.
pub fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// What the event loop woke up for.
#[cfg(not(target_os = "android"))]
enum Wake {
    Read(nusb::transfer::Completion<Vec<u8>>),
    Command(std::result::Result<Command, async_channel::RecvError>),
}

/// Spawn the background thread for a claimed device and return its handles.
#[cfg(not(target_os = "android"))]
pub fn spawn(model: &'static dyn DeviceModel, iface: Interface) -> Actor {
    let (cmd_tx, cmd_rx) = async_channel::unbounded::<Command>();
    let status = Arc::new(Mutex::new(DeviceStatus::disconnected(model.id())));
    let alive = Arc::new(AtomicBool::new(true));
    let last_seen = Arc::new(AtomicU64::new(0));

    let status_thread = status.clone();
    let alive_thread = alive.clone();
    let last_seen_thread = last_seen.clone();
    std::thread::spawn(move || {
        future::block_on(run(model, iface, cmd_rx, status_thread, last_seen_thread));
        alive_thread.store(false, Ordering::Relaxed);
    });

    Actor { cmd_tx, status, alive, last_seen }
}

/// Handle used by an external USB transport (such as Android's `UsbManager`)
/// to feed incoming Bulk-IN bytes and drain outgoing Bulk-OUT command packets.
pub struct BridgeHandle {
    model: &'static dyn DeviceModel,
    cmd_rx: async_channel::Receiver<Command>,
    status: Arc<Mutex<DeviceStatus>>,
    alive: Arc<AtomicBool>,
    last_seen: Arc<AtomicU64>,
    rx_buf: Mutex<Vec<u8>>,
    /// 1 = Dialect::V1, 2 = Dialect::V2.
    dialect: AtomicU8,
    seq: AtomicU16,
    pending_resp: Mutex<Option<std::sync::mpsc::Sender<Result<()>>>>,
}

impl BridgeHandle {
    /// Feed raw Bulk-IN bytes read from the USB endpoint into the frame decoder.
    pub fn feed_bytes(&self, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        let debug = std::env::var_os("DJIMIC_DEBUG").is_some();
        if debug {
            eprintln!("[bridge-read] len={} data={}", data.len(), hex(data));
        }
        let mut buf = self.rx_buf.lock().unwrap();
        buf.extend_from_slice(data);
        while let Some(frame) = packet::take_frame(&mut buf) {
            if let Some(d) = packet::heartbeat_dialect(&frame) {
                let code = match d {
                    Dialect::V1 => 1,
                    Dialect::V2 => 2,
                };
                self.dialect.store(code, Ordering::Relaxed);
            }
            let previous = self.status.lock().unwrap().clone();
            let new_status = self.model.decode(&previous, &frame);
            if debug {
                eprintln!(
                    "[bridge-frame] kind={:?} decoded={} {}",
                    packet::frame_kind(&frame),
                    new_status.is_some(),
                    hex(&frame),
                );
            }
            if let Some(new_status) = new_status {
                *self.status.lock().unwrap() = new_status;
                self.last_seen.store(now_millis(), Ordering::Relaxed);
            }
        }
    }

    /// Block until the next outgoing command packet is requested, or return
    /// `None` once the device has been disconnected.
    pub fn next_packet(&self) -> Option<Vec<u8>> {
        loop {
            if !self.alive.load(Ordering::Relaxed) {
                return None;
            }
            let cmd = self.cmd_rx.recv_blocking().ok()?;
            let dialect = match self.dialect.load(Ordering::Relaxed) {
                1 => Dialect::V1,
                _ => Dialect::V2,
            };
            let seq = self.seq.fetch_add(1, Ordering::Relaxed);
            match self.model.build_command(seq, dialect, &cmd.setting_id, &cmd.value, cmd.tx) {
                Some(pkt) => {
                    *self.pending_resp.lock().unwrap() = Some(cmd.resp);
                    return Some(pkt);
                }
                None => {
                    let _ = cmd.resp.send(Err(DeviceError::UnknownSetting(cmd.setting_id)));
                }
            }
        }
    }

    /// Report the completion of the Bulk-OUT transfer for the packet returned
    /// by [`next_packet`](Self::next_packet).
    pub fn complete_packet(&self, result: Result<()>) {
        if let Some(resp) = self.pending_resp.lock().unwrap().take() {
            let _ = resp.send(result);
        }
    }

    /// Mark the bridged device as disconnected and wake any blocked writer/caller.
    pub fn disconnect(&self) {
        self.alive.store(false, Ordering::Relaxed);
        self.cmd_rx.close();
        if let Some(resp) = self.pending_resp.lock().unwrap().take() {
            let _ = resp.send(Err(DeviceError::Disconnected));
        }
    }
}

/// Create an [`Actor`] and [`BridgeHandle`] pair for an externally-managed USB
/// connection (used on Android via `UsbManager`).
pub fn spawn_bridged(model: &'static dyn DeviceModel) -> (Actor, Arc<BridgeHandle>) {
    let (cmd_tx, cmd_rx) = async_channel::unbounded::<Command>();
    let status = Arc::new(Mutex::new(DeviceStatus::disconnected(model.id())));
    let alive = Arc::new(AtomicBool::new(true));
    let last_seen = Arc::new(AtomicU64::new(0));

    let bridge = Arc::new(BridgeHandle {
        model,
        cmd_rx,
        status: status.clone(),
        alive: alive.clone(),
        last_seen: last_seen.clone(),
        rx_buf: Mutex::new(Vec::with_capacity(1024)),
        dialect: AtomicU8::new(2),
        seq: AtomicU16::new(1),
        pending_resp: Mutex::new(None),
    });

    (Actor { cmd_tx, status, alive, last_seen }, bridge)
}

#[cfg(not(target_os = "android"))]
async fn run(
    model: &'static dyn DeviceModel,
    iface: Interface,
    cmd_rx: async_channel::Receiver<Command>,
    status: Arc<Mutex<DeviceStatus>>,
    last_seen: Arc<AtomicU64>,
) {
    let ep_in = model.bulk_in();
    let ep_out = model.bulk_out();
    let mut seq: u16 = 1;
    let mut buf: Vec<u8> = Vec::with_capacity(1024);
    let debug = std::env::var_os("DJIMIC_DEBUG").is_some();
    // Learned from the first heartbeat; defaults to v2 since a command sent
    // before any heartbeat arrives is the rare case (heartbeats stream
    // continuously as soon as the device is opened).
    let mut dialect = Dialect::V2;

    loop {
        let read = iface.bulk_in(ep_in, RequestBuffer::new(512));
        let wake = future::or(
            async { Wake::Read(read.await) },
            async { Wake::Command(cmd_rx.recv().await) },
        )
        .await;

        match wake {
            Wake::Read(comp) => {
                if debug {
                    eprintln!(
                        "[read] status={:?} len={} data={}",
                        comp.status,
                        comp.data.len(),
                        hex(&comp.data),
                    );
                }
                if comp.status.is_err() {
                    break; // device unplugged or fatal transfer error
                }
                buf.extend_from_slice(&comp.data);
                while let Some(frame) = packet::take_frame(&mut buf) {
                    if let Some(d) = packet::heartbeat_dialect(&frame) {
                        dialect = d;
                    }
                    let previous = status.lock().unwrap().clone();
                    let new_status = model.decode(&previous, &frame);
                    if debug {
                        eprintln!(
                            "[frame] kind={:?} decoded={} {}",
                            packet::frame_kind(&frame),
                            new_status.is_some(),
                            hex(&frame),
                        );
                    }
                    if let Some(new_status) = new_status {
                        *status.lock().unwrap() = new_status;
                        last_seen.store(now_millis(), Ordering::Relaxed);
                    }
                }
            }
            Wake::Command(Ok(cmd)) => {
                let result = match model.build_command(seq, dialect, &cmd.setting_id, &cmd.value, cmd.tx) {
                    Some(pkt) => {
                        seq = seq.wrapping_add(1);
                        let comp = iface.bulk_out(ep_out, pkt).await;
                        comp.status
                            .map_err(|e| DeviceError::Usb(format!("{e:?}")))
                    }
                    None => Err(DeviceError::UnknownSetting(cmd.setting_id.clone())),
                };
                let _ = cmd.resp.send(result);
            }
            Wake::Command(Err(_)) => break, // manager dropped the sender
        }
    }
}
