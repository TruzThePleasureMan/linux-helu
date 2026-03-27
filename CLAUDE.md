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

### `helu-server` Details
`helu-server` provides a lightweight network auth replacement. It connects natively to `helud` via local D-Bus to issue biometric challenges, verifying responses by directly capturing return values of `Authenticate` method calls.
Its API endpoints include:
- `POST /auth/challenge`: Creates an authentication challenge for a user and biometric method, stored in the postgres backend with a short expiration.
- `POST /auth/verify`: Using the challenge data, initiates blocking D-Bus calls via `helud` allowing the client to verify biometrics. Returns a valid signed JWT (HS256 or RS256).
- `POST /auth/direct`: A single step alternative endpoint that operates similar to `/verify` but receives both username and credentials (`pin`) in one step.
- `POST /admin/*`: Admin APIs configured via Argon2 API key headers to enable creating API keys or configuring authentication nodes/users.

## Biometric Authentication Methods and Fallback Chain
helud automatically handles degradation based on availability and enrollment across the following methods:
1. **Face Recognition**: Primary method. Checked for ONNX model presence and `/dev/video*`.
2. **Fingerprint**: Uses `fprintd` over D-Bus (`net.reactivated.Fprint`). Falls back cleanly if daemon is missing or no device is registered.
3. **FIDO2**: Uses CTAP2 natively over `hidraw` (no browser required). Checks for `/dev/hidraw*` proxy presence. Requires physical touch.
4. **PIN**: Local encrypted fallback mechanism. Always available as the final link in the chain.

The `Authenticate("auto")` call iterates this chain synchronously in order, terminating and returning success on the first match.

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

## Known Edges
- **`HELU_MOCK_PIN`**: Removed. PIN authentication fallback in PAM handles verification locally and passes the PIN over D-Bus via `AuthenticateWithCredential`.
- **UI Startup Grace Period**: Previously `pam_helu` had a race condition firing before `helu-ui` was fully awake. This is now mitigated via a 3-second UI readiness check and polling of the session bus.

## TPM2 Crypto Model
Face embeddings are strictly encrypted locally with AES-256-GCM. The AES key is sealed into the system's TPM2 hardware under the owner hierarchy, using a policy bound to PCRs 0, 1, and 7. If hardware TPM is unavailable, `helud` falls back to software derivation via HKDF-SHA256 of the machine ID and a random salt (though this compromises absolute hardware security guarantees). See `ARCHITECTURE.md` for more details.

## Coding Conventions
- Use `anyhow` for daemon/server error handling, but implement specific error codes where D-Bus needs them.
- Follow Rust standard styling (`cargo fmt`, `cargo clippy`).
