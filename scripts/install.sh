#!/usr/bin/env bash
# ==============================================================================
# Smoooth v2 Beta — Production Installer & Lifecycle Script for Linux VPS
# Idempotent installation, dedicated service account, Docker host-root security
# disclosures, preflight diagnostics, safe upgrade/rollback, and systemd confinement.
# Target Baseline: Ubuntu 24.04 LTS (x86_64)
# ==============================================================================

set -euo pipefail

SMOOOTH_VERSION="0.1.1"
SMOOOTH_USER="smoooth"
SMOOOTH_GROUP="smoooth"
INSTALL_BIN_DIR="/usr/local/bin"
SMOOOTH_BIN="${INSTALL_BIN_DIR}/smoooth"
CONFIG_DIR="/etc/smoooth"
TOKEN_FILE="${CONFIG_DIR}/auth.token"
STATE_DIR="/var/lib/smoooth"
SYSTEMD_SERVICE_FILE="/etc/systemd/system/smoooth.service"

ROTATE_TOKEN=false
SHOW_TOKEN=false
ACTION="install"
PURGE_DATA=false
ENABLE_REMOTE=false

# ------------------------------------------------------------------------------
# Parse Flags
# ------------------------------------------------------------------------------
while [[ $# -gt 0 ]]; do
  case "$1" in
    --upgrade)
      ACTION="upgrade"
      shift
      ;;
    --rotate-token)
      ROTATE_TOKEN=true
      shift
      ;;
    --show-token)
      SHOW_TOKEN=true
      shift
      ;;
    --enable-remote)
      ENABLE_REMOTE=true
      shift
      ;;
    --uninstall)
      ACTION="uninstall"
      shift
      ;;
    --purge-data)
      PURGE_DATA=true
      shift
      ;;
    --help|-h)
      echo "Smoooth v${SMOOOTH_VERSION} Beta Installer & Lifecycle Manager"
      echo ""
      echo "Usage: sudo $0 [OPTIONS]"
      echo ""
      echo "Options:"
      echo "  --upgrade        Safely upgrade daemon binary with automated rollback on failure"
      echo "  --rotate-token   Generate a new cryptographically secure 256-bit auth token"
      echo "  --show-token     Display token summary upon completion"
      echo "  --enable-remote  Explicit opt-in to bind to 0.0.0.0 (requires TLS reverse proxy)"
      echo "  --uninstall      Safely uninstall systemd service and binaries (preserves application data)"
      echo "  --purge-data     Used with --uninstall to also delete application databases and configs"
      echo "  --help, -h       Display this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

# ------------------------------------------------------------------------------
# 1. Root & Privilege Verification
# ------------------------------------------------------------------------------
if [[ $EUID -ne 0 ]]; then
  echo "[ERROR] This installer must be executed as root (e.g. via sudo)." >&2
  exit 1
fi

# ------------------------------------------------------------------------------
# Safe Uninstall Handler
# ------------------------------------------------------------------------------
if [[ "${ACTION}" == "uninstall" ]]; then
  echo "[INFO] Uninstalling Smoooth daemon..."
  if systemctl is-active --quiet smoooth.service 2>/dev/null; then
    systemctl stop smoooth.service || true
  fi
  if systemctl is-enabled --quiet smoooth.service 2>/dev/null; then
    systemctl disable smoooth.service || true
  fi
  rm -f "${SYSTEMD_SERVICE_FILE}"
  systemctl daemon-reload || true
  rm -f "${SMOOOTH_BIN}"
  echo "[INFO] Smoooth service unit and binary removed."

  if [[ "${PURGE_DATA}" == "true" ]]; then
    echo "[WARN] Explicit data purge requested: Removing state and configuration directories..."
    rm -rf "${STATE_DIR}" "${CONFIG_DIR}"
    echo "[INFO] Data directories purged."
  else
    echo "[INFO] Safe uninstall: Application data in '${STATE_DIR}' and configuration in '${CONFIG_DIR}' PRESERVED."
    echo "[INFO] To purge all data in the future, pass --purge-data."
  fi
  echo "[INFO] Uninstall completed."
  exit 0
