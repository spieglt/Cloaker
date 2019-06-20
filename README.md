# Cloaker

Ready-to-use downloads on the [Releases](https://github.com/spieglt/Cloaker/releases) page
Version 2.0 improvements: removed Encrypt and Decrypt buttons, it now automatically detects mode.

### Very simple cross-platform file encryption

Have you ever wanted to protect a file with a password and found it unnecessarily difficult to do so? Cloaker aims to provide the most straightforward file encryption possible. Just drop a file onto the window, set a password, and choose where to save it. To decrypt, drop the encrypted file on the window, enter the password, and choose the output location. (Tip: decrypt to a ramdisk to avoid touching the filesystem.)

![Demo](demo.gif)

**Data Loss Disclaimer:** if you lose or forget your password, **your data cannot be recovered!** Use a password manager or another secure form of backup. Cloaker uses stream encryption from the sodium-oxide Rust wrapper of libsodium (xchacha20poly1305).

# Compilation instructions:
`cd gui_adapter && cargo build && cargo build --release`. 

Then open `unix_gui/cloaker/cloaker.pro` in Qt Creator (Qt 5.12), make sure kit is 64bit, and build.

If you want to make a distributable on... 

**Mac:**, use `macqtdeploy` with the built .app bundle as argument. 

**Linux:** compile a static version of Qt with something like:
```
$ mkdir ~/qt-static && cd ~/qt-static
$ ~/Qt/5.12.3/Src/configure -prefix ~/qt-static/5.12.3 -static -release -opensource -confirm-license -skip multimedia -no-compile-examples -nomake examples -no-openssl -no-libpng -skip wayland -qt-xcb
$ make
```

**Windows:** compile Qt statically with something like:
```
> cd c:\; mkdir qt-static; cd qt-static
> C:\Qt\5.12.0\Src\configure.bat -prefix C:\qt-static\5.12.0 -static -release -opensource -confirm-license -skip multimedia -no-compile-examples -nomake examples -no-openssl -no-opengl
```

**Then, on Linux and Windows:** go to Qt Creator settings, add a new version of Qt, and point to `wherever/it/is/qt-static/qtbase/bin/qmake`. Then add a new "Kit" that points to this Qt version, and build Release version with that kit selected. (Use your preexisting dynamically-linked version of Qt for debugging.)

# Planned features:
- Progress indicator/speed staticstics?
- CLI: add password length requirement, and a real flag parser with an output parameter
- Mobile version someday?

# Issues:
- Tell me about them

If you've used Cloaker, please send me feedback and thank you for your interest!

**You might also like:** 

https://github.com/spieglt/flyingcarpet

