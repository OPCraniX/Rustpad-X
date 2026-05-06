# Rustpad-X

Rustpad-X is a native Windows text/code editor written in Rust with a low-level Win32 interface. It is designed for fast local editing, tabbed documents, project file opening, dark/light themes, and everyday source/text workflows.

## Features

- Native Windows GUI with no runtime framework dependency.
- Multiple tabs with recent files and persistent session restore.
- Full tab labels without file extensions.
- Cleaner tab close buttons and hover feedback.
- Open single files or scan a project folder for matching source/text/code files.
- Broader text/code file opening support, including common config and programming extensions such as `.txt`, `.ini`, `.cfg`, `.log`, `.md`, `.rs`, `.py`, `.js`, `.ts`, `.html`, `.css`, `.json`, `.xml`, `.yml`, `.yaml`, `.toml`, `.vbs`, `.bat`, `.ps1`, `.c`, `.cpp`, `.h`, `.hpp`, `.java`, `.cs`, `.go`, `.php`, `.rb`, `.lua`, `.sql`, `.rtf`, and `.doc` when readable as text.
- Save, Save As, Save All, print, and reveal files in File Explorer.
- Find, find next/previous, replace, go to line, and top/bottom navigation.
- Right-click editor menu with Cut, Copy, Paste, and Select All.
- Click the path bar to copy the current file name or location, with a Copied indicator beside the cursor.
- Click a gutter line number to copy that line of code.
- Word wrap is enabled by default and persists with the session.
- Line numbers, code folding, zoom, font selection, fullscreen mode, and light/dark themes.
- Side-by-side tab comparison with optional synchronized Page Up/Page Down scrolling.
- Compare Page Up/Page Down sync is persisted with the session.
- Font and theme formatting fixes so newly visible text keeps the selected font/color while scrolling.
- Fullscreen layout fixes for a cleaner borderless editor view.
- Startup CPU affinity reset to allow all available CPU cores.
- Embedded Windows icon and executable metadata for Rustpad-X.

## Project Layout

- `src\main.rs` contains the app entrypoint.
- `src\gui\` contains the Win32 editor implementation split into focused modules.
- `assets\Rustpad-X.ico` is embedded into Windows builds, if present.
- `assets\Rustpad-X.resources.o` may be linked by `.cargo\config.toml` to provide the app icon and executable metadata.
- `help.txt` provides end-user usage notes.
- `security.md` explains how to report security issues.

Rustpad-X currently targets Windows. On non-Windows systems, the binary prints a short message instead of launching the GUI.
