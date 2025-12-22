<h1 align="center">
    Click
</h1>

A fast, cross‑platform autoclicker built with Rust and **egui**. Supports Windows, macOS, and Linux (X11/Wayland).

## Features
* Very fast clicking (~1K CPS)
* Adjustable interval
* Configurable click action [Single/Double]
* Configurable mouse button [Left/Right/Middle]

## Installation

TODO

## Building

### Prerequisites
* Install [rustup](https://rustup.rs/)

---

### Linux / macOS / Windows
```bash
git clone https://github.com/maskeddd/click.git
cd click
cargo build --release
```

The compiled binary can be found at `./target/release/click`

---

## Roadmap
- [ ] Customizable global hotkey
- [ ] Randomized click interval
- [ ] Clicking a set location
- [ ] Hold clicks

## License
MIT
