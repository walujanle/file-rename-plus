# File Rename Plus

A fast, lightweight, and portable file renaming tool with a native GUI. Built with Rust.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-0.1.0-green.svg)
![Language](https://img.shields.io/badge/language-Rust-orange.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)

## Features

- **Find & Replace Mode** - Replace text patterns in filenames with support for:

  - Plain text matching
  - Regular expressions (Regex)
  - Case-sensitive or case-insensitive search

- **Iteration Numbering Mode** - Rename files with sequential numbers:

  - Customizable template with `{n}` placeholder
  - Configurable start number and padding

- **Smart Sorting** - Natural sort order (file1, file2, file10 instead of file1, file10, file2)

- **Live Preview** - See all changes before executing

- **Conflict Detection** - Visual warnings for duplicate filenames

- **Dark/Light Theme** - User-selectable theme preference

- **Settings Persistence** - Remembers your preferences across sessions

- **Keyboard Shortcuts**:
  - `Ctrl+O` - Open folder
  - `Delete` - Remove selected file
  - `Ctrl+Enter` - Execute rename

## System Requirements

| Component | Minimum                                           |
| --------- | ------------------------------------------------- |
| OS        | Windows 10+, macOS 10.15+, or Linux (X11/Wayland) |
| RAM       | 64 MB                                             |
| Disk      | 10 MB                                             |
| Display   | 900x650 resolution                                |

> **Platform Support Status**
>
> - **Windows**: Fully tested, verified, and distributed via releases.
> - **macOS / Linux**: Functionality is implemented but currently unverified. Users on these platforms are encouraged to build from source using `cargo build --release`.

1. **Select Files** - Click "Add Folder" to choose a directory containing files to rename
2. **Configure** - Choose between Find & Replace or Iteration Numbering mode
3. **Preview** - See the proposed changes in real-time
4. **Execute** - Click "Execute Rename" to apply the changes

The application uses atomic two-phase renaming to ensure data safety:

- Files are first renamed to temporary names
- Then renamed to final names
- This prevents data loss even if the process is interrupted

## Dependencies

### Runtime

- No external dependencies required
- The application is fully portable

### Build Dependencies

| Crate      | Purpose              |
| ---------- | -------------------- |
| `iced`     | Native GUI framework |
| `rfd`      | Native file dialogs  |
| `regex`    | Pattern matching     |
| `rusqlite` | Settings persistence |
| `dirs`     | Cross-platform paths |

## Installation

### Pre-built Binary (Windows)

1. Download the latest release from [Releases](https://github.com/walujanle/file-rename-plus/releases)
2. Extract the archive
3. Run `file-rename-plus.exe`

No installation required - the application is fully portable.

### Build from Source

#### Prerequisites

- [Rust](https://rustup.rs/) 1.75 or later

#### Build Steps

**Windows:**

```batch
# Run the automated build script
build_release_windows.bat

# Output will be in build_release_windows/
```

**macOS / Linux:**

```bash
# Build release binary
cargo build --release

# Binary location
./target/release/file-rename-plus
```

## Usage

### Find & Replace Mode

1. Add files using "Add Folder" button
2. Enter the text pattern to find
3. Enter the replacement text
4. Toggle Regex mode for pattern matching
5. Toggle Case Sensitive as needed
6. Review the preview
7. Click "Execute Rename"

**Example:**

- Find: `IMG_`
- Replace: `Photo_`
- Result: `IMG_001.jpg` → `Photo_001.jpg`

### Iteration Numbering Mode

1. Add files using "Add Folder" button
2. Enter template with `{n}` placeholder
3. Set start number and padding
4. Review the preview
5. Click "Execute Rename"

**Example:**

- Template: `vacation_{n}`
- Start: `1`, Padding: `3`
- Result: `DSC001.jpg` → `vacation_001.jpg`

## Settings Location

Settings are stored in an SQLite database at:

| Platform | Location                                                     |
| -------- | ------------------------------------------------------------ |
| Windows  | `%LOCALAPPDATA%\file-rename-plus\settings.db`                |
| macOS    | `~/Library/Application Support/file-rename-plus/settings.db` |
| Linux    | `~/.local/share/file-rename-plus/settings.db`                |

**Stored Settings:**

- Theme preference (Dark/Light)
- Regex mode toggle
- Case sensitivity toggle
- Template string
- Start number
- Padding value

## Project Structure

```
file-rename-plus/
├── src/
│   ├── main.rs          # Application entry point
│   ├── app/
│   │   └── mod.rs       # GUI state and message handling
│   ├── file_ops/
│   │   └── mod.rs       # Directory scanning and atomic rename
│   ├── rename/
│   │   └── mod.rs       # Find/replace and iteration logic
│   ├── security/
│   │   └── mod.rs       # Permission and privilege checks
│   ├── settings.rs      # SQLite settings persistence
│   ├── theme.rs         # Design system tokens
│   └── types.rs         # Shared data structures
├── Cargo.toml           # Dependencies and build config
├── LICENSE              # MIT License
├── README.md            # This file
└── build_release_windows.bat  # Windows build script
```

## Security

- **Permission Checks** - Validates write access before renaming
- **Admin Detection** - Warns if elevated privileges are needed
- **Input Validation** - Pattern length limits to prevent ReDoS attacks
- **Atomic Operations** - Two-phase rename prevents partial failures

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

**Leonard Walujan's Public Projects**
