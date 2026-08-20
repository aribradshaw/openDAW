# DC Blocker (2 Hz)

This directory contains the Werkstatt preset proposed as the replacement for
the Stereo Tool implementation in [openDAW issue #91](https://github.com/andremichelle/openDAW/issues/91).

## Files

- `DC Blocker (2 Hz).opb`: preset bundle exported from openDAW
- `werkstatt-dc-blocker.js`: readable Werkstatt source
- `validate-werkstatt-dc-blocker.mjs`: offline DSP regression check

## Design

The preset is a fixed 2 Hz Butterworth high-pass filter. It processes both
channels independently and resets its filter history when openDAW reports a
transport discontinuity. The device's existing enabled control provides bypass.

## Validation

The source compiled successfully in openDAW at 48 kHz. The offline check sends
both DC and 1 kHz test signals through the processor and verifies:

- maximum DC tail below `1.3e-12`
- 1 kHz gain within `3.3e-9` of unity

Run the check with:

```sh
node validate-werkstatt-dc-blocker.mjs
```

Import `DC Blocker (2 Hz).opb` with openDAW's preset-bundle importer.
