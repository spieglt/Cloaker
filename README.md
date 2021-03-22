# Cloaker

**Version 4:** Changed password hashing from `scryptsalsa208sha256` to `Argon2id`. (Version 4 can still decrypt files encrypted with earlier versions, but version 3 and below cannot decrypt files encrypted with version 4+.) Increased speed by ??. 
Ready-to-use downloads on the [Releases](https://github.com/spieglt/Cloaker/releases) page

### Very simple cross-platform file encryption

Have you ever wanted to protect a file with a password and found it unnecessarily difficult to do so? Cloaker aims to provide the most straightforward file encryption possible. Just drop a file onto the window, set a password, and choose where to save it. To decrypt, drop the encrypted file on the window, enter the password, and choose the output location. (Tip: decrypt to a ramdisk for temporary use to avoid writing data to permanent storage.) No installation required! On Windows it's a single `.exe`, on Mac a standard `.app` bundle, and on Linux a single executable `.run` file.

![Demo](demo.gif)

**Data Loss Disclaimer:** if you lose or forget your password, **your data cannot be recovered!** Use a password manager or another secure form of backup. Cloaker uses stream encryption from the sodium-oxide Rust wrapper of libsodium (xchacha20poly1305).

# Compilation instructions:
`cd adapter && cargo build && cargo build --release`. 

Then open `gui/cloaker/cloaker.pro` in Qt Creator (Qt 5.12), make sure kit is 64bit, and build.

If you want to make a distributable on... 

**Mac:** use the `macdeployqt` script in your Qt installation's `bin/` directory with the built .app bundle as argument.

**Linux:** Just use the dynamically linked binary that Qt Creator builds by default, it's more portable on Linux.

**Windows only:** compile Qt statically with something like:
```
> cd c:\; mkdir qt-static; cd qt-static
> C:\Qt\5.15.2\Src\configure.bat -prefix C:\qt-static\5.15.2 -static -release -opensource -confirm-license -skip multimedia -no-compile-examples -nomake examples -no-openssl -opengl desktop -platform win32-g++
> mingw32-make.exe
```

Then run `rustup toolchain install stable-x86_64-pc-windows-gnu` and `rustup set default-host x86_64-pc-windows-gnu`. (Linking Qt statically requires compiling with MinGW, which requires linking against Rust libs compiled with MinGW.) Then `cd` to `cloaker\adapter`, delete the `target` directory and run `cargo build --release` again.

Then go to Qt Creator settings, add a new version of Qt, and point to `wherever\it\is\qt-static\qtbase\bin\qmake`. Then add a new "Kit" that points to this Qt version, and build Release version with that kit selected.

# CLI compilation instructions
`cd cli; cargo build --release`. Executable will be at `cloaker/cli/target/release/cloaker_cli`(`.exe`).

# Planned features:
- Progress indicator/speed staticstics?
- Change minimum password length to 14 or 16?
- Mobile version someday?

# Issues:
- Tell me about them
- Backward compatibility note: to decrypt a file made with version 1.0 or 1.1 of Cloaker (with Encrypt and Decrypt buttons), the filename must end with the ".cloaker" extension. Files encrypted with later versions are not subject to this restriction.

If you've used Cloaker, please send me feedback and thank you for your interest!

**You might also like:** https://github.com/spieglt/flyingcarpet

