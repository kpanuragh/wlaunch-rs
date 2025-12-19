# WLaunch

A fast, Raycast-like application launcher for Linux built with Rust and [Iced](https://github.com/iced-rs/iced) GUI framework.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Build](https://github.com/kpanuragh/wlaunch-rs/actions/workflows/build.yml/badge.svg)

## Features

- **Application Launcher** - Quick access to all your installed applications
- **Multiple Search Modes** - Switch between different modes using prefixes
- **Clipboard History** - Background daemon monitors and saves clipboard history
- **Fuzzy Search** - Smart fuzzy matching for faster results
- **Keyboard Navigation** - Full keyboard control with vim-style bindings
- **Dark Theme** - Modern dark UI with accent colors
- **Lightweight** - Fast startup, minimal resource usage

### Search Modes

| Prefix | Mode | Description |
|--------|------|-------------|
| *(none)* | Apps | Search installed applications |
| `w` | Windows | Switch between open windows |
| `ps` | Processes | View and kill running processes |
| `wifi` | Network | Connect to WiFi networks |
| `bt` | Bluetooth | Manage Bluetooth devices |
| `vol` | Audio | Control audio sinks and volume |
| `cb` | Clipboard | Browse clipboard history |
| `note` | Notes | Quick notes |
| `todo` | Todos | Task management |
| `snip` | Snippets | Code/text snippets |
| `ssh` | SSH | SSH connections from ~/.ssh/config |
| `docker` | Docker | Manage Docker containers |
| `e` | Emoji | Emoji picker |
| `f` | Files | Search files |
| `r` | Recent | Recently opened files |
| `timer` | Timer | Stopwatch and timers |
| `bw` | Bitwarden | Password manager integration |
| `ai` | AI | AI assistant queries |
| `g` | Google | Web search |
| `gh` | GitHub | GitHub search |
| `yt` | YouTube | YouTube search |

**Auto-detected modes:**
- Calculator: Type math expressions (e.g., `2+2`, `sqrt(16)`)
- Converter: Type conversions (e.g., `100 usd to eur`, `5 km in miles`)

## Installation

### From Releases

Download the latest release for your distribution:

```bash
# Debian/Ubuntu
sudo dpkg -i wlaunch_*.deb

# Fedora/RHEL
sudo rpm -i wlaunch-*.rpm

# Binary
tar -xzf wlaunch-*-linux-x86_64.tar.gz
sudo mv wlaunch /usr/local/bin/
```

### Building from Source

```bash
# Install dependencies (Debian/Ubuntu)
sudo apt install libxkbcommon-dev libwayland-dev libxcb-render0-dev \
    libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-x11-dev \
    libx11-dev libx11-xcb-dev libxcursor-dev libxrandr-dev libxi-dev \
    libfontconfig1-dev libfreetype6-dev libssl-dev pkg-config

# Clone and build
git clone https://github.com/kpanuragh/wlaunch-rs.git
cd wlaunch-rs
cargo build --release

# Install
sudo cp target/release/wlaunch /usr/local/bin/
```

## Usage

### Launch the Application

```bash
wlaunch
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate results |
| `Ctrl+J` / `Ctrl+K` | Navigate results (vim-style) |
| `Ctrl+N` / `Ctrl+P` | Navigate results (emacs-style) |
| `Enter` | Execute selected item |
| `Escape` | Close launcher |

### Command Line Options

```bash
wlaunch              # Launch GUI
wlaunch --daemon     # Run clipboard monitor daemon
wlaunch --help       # Show help
```

## Clipboard Daemon

WLaunch includes a clipboard history manager that runs in the background.

### Enable with Systemd (Recommended)

```bash
# Enable and start the service
systemctl --user enable --now wlaunch-clipboard.service

# Check status
systemctl --user status wlaunch-clipboard.service

# View logs
journalctl --user -u wlaunch-clipboard.service -f
```

### XDG Autostart

The clipboard daemon automatically starts on login if installed from .deb/.rpm packages. To disable:

```bash
# Disable autostart
rm ~/.config/autostart/wlaunch-clipboard.desktop
```

## Configuration

Configuration is stored in `~/.config/wlaunch/config.json`:

```json
{
  "clipboard_history_size": 50,
  "max_recent_files": 100,
  "gemini_api_key": "your-api-key",
  "bitwarden_email": "your-email"
}
```

### Data Locations

| File | Description |
|------|-------------|
| `~/.config/wlaunch/config.json` | Configuration |
| `~/.config/wlaunch/clipboard_history.json` | Clipboard history |
| `~/.config/wlaunch/notes.json` | Notes |
| `~/.config/wlaunch/todos.json` | Todos |
| `~/.config/wlaunch/snippets.json` | Snippets |
| `~/.config/wlaunch/scripts/` | Custom scripts |

## Custom Scripts

Place executable scripts in `~/.config/wlaunch/scripts/` to make them searchable:

```bash
mkdir -p ~/.config/wlaunch/scripts
echo '#!/bin/bash
notify-send "Hello from WLaunch!"' > ~/.config/wlaunch/scripts/hello
chmod +x ~/.config/wlaunch/scripts/hello
```

## Development

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run
cargo run

# Run with logging
RUST_LOG=debug cargo run

# Lint
cargo clippy

# Format
cargo fmt
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Iced](https://github.com/iced-rs/iced) - Cross-platform GUI library for Rust
- [Raycast](https://raycast.com/) - Inspiration for the UI/UX design
