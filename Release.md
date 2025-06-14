## Defender-rs Build.v0.1.1

An even funnier way to disable Windows Defender.

A fully Rust rewrite of defendnot, 100% compatible with the original [C++ version](https://github.com/es3n1n/defendnot). You can use the Rust loader to inject the C++ DLL, or the C++ loader to inject the Rust DLL.

- Register/unregister custom AV/AS to Windows Security Center (WSC)
- Automatic scheduled task for persistence (boot/login)
- Minimal (Just 300kb), dependency-free

**Note:**

Defender will flag/block the binaries. Please temporarily disable Defender real-time/tamper protection or add an exclusion before use.

The first public release!
