# Setup & Development Guide for Linux Helu

## Dependencies
You will need Rust installed on your machine (`rustup` recommended) and the build dependencies for Tauri and OpenCV/v4l2. On Ubuntu/Debian:
```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libv4l-dev clang libclang-dev llvm
```

## Running the Mock Environment Locally
To test the D-Bus Daemon and UI interaction without touching your actual system PAM, you can run the mock system on your Session D-Bus.

1. **Build and start the Daemon (`helud`)**
   In one terminal, start the daemon with hardware mocks enabled:
   ```bash
   cargo run --bin helud -- --mock --bus session
   ```
   *The daemon will register itself as `net.helu.Auth` on the session bus.*

2. **Start the Frontend UI (`helu-ui`)**
   In another terminal, launch the GTK4 UI:
   ```bash
   cargo run --bin helu-ui -- --mock
   ```
   *The UI will start hidden and listen for D-Bus auth signals from the daemon.*

3. **Test the Flow using CLI (`helu-cli`)**
   In a third terminal, use the CLI tool to request an authentication:
   ```bash
   cargo run --bin helu-cli -- test --method face
   ```
   *Watch the UI pop up, "scan" your mock face, and report success back to the daemon!*

4. **Run the Setup Utility (`helu-setup`)**
   To configure the system and enroll faces, you can run the separate Tauri setup app:
   ```bash
   cd helu-setup
   npm install
   npm run tauri dev
   ```

## Real PAM Integration (Advanced)
If you wish to test `pam_helu.so` in real scenarios, you need to configure D-Bus policy and install the module:

1. **Build the PAM module**
   ```bash
   cargo build --package pam_helu --release
   ```
   *The resulting `.so` file will be at `target/release/libpam_helu.so`.*
2. **Install to system PAM directory**
   ```bash
   sudo cp target/release/libpam_helu.so /lib/security/pam_helu.so
   ```
3. **Configure System D-Bus Policy**
   Copy `dist/helu.policy` to `/etc/dbus-1/system.d/net.helu.Auth.conf` and restart D-Bus.
4. **Edit PAM Stack**
   Add `auth sufficient pam_helu.so` to `/etc/pam.d/sudo` (test with caution!).

## Fetching the Real ONNX Model
When running on real hardware (no `--mock`), the face pipeline will attempt to load a model.
By default, the daemon looks for `/usr/share/helu/models/mobilefacenet.onnx`.
You can override this in `/etc/helu/helu.toml` or `~/.config/helu/helu.toml` under `[face].model_path`. Download the `mobilefacenet.onnx` from the canonical InsightFace repository and place it in the configured path.
