# Pocket Computer (ESP32-S3)
An experimental pocket computer built on a **LilyGo T-HMI (ESP32-S3 + touchscreen)**.
The project explores building a small interactive system in Rust, with a focus on **responsive input, incremental rendering, and simple app-style software structure**.

![Pocket Computer Demo](media/demo.gif)

## Features
- Rust (no_std)
- Custom grid-based screen model
- Dirty rendering (only redraws what changed)
- Touch input + on-screen buttons
- App framework with launcher
- System title bar and status bar
- Color picker app

## Performance
The UI originally used full-screen redraws (~200ms per update). The renderer was reworked to use dirty cell tracking and incremental updates.

Typical timings now:

- Full screen clear: ~30–40ms
- Normal UI updates: ~8–15ms

This significantly improves input responsiveness.

## Hardware / Stack
- LilyGo T-HMI
- ESP32-S3
- ST7789 LCD
- Rust (`no_std`)
- `embedded-graphics`, `mipidsi`

## Roadmap / Ideas
- Calibration & settings app
- Persistent storage (mem-fs integration)
- Games (Snake, Breakout, etc.)
- Keyboard input (on-screen / Bluetooth)
- Optional DMA-backed graphics backend
- Additional apps & tools