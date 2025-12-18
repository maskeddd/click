<h1 align="center">
    Click
</h1>

A fast, cross-platform autoclicker built with Rust, GTK4, and libadwaita. Supports Windows, macOS, and Linux (X11/Wayland)

## Features
* Very fast clicking (~1K CPS)
* Adjustable interval
* Configurable click action [Single/Double]
* Configurable mouse button [Left/Right/Middle]

## Installation

TODO

## Building

### Linux
* Install [rustup](https://rustup.rs/)
* Install GTK4, libadwaita, and build tools:

#### Fedora:
```bash
sudo dnf install gtk4-devel libadwaita-devel gcc
```

#### Debian/Ubuntu:
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev build-essential
```

#### Arch:
```bash
sudo pacman -S gtk4 libadwaita base-devel
```

---

### macOS
* Install [rustup](https://rustup.rs/)
* Install [Homebrew](https://brew.sh/)
* Then run:
```bash
brew install gtk4 libadwaita
```

---

### Windows

See [gtk-rs](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation_windows.html)

---

### Build & Run
```bash
git clone https://codeberg.org/masked/click.git
cd click
cargo build --release
```

The compiled binary can be found at `./target/release/click`

## Roadmap
- [ ] Customizable global hotkey
- [ ] Randomized click interval
- [ ] Clicking a set location
- [ ] Hold clicks
- [ ] A logo

## Contributing
Go for it.

## License
MIT
