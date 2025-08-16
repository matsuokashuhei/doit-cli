# doit - Just Do It!

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)


**doit** is a CLI tool to visualize your time and boost your focus.
Set a duration or deadline, and see your progress in real time.
Use this tool to maximize your concentration and motivation!

```
$ doit -s "2025-08-12 08:00:00" -d 9h -t "Just Do It!"

Just Do It!
2025-08-12 08:00 → 2025-08-12 17:00   |   92%   |   8 h 14 m / 9h

█████████████████████████████████████████████████████████████████░░░░░░

46 m remaining
```
---

## Features

- ⏱️ Time-based progress bar
- 🎯 Flexible time formats (`2h`, `2025-08-10 09:00:00`, etc.)
- 🔄 Real-time updates
- 🎨 Colored output with decorative borders
- 📝 Custom title support for progress sessions
- 🎭 Multiple themes (default, retro, synthwave)
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
# Basic usage with duration
doit --duration "3h"

# Set custom start and end times
doit --start "2025-08-10 09:00:00" --end "2025-08-10 17:00:00"

# Add a custom title to your progress session
doit --start "2025-08-10 09:00:00" --duration "8h" --title "Deep Work Session"

# Use retro theme for military-style motivation
doit --start "2025-08-10 09:00:00" --duration "8h" --title "JUST DO IT!" --theme retro

# Use synthwave theme for synthwave-style aesthetic
doit --start "2025-08-10 09:00:00" --duration "8h" --title "CYBER FOCUS" --theme synthwave

# Short form options
doit -s "2025-08-10 09:00:00" -d "8h" -t "My Task"
```

### Options

- `--start` / `-s` Start time (default: now)
- `--end` / `-e` End time
- `--duration` / `-d` Duration (e.g. `25m`, `2h`)
- `--title` / `-t` Custom title for the progress session
- `--theme` Theme for the progress display (default, retro, synthwave)
- `--interval` / `-i` Update interval (seconds)
- `--verbose` / `-v` Display verbose output

## Example Output

### Default Theme (With Custom Title)

```
Just Do It!
2025-08-16 05:51 → 2025-08-16 14:51   |   92%   |   8 h 14 m / 9h

█████████████████████████████████████████████████████████████████░░░░░░

46 m remaining
```

### Default Theme (Without Title)

```
2025-08-16 05:51 → 2025-08-16 14:51   |   92%   |   8 h 14 m / 9h

█████████████████████████████████████████████████████████████████░░░░░░

46 m remaining
```

### Retro Theme Example

```
[JUST DO IT!] FOCUS SESSION INITIATED
============================================================
[START]     2025-08-16 05:51
[END]       2025-08-16 14:51
[ELAPSED]   92% | 8 h 14 m
[REMAINING] 46 m

[PROGRESS]
[███████████████████████████████████████████████████░░░░░░░]
============================================================
STATUS: > ALMOST THERE, SOLDIER! HOLD YOUR POSITION.
============================================================
(Q) QUIT | (CTRL+C) ABORT
```

### Synthwave Theme Example

![synthwave](images/synthwave.png)

```
═ JUST DO IT ═
╔════════════════════════════════════════════════════════════════════════════╗
║ 2025-01-01 00:00  ████████████████████████░░░░░░░░░░░░░░  2025-12-31 23:59 ║
║                   62% | 227 d elapsed | 137 d 11 h remaining               ║
╚════════════════════════════════════════════════════════════════════════════╝
                       ⚡ KEEP THE ENERGY FLOWING ⚡
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
