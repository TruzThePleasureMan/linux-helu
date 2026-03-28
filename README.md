# Linux Helu 🐧👋

```text
     .-.
    (o o)    Linux Helu 🐧👋
    | O |    Real biometric auth for Linux.
    '\_/'    Born from a meme. Built for production.
```

## What it is
Unified biometric auth daemon for Linux
Face, fingerprint, FIDO2, PIN under one D-Bus interface
PAM integration — works with any PAM-aware application
Network auth server replacing RADIUS for enterprise use

## Why
"RADIUS is from 1991. fprintd, face recognition, and FIDO2 have never had a unified Linux auth layer. Linux Helu is that layer."

## Display server support
| Compositor Status | Wayland (wlroots: Sway, Hyprland, River) |
| --- | --- |
| ✅ Full layer-shell support | Wayland (GNOME Shell 45+) |
| ✅ Supported | Wayland (KDE Plasma 6) |
| ✅ Supported | X11 |
| ✅ Fallback mode, keep-above window | Mir |
| 🤷 Untested. Good luck. | |

## Architecture diagram
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

## Prerequisites
- Rust toolchain (`cargo`)
- `libwebkit2gtk-4.1-dev` and common build essentials.
- Tauri CLI (`npm install -g @tauri-apps/cli`)
- ONNX Runtime and a valid `mobilefacenet.onnx` model (InsightFace).
- PostgreSQL (for `helu-server`).

## Quick start
1. Build the workspace: `cargo build --workspace --release`
2. Download the ONNX model: `./scripts/download-model.sh`
3. Install the PAM module (see PAM setup below).
4. Run the daemon: `sudo target/release/helud`
5. Enroll your biometrics: `cd helu-setup && npm run tauri dev`

## Component overview
- **helud**: The core daemon running on D-Bus. Handles auth requests, configuration, and biometric verification logic.
- **pam_helu**: A C-compatible PAM library (`pam_sm_authenticate`) that forwards auth requests to `helud`.
- **helu-cli**: Command-line tool for enrolling biometric data and checking system status.
- **helu-ui**: GTK4 system overlay, layer-shell, Rust only, no web tech. Provides the lockscreen and auth UI.
- **helu-setup**: Tauri + Svelte, normal app window, enrollment only.
- **helu-server**: A lightweight network server that functions as a RADIUS replacement, issuing biometric challenges and verifying JWTs.

## PAM setup
```text
auth    sufficient    pam_helu.so
auth    required      pam_unix.so try_first_pass
```

## helu-server deployment
To deploy `helu-server` using Docker:
```bash
docker build -t helu-server -f helu-server/Dockerfile .
docker run -d -p 8080:8080 --env-file .env helu-server
```
Or run as a systemd service:
```ini
[Unit]
Description=Linux Helu Network Auth Server
After=network.target postgresql.service

[Service]
ExecStart=/usr/local/bin/helu-server
EnvironmentFile=/etc/helu/server.env
Restart=always

[Install]
WantedBy=multi-user.target
```

## Config reference
`/etc/helu/helu.toml`:
```toml
[daemon]
bus = "session" # 'system' or 'session'
socket = "/run/helu/helu.sock"
log_level = "info"

[face]
enabled = true
model_path = "/usr/share/helu/models/mobilefacenet.onnx"
threshold = 0.6
camera_index = 0
mock_hardware = true # for development

[fingerprint]
enabled = true
timeout_secs = 15
default_finger = "right-index-finger"

[pin]
enabled = true
min_length = 4

[fido2]
enabled = true
timeout_secs = 30
credential_path = "/var/lib/helu/fido2"

[crypto]
tpm_device = "/dev/tpmrm0"
tpm_pcrs = [0, 1, 7]
face_store_path = "/var/lib/helu/faces.enc"
sealed_key_path = "/var/lib/helu/tpm_sealed_key"
software_fallback = true # Set to false to require TPM2 hardware

[ui]
accent_color = "#e95420"
greeting = "Helu" # Change to "Hello" if you hate fun
```
`/etc/helu/helu-server.toml`:
```toml
[server]
bind = "0.0.0.0:8080"
tls_cert = "/path/to/cert.pem" # Optional
tls_key = "/path/to/key.pem" # Optional

[database]
url = "postgres://helu:password@localhost/helu"
max_connections = 10

[jwt]
secret = "change-me-in-production"
ttl_secs = 3600
algorithm = "HS256" # Or "RS256"
private_key_path = "/path/to/private.pem" # Optional, required for RS256
public_key_path = "/path/to/public.pem" # Optional, required for RS256

[challenge]
ttl_secs = 60

[dbus]
bus = "session" # 'system' or 'session'

[auth]
allowed_methods = ["face", "fingerprint", "fido2", "pin"]
```

## Wayland Gotchas
`gtk4_layer_shell::is_supported()` must always be checked before calling any layer shell API. Some compositors might not support layer shell, in which case the UI falls back to a normal window.

## Model download
Usage: `./scripts/download-model.sh`
This script will fetch the `mobilefacenet.onnx` model from the required source and place it in the correct directory.

## Contributing
We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to set up your dev environment, run tests, and our expectations for PRs. Open issues are a great place to start! Note that this project started as a meme, so contributions of all sizes are welcome.

## Roadmap
Phase 1: Core local auth ✅
Phase 2: Network auth server ✅
Phase 3: helu-setup Tauri enrollment UI
Phase 4: Multi-node helu-server with remote helud agents
Phase 5: freedesktop.org D-Bus spec proposal
Phase 6: Distro packaging (Fedora, Debian, Arch AUR)

## Known issues
"It's Linux."

## Why Not RADIUS?
RADIUS was designed in 1991 for dial-up modem pools. It uses UDP with weak MD5-based auth by default, and has no native biometric support or modern token format. We replace it completely: `helu-server` uses HTTPS, Argon2, JWT, and biometric-gated issuance natively tied to your local OS daemon. We were born in a meme, but we're still more modern.

## License
MIT
