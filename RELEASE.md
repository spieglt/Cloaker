# Releasing Cloaker

Version 5.0 replaces the Qt GUI with an egui one and the libsodium bindings with pure Rust. The
**file format is unchanged**, so Cloaker 4 and 5 read each other's files and Cloaker.js is
unaffected. That is the claim to be most careful about before publishing.

## Before tagging

Things the test suite cannot cover:

- [ ] **Linux GUI pass.** Drag and drop a file (the tests can't simulate a real drop), drop a folder
      and several files at once, cancel the save dialog, save over an existing file, Tab and Enter
      through the password prompt, and run a multi-gigabyte file to watch the progress bar move and
      the window stay responsive.
- [ ] **Windows build.** Never compiled there. Check the window opens with no console behind it,
      drag and drop works, and the file dialog looks right. The exe has no icon yet — see gaps below.
- [ ] **macOS build.** Never compiled there. Check the `.app` opens, the icon appears, and the file
      dialog works. It will be unsigned: first launch needs right-click → Open.
- [ ] **Cloaker.js interop.** Encrypt at cloaker.mobi and decrypt on the desktop, then the reverse.
      The format is verified against libsodium in `core/tests/compatibility.rs`, but nothing has
      exercised the browser implementation.
- [x] **Old-version file.** A file from an earlier Cloaker decrypts correctly (confirmed by hand).
- [ ] **Security review of the diff**, particularly the KDF parameter translation in
      `core/src/lib.rs` and `core/src/legacy.rs`.
- [ ] Decide the password-minimum question still listed under "Planned features" in the README.

## Cutting it

1. Versions are already 5.0.0 in `core/`, `cli/` and `gui/` `Cargo.toml`, the CLI's `--version`,
   the About box and `gui/assets/Info.plist`.
2. Commit, then `git tag v5.0.0 && git push --tags`.
3. `.github/workflows/release.yml` builds all three platforms, smoke-tests each binary, and opens a
   **draft** release with the artifacts attached.
4. Download the artifacts, run each one, then publish the draft.
5. The README's download line points at the Releases page and needs no change per release.

## Known gaps

- **Windows icon.** The exe ships without one; wiring it needs a `build.rs` using
  [winresource](https://github.com/BenjaminRi/winresource) pointing at `gui/assets/cloaker.ico`.
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

- 41 tests: 24 in `core` (including committed fixtures produced by libsodium before the crypto
  migration, and both interop directions), 2 in `cli`, 15 in `gui` (headless UI tests via
  `egui_kittest`). `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are clean.
- A CLI built from the pre-rewrite commit and the current one encrypt and decrypt each other's
  files in both directions.
- The AppImage has been built with linuxdeploy on this machine and launches, both headless
  (`--version`) and with a file argument.

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
