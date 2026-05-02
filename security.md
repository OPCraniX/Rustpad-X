# Security Policy

## Supported Versions

Rustpad-X is handled on the current source tree and latest build artifacts. Security fixes should target the latest public commit unless a maintainer states otherwise.

## Reporting a Vulnerability

Please report suspected security issues privately to the project maintainer before sharing details publicly. Include:

- A clear description of the issue.
- Steps to reproduce it.
- The affected Rustpad-X version or commit/build date.
- Any files, input, or environment details needed to verify the behavior.
- Whether the issue happens when opening a specific text/code/config file type.

## Scope

Useful reports include:

- Crashes or memory-safety issues triggered by opening files.
- Unsafe file handling or path handling bugs.
- Unexpected command execution behavior.
- Path traversal or project-folder scanning issues.
- Bugs that could corrupt, overwrite, or lose user data unexpectedly.
- Problems caused by malformed text-like files, including INI, config, script, source, RTF, or DOC-like content that Rustpad-X attempts to open as text.

## Out of Scope

Normally out of scope unless paired with a concrete exploit or data-loss issue:

- General UI glitches.
- Expected failure to parse binary-only document formats.
- Issues requiring modified local builds not based on the public source.
- Reports against old ZIP artifacts after a newer source fix is available.

## Handling

The maintainer should acknowledge reports, reproduce the issue, prepare a fix, and publish release notes once a safe update is available. If a report turns out not to be security-sensitive, it can be handled as a normal bug.

## Safety Notes

Rustpad-X is a local native Windows editor using low-level Win32 and RichEdit APIs. It should be treated as a local desktop application, not as a sandbox for untrusted files.
