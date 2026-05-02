# Rustpad-X

Rustpad-X is a native Windows text/code editor written in Rust with a low-level Win32 interface. It is designed for fast local editing, tabbed documents, project file opening, dark/light themes, and everyday source/text workflows.

## Features

- Native Windows GUI with no runtime framework dependency.
- Multiple tabs with recent files and persistent session restore.
- Short tab labels for readability, with full file paths shown on tab hover tooltips.
- Cleaner tab close buttons and hover feedback.
- Open single files or scan a project folder for matching source/text/code files.
- Broader text/code file opening support, including common config and programming extensions such as `.txt`, `.ini`, `.cfg`, `.log`, `.md`, `.rs`, `.py`, `.js`, `.ts`, `.html`, `.css`, `.json`, `.xml`, `.yml`, `.yaml`, `.toml`, `.vbs`, `.bat`, `.ps1`, `.c`, `.cpp`, `.h`, `.hpp`, `.java`, `.cs`, `.go`, `.php`, `.rb`, `.lua`, `.sql`, `.rtf`, and `.doc` when readable as text.
- Save, Save As, Save All, print, and reveal files in File Explorer.
- Find, find next/previous, replace, go to line, and top/bottom navigation.
- Right-click editor menu with Cut, Copy, Paste, and Select All.
- Line numbers, code folding, zoom, font selection, fullscreen mode, and light/dark themes.
- Side-by-side tab comparison with optional synchronized Page Up/Page Down scrolling.
- Compare Page Up/Page Down sync is persisted with the session.
- Font and theme formatting fixes so newly visible text keeps the selected font/color while scrolling.
- Fullscreen layout fixes for a cleaner borderless editor view.
- Startup CPU affinity reset to allow all available CPU cores.
- Embedded Windows icon and executable metadata for Rustpad-X.

> Note: Bracket highlighting was intentionally removed to avoid RichEdit formatting conflicts.

## Build

Install Rust for Windows, then build from the project root:

```powershell
cargo build --release
```

The release executable is written to:

```text
target\release\Rustpad-X.exe
```

For a development build, use:

```powershell
cargo build
```

## Run

```powershell
cargo run
```

You can also pass file paths on the command line:

```powershell
cargo run -- C:\path\to\file.txt
```

## Project Layout

- `src\main.rs` contains the Win32 editor implementation.
- `assets\Rustpad-X.ico` is embedded into Windows builds, if present.
- `assets\Rustpad-X.resources.o` may be linked by `.cargo\config.toml` to provide the app icon and executable metadata.
- `help.txt` provides end-user usage notes.
- `security.md` explains how to report security issues.

## GitHub / Open Source Notes

Recommended files to commit:

```text
Cargo.toml
Cargo.lock
src/main.rs
assets/        optional
README.md
help.txt
security.md
LICENSE
.gitignore
```

Recommended files to ignore:

```text
target/
*.exe
*.pdb
*.dll
*.zip
*.rs.bk
session.txt
.vscode/
.idea/
```

Rustpad-X currently targets Windows. On non-Windows systems, the binary prints a short message instead of launching the GUI.
