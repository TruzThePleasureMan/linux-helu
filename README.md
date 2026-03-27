# Linux Helu 🐧👋
```text
     .-.
    (o o)    Linux Helu 🐧👋
    | O |    Real biometric auth for Linux.
    '\_/'    Born from a meme. Built for production.
```
**Disclaimer:** Unlike certain other auth systems from 1991, Linux Helu was designed after the invention of the internet.

## What is Linux Helu?
Linux Helu is a genuine, production-grade biometric authentication system for Linux — unifying face recognition, fingerprint, PIN, and FIDO2 under a single D-Bus daemon and a modern Tauri frontend. It is designed to eventually replace RADIUS in enterprise environments.

The name is a joke. The software is not. It’s a real, robust PAM module, D-Bus daemon, and GUI tool chain built entirely in Rust and Web Technologies.

## Architecture Overview
```text
[PAM Trigger] -> pam_helu.so -> (D-Bus) -> helud
                                              |
      [Tauri Svelte UI] <--(D-Bus)------------+--> Face Recognition (ONNX)
                                              +--> Fingerprint (fprintd)
                                              +--> FIDO2
                                              +--> PIN Fallback
```

## Prerequisites and Install
- Rust toolchain (`cargo`)
- `libwebkit2gtk-4.1-dev` and common build essentials.
- Tauri CLI (`npm install -g @tauri-apps/cli`)
- ONNX Runtime and a valid `mobilefacenet.onnx` model (InsightFace).

## PAM Setup
To install Linux Helu for PAM authentication, add this to your PAM configuration (e.g. `/etc/pam.d/system-auth` or `/etc/pam.d/sudo`):
```text
auth    sufficient    pam_helu.so
auth    required      pam_unix.so try_first_pass
```

## Config
The primary configuration is loaded from `/etc/helu/helu.toml` (system) and `~/.config/helu/user.toml` (user mode).
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

[ui]
accent_color = "#e95420"
greeting = "Helu" # Change to "Hello" if you hate fun
```

## Display Server Support
| Compositor Status | Wayland (wlroots: Sway, Hyprland, River) |
| --- | --- |
| ✅ Full layer-shell support | Wayland (GNOME Shell 45+) |
| ✅ Supported | Wayland (KDE Plasma 6) |
| ✅ Supported | X11 |
| ✅ Fallback mode, keep-above window | Mir |
| 🤷 Untested. Good luck. | |

## Known Issues
"It's Linux."

## Why?
RADIUS was released in 1991. We were born in a meme. We're still more modern.

## Roadmap
1. `helud` daemon skeleton + D-Bus interface + PIN auth.
2. `pam_helu.so` PAM module.
3. `helu-ui` frontend wired over D-Bus.
4. Face recognition pipeline (ONNX).
5. Fingerprint via `fprintd`.
6. FIDO2 support.
7. `helu-server` network auth replacement.
