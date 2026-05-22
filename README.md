# doit - Just Do It!

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/matsuokashuhei/doit-cli)

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

- ⏱️ **Time-based progress bar** with real-time updates (60-second intervals)
- 🎯 **Flexible time formats** (`2h`, `2025-08-10 09:00:00`, etc.)
- 🎨 **Smart dynamic formatting** - automatically adjusts time display based on duration
- 🔄 **Real-time updates** with optimized 60-second refresh
- 📝 **Custom title support** for motivational progress sessions
- 🎭 **Multiple styles** (default, hourglass, retro, synthwave, dotgrid)
- 🖥️ **Cross-platform** (Linux/macOS/Windows)
- ⚡ **Colored output** with intelligent time calculations

## Dynamic Time Display

**doit** automatically chooses the best time format based on your session duration:

- **≤24 hours**: `14:00 → 16:00`
- **≤7 days**: `08-16 09:00 → 08-19 09:00`
- **>7 days**: `2025-08-01 → 2025-12-31`

Progress and remaining time are also smartly formatted:
- **≤24 hours**: `2h 30m`
- **≤7 days**: `2d 5h`
- **>7 days**: `45d`

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

# Use retro style for military-style motivation
doit --start "2025-08-10 09:00:00" --duration "8h" --title "JUST DO IT!" --style retro

# Use synthwave style for synthwave-style aesthetic
doit --start "2025-08-10 09:00:00" --duration "8h" --title "CYBER FOCUS" --style synthwave

# Use hourglass style for a visual sand timer effect
doit --start "2025-08-10 09:00:00" --duration "8h" --title "Time is Flowing" --style hourglass

# Use dotgrid style for a dot-matrix long-range countdown
doit --start "2026-01-01" --end "2026-12-31" --title "Year Focus" --style dotgrid

# Short form options
doit -s "2025-08-10 09:00:00" -d "8h" -t "My Task"
```

### Options

- `--start` / `-s` Start time (optional, default: current time)
- `--end` / `-e` End time (mutually exclusive with --duration)
- `--duration` / `-d` Duration (e.g. `25m`, `2h`) (mutually exclusive with --end)
- `--title` / `-t` title message for motivation
- `--style` / `-S` Display style [default|hourglass|retro|synthwave|dotgrid]

## Example Output

### Default Style (With Custom Title)

```
Just Do It!
08:00 → 17:00   |   92%   |   8h 14m / 9h

█████████████████████████████████████████████████████████████████░░░░░░

46 m remaining
```

### Default Style (Without Title)

```
08:00 → 17:00   |   92%   |   8h 14m / 9h

█████████████████████████████████████████████████████████████████░░░░░░

46 m remaining
```

### Retro Style Example

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

### Synthwave Style Example

![synthwave](images/synthwave.png)

```
═ JUST DO IT ═
╔════════════════════════════════════════════════════════════════════════════╗
║ 2025-01-01 00:00  ████████████████████████░░░░░░░░░░░░░░  2025-12-31 23:59 ║
║                   62% | 227d elapsed | 137d remaining                     ║
╚════════════════════════════════════════════════════════════════════════════╝
                       ⚡ KEEP THE ENERGY FLOWING ⚡
```

### Hourglass Style Example

```
Time is Flowing
14:30 → 16:30   |   25%
           ┏━━━━━━━━━┓
           ┃░░░░░░░░░┃
           ┃██░░░░███┃
           ┃█████████┃
           ┃█████████┃
           ┃█████████┃
           ┃█████████┃
           ┗━┓█████┏━┛
             ┗━┓█┏━┛
               ┃┊┃
             ┏━┛┊┗━┓
           ┏━┛░░┊░░┗━┓
           ┃░░░░┊░░░░┃
           ┃░░░░┊░░░░┃
           ┃░░░░┊░░░░┃
           ┃░░░░┊░░░░┃
           ┃░░░░┊░░░░┃
           ┃░░░░┊░░░░┃
           ┃░░░░┊░░░░┃
           ┗━━━━━━━━━┛
elapsed: 30m   |   remaining: 1h 30m
```

### Dotgrid Style Example

```
•  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •
•  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •
•  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •
•  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •
Year Focus                                      348d left
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
cargo test
cargo clippy
cargo fmt
cargo build --release
```

## License

MIT

---

**Maximize your focus and motivation with this tool!**
