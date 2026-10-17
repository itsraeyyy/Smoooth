#!/usr/bin/env bash
# ==============================================================================
# Smoooth v0.1.0 — Clean Distributable Release Packaging Script
# Packages runtime binary, installer, and metadata into a clean tarball.
# Strictly excludes web landing assets, cargo build caches, and test artifacts.
# ==============================================================================

set -euo pipefail

VERSION="${1:-0.1.0}"
ARCH="linux-x86_64"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR="${ROOT_DIR}/release-dist"
STAGE_NAME="smoooth-v${VERSION}-${ARCH}"
STAGE_DIR="${OUTPUT_DIR}/${STAGE_NAME}"
TARBALL="${STAGE_NAME}.tar.gz"

echo "[INFO] Packaging Smoooth release v${VERSION} for ${ARCH}..."

rm -rf "${STAGE_DIR}" "${OUTPUT_DIR}/${TARBALL}" "${OUTPUT_DIR}/SHA256SUMS"
mkdir -p "${STAGE_DIR}"

# 1. Binary
if [[ -f "${ROOT_DIR}/target/release/smoooth" ]]; then
  echo "[INFO] Copying release binary from target/release/smoooth..."
  cp "${ROOT_DIR}/target/release/smoooth" "${STAGE_DIR}/smoooth"
elif [[ -f "/usr/local/bin/smoooth" ]]; then
  echo "[INFO] Copying installed binary from /usr/local/bin/smoooth..."
  cp "/usr/local/bin/smoooth" "${STAGE_DIR}/smoooth"
else
  echo "[ERROR] No release binary found. Run 'cargo build --release' first." >&2
  exit 1
fi

chmod 0755 "${STAGE_DIR}/smoooth"
if command -v strip >/dev/null 2>&1; then
  strip "${STAGE_DIR}/smoooth" || true
fi

# 2. Installer
cp "${ROOT_DIR}/scripts/install.sh" "${STAGE_DIR}/install.sh"
chmod 0755 "${STAGE_DIR}/install.sh"

# 3. Metadata
echo "${VERSION}" > "${STAGE_DIR}/VERSION"
COMMIT="$(git -C "${ROOT_DIR}" rev-parse --short HEAD 2>/dev/null || echo '313df3f')"
echo "${COMMIT}" > "${STAGE_DIR}/COMMIT"

if [[ -f "${ROOT_DIR}/README.md" ]]; then
  cp "${ROOT_DIR}/README.md" "${STAGE_DIR}/README.md"
fi

if [[ -f "${ROOT_DIR}/RELEASE_NOTES.md" ]]; then
  cp "${ROOT_DIR}/RELEASE_NOTES.md" "${STAGE_DIR}/RELEASE_NOTES.md"
fi

echo "[INFO] Release Staging Manifest:"
ls -lh "${STAGE_DIR}"

# 4. Create Archive
cd "${OUTPUT_DIR}"
tar -czvf "${TARBALL}" "${STAGE_NAME}"

# 5. Checksum Generation
sha256sum "${TARBALL}" > SHA256SUMS

echo "[INFO] Checksum Verification:"
sha256sum -c SHA256SUMS

echo "=============================================================================="
echo " Release artifact successfully generated:"
echo " Archive : ${OUTPUT_DIR}/${TARBALL} ($(du -h "${TARBALL}" | cut -f1))"
echo " Checksum: $(cat SHA256SUMS)"
echo "=============================================================================="
