# Tauri VST3 Host

## Goal

Add production-grade VST3 instrument and effect hosting to the offline Tauri app without weakening the browser build or putting native plugin code inside the webview process.

## Supported format

- VST3 on Windows, macOS, and Linux.
- Instruments, audio effects, MIDI input and output, parameter automation, project state, presets, multi-bus audio, transport context, and native plugin editors.
- Legacy VST2 is not bundled. Its discontinued SDK and distribution terms require a separate legal and technical decision.

## Architecture

1. The studio UI remains in the Tauri webview.
2. A native Rust host owns VST3 discovery, plugin instances, parameters, state, MIDI, editor windows, and DSP calls.
3. Control-plane operations use typed Tauri commands.
4. Audio must not use JSON IPC. The real-time path will move the openDAW Rust engine into the native audio process and connect VST3 processors directly to its audio graph.
5. Each third-party plugin will run in a helper process. A plugin crash must mute only that instance and allow state-backed recovery.

## Delivery phases

### Phase 1: Native control plane

- Scan standard and user-supplied VST3 locations.
- Return stable plugin identity and capability metadata.
- Load and unload plugin instances.
- List, format, read, and set parameters.
- Send timestamped MIDI and automation.
- Save and restore opaque plugin state.
- Process typed audio blocks for offline rendering and integration tests.

### Phase 2: Native real-time graph

- Run the openDAW Rust engine from the desktop audio callback.
- Add VST3 instrument and effect processor nodes.
- Support input and output buses, sidechains, latency, tail length, silence flags, and transport context.
- Use bounded lock-free queues for control changes.
- Keep allocation, locks, filesystem access, and Tauri IPC off the audio thread.

### Phase 3: Project and UI integration

- Add serializable VST3 device boxes keyed by class UID.
- Persist plugin path hints, vendor, version, parameter mapping, and opaque state.
- Restore missing plugins as disabled placeholders without losing state.
- Open native plugin editors in helper-owned windows.
- Expose generic openDAW controls when a plugin has no editor.

### Phase 4: Isolation and compatibility

- Bundle and supervise one helper process per plugin instance.
- Cache scan results and quarantine plugins that crash or time out during probing.
- Add block-size, sample-rate, mono/stereo, sidechain, instrument, effect, state, automation, MIDI, and editor conformance tests.
- Validate Windows with installed commercial plugins and a redistributable open-source test plugin.

## Acceptance criteria

- A scanned VST3 instrument can receive MIDI and produce audio on an openDAW track.
- A scanned VST3 effect can process track audio and sidechain input.
- Parameter changes are sample-offset aware and automatable.
- Save, close, reopen, and project reload reproduce plugin state.
- Plugin latency is compensated and updated when the plugin requests a restart.
- Native editors open, resize, and close without hanging the studio.
- A crashing or hanging plugin cannot terminate the desktop app or block its audio callback.
- Offline bounce and live playback produce equivalent output within floating-point tolerance.

## Current implementation boundary

The first implementation lands the typed native control plane and offline block processor. It deliberately does not route real-time audio through Tauri command IPC. Full live-track support depends on Phase 2, the native openDAW engine host.
