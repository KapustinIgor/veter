#!/bin/sh
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"

mkdir -p "$repo_root/apps" "$repo_root/core" "$repo_root/proto" "$repo_root/docs" "$repo_root/ci"

if [ ! -d "$repo_root/apps/veter_app" ]; then
  if command -v flutter >/dev/null 2>&1; then
    flutter create --org com.veter --project-name veter_app "$repo_root/apps/veter_app"
  else
    echo "[WARN] Flutter not found. Skipping Flutter app creation."
  fi
fi

if [ ! -d "$repo_root/core/veter_core" ]; then
  if command -v cargo >/dev/null 2>&1; then
    cargo new --lib "$repo_root/core/veter_core"
  else
    echo "[WARN] Cargo not found. Skipping Rust core creation."
  fi
fi

if [ -f "$repo_root/core/veter_core/Cargo.toml" ]; then
  if ! grep -q "crate-type" "$repo_root/core/veter_core/Cargo.toml"; then
    cat >> "$repo_root/core/veter_core/Cargo.toml" <<'EOF'

[lib]
crate-type = ["staticlib", "cdylib", "rlib"]

[dependencies]
anyhow = "1"
thiserror = "1"

EOF
  fi
  mkdir -p "$repo_root/core/veter_core/src"
  cat > "$repo_root/core/veter_core/src/lib.rs" <<'EOF'
//! Veter core placeholder. Expose safe FFI later.

pub fn hello() -> &'static str { "veter-core" }
EOF
fi

if [ ! -f "$repo_root/proto/relay.proto" ]; then
  cat > "$repo_root/proto/relay.proto" <<'EOF'
syntax = "proto3";
package veter.relay.v1;

message Ciphertext {
  bytes id = 1;
  bytes sender_device_id = 2;
  bytes room_id = 3;
  bytes payload = 4; // E2EE blob
  int64 sent_ts = 5;
}

message EnqueueRequest { repeated Ciphertext messages = 1; }
message EnqueueResponse { repeated bytes accepted_ids = 1; }

message DequeueRequest { bytes device_id = 1; uint32 max_items = 2; uint32 credits = 3; }
message DequeueResponse { repeated Ciphertext messages = 1; }

message AckRequest { repeated bytes ids = 1; }
message AckResponse {}

service Relay {
  rpc Enqueue(EnqueueRequest) returns (EnqueueResponse);
  rpc Dequeue(DequeueRequest) returns (DequeueResponse);
  rpc Ack(AckRequest) returns (AckResponse);
}
EOF
fi

echo "Bootstrap complete."


