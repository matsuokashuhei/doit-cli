# doit - Just Do It!

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)


**doit** is a CLI tool to visualize your time and boost your focus.
Set a duration or deadline, and see your progress in real time.
Use this tool to maximize your concentration and motivation!

```
$ doit -s "2025-08-12 08:00:00" -d 9h -t "Just Do It!"

Just Do It!
08:00 → 17:00   |   92%   |   8h 14m / 9h

█████████████████████████████████████████████████████████████████░░░░░░

46 m remaining
```
---

## Features

- ⏱️ **Time-based progress bar** with real-time updates
- 🎯 **Flexible time formats** (`2h`, `2025-08-10 09:00:00`, etc.)
- 🎨 **Smart dynamic formatting** - automatically adjusts time display based on duration
- 🔄 **Real-time updates** with customizable intervals
- 📝 **Custom title support** for progress sessions
- 🎭 **Multiple themes** (default, retro, synthwave)
- 🖥️ **Cross-platform** (Linux/macOS/Windows)
- ⚡ **Colored output** with intelligent time calculations

## Dynamic Time Display

**doit** automatically chooses the best time format based on your session duration:

- **≤24h**: `14:30` (clean time format)
- **≤7d**: `8/16 14:30` (includes date)
- **>7d**: `2025-08-16 14:30` (full datetime)

Total duration is also smartly formatted:
- **≤1h**: `1h 23m 45s` (precise to seconds)
- **≤24h**: `12h 34m` (hours and minutes)
- **≤7d**: `3d 14h` (days and hours)
- **>7d**: `2w 4d` (weeks and days)

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
08:00 → 17:00   |   92%   |   8h 14m / 9h

█████████████████████████████████████████████████████████████████░░░░░░

46 m remaining
```

### Default Theme (Without Title)

```
08:00 → 17:00   |   92%   |   8h 14m / 9h

█████████████████████████████████████████████████████████████████░░░░░░

46 m remaining
```

### Retro Theme Example

```
[JUST DO IT!] FOCUS SESSION INITIATED
============================================================
[START]     2025-08-16 08:00:00
[END]       2025-08-16 17:00:00
[ELAPSED]   92% | 8h 14m
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
║                   62% | 227d elapsed | 137d remaining                     ║
╚════════════════════════════════════════════════════════════════════════════╝
                       ⚡ KEEP THE ENERGY FLOWING ⚡
```

## Time Examples

**doit** smartly formats time displays based on session length:

### Short Sessions (≤24 hours)
```bash
$ doit -s "14:00:00" -d "2h" -t "Focus Session"

Focus Session
14:00 → 16:00   |   25%   |   30m / 2h

██████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░

1h 30m remaining
```

### Medium Sessions (≤7 days)
```bash
$ doit -s "2025-08-16 09:00:00" -d "3d" -t "Sprint Week"

Sprint Week
8/16 09:00 → 8/19 09:00   |   33%   |   1d 2h / 3d

█████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░

1d 22h remaining
```

### Long Sessions (>7 days)
```bash
$ doit -s "2025-08-01 00:00:00" -e "2025-12-31 23:59:59" -t "Annual Goal"

Annual Goal
2025-08-01 00:00 → 2025-12-31 23:59   |   12%   |   15d / 152d

████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░

137d remaining
```

## Development & Testing

```bash
cargo test        # Run all tests (36 comprehensive test cases)
cargo clippy      # Lint checking
cargo fmt         # Code formatting
cargo build --release  # Optimized build
```

## Recent Updates (v0.7.0)

### ✨ New Features
- **Smart Dynamic Formatting**: Time displays automatically adapt to session duration
- **Enhanced Time Calculation**: Accurate progress for past, present, and future time ranges
- **Improved Total Duration Display**: 4-tier formatting system for better readability

### 🐛 Bug Fixes
- Fixed negative elapsed time display for past sessions
- Corrected remaining time calculations across all time scenarios
- Resolved timezone handling issues

### 🧪 Testing
- Expanded test coverage to 36 comprehensive test cases
- Added edge case validation for all time range scenarios
- Enhanced theme consistency testing

## License

MIT

---

**Maximize your focus and motivation with this tool!**
