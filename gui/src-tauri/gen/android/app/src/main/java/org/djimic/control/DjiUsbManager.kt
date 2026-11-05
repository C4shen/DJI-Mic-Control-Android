package org.djimic.control

import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.hardware.usb.UsbConstants
import android.hardware.usb.UsbDevice
import android.hardware.usb.UsbDeviceConnection
import android.hardware.usb.UsbEndpoint
import android.hardware.usb.UsbInterface
import android.hardware.usb.UsbManager
import android.os.Build
import android.util.Log
import androidx.core.content.ContextCompat
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicBoolean

object DjiUsbManager {
    private const val TAG = "DjiUsbManager"
    private const val ACTION_USB_PERMISSION = "org.djimic.control.USB_PERMISSION"

    private const val DJI_VID = 0x2ca3
    private const val MIC_MINI_PID = 0x4011
    private const val VENDOR_INTERFACE_ID = 6
    private const val EP_BULK_IN = 0x86
    private const val EP_BULK_OUT = 0x06

    init {
        try {
            System.loadLibrary("djimic_gui_lib")
        } catch (t: Throwable) {
            Log.w(TAG, "Native library not yet loaded in init: ${t.message}")
        }
    }

    @JvmStatic
    external fun nativeOnProbe(present: Int, accessible: Int, permissionIssue: Boolean): Boolean

    @JvmStatic
    external fun nativeOnDeviceConnected(deviceId: String, vid: Int, pid: Int): Boolean

    @JvmStatic
    external fun nativeOnDeviceDisconnected(deviceId: String)

    @JvmStatic
    external fun nativeFeedBytes(deviceId: String, data: ByteArray, len: Int)

    @JvmStatic
    external fun nativeNextPacket(deviceId: String): ByteArray?

    @JvmStatic
    external fun nativeCompletePacket(deviceId: String, ok: Boolean, errMsg: String?)

    private class UsbSession(
        val deviceId: String,
        val deviceName: String,
        val connection: UsbDeviceConnection,
        val usbInterface: UsbInterface,
        val epIn: UsbEndpoint,
        val epOut: UsbEndpoint,
    ) {
        val running = AtomicBoolean(true)
        var readerThread: Thread? = null
        var writerThread: Thread? = null

        fun close() {
            if (!running.compareAndSet(true, false)) return
            try {
                nativeOnDeviceDisconnected(deviceId)
            } catch (t: Throwable) {
                Log.w(TAG, "Error notifying disconnect for $deviceId", t)
            }
            try {
                connection.releaseInterface(usbInterface)
            } catch (_: Throwable) {
            }
            try {
                connection.close()
            } catch (_: Throwable) {
            }
        }
    }

    private val started = AtomicBoolean(false)
    private val sessions = ConcurrentHashMap<String, UsbSession>()
    private val promptedDevices = ConcurrentHashMap.newKeySet<String>()
    private var appContext: Context? = null

