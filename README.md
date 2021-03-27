# Cloaker

**Mobile version available at https://cloaker.mobi! Interoperable with the GUI and CLI versions!** [Repo here.](https://github.com/spieglt/Cloaker.js)

**Version 4 Release Notes:**
- Increased speed by ??.
- Added progress bar to GUI and progress output to CLI.
- Added CLI flags to:
    - encrypt/decrypt from stdin (`-E`, `-D`)
    - encrypt/decrypt to stdout (`-O`)
    - read password from a file (`-f`) instead of prompting for it
    - accept password directly with `-p` instead of prompting (not recommended: password will appear in shell history)
- Changed password hashing from `scryptsalsa208sha256` to `Argon2id`.
- Windows GUI version builds with MSVC instead of MinGW.

Ready-to-use downloads on the [Releases](https://github.com/spieglt/Cloaker/releases) page

### Very simple cross-platform file encryption

Have you ever wanted to protect a file with a password and found it unnecessarily difficult to do so? Cloaker aims to provide the most straightforward file encryption possible. Just drop a file onto the window, set a password, and choose where to save it. To decrypt, drop the encrypted file on the window, enter the password, and choose the output location. No installation required: on Windows it's a single `.exe`, on Mac an `.app` bundle, and on Linux a `.run` file that links to your system's Qt.

![Demo](demo.gif)

**Data Loss Disclaimer:** if you lose or forget your password, **your data cannot be recovered!** Use a password manager or another secure form of backup. Cloaker uses the pwhash and secretstream APIs of [libsodium](https://github.com/jedisct1/libsodium) via [sodiumoxide](https://github.com/sodiumoxide/sodiumoxide).

# Compilation instructions:
`cd adapter && cargo build && cargo build --release`. 

Then open `gui/cloaker/cloaker.pro` in Qt Creator (Qt 5.15.2), make sure kit is 64bit, and build.

If you want to make a distributable on... 

**Mac:** use the `macdeployqt` script in your Qt installation's `bin/` directory with the built `.app` bundle as argument.

**Linux:** just use the dynamically linked binary that Qt Creator builds by default, it's more portable on Linux.

**Windows only:** install Visual Studio 2019 Community (including the `Desktop development with C++` feature), launch the `x64 Native Tools Command Prompt` (found in `Start Menu > Visual Studio 2019`) and compile Qt statically with something like:
```
> cd C:\; mkdir qt-static; cd qt-static
> C:\Qt\5.15.2\Src\configure.bat -release -static -no-pch -optimize-size -opengl desktop -platform win32-msvc -skip webengine -nomake tools -nomake tests -nomake examples
> nmake.exe
```
Run `rustup default stable-x86_64-pc-windows-msvc` to make sure you're using MSVC, and rerun `cargo build --release` from `adapter/` if you weren't.

Then go to Qt Creator settings, add a new version of Qt, and point to `C:\qt-static\qtbase\bin\qmake.exe`. Then add a new "Kit" that points to this Qt version, and build Release version with that kit selected.

# CLI compilation instructions
`cd cli; cargo build --release`. Executable will be at `cloaker/cli/target/release/cloaker_cli`(`.exe`).

# Planned features:
- Change minimum password length to 14 or 16?

# Issues:
- Please tell me about them.
- Backward compatibility notes:
    - to decrypt a file made with version 1.0 or 1.1 of Cloaker (with Encrypt and Decrypt buttons), the filename must end with the ".cloaker" extension. Files encrypted with later versions are not subject to this restriction.
    - Cloaker version 4 can decrypt files that were encrypted with previous versions, but previous versions cannot decrypt files encrypted with version 4+.

If you've used Cloaker, please send me feedback and thank you for your interest!

**You might also like:** https://github.com/spieglt/flyingcarpet

