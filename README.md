# doit - Progress Monitor

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A command-line tool for time-based progress visualization. Specify start and end times (or duration) to display real-time progress bars.

## Features

- ⏱️ **Time-based Progress Display** - Visualize progress across specified time ranges
- 🎯 **Flexible Time Specification** - Support for multiple datetime formats and relative time
- 🔄 **Real-time Updates** - Live display with configurable update intervals
- 🎨 **Color Text** - Clear and visible progress bar display
- ⚡ **Lightweight & Fast** - Minimal resource usage
- 🖥️ **Cross-platform** - Linux, macOS, Windows support

## Installation

### Using Cargo (Recommended)

```bash
# Clone the repository
git clone https://github.com/matsuokashuhei/doit.git
cd doit

# Build and install
cargo build --release
sudo cp target/release/doit /usr/local/bin/
```

### Build from Source

```bash
git clone https://github.com/matsuokashuhei/doit.git
cd doit
cargo install --path .
```

## Usage

### Basic Syntax

```bash
doit [OPTIONS] --end <END_TIME> | --duration <DURATION>
```

### Command Line Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--start` | `-s` | Start time | Current time |
| `--end` | `-e` | End time | - |
| `--duration` | `-d` | Duration | - |
| `--interval` | `-i` | Update interval in seconds | 5 |
| `--verbose` | `-v` | Enable verbose output | false |
| `--help` | `-h` | Show help | - |
| `--version` | `-V` | Show version | - |

**Note**: `--end` and `--duration` are mutually exclusive. Specify only one of them.

### Time Formats

#### DateTime Formats
```bash
# Date only (time defaults to 00:00:00 or 23:59:59)
doit --start "2025-08-10" --end "2025-08-11"

# Complete datetime
doit --start "2025-08-10 09:00:00" --end "2025-08-10 17:00:00"

# ISO 8601 format
doit --start "2025-08-10T09:00:00" --end "2025-08-10T17:00:00"

# With timezone
doit --start "2025-08-10T09:00:00+09:00" --end "2025-08-10T17:00:00+09:00"
```

#### Relative Time Formats (Duration)
```bash
# Seconds
doit --duration "30s"

# Minutes
doit --duration "45m"

# Hours
doit --duration "2h"

# Days
doit --duration "7d"
```

### Usage Examples

#### 1. Work Session Tracking
```bash
# Track an 8-hour work day
doit --start "2025-08-10 09:00:00" --end "2025-08-10 17:00:00"

# 3-hour work session from now
doit --duration "3h"
```

#### 2. Meeting Timer
```bash
# 1-hour meeting with 30-second updates
doit --duration "1h" --interval 30

# 2-hour meeting from specific time
doit --start "2025-08-10 14:00:00" --duration "2h"
```

#### 3. Project Deadline Tracking
```bash
# Track progress to project deadline (hourly updates)
doit --start "2025-08-01" --end "2025-08-31" --interval 3600
```

#### 4. Short-term Timers
```bash
# Pomodoro timer (25 minutes)
doit --duration "25m" --interval 60

# Short break (5 minutes)
doit --duration "5m" --interval 10
```

### Output Example

```
██████████████████████████████████████████████████████░░░░░░
██████████████████████████████████████████████████████░░░░░░

Start:   2025-08-10 09:00:00
End:     2025-08-10 18:00:00
Elapsed: 89 % | 8 h 3 m

(Quit: q, ESC, or Ctrl+C)
```

### Controls

While the program is running, you can use these keys:

- **q** - Quit
- **ESC** - Quit
- **Ctrl+C** - Quit

## Development

### Development Environment Setup

```bash
# Install dependencies
cargo build

# Run tests
cargo test

# Run in development mode
cargo run -- --duration "10s" --interval 1
```

### Project Structure

```
src/
 main.rs          # Application entry point
 lib.rs           # Library exports
 cli.rs           # Command-line argument parsing
 progress_bar.rs  # Progress bar calculation and rendering
```

### Dependencies

- **clap** - Command-line argument parsing
- **chrono** - Date and time operations
- **colored** - Color text output
- **crossterm** - Cross-platform terminal control
- **anyhow** - Error handling
- **thiserror** - Custom error types
- **regex** - Regular expressions (for duration parsing)

## Testing

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Test specific functionality
cargo test parse_duration
```

## Performance

doit is designed to be lightweight and efficient:

- **Startup time**: <50ms
- **Memory usage**: <10MB during operation
- **CPU usage**: Minimal, only during updates
- **Update frequency**: Configurable from 1-59 seconds

## Platform Support

| Platform | Architecture | Status |
|----------|-------------|--------|
| Linux | x86_64 | ✅ Fully supported |
| Linux | ARM64 Fully supported | |
| macOS | Intel (x86_64) | ✅ Fully supported |
| macOS | Apple Silicon (ARM64) | ✅ Fully supported |
| Windows | x86_64 | ✅ Fully supported |

## Troubleshooting

### Common Issues

**Q: Program doesn't start or shows errors**
- Check that start time is before end time
- Verify time format is correct
- Ensure interval is between 1-59 seconds

**Q: Progress bar doesn't update**
- Check terminal compatibility
- Verify update interval setting
- Try reducing interval value

**Q: Time parsing errors**
- Use supported formats: `YYYY-MM-DD HH:MM:SS`, `YYYY-MM-DD`, or relative time (`1h`, `30m`, etc.)
- Check timezone format if using timezone-aware times

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Author

Shuhei Matsuoka - [matsuokashuheiii@gmail.com](mailto:matsuokashuheiii@gmail.com)

## Contributing

Pull requests and issue reports are welcome. Please check existing issues before contributing.

### Contributing Guidelines

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Ensure all tests pass: `cargo test`
5. Check code formatting: `cargo fmt`
6. Submit a pull request

## Changelog

### v0.2.0
- Added duration-based time specification
- Improved time parsing with multiple formats
- Enhanced progress bar visualization
- Added comprehensive test coverage

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) for performance and reliability
- Uses [clap](https://clap.rs/) for command-line parsing
- Uses [chrono](https://github.com/chronotope/chrono) for robust time handling
- Uses [crossterm](https://github.com/crossterm-rs/crossterm) for cross-platform terminal support

---

**⭐ If you find this tool useful, please give it a star!**