    private val usbReceiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            when (intent.action) {
                UsbManager.ACTION_USB_DEVICE_ATTACHED -> {
                    scanNow()
                }
                UsbManager.ACTION_USB_DEVICE_DETACHED -> {
                    val dev = getParcelableDevice(intent)
                    if (dev != null) {
                        val id = deviceKey(dev)
                        promptedDevices.remove(dev.deviceName)
                        sessions.remove(id)?.close()
                    }
                    scanNow()
                }
                ACTION_USB_PERMISSION -> {
                    val granted = intent.getBooleanExtra(UsbManager.EXTRA_PERMISSION_GRANTED, false)
                    val dev = getParcelableDevice(intent)
                    Log.i(TAG, "USB permission result for ${dev?.deviceName}: granted=$granted")
                    scanNow()
                }
            }
        }
    }

    @Suppress("DEPRECATION")
    private fun getParcelableDevice(intent: Intent): UsbDevice? {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            intent.getParcelableExtra(UsbManager.EXTRA_DEVICE, UsbDevice::class.java)
        } else {
            intent.getParcelableExtra(UsbManager.EXTRA_DEVICE)
        }
    }

    fun start(context: Context) {
        appContext = context.applicationContext
        if (!started.compareAndSet(false, true)) {
            scanNow()
            return
        }

        val filter = IntentFilter().apply {
            addAction(UsbManager.ACTION_USB_DEVICE_ATTACHED)
            addAction(UsbManager.ACTION_USB_DEVICE_DETACHED)
            addAction(ACTION_USB_PERMISSION)
        }
        ContextCompat.registerReceiver(
            appContext!!,
            usbReceiver,
            filter,
            ContextCompat.RECEIVER_EXPORTED
        )

        Thread({
            while (true) {
                try {
                    scanBus()
                } catch (t: Throwable) {
                    Log.e(TAG, "Error during USB bus scan", t)
                }
                try {
                    Thread.sleep(500)
                } catch (_: InterruptedException) {
                    break
                }
            }
        }, "dji-usb-scanner").apply {
            isDaemon = true
            start()
        }
    }

    fun scanNow() {
        Thread({
            try {
                scanBus()
            } catch (t: Throwable) {
                Log.e(TAG, "Immediate USB scan failed", t)
            }
        }, "dji-usb-scan-once").start()
    }

    private fun isSupported(device: UsbDevice): Boolean {
        return device.vendorId == DJI_VID && device.productId == MIC_MINI_PID
    }

    private fun deviceKey(device: UsbDevice): String {
        return "usb-${device.deviceName.replace('/', '-')}"
    }

    @Synchronized
    private fun scanBus() {
        val ctx = appContext ?: return
        val usbManager = ctx.getSystemService(Context.USB_SERVICE) as? UsbManager ?: return
        val currentDevices = usbManager.deviceList ?: emptyMap()

        // Clean up disconnected sessions
        val activeKeys = currentDevices.values.filter { isSupported(it) }.map { deviceKey(it) }.toSet()
        for ((key, session) in sessions.entries.toList()) {
            if (!activeKeys.contains(key) || !session.running.get()) {
                sessions.remove(key)?.close()
            }
        }

        var present = 0
        var permissionIssue = false
        val unpermitted = mutableListOf<UsbDevice>()

        for (device in currentDevices.values) {
            if (!isSupported(device)) continue
            present += 1
            val key = deviceKey(device)

            if (!usbManager.hasPermission(device)) {
                permissionIssue = true
                unpermitted.add(device)
                if (promptedDevices.add(device.deviceName)) {
                    requestUsbPermission(ctx, usbManager, device)
                }
                continue
            }

            if (!sessions.containsKey(key)) {
                openSession(usbManager, key, device)
            }
        }

        val forcePrompt = try {
            nativeOnProbe(present, sessions.size, permissionIssue)
        } catch (t: Throwable) {
            false
        }

        if (forcePrompt && unpermitted.isNotEmpty()) {
            for (dev in unpermitted) {
                requestUsbPermission(ctx, usbManager, dev)
            }
        }
    }

    private fun requestUsbPermission(ctx: Context, usbManager: UsbManager, device: UsbDevice) {
        try {
            val intent = Intent(ACTION_USB_PERMISSION).setPackage(ctx.packageName)
            val flags = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                PendingIntent.FLAG_MUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
            } else {
                PendingIntent.FLAG_UPDATE_CURRENT
            }
            val pi = PendingIntent.getBroadcast(ctx, 0, intent, flags)
            usbManager.requestPermission(device, pi)
        } catch (t: Throwable) {
            Log.e(TAG, "Failed to request USB permission", t)
        }
    }

    private fun openSession(usbManager: UsbManager, key: String, device: UsbDevice) {
        val intf = (0 until device.interfaceCount)
            .map { device.getInterface(it) }
            .firstOrNull { it.id == VENDOR_INTERFACE_ID }
            ?: run {
                Log.w(TAG, "Device ${device.deviceName} has no interface $VENDOR_INTERFACE_ID")
                return
            }

        var epIn: UsbEndpoint? = null
        var epOut: UsbEndpoint? = null
        for (i in 0 until intf.endpointCount) {
            val ep = intf.getEndpoint(i)
            if (ep.address == EP_BULK_IN ||
                (ep.direction == UsbConstants.USB_DIR_IN && ep.type == UsbConstants.USB_ENDPOINT_XFER_BULK)
            ) {
                epIn = ep
            } else if (ep.address == EP_BULK_OUT ||
                (ep.direction == UsbConstants.USB_DIR_OUT && ep.type == UsbConstants.USB_ENDPOINT_XFER_BULK)
            ) {
                epOut = ep
            }
        }

        val inEndpoint = epIn ?: run {
            Log.w(TAG, "Missing Bulk IN endpoint on interface $VENDOR_INTERFACE_ID")
            return
        }
        val outEndpoint = epOut ?: run {
            Log.w(TAG, "Missing Bulk OUT endpoint on interface $VENDOR_INTERFACE_ID")
            return
        }

        val conn = usbManager.openDevice(device) ?: run {
            Log.w(TAG, "usbManager.openDevice returned null for ${device.deviceName}")
            return
        }

        if (!conn.claimInterface(intf, true)) {
            Log.w(TAG, "Failed to claim interface $VENDOR_INTERFACE_ID on ${device.deviceName}")
            conn.close()
            return
        }

        val accepted = try {
            nativeOnDeviceConnected(key, device.vendorId, device.productId)
        } catch (t: Throwable) {
            Log.e(TAG, "nativeOnDeviceConnected failed", t)
            false
        }
        if (!accepted) {
            conn.releaseInterface(intf)
            conn.close()
            return
        }

        val session = UsbSession(key, device.deviceName, conn, intf, inEndpoint, outEndpoint)
        sessions[key] = session

        val reader = Thread({
            val buf = ByteArray(1024)
            while (session.running.get()) {
                val n = try {
                    conn.bulkTransfer(inEndpoint, buf, buf.size, 500)
                } catch (t: Throwable) {
                    -1
                }
                if (n > 0) {
                    try {
                        nativeFeedBytes(key, buf, n)
                    } catch (t: Throwable) {
                        Log.e(TAG, "nativeFeedBytes error", t)
                    }
                } else if (n < 0) {
                    // Check if the device is still physically attached
                    val stillPresent = usbManager.deviceList?.containsKey(session.deviceName) == true
                    if (!stillPresent) {
                        break
                    }
                }
            }
            sessions.remove(key)?.close()
        }, "dji-usb-rx-$key").apply {
            isDaemon = true
            start()
        }

        val writer = Thread({
            while (session.running.get()) {
                val pkt = try {
                    nativeNextPacket(key)
                } catch (t: Throwable) {
                    null
                } ?: break

                val written = try {
                    conn.bulkTransfer(outEndpoint, pkt, pkt.size, 1000)
                } catch (t: Throwable) {
                    -1
                }
                try {
                    if (written >= 0) {
                        nativeCompletePacket(key, true, null)
                    } else {
                        nativeCompletePacket(key, false, "USB bulkTransfer OUT failed ($written)")
                    }
                } catch (t: Throwable) {
                    Log.e(TAG, "nativeCompletePacket error", t)
                }
            }
        }, "dji-usb-tx-$key").apply {
            isDaemon = true
            start()
        }

        session.readerThread = reader
        session.writerThread = writer
        Log.i(TAG, "Opened DJI Mic session $key on ${device.deviceName}")
    }
}
