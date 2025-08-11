# doit - Just Do It!

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**doit** is a CLI tool to visualize your time and boost your focus.
Set a duration or deadline, and see your progress in real time.
Use this tool to maximize your concentration and motivation!

---

## Features

- ⏱️ Time-based progress bar
- 🎯 Flexible time formats (`2h`, `2025-08-10 09:00:00`, etc.)
- 🔄 Real-time updates
- 🎨 Colored output
- 🖥️ Cross-platform (Linux/macOS/Windows)

## Install

### Via Homebrew (Recommended)

```bash
# Add the tap
brew tap matsuokashuhei/doit

# Install doit
brew install doit
```

### Manual Installation

```bash
git clone https://github.com/matsuokashuhei/doit.git
cd doit
cargo build --release
sudo cp target/release/doit /usr/local/bin/
```

### Pre-built Binaries

Download the latest release from [GitHub Releases](https://github.com/matsuokashuhei/doit/releases)

## Usage

```bash
doit --duration "3h"
doit --start "2025-08-10 09:00:00" --end "2025-08-10 17:00:00"
```

- `--start` Start time (default: now)
- `--end` End time
- `--duration` Duration (e.g. `25m`, `2h`)
- `--interval` Update interval (seconds)

## Example Output

```
██████████████████████████████████████░░░░░░
Start:   2025-08-10 09:00:00
End:     2025-08-10 12:00:00
Elapsed: 60 % | 1 h 48 m
(Quit: q, ESC, Ctrl+C)
```

## Development & Testing

```bash
cargo test
cargo clippy
cargo fmt
```

## License

MIT

---

**Maximize your focus and motivation with this tool!**