fi

# ------------------------------------------------------------------------------
# Safe Upgrade & Rollback Handler
# ------------------------------------------------------------------------------
if [[ "${ACTION}" == "upgrade" ]]; then
  echo "[INFO] Starting safe Smoooth daemon upgrade..."
  if [[ ! -f "${SMOOOTH_BIN}" ]]; then
    echo "[ERROR] No existing installation found at ${SMOOOTH_BIN}. Run normal installation first." >&2
    exit 1
  fi

  BACKUP_BIN="${SMOOOTH_BIN}.bak.$(date +%s)"
  echo "[INFO] Creating binary rollback backup at ${BACKUP_BIN}..."
  cp "${SMOOOTH_BIN}" "${BACKUP_BIN}"

  NEW_BIN=""
  if [[ -f "./target/release/smoooth" ]]; then
    NEW_BIN="./target/release/smoooth"
  elif [[ -f "./smoooth" ]]; then
    NEW_BIN="./smoooth"
  fi

  if [[ -z "${NEW_BIN}" ]]; then
    echo "[ERROR] No new 'smoooth' binary found in release target or current directory." >&2
    exit 1
  fi

  echo "[INFO] Installing new binary from ${NEW_BIN}..."
  install -m 0755 "${NEW_BIN}" "${SMOOOTH_BIN}"

  echo "[INFO] Restarting Smoooth service..."
  systemctl restart smoooth.service || true
  sleep 2

  if systemctl is-active --quiet smoooth.service; then
    echo "[INFO] Smoooth service is ACTIVE after upgrade."
    echo "[INFO] Running post-upgrade diagnostics with 'smoooth doctor'..."
    "${SMOOOTH_BIN}" doctor || true
    echo "=============================================================================="
    echo " Smoooth upgraded successfully to v${SMOOOTH_VERSION}!"
    echo " Rollback binary preserved at: ${BACKUP_BIN}"
    echo "=============================================================================="
    exit 0
  else
    echo "[ERROR] Service failed to start with new binary! Initiating automatic rollback..." >&2
    cp "${BACKUP_BIN}" "${SMOOOTH_BIN}"
    systemctl restart smoooth.service
    echo "[WARN] Service safely restored from rollback backup."
    exit 1
  fi
fi

# ------------------------------------------------------------------------------
# 2. Preflight Architecture & OS Verification
# ------------------------------------------------------------------------------
echo "[INFO] Running preflight system checks..."

# Architecture check: official baseline is x86_64
ARCH=$(uname -m)
if [[ "${ARCH}" != "x86_64" ]]; then
  if [[ "${ARCH}" == "aarch64" ]]; then
    echo "[WARN] Architecture is aarch64 (ARM64). Experimental mode; official verified target is x86_64."
  else
    echo "[ERROR] Unsupported CPU architecture: '${ARCH}'. Smoooth Beta requires x86_64." >&2
    exit 1
  fi
else
  echo "[INFO] Architecture verified: ${ARCH} (x86_64 verified baseline)."
fi

# Operating System verification
if [[ ! -f /etc/os-release ]]; then
  echo "[ERROR] Unsupported operating system: /etc/os-release not found." >&2
  exit 1
fi

source /etc/os-release
echo "[INFO] Detected OS: ${PRETTY_NAME:-$ID} (Version: ${VERSION_ID:-unknown})"

case "${ID:-}" in
  ubuntu)
    if [[ "${VERSION_ID:-}" == "24.04"* ]]; then
      echo "[INFO] Verified target operating system: Ubuntu 24.04 LTS confirmed."
    else
      echo "[INFO] Compatible Ubuntu release detected (${VERSION_ID:-})."
    fi
    ;;
  debian|centos|rhel|rocky|almalinux)
    echo "[INFO] Compatible Linux distribution confirmed (${ID:-})."
    ;;
  *)
    echo "[WARN] Operating system '${ID:-}' is not officially verified, continuing standard systemd setup..."
    ;;
