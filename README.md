# DJI Mic Control for Android

An open-source, lightweight, and privacy-friendly Android app to control your **DJI Mic Mini** and **DJI Mic Mini 2** wireless microphones over USB-C — designed as a clean, bloat-free alternative to the official DJI Mimo app.

> [!NOTE]
> **Project Attribution & Background**  
> This project is an AI-modified Android version of the original project by **[ShadowBitBasher](https://github.com/ShadowBitBasher/DJI-Mic-Control)** to replace the bloated official DJI Mimo app.

---

## Download & Installation

You can download the pre-built, signed APK directly from this repository:

👉 **[Download DJI-Mic-Control-1.1.0-android-arm64.apk](Release/android/DJI-Mic-Control-1.1.0-android-arm64.apk)**

1. Transfer the `.apk` file to your Android device (or download it directly on your phone).
2. Open the `.apk` file in your phone's file manager to install it (enable "Install unknown apps" if prompted).
3. Plug your DJI Mic Mini / Mic Mini 2 receiver into your phone's USB-C port.
4. When Android prompts *"Allow DJI Mic Control to access DJI Mic Mini?"*, tap **OK**.

---

## Building from Source

### Prerequisites
1. **Rust:** 1.77+ with Android target installed:
   ```bash
   rustup target add aarch64-linux-android
   ```
2. **Android SDK & NDK:** Android SDK with Build-Tools 35+, Platform 35+, and NDK (e.g. `27.2.12479018`).
3. **Java:** JDK 17 or JDK 21.
4. **Node.js:** v18+ with npm.

### Build the Signed APK
Set your Android environment variables and run the build script:

```bash
export ANDROID_HOME="$HOME/Android/Sdk"
export NDK_HOME="$ANDROID_HOME/ndk/27.2.12479018"
export JAVA_HOME="/path/to/jdk"

./build-android.sh
```

The output signed APK will be generated at:
```
Release/android/DJI-Mic-Control-1.1.0-android-arm64.apk
```

---

## Credits & License

- Original desktop implementation & reverse engineering by **[ShadowBitBasher](https://github.com/ShadowBitBasher/DJI-Mic-Control)**.
- Released into the public domain under [The Unlicense](LICENSE).
