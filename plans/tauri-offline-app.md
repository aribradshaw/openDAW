# Tauri Offline App

## Goal

Package the existing openDAW Studio Vite application as a Tauri 2 desktop application without changing its audio engine or storage model.

## First milestone

- Build the existing `@opendaw/app-studio` workspace as the desktop frontend.
- Preserve cross-origin isolation for AudioWorklet and `SharedArrayBuffer` use.
- Produce a Windows NSIS installer.
- Grant only Tauri's base application capability.
- Keep OPFS storage and existing web APIs unchanged.
- Make the generated box-source cleanup step work from Windows so the desktop prerequisite build can complete.
- Allow the WASM build to use Cargo from `PATH` on Windows when the Unix-style Cargo environment file is absent.

## Development

The studio development server requires the repository's localhost certificate:

```shell
npm run cert
npm run tauri -w @opendaw/app-desktop -- dev
```

Create a production installer with:

```shell
npm run tauri -w @opendaw/app-desktop -- build
```

## Follow-up milestones

1. Verify recording, playback, MIDI input, OPFS persistence, and WASM devices in WebView2.
2. Add native file dialogs and explicit import/export capabilities.
3. Add signed installers and automated Windows, macOS, and Linux builds.
4. Design an out-of-process VST3 host. Real-time audio must not use Tauri's JSON IPC.

## AI-assisted work

Codex prepared the initial Tauri scaffold, build integration, cross-origin isolation configuration, and verification notes. The implementation intentionally excludes native filesystem and VST permissions until the offline shell is validated.