esac

if ! command -v systemctl >/dev/null 2>&1; then
  echo "[ERROR] systemd is required to run the Smoooth daemon." >&2
  exit 1
fi

# ------------------------------------------------------------------------------
# 3. Disk Space & Port Preflight Checks
# ------------------------------------------------------------------------------
FREE_KB=$(df -k /var 2>/dev/null | awk 'NR==2 {print $4}' || echo "")
if [[ -n "${FREE_KB}" && "${FREE_KB}" =~ ^[0-9]+$ ]]; then
  if [[ "${FREE_KB}" -lt 524288 ]]; then
    echo "[ERROR] Insufficient disk space on /var ($((FREE_KB / 1024))MB available). Smoooth requires at least 512MB free space." >&2
    exit 1
  else
    echo "[INFO] Available disk space on /var: $((FREE_KB / 1024))MB (healthy)."
  fi
fi

# Port 4040 conflict check
if command -v ss >/dev/null 2>&1; then
  if ss -tulpn | grep -q ":4040\b"; then
    echo "[WARN] Port 4040 is currently in use. Existing Smoooth service will be restarted on update."
  fi
fi

# ------------------------------------------------------------------------------
# 4. Prerequisites Verification & Installation
# ------------------------------------------------------------------------------
for cmd in curl openssl jq; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "[WARN] Required prerequisite tool '$cmd' is not installed."
    echo "[INFO] Attempting to install missing package '$cmd'..."
    if command -v apt-get >/dev/null 2>&1; then
      apt-get update -y && apt-get install -y "$cmd"
    elif command -v dnf >/dev/null 2>&1; then
      dnf install -y "$cmd"
    elif command -v yum >/dev/null 2>&1; then
      yum install -y "$cmd"
    else
      echo "[ERROR] Package manager not recognized. Please install '$cmd' manually." >&2
      exit 1
    fi
  fi
done

# Verify Docker Engine availability
if ! command -v docker >/dev/null 2>&1; then
  echo "[WARN] Docker CLI not found. Smoooth requires Docker Engine to provision containers and databases."
  echo "[INFO] Install Docker on Ubuntu: curl -fsSL https://get.docker.com | sh"
else
  DOCKER_VER=$(docker --version)
  echo "[INFO] ${DOCKER_VER} detected."
  if ! systemctl is-active --quiet docker; then
    echo "[INFO] Starting Docker daemon..."
    systemctl enable --now docker || true
  fi
fi

# ------------------------------------------------------------------------------
# 5. Security Disclosure: Docker Effective Host-Root Access
# ------------------------------------------------------------------------------
echo "=============================================================================="
echo " SECURITY NOTICE: DOCKER ENGINE ACCESS & PRIVILEGE CONFINEMENT"
echo " Smoooth orchestrates containers via the local Docker daemon socket"
echo " (/var/run/docker.sock) using a dedicated '${SMOOOTH_USER}' system service account."
echo " In Docker's architecture, socket access is effectively root-equivalent on the"
echo " host. To protect your server, Smoooth enforces:"
echo "   1. Private loopback binding (127.0.0.1:4040) by default"
echo "   2. Cryptographic token authorization for all HTTP, MCP, and CLI operations"
echo "   3. Systemd sandbox confinement (NoNewPrivileges=yes, ProtectSystem=strict)"
echo "   4. Server-side capability and scope enforcement"
echo " Do not expose port 4040 or /var/run/docker.sock to public interfaces without"
echo " a TLS reverse proxy and strict network ACLs."
echo "=============================================================================="

# ------------------------------------------------------------------------------
# 6. Dedicated Service Account & Group Provisioning
# ------------------------------------------------------------------------------
echo "[INFO] Provisioning dedicated system account '${SMOOOTH_USER}'..."

