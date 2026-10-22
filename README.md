# Button Beam Desktop

Desktop companion for [Button Beam Mobile](https://github.com/multipliedtwice/button-beam-mobile). It manages shortcut sequences, displays a QR connection flow, accepts one phone connection over WebSocket, and runs configured shortcuts on the desktop.

Built with Tauri, React, and Rust.

![Button Beam Desktop](bb-1024.png)

## Run locally

Requirements: the Tauri development prerequisites, Rust, and Yarn.

```bash
yarn install
yarn tauri dev
```

Open the mobile companion, scan the QR code shown by the desktop app, then choose a configured shortcut on the phone.

## Architecture

- `src/` — React interface for connection and shortcut management
- `src-tauri/src/shortcuts.rs` — local shortcut storage and execution
- `src-tauri/src/sockets.rs` — WebSocket server and mobile-client protocol
- `src-tauri/src/main.rs` — Tauri application setup

Shortcuts are stored locally by the desktop app. The current source accepts one mobile device connection at a time.

## Companion

[Button Beam Mobile](https://github.com/multipliedtwice/button-beam-mobile) is the Flutter client that connects over the local network.

## Status

There are no GitHub Releases for this repository at the time of this update.
