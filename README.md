# Cloaker

**New Cloaker 4.0 downloads on the [Releases](https://github.com/spieglt/Cloaker/releases) page!**

**Mobile version available at https://cloaker.mobi! Static HTML/CSS/JS/WASM and interoperable with this version of Cloaker.** [Code here.](https://github.com/spieglt/Cloaker.js)

## Very simple cross-platform file encryption

Have you ever wanted to protect a file with a password and found it unnecessarily difficult to do so? Cloaker aims to provide the most straightforward file encryption possible. Just drop a file onto the window, set a password, and choose where to save it. To decrypt, drop the encrypted file on the window, enter the password, and choose the output location. No installation required: on Windows it's a single `.exe`, on Mac an `.app` bundle, and on Linux an executable `.AppImage` file.

![Demo](demo.gif)

**Data Loss Disclaimer:** if you lose or forget your password, **your data cannot be recovered!** Use a password manager or another secure form of backup. Cloaker uses the `pwhash` and `secretstream` APIs of [libsodium](https://doc.libsodium.org/) via [sodiumoxide](https://github.com/sodiumoxide/sodiumoxide).

# Compilation instructions:
`cd cloaker/adapter; cargo build --release`.

Then open `gui/cloaker/cloaker.pro` in Qt Creator (Qt 5.15.2), make sure kit is Release and 64bit, and build.

If you want to make a distributable on...

**Linux:** download [linuxdeployqt](https://github.com/probonopd/linuxdeployqt/releases). Navigate to the folder with the executable built by Qt (i.e. `cloaker/gui/\[release build folder\]`) and run something like `linuxdeployqt ./cloaker -appimage`. Read the instructions at [linuxdeployqt](https://github.com/probonopd/linuxdeployqt), you may have to make a `cloaker.desktop` file or add a line to it that says `Categories=Utilities;`. It should output an `.AppImage` file that includes all the libraries Cloaker needs, and will run on a wide variety of Linux distributions.

**Mac:** use the `macdeployqt` script in your Qt installation's `bin/` directory with the built `.app` bundle as argument.

**Windows:** make sure Sources are installed for Qt 5.15.2 through the Qt Maintenance Tool. Then install Visual Studio 2019 Community (including the `Desktop development with C++` feature), launch the `x64 Native Tools Command Prompt` (found in `Start Menu > Visual Studio 2019`) and compile Qt statically with something like:
```
> cd C:\; mkdir qt-static; cd qt-static
> C:\Qt\5.15.2\Src\configure.bat -release -static -no-pch -optimize-size -opengl desktop -platform win32-msvc -skip webengine -nomake tools -nomake tests -nomake examples
> nmake.exe
```
Run `rustup default stable-x86_64-pc-windows-msvc` to make sure you're using MSVC, and rerun `cargo build --release` from `adapter/` if you weren't.

Finally, go to `Qt Creator > Project > Manage Kits > Qt Versions`, add a new version of Qt, and point to `C:\qt-static\qtbase\bin\qmake.exe`. Add a new Kit in the `Kits` tab, and set its `Qt version` to be the static one you just added. On the Projects page, click the plus button by the new Kit under `Build & Run`. Now you can build with the static kit's Release profile in the bottom-left above the play and build buttons.

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
