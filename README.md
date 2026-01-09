# defaults-util

A TUI application that detects macOS `defaults` setting changes and generates reproducible commands.

## Overview

After making changes in System Settings, you can retrieve those changes as `defaults write` commands. Useful for dotfiles integration and reproducing settings.

## Installation

From crates.io:

```bash
cargo install defaults-util
```

From source:

```bash
cargo build --release
```

## Limitations

- macOS only
- Some domains may not be readable (they will be skipped)
- Clipboard copy uses `pbcopy`

## License

MIT
