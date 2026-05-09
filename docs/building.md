# Building

## Requirements

| Tool | Version | Notes |
|---|---|---|
| Rust | stable | Install via [rustup](https://rustup.rs/) |
| gcc or clang | any recent | For compiling the C projects newc manages |
| make | any | Required for the generated Makefile targets |
| git | 2.x | Optional; required for git panel features |
| clang-format | any | Optional; required for "Format" button in editor |

---

## Linux

### Ubuntu / Debian

```bash
sudo apt update
sudo apt install build-essential git clang-format curl

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Build
git clone https://github.com/TheHomelessTwig/newc-rs.git
cd newc-rs
cargo build --release

# Install
sudo cp target/release/newc /usr/local/bin/newc
```

### Arch Linux

```bash
sudo pacman -S base-devel git clang rustup

rustup toolchain install stable
rustup default stable

git clone https://github.com/TheHomelessTwig/newc-rs.git
cd newc-rs
cargo build --release
sudo cp target/release/newc /usr/local/bin/newc
```

### Fedora / RHEL

```bash
sudo dnf install gcc gcc-c++ make git clang-tools-extra curl

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

git clone https://github.com/TheHomelessTwig/newc-rs.git
cd newc-rs
cargo build --release
sudo cp target/release/newc /usr/local/bin/newc
```

### WSL2 (Windows Subsystem for Linux)

The GUI auto-detects WSL2 by reading `/proc/version`. When Microsoft's kernel string is detected, it sets:

```
LIBGL_ALWAYS_SOFTWARE=1
GALLIUM_DRIVER=llvmpipe
```

This forces Mesa's software renderer and avoids the crash caused by WSL2's incomplete EGL/Zink stack.

**Additional requirement:** Install a Mesa software driver if not already present:

```bash
# Ubuntu/Debian WSL2
sudo apt install libgl1-mesa-dri

# Arch WSL2
sudo pacman -S mesa
```

You may also need an X server or Wayland compositor. With WSLg (Windows 11), this is provided automatically. On Windows 10, install [VcXsrv](https://sourceforge.net/projects/vcxsrv/) and set `DISPLAY=:0`.

Build steps are the same as the distribution-specific instructions above.

---

## macOS

```bash
# Install Xcode command line tools
xcode-select --install

# Install Homebrew if not present
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install llvm          # provides clang-format
brew install rustup-init
rustup-init
source "$HOME/.cargo/env"

git clone https://github.com/TheHomelessTwig/newc-rs.git
cd newc-rs
cargo build --release
cp target/release/newc /usr/local/bin/newc
```

The GUI uses macOS's native window backend via eframe. No additional display server is required.

The "Open in Editor" feature uses AppleScript to open a new Terminal window.

---

## Windows

### Prerequisites

1. Install [Rust](https://www.rust-lang.org/tools/install) (rustup-init.exe)
2. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (MSVC toolchain) or [MinGW-w64](https://www.mingw-w64.org/) (GNU toolchain)
3. Install [Git for Windows](https://git-scm.com/download/win)
4. Optional: Install [LLVM](https://releases.llvm.org/download.html) for `clang-format`

### Build

```powershell
git clone https://github.com/TheHomelessTwig/newc-rs.git
cd newc-rs
cargo build --release
```

The binary is at `target\release\newc.exe`. Add it to your PATH.

### Notes

- `which` is replaced by `where` automatically via `cfg(target_os = "windows")`
- The "Open in Editor" feature uses `cmd /c start <terminal>` 
- `make` is not included with Windows — install [GnuWin32 make](http://gnuwin32.sourceforge.net/packages/make.htm) or use WSL2 instead
- Windows Terminal (`wt`) is the default terminal for the "Open in Editor" feature

---

## Cross-compilation

Cross-compilation is not officially supported. The GUI depends on platform-native window system bindings (X11/Wayland/AppKit/Win32) which complicate cross-compilation significantly.

For Linux→Linux cross-compilation (e.g. for a different architecture), the standard `cargo` cross-compilation workflow applies. Ensure the target toolchain and window system libraries are available.

---

## Feature flags

The `newc` crate exposes no user-facing feature flags. Platform-specific features are selected automatically:

```toml
# Base features (all platforms)
eframe = { features = ["glow", "persistence"] }

# Linux only
eframe = { features = ["wayland", "x11"] }
```

The `glow` backend (OpenGL via `glow` crate) is used instead of `wgpu` because `glow` works correctly under Mesa software rendering in WSL2.

---

## Troubleshooting

### GUI window does not appear

- **WSL2 without WSLg**: Set `DISPLAY=:0` and ensure an X server is running
- **Wayland session**: Try `WAYLAND_DISPLAY=` unset to force X11 fallback
- **Mesa missing**: Install `libgl1-mesa-dri` (Debian/Ubuntu) or `mesa` (Arch)

### `cargo build` fails with linker errors on Linux

Install the X11 and Wayland development libraries:

```bash
# Ubuntu/Debian
sudo apt install libx11-dev libxrandr-dev libxi-dev libxcursor-dev libxinerama-dev \
                 libwayland-dev libxkbcommon-dev

# Arch
sudo pacman -S libx11 libxrandr libxi libxcursor libxinerama wayland libxkbcommon
```

### `cargo build` fails with linker errors on macOS

Ensure Xcode command line tools are installed:
```bash
xcode-select --install
```

### GUI crashes immediately on WSL2

Ensure Mesa software rendering is enabled. The app sets these environment variables automatically when WSL2 is detected, but they can be set manually if needed:
```bash
LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe newc
```
