# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an ESP32 embedded Rust project for a remote controller using ESP-NOW wireless communication protocol. The project is `no_std` (bare metal) and uses the `esp-hal` ecosystem for ESP32 hardware abstraction.

## Build Commands

### Building
```bash
cargo build
cargo build --release
```

### Flashing and Monitoring
The default runner is configured to automatically flash and monitor:
```bash
cargo run           # Flashes to ESP32 and starts serial monitor with defmt logging
cargo run --release # Same, but with release optimizations
```

Manual flashing (if needed):
```bash
espflash flash --monitor --chip esp32 --log-format defmt target/xtensa-esp32-none-elf/debug/remote-controller
```

### Checking Code
```bash
cargo check         # Fast syntax and type checking
cargo clippy        # Linting (note: clippy::mem_forget is denied)
```

## Target Architecture

- **Target triple**: `xtensa-esp32-none-elf` (Xtensa architecture, ESP32 chip)
- **Toolchain**: Uses the `esp` Rust channel via `rust-toolchain.toml`
- **Build-std**: Compiles `core` and `alloc` from source for the target

## Key Dependencies and HAL Usage

- **esp-hal v1.0.0-rc.0**: Main hardware abstraction layer (GPIO, timers, RNG, etc.)
- **esp-wifi v0.15.0**: WiFi and ESP-NOW protocol support
- **defmt**: Logging framework optimized for embedded systems
- **esp-println**: Serial output with defmt integration

## Architecture Notes

### Memory Allocation
- Heap size is configured to 64KB via `esp_alloc::heap_allocator!` in [main.rs:37](src/bin/main.rs#L37)
- Uses `#![no_std]` - no standard library, only `core` and `alloc`

### ESP-NOW Communication
The main application implements a bidirectional ESP-NOW communication system:
- Broadcasts messages every 5 seconds to `BROADCAST_ADDRESS`
- Listens for incoming messages from peers
- Automatically adds responding peers to the peer list
- Channel 11 is used for communication

### Entry Point
- Uses `#[main]` attribute from esp-hal (not standard Rust main)
- Configures CPU clock to maximum via `CpuClock::max()`
- Minimal panic handler that loops forever (no panic unwinding)

### Linker Configuration
The [build.rs](build.rs) provides helpful error messages for common linker issues:
- Missing `defmt.x` linker script
- Missing `linkall.x` linker script
- ESP-WiFi scheduler configuration problems

## Environment Variables

- `DEFMT_LOG`: Set to `"info"` by default in [.cargo/config.toml](.cargo/config.toml)
- Can be overridden to control log levels: `trace`, `debug`, `info`, `warn`, `error`

## Code Quality Rules

- **Denied lint**: `clippy::mem_forget` - Using `mem::forget` with esp_hal types is unsafe, especially those holding buffers during data transfers
- Stack protector is enabled via rustflags: `-Z stack-protector=all`

## GPIO Configuration

- GPIO2 is configured as an output (typically the built-in LED on ESP32 dev boards)
- Toggles on each loop iteration for visual feedback
