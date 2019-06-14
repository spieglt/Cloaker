# Cloaker
### Very simple cross-platform file encryption

Have you ever wanted to protect a file with a password and found it unnecessarily difficult to do so? Cloaker aims to be the most straightforward file encrypter possible. Just drop a file onto the window and set a password. To decrypt, select "Decrypt", drop the encrypted file on the window, and enter the password.

![Demo](demo.gif)

**Data Loss Disclaimer:** if you lose or forget your password, **your data cannot be recovered!** Use a password manager or another secure form of backup. Cloaker uses stream encryption from the sodium-oxide Rust wrapper of libsodium (xchacha20poly1305).

# Compilation instructions:
`cd gui_adapter && cargo build`. Then...

**Windows**: Open `windows_gui\cloaker\cloaker.sln` in Visual Studio, make sure architecture is set to x64, and build.

**Mac/Linux**: Open `unix_gui/cloaker/cloaker.pro` in Qt Creator (Qt 5.12), make sure kit is 5.12.0 64bit, and build.

# Planned features:
- Progress indicator/speed staticstics?
- Use a real flag parser for the CLI
- Mobile version someday?

# Issues:
