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
- [ ] Decide the password question still listed under "Planned features" in the README. See
      [Key derivation strength](#key-derivation-strength) below: at the current settings the
      password itself decides whether a file can be broken, and neither a longer minimum nor
      costlier settings changes that much.

## Cutting it

1. Versions are already 5.0.0 in `core/`, `cli/` and `gui/` `Cargo.toml`, the CLI's `--version`,
   the About box and `gui/assets/Info.plist`.
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

## Key derivation strength

An open question, not a release blocker. The current format uses Argon2id with 64 MiB of memory
and 2 passes: libsodium's INTERACTIVE level.

**Can anything break it?** No known technology breaks the cryptography. There is no cryptanalytic
attack on Argon2id (the Password Hashing Competition winner, RFC 9106), and XChaCha20-Poly1305's
256-bit key can't be brute-forced by any physical means. The only way in is guessing the password,
and every guess costs a full key derivation. So whether a file falls depends on its password, not
on these settings.

To bound the biggest attacker: each guess moves roughly 256–384 MiB through memory, and memory
bandwidth is the bottleneck. A top GPU has about 1–3 TB/s, so it manages on the order of 10³–10⁴
guesses a second. An attacker far beyond any real cracking operation — a million top GPUs, about
the size of the largest GPU clusters in existence, all working on one file — gets about 10¹⁰
guesses a second. These are order-of-magnitude estimates from that bandwidth arithmetic, not
measured attack rates. Time for that attacker to try every possible password:

| Password | Possibilities | Time |
|---|---|---|
| Typical human-chosen, 12 characters | cracking dictionaries plus mangling rules find most within ~10⁹–10¹² guesses | **seconds to minutes** |
| 4 random words (diceware) | 3.7 × 10¹⁵ | **~4 days** |
| 5 random words | 2.8 × 10¹⁹ | ~90 years |
| 6 random words | 2.2 × 10²³ | ~700,000 years |
| 12 random characters | 4.8 × 10²³ | ~1.5 million years |

On average an attacker finds the password halfway through. Two things that don't change the
answer:

- **Custom chips (ASICs).** Argon2 is memory-hard, so they hit the same memory limits as GPUs.
  They might gain around 10×, which moves every row by less than an order of magnitude.
- **Quantum computers.** Grover's algorithm offers at most a quadratic speedup, and would need
  Argon2 running as a quantum circuit with 64 MiB of coherent quantum memory, orders of magnitude
  beyond any existing machine. Against the 256-bit cipher key it still leaves 128-bit security.

So a strong password (12 truly random characters, or five or more random words) is safe at these
settings against any known attacker, nation states included, and a weak password isn't safe at any
setting.

**Raising the settings.** Measured with the argon2 crate Cloaker uses, on a Ryzen 5 5600X:

| libsodium level | Memory | Passes | One key derivation | Cost to an attacker |
|---|---|---|---|---|
| **INTERACTIVE (current)** | 64 MiB | 2 | **76 ms** | 1× |
| MODERATE | 256 MiB | 3 | 422 ms | ~5.6× |
| SENSITIVE | 1 GiB | 4 | 2.3 s | ~30× |

By the standards' own wording the current level is on the light side for file encryption:
libsodium intends INTERACTIVE for online logins and points to SENSITIVE for "highly sensitive data
and non-interactive operations", and RFC 9106's fallback for memory-constrained environments is
64 MiB with 3 passes. But a higher level only multiplies the times in the first table, and the one
row where that changes the outcome is four random words against the extreme attacker, which goes
from days to weeks or months. It is a marginal improvement, not an urgent one. A longer minimum
password length helps even less: 12 random characters are already out of reach, and people meet a
longer minimum by padding familiar patterns.

**What's in the way of raising them.**

1. The parameters are not stored in the file, so changing them needs a new file format. New
   versions can keep reading today's format, but older ones can't read the new one — and Cloaker 4
   would misreport it: an unknown signature falls through to its legacy path, so it tells the user
   their password is wrong. Cloaker.js would have to change at the same time.
2. Cloaker.js caps how high this can go. 1 GiB is likely too much for browser WebAssembly on
   phones, and a 30× slowdown there could run to many seconds. 256 MiB is a far safer bet, but
   nobody has measured Cloaker.js yet.

**Recommendation.** Ship 5.0 at INTERACTIVE, keeping full compatibility with Cloaker 4 and
Cloaker.js. Treat stronger settings as future work: a new file format that records its parameters
(with the reader capping the stored values, so a crafted file can't make it allocate an absurd
amount of memory), probably at MODERATE, after measuring Cloaker.js on a phone. Encouraging
passphrases buys more than either. The GUI's hint already does; it could name "five or more random
words" to make the target concrete.

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
