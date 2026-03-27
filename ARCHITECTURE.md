# Linux Helu Architecture

## Components

[PAM Trigger] -> pam_helu.so -> (D-Bus) -> helud -> (Face/FP/FIDO2 logic)

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
