Veter – Local‑first E2EE Corporate Messenger
===========================================

Veter is a cross‑platform, local‑first, end‑to‑end encrypted messenger for enterprise use. This repository hosts a mono‑repo layout for the Flutter apps, Rust core, protobuf APIs, CI, and tooling.

Quick Start
-----------

1. Ensure Flutter (3.24+), Rust (stable), protoc, and make are installed.
2. Run `scripts/bootstrap.sh` to scaffold the Flutter app and Rust core crate.
3. Build and run the Flutter app for your target platform.

Repository Structure
--------------------

- `apps/` – Flutter application(s)
- `core/` – Rust core library (crypto, CRDT, storage, networking) exposed via FFI
- `proto/` – Protobuf definitions (gRPC over QUIC APIs)
- `scripts/` – Tooling and helper scripts
- `ci/` – CI/CD configuration (to be added)
- `docs/` – Technical plans and specifications

References
----------

- GitHub repo: https://github.com/KapustinIgor/veter.git


