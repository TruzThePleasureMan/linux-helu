#!/usr/bin/env bash
set -euo pipefail
mkdir -p ~/.config/systemd/user
cp dist/helu-ui.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now helu-ui
echo "helu-ui service installed and started."
