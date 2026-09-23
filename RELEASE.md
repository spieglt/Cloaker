# Releasing Cloaker

Version 5.0 replaces the Qt GUI with an egui one and the libsodium bindings with pure Rust. The
**file format is unchanged**, so Cloaker 4 and 5 read each other's files and Cloaker.js is
unaffected. That is the claim to be most careful about before publishing.

## Before tagging

Things the test suite cannot cover:

- [x] **Windows** builds and works (confirmed by hand). `gui/build.rs` now embeds the exe icon and
      version info with [winresource](https://github.com/BenjaminRi/winresource); the icon
      extracts at 16 to 256 px from a local release build.
- [x] **macOS** builds and works (confirmed by hand). Still unsigned — see gaps below.
- [x] **Linux: drag and drop, and saving** work (confirmed by hand; the tests can't simulate a real
      drop).
- [x] **Linux: folder and multi-file drops** show the right messages, **Tab and Enter** work through
      the password prompt, and the **progress bar** works (confirmed by hand).
- [x] **Linux: the rest of the GUI pass.** Cancelling the save dialog works, saving over an
      existing file gives the right warning, and the window stays responsive on large files
      (confirmed by hand).
- [x] **Cloaker.js interop** works (confirmed by hand).
- [x] **Old-version file.** A file from an earlier Cloaker decrypts correctly (confirmed by hand).
- [x] **Security review of the diff** found no vulnerabilities. Two items below the bar were fixed
      anyway: an output that is a hard link to the input is now refused (it used to destroy the
      input), and the release workflow pins linuxdeploy with a checksum and every action by commit
      SHA.
- [x] **The CLI's interactive password prompt** still prompts, and still hides what is typed, on
      rpassword 7 (confirmed by hand). The prompt now goes to the terminal device rather than
      stdout, which keeps it out of `-E`'s output stream.
- [ ] **CI on the current commits.** The Windows resource step in `gui/build.rs` has only been
      built on this machine; run 35304834017 predates it.

## Cutting it

1. Versions are already 5.0.0 in `core/`, `cli/` and `gui/` `Cargo.toml` and in
   `gui/assets/Info.plist`. `--version` and the About box follow `Cargo.toml`.
2. Commit, then `git tag v5.0.0 && git push --tags`.
3. `.github/workflows/release.yml` builds all three platforms, smoke-tests each binary, and opens a
   **draft** release with the artifacts attached.
4. Download the artifacts, run each one, then publish the draft.
5. The README's download line points at the Releases page and needs no change per release.

## Known gaps

- **Windows console output.** As a GUI-subsystem binary, `--help` and `--version` write to a console
  that isn't attached, so the text is dropped (the exit code is still 0). Harmless, but it means
  `cloaker --help` from cmd.exe prints nothing.
- **macOS signing.** Unsigned and un-notarized, so Gatekeeper will complain on first run. Signing
  needs a paid Apple developer account.
- **AppImage portability.** linuxdeploy bundles no extra libraries (the AppImage is 7.4 MB), so it
  relies on the host having libGL and libxkbcommon — true of any desktop, but worth testing on an
  older distribution than the one it was built on.
- **File associations.** `gui/assets/cloaker.desktop` declares `Exec=cloaker %f`, so "Open with"
  works once installed on Linux. macOS and Windows would need their own registration.

## What is already verified

- 42 tests: 25 in `core` (including committed fixtures produced by libsodium before the crypto
  migration, and both interop directions), 2 in `cli`, 15 in `gui` (headless UI tests via
  `egui_kittest`). A 43rd, which rewrites the fixtures, is `#[ignore]`d. `cargo clippy
  --all-targets -- -D warnings` and `cargo fmt --check` are clean.
- A CLI built from the pre-rewrite commit and the current one encrypt and decrypt each other's
  files in both directions.
- The AppImage has been built with linuxdeploy on this machine and launches, both headless
  (`--version`) and with a file argument.
- Built and run by hand on Windows and macOS; on Linux, drag and drop and saving confirmed.
- A fresh clone of the branch passes all 42 tests, fixtures included.
- CI passed on ubuntu-latest, macos-latest and windows-latest (run 35304834017).
- The CLI was exercised end to end against the release build: round trips including an empty file
  and a 3 MB one, wrong password and tampered data rejected with no output left behind, truncation
  after an authenticated chunk reported as truncation, `-o <directory>`, refusing an existing
  output file, refusing an output that is the input or a hard link to it, password files with a
  trailing newline, stdin/stdout piping, the short-password rule, all three committed fixtures, and
  round trips both ways against a CLI built from `master`.

## Release notes draft

**Cloaker 5.0**

- The desktop app has been rewritten in Rust using egui. Qt and the C++/Rust FFI layer are gone,
  which removes a class of crash and makes the app a single self-contained binary per platform.
- The crypto is now pure Rust — dryoc for libsodium's secretstream, RustCrypto's argon2 and scrypt
  for key derivation. **The file format has not changed**: Cloaker 4 and 5 read each other's files,
  files from versions 1 to 3 still decrypt, and Cloaker.js stays interoperable.
- New: a File > Open menu item, `cloaker <file>` on the command line, and a window that stays
  responsive while encrypting instead of freezing.
- CLI fixes: a failure now exits 1 instead of 0; `-o <directory>` is honoured when the input is
  elsewhere on disk; the 12-character minimum applies to `--password` and `--password-file`, not
  just the prompt; a password file's trailing newline is ignored (files made with the old behaviour
  are still readable); progress output is flushed.
- Safety fixes: a bad input can no longer truncate an existing output file, Cloaker refuses to write
  over its own input, and a truncated file is reported as truncated rather than as a wrong password.