if ! getent group "${SMOOOTH_GROUP}" >/dev/null 2>&1; then
  groupadd --system "${SMOOOTH_GROUP}"
  echo "[INFO] Created system group '${SMOOOTH_GROUP}'."
fi

if ! id -u "${SMOOOTH_USER}" >/dev/null 2>&1; then
  useradd \
    --system \
    --gid "${SMOOOTH_GROUP}" \
    --home-dir "${STATE_DIR}" \
    --shell /usr/sbin/nologin \
    --comment "Smoooth Autonomous Infrastructure Daemon" \
    "${SMOOOTH_USER}"
  echo "[INFO] Created system user '${SMOOOTH_USER}'."
fi

# Grant Docker daemon access if docker group exists
if getent group docker >/dev/null 2>&1; then
  usermod -aG docker "${SMOOOTH_USER}"
  echo "[INFO] Added '${SMOOOTH_USER}' to 'docker' group for engine access."
fi

# ------------------------------------------------------------------------------
# 7. Directory Structure & Permissions (Strict Least Privilege)
# ------------------------------------------------------------------------------
echo "[INFO] Configuring secure directory layout..."

mkdir -p "${CONFIG_DIR}"
chmod 0700 "${CONFIG_DIR}"
chown "${SMOOOTH_USER}:${SMOOOTH_GROUP}" "${CONFIG_DIR}"

mkdir -p "${STATE_DIR}"
chmod 0750 "${STATE_DIR}"
chown "${SMOOOTH_USER}:${SMOOOTH_GROUP}" "${STATE_DIR}"

# ------------------------------------------------------------------------------
# 8. Cryptographic Token Generation & Storage
# ------------------------------------------------------------------------------
if [[ ! -f "${TOKEN_FILE}" ]] || [[ "${ROTATE_TOKEN}" == "true" ]]; then
  echo "[INFO] Generating cryptographically secure 256-bit authentication token..."
  TOKEN=$(openssl rand -hex 32)
  echo "${TOKEN}" > "${TOKEN_FILE}"
  chmod 0600 "${TOKEN_FILE}"
  chown "${SMOOOTH_USER}:${SMOOOTH_GROUP}" "${TOKEN_FILE}"
  echo "[INFO] Master auth token securely stored in ${TOKEN_FILE} (permissions: 0600)."
else
  echo "[INFO] Existing master auth token retained at ${TOKEN_FILE}."
fi

# ------------------------------------------------------------------------------
# 9. Binary Installation (Idempotent)
# ------------------------------------------------------------------------------
if [[ -f "./target/release/smoooth" ]]; then
  echo "[INFO] Installing release binary from ./target/release/smoooth..."
  install -m 0755 ./target/release/smoooth "${SMOOOTH_BIN}"
elif [[ -f "./target/debug/smoooth" ]]; then
  echo "[INFO] Installing debug binary from ./target/debug/smoooth..."
  install -m 0755 ./target/debug/smoooth "${SMOOOTH_BIN}"
elif [[ -f "./smoooth" ]]; then
  echo "[INFO] Installing local binary from ./smoooth..."
  install -m 0755 ./smoooth "${SMOOOTH_BIN}"
elif command -v "${SMOOOTH_BIN}" >/dev/null 2>&1; then
  echo "[INFO] Binary already installed at ${SMOOOTH_BIN}."
else
  echo "[WARN] No pre-built 'smoooth' binary found in current path."
  echo "[INFO] Compile via: cargo build --release && sudo install -m 0755 target/release/smoooth ${SMOOOTH_BIN}"
fi

# ------------------------------------------------------------------------------
# 10. Systemd Service Unit (Hardened Confinement & Private by Default)
# ------------------------------------------------------------------------------
if [[ "${ENABLE_REMOTE}" == "true" ]]; then
  BIND_ADDR="0.0.0.0"
  echo "[WARN] Remote access enabled (binding to 0.0.0.0:4040). Ensure TLS termination and authentication tokens are enforced."
