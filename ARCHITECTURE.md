# Linux Helu Architecture

## System Overview

```text
[PAM Trigger: sudo, login, etc]
       |
       v
   pam_helu.so   ----(D-Bus Authenticate(username, "auto"))---->   helud (Daemon)
                                                                     |
                                                                     +--> Loads Config / User Data
                                                                     |
   [helu-ui] <-----(D-Bus AuthRequested Signal)----------------------+
     (Shows    <-----(D-Bus AuthStateChanged / Fallbacks)------------+
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

## Component Responsibilities

- **`helud`**: The central D-Bus daemon running as root. Orchestrates authentication, interacts directly with hardware components (FIDO2, Face, Fingerprint) and performs validation.
- **`pam_helu`**: The PAM module that applications integrate with. It translates PAM requests into D-Bus calls targeting `helud` and handles passing passwords via `AuthenticateWithCredential` when appropriate.
- **`helu-ui`**: The visual frontend (lockscreen) displaying auth state, capturing PIN, and reacting to D-Bus signals from `helud`.
- **`helu-setup`**: A user-facing application for enrolling new biometric factors.
- **`helu-server`**: The enterprise network authentication piece designed as an alternative to RADIUS, acting as an identity provider by bridging remote requests to local D-Bus calls to `helud`.
- **`helu-cli`**: Command-line tool for testing the daemon and checking biometric enrollment status without needing a UI.
- **`helu-common`**: Shared Rust definitions, DBus proxy traits, and types used across the entire workspace.

## D-Bus Interface Spec
See `helu-common/src/dbus.rs` for exact definitions.

Exposed by `helud` (`net.helu.Auth`):
- `Authenticate(username: String, method: String) -> (success: Bool, message: String)`
- `AuthenticateWithCredential(username: String, method: String, credential: String) -> (success: Bool, message: String)`
- `Enroll(username: String, method: String) -> (success: Bool)`
- `ListMethods(username: String) -> (methods: Array<String>)`
- `Status() -> (daemon_version: String, loaded_methods: Array<String>)`

Signals on `net.helu.Auth`:
- `AuthRequested(username: String)`
- `AuthSuccess(username: String)`
- `AuthFailure(username: String, reason: String)`
- `AuthStateChanged(state: AuthState)`

Exposed by `helu-ui` (`net.helu.UI`):
Signals on `net.helu.UI`:
- `PinSubmitted(username: String, pin: String)`
- `UiReady()`

## Biometric Security Model

### Face Recognition Pipeline
We use ONNX Runtime to execute the `MobileFaceNet` model. The pipeline does not store facial images. Instead, it extracts a 512-dimensional embedding vector, which is compared using cosine similarity.
It is computationally infeasible to reconstruct a human face from these embeddings, providing a strong privacy guarantee.

### TPM2 Crypto Integration
When available, we seal a 32-byte AES key using a TPM 2.0. This key encrypts the 512-dimensional embeddings via AES-256-GCM.
The key is bound to the following PCRs (Platform Configuration Registers):
- **PCR 0**: Core firmware measurements (detects BIOS tampering).
- **PCR 1**: Firmware configuration (detects boot order modification).
- **PCR 7**: Secure boot state (detects if secure boot is disabled).

By binding to these registers, the key cannot be unsealed if the system has been tampered with at the boot level, providing a robust defense-in-depth security model.

### Software Fallback
If no TPM is present on the system, the platform falls back to software key derivation using a combination of the stable machine ID (`/etc/machine-id`) and a randomly generated salt (`/var/lib/helu/salt`) processed via HKDF-SHA256.
*Warning*: While better than plaintext storage, this fallback cannot provide the hardware-backed security guarantees of TPM and leaves data vulnerable if physical access to the disk is compromised.

### Cosine Similarity & Thresholds
Embeddings are L2 normalized, allowing cosine similarity to be calculated easily as a simple dot product.
Thresholds for a match typically range from 0.5 to 0.7. Lower thresholds imply a stricter match. The default value is configurable in `helu.toml`.

## Network Authentication Flow

```text
┌─────────────┐     POST /auth/challenge      ┌──────────────┐
│  VPN / App  │ ────────────────────────────► │ helu-server  │
│  (client)   │                               │   (axum)     │
│             │ ◄──────────────────────────── │              │
│             │     { challenge_id, nonce }   │              │
│             │                               │      │       │
│             │     POST /auth/verify         │      │ D-Bus │
│             │ ────────────────────────────► │      │       │
│             │                               │      ▼       │
│             │                               │    helud     │
│             │                               │      │       │
│             │                               │      ▼       │
│             │                               │   helu-ui    │
│             │                               │  (biometric) │
│             │     { JWT }                   │      │       │
│             │ ◄──────────────────────────── │      │       │
└─────────────┘                               └──────────────┘
```

**Note:** Currently, `helu-server` assumes local D-Bus IPC to `helud` in an "auth node" model. The server directly blocks and awaits the return values from the `Authenticate` method call without the complexity of a background task for signals. A future architecture could have `helu-server` trigger auth remotely via a lightweight gRPC call.

## Why Not RADIUS
- RADIUS was designed in 1991 for dial-up modem pools
- Uses UDP with weak MD5-based auth
- No native biometric support
- No modern token format
- helu-server uses HTTPS, Argon2, JWT, and biometric-gated issuance

## Future Architecture Notes
- Remote `helud` agents allowing `helu-server` to operate as a centralized multi-node orchestrator over gRPC instead of local D-Bus.

## Wayland and Layer Shell
The `helu-ui` overlay uses GTK4 and the `gtk4-layer-shell` library. Layer shell is an extension to the Wayland protocol that allows applications to create panels, lockscreens, and overlays. We use it to ensure the biometric prompt correctly draws over other windows, acting as an un-ignorable system prompt, unlike a standard application window.
