# Releasing Cloaker

Version 5.0 replaces the Qt GUI with an egui one and the libsodium bindings with pure Rust. The
**file format is unchanged**, so Cloaker 4 and 5 read each other's files and Cloaker.js is
unaffected. That is the claim to be most careful about before publishing.

## Before tagging

Things the test suite cannot cover:

- [x] **Windows** builds and works (confirmed by hand). The exe still has no icon — see gaps below.
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
- [ ] Decide the password question still listed under "Planned features" in the README. The
      minimum length matters less than the cost of each guess — see
      [Key derivation strength](#key-derivation-strength) below.

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

## Key derivation strength

An open question, not a release blocker. The current format uses Argon2id with 64 MiB of memory
and 2 passes: libsodium's INTERACTIVE level. Measured with the argon2 crate Cloaker uses, on a
Ryzen 5 5600X:

| libsodium level | Memory | Passes | One key derivation | Cost to an attacker |
|---|---|---|---|---|
| **INTERACTIVE (current)** | 64 MiB | 2 | **76 ms** | 1× |
| MODERATE | 256 MiB | 3 | 422 ms | ~5.6× |
| SENSITIVE | 1 GiB | 4 | 2.3 s | ~30× |

**Adequate, but on the light side for file encryption.**

- It is above OWASP's Argon2id minimums (for example 19 MiB with 2 passes, or 46 MiB with 1).
- libsodium means INTERACTIVE for online logins, where a server derives a key on every request,
  and points to SENSITIVE for "highly sensitive data and non-interactive operations". File
  encryption is the non-interactive case: one derivation per file, and anyone holding the file
  can guess offline for as long as they like.
- RFC 9106's fallback for memory-constrained environments is 64 MiB with 3 passes; Cloaker is
  slightly below it.
- At 76 ms nobody would notice going several times higher, and the multiplier applies directly to
  an attacker's time. It won't save a bad password, but it raises the bar for mediocre ones.
- The minimum password length matters less by comparison: 12 random characters are already out of
  reach, and people meet a longer minimum by padding familiar patterns.

**What's in the way.**

1. The parameters are not stored in the file, so changing them needs a new file format. New
   versions can keep reading today's format, but older ones can't read the new one — and Cloaker 4
   would misreport it: an unknown signature falls through to its legacy path, so it tells the user
   their password is wrong. Cloaker.js would have to change at the same time.
2. Cloaker.js caps how high this can go. 1 GiB is likely too much for browser WebAssembly on
   phones, and a 30× slowdown there could run to many seconds. 256 MiB is a far safer bet, but
   nobody has measured Cloaker.js yet — check it on a phone before choosing a level.

**Recommendation.** Move to MODERATE (256 MiB, 3 passes) in a new file format that records its
parameters, so they can be raised later without another break. The reader must cap the stored
values so a crafted file can't make it allocate an absurd amount of memory. The open decision is
timing: in 5.0, which is already the major release but means Cloaker 4 can't open the new files,
or in a later release, which keeps 5.0 fully compatible with 4.

## What is already verified

- 41 tests: 24 in `core` (including committed fixtures produced by libsodium before the crypto
  migration, and both interop directions), 2 in `cli`, 15 in `gui` (headless UI tests via
  `egui_kittest`). `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are clean.
- A CLI built from the pre-rewrite commit and the current one encrypt and decrypt each other's
  files in both directions.
- The AppImage has been built with linuxdeploy on this machine and launches, both headless
  (`--version`) and with a file argument.
- Built and run by hand on Windows and macOS; on Linux, drag and drop and saving confirmed.
- A fresh clone of the branch passes all 43 tests, fixtures included.
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
