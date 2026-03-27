#!/usr/bin/env bash
# Downloads MobileFaceNet ONNX model from InsightFace
set -euo pipefail

MODEL_DIR="/usr/share/helu/models"
MODEL_URL="https://github.com/deepinsight/insightface/releases/download/v0.7/mobilefacenet.onnx"

echo "Downloading MobileFaceNet ONNX model..."
sudo mkdir -p "$MODEL_DIR"
sudo curl -L "$MODEL_URL" -o "$MODEL_DIR/mobilefacenet.onnx"
echo "Model saved to $MODEL_DIR/mobilefacenet.onnx"
echo "Update model_path in /etc/helu/helu.toml if needed."
