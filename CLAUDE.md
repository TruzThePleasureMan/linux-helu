# Linux Helu 🐧👋

A real biometric authentication system for Linux. Born from a meme. Died a standard.

## Purpose and Scope
Linux Helu provides a unified, production-grade biometric authentication experience for Linux desktops. It supports Face Recognition, Fingerprint (via `fprintd`), FIDO2, and PIN. The project carries the funny and self-aware personality of a "Windows Hello" parody, while fundamentally being a robust D-Bus daemon and PAM module combination designed as a serious replacement for older technologies like RADIUS.

## Workspace Crates
- **`helud`**: The core daemon running on D-Bus. Handles auth requests, configuration, and biometric verification logic.
- **`pam_helu`**: A C-compatible PAM library (`pam_sm_authenticate`) that forwards auth requests to `helud`.
- **`helu-cli`**: Command-line tool for enrolling biometric data and checking system status.
- **`helu-ui`**: GTK4 system overlay, layer-shell, Rust only, no web tech. Provides the lockscreen and auth UI.
- **`helu-setup`**: Tauri + Svelte, normal app window, enrollment only.
- **`helu-server`**: A lightweight network server that functions as a RADIUS replacement, issuing biometric challenges and verifying JWTs.

## Architecture & Auth Flow
```text
[PAM Trigger: sudo, login, etc]
       |
       v
   pam_helu.so   ----(D-Bus Authenticate(username, "auto"))---->   helud (Daemon)
                                                                     |
                                                                     +--> Loads Config / User Data
                                                                     |
   [helu-ui] <-----(D-Bus AuthRequested Signal)----------------------+
     (Shows    <-----(D-Bus AuthStateUpdates / Fallbacks)------------+
      UI)                                                            |
                                                                     +--> Face Recognition Pipeline (ONNX)
                                                                     +--> Fingerprint (fprintd)
                                                                     +--> FIDO2
                                                                     +--> PIN verification
                                                                     |
   pam_helu.so   <-------(D-Bus Returns Success/Fail)----------------+
       |
       v
  [Access Granted / Denied]
```

## D-Bus Interface (`net.helu.Auth`)
Exposed by `helud`.
- **Methods:**
  - `Authenticate(username: String, method: String) -> (success: Bool, message: String)`
  - `AuthenticateWithCredential(username: String, method: String, credential: String) -> (success: Bool, message: String)`
  - `Enroll(username: String, method: String) -> (success: Bool)`
  - `ListMethods(username: String) -> (methods: Array<String>)`
  - `Status() -> (daemon_version: String, loaded_methods: Array<String>)`
- **Signals:**
  - `AuthRequested(username: String, method: String)`
  - `AuthSuccess(username: String, method: String)`
  - `AuthFailure(username: String, reason: String)`

## Key Design Decisions
- **ONNX Runtime (`ort`)**: Chosen over PyTorch/TensorFlow C++ for a lightweight, dependency-free CPU inference pipeline. Allows bundling mobile face models directly without CUDA bloat.
- **zbus**: The canonical Rust D-Bus library. Simple, safe, and macro-driven.
- **Tauri**: Provides a cross-platform, web-tech UI (Svelte) for the enrollment app (`helu-setup`) with a Rust backend that can natively speak D-Bus.
- **GTK4/layer-shell**: Used for `helu-ui` to allow it to be a true system overlay behaving like a lockscreen on both X11 and Wayland.

## Runtime Wayland Detection
Note the runtime Wayland detection pattern as a key convention in the codebase. `gtk4_layer_shell::is_supported()` must always be checked before calling any layer shell API.

## Running Locally for Development
You can run the stack using session D-Bus and mocked hardware:
1. Ensure your config uses `bus = "session"` and `--mock` hardware flags.
2. Start the daemon: `cargo run --bin helud`
3. Run the setup UI: `cd helu-setup && npm run tauri dev`
4. Run the auth UI (mocked): `cargo run --bin helu-ui -- --mock`
5. Test with CLI: `cargo run --bin helu-cli test`

## Known Edges & TODOs
- **Face Model**: You must provide `mobilefacenet.onnx` from the InsightFace repository and configure its path. It is not bundled in the repo.
- **`HELU_MOCK_PIN`**: Removed. PIN authentication fallback in PAM handles verification locally and passes the PIN over D-Bus via `AuthenticateWithCredential`.
- **UI Startup Grace Period**: Previously `pam_helu` had a race condition firing before `helu-ui` was fully awake. This is now mitigated via a 3-second UI readiness check and polling of the session bus.

## Coding Conventions
- Use `anyhow` for daemon/server error handling, but implement specific error codes where D-Bus needs them.
- Follow Rust standard styling (`cargo fmt`, `cargo clippy`).
