# <img src="https://raw.githubusercontent.com/LiveSplit/LiveSplit/master/LiveSplit/Resources/Icon.png" alt="LiveSplit" height="42" width="45" align="top"/> LiveSplit One

This repository hosts a modified Desktop version of LiveSplit One that includes changing splits files, changing layout files, and a stateful timer allowing closing and reopening (crashes) of Livesplit One through a modified livesplit_core.

LiveSplit One is a
version of LiveSplit that uses the multiplatform
[livesplit-core](https://github.com/LiveSplit/livesplit-core) library to create
a new LiveSplit experience that works on a lot of different platforms.

The Web Version is available [here](https://one.livesplit.org/).

## Build Instructions

In order to build LiveSplit One you need the [Rust
Compiler](https://www.rust-lang.org/). You can then build and run the project
with:

```bash
cargo run
```

In order to build and run a release build, use the following command:

```bash
cargo run --release
```