else
  BIND_ADDR="127.0.0.1"
fi

cat <<EOF > "${SYSTEMD_SERVICE_FILE}"
[Unit]
Description=Smoooth Infrastructure Engine & MCP Server
Documentation=https://github.com/smoooth/smoooth
After=network.target docker.service
Wants=docker.service

[Service]
Type=simple
User=${SMOOOTH_USER}
Group=${SMOOOTH_GROUP}
WorkingDirectory=${STATE_DIR}

Environment=SMOOOTH_BIND_ADDR=${BIND_ADDR}
Environment=SMOOOTH_PORT=4040
Environment=SMOOOTH_TOKEN_FILE=${TOKEN_FILE}
Environment=SMOOOTH_STORAGE_PATH=${STATE_DIR}/smoooth_state.json

# Private loopback management API by default; remote access requires explicit opt-in
ExecStart=${SMOOOTH_BIN} server --bind ${BIND_ADDR} --port 4040

Restart=on-failure
RestartSec=5s
LimitNOFILE=65536

# Hardened security confinement
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=${STATE_DIR} ${CONFIG_DIR} /tmp
PrivateTmp=yes

[Install]
WantedBy=multi-user.target
EOF

chmod 0644 "${SYSTEMD_SERVICE_FILE}"

# ------------------------------------------------------------------------------
# 11. Reload & Enable Systemd Service
# ------------------------------------------------------------------------------
echo "[INFO] Reloading systemd daemon..."
systemctl daemon-reload
systemctl enable smoooth.service

if [[ -f "${SMOOOTH_BIN}" ]]; then
  echo "[INFO] Starting Smoooth service..."
  systemctl restart smoooth.service
  sleep 1
  if systemctl is-active --quiet smoooth.service; then
    echo "[INFO] Smoooth service is ACTIVE and healthy."
  else
    echo "[WARN] Service restart was attempted; check logs with: journalctl -u smoooth -n 50"
  fi
fi

# ------------------------------------------------------------------------------
# 12. Post-Install Diagnostics Check
# ------------------------------------------------------------------------------
if [[ -f "${SMOOOTH_BIN}" ]] && command -v "${SMOOOTH_BIN}" >/dev/null 2>&1; then
  echo ""
  echo "[INFO] Executing post-install preflight check via 'smoooth doctor'..."
  "${SMOOOTH_BIN}" doctor || true
fi

# ------------------------------------------------------------------------------
# 13. Summary & Security Guidance
# ------------------------------------------------------------------------------
echo ""
echo "=============================================================================="
echo " Smoooth v${SMOOOTH_VERSION} Beta successfully installed!"
echo "=============================================================================="
echo " - Daemon Binary:      ${SMOOOTH_BIN}"
echo " - Config & Tokens:    ${CONFIG_DIR} (mode 0700, ${SMOOOTH_USER})"
echo " - State & Storage:    ${STATE_DIR} (mode 0750, ${SMOOOTH_USER})"
echo " - Systemd Service:    ${SYSTEMD_SERVICE_FILE}"
echo " - Network Boundary:   Private 127.0.0.1:4040 (Zero unauthenticated exposure)"
echo ""
echo " Operator Quickstart:"
echo "   smoooth doctor                  # Verify host diagnostics & security"
echo "   smoooth agent pair --name agent # Pair an AI coding agent with scoped token"
echo "   smoooth agent list              # View active agent credentials"
echo "   systemctl status smoooth        # Service status"
echo "   journalctl -u smoooth -f        # Live audit log"
echo ""
if [[ "${SHOW_TOKEN}" == "true" ]]; then
  echo " Master Operator Token:"
  cat "${TOKEN_FILE}"
  echo ""
else
  echo " Master Token File: ${TOKEN_FILE}"
  echo " (To view master token: sudo cat ${TOKEN_FILE} or rerun with --show-token)"
fi
echo " Control Dashboard: http://127.0.0.1:4040/dashboard (Local access)"
echo "=============================================================================="
