Veter Technical Plan (Initial)
==============================

Scope: Local-first, E2EE messenger across iOS/Android/macOS/Windows/Linux with WebRTC calling, QUIC/gRPC transport, and strict privacy/security.

Components
----------

- Flutter UI (Material 3), desktop+mobile
- Rust core (libsignal integration, CRDT, storage, networking)
- Protobuf APIs (`proto/`) for Directory, Relay, Compliance

Next Steps
----------

- Flesh out protobufs for Directory/Compliance
- Add Rust FFI surface for sessions and storage
- Implement local encrypted DB and indexing
- Spike WebRTC with TURN/TLS via flutter_webrtc

Repo: https://github.com/KapustinIgor/veter.git


