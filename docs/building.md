# Building

## Pre-built binaries (recommended)

No Rust installation required. Download from [GitHub Releases](https://github.com/TheHomelessTwig/newc-rs/releases/latest):

| Platform | File |
|---|---|
| Linux x86_64 | `newc-x86_64-linux` |
| Linux aarch64 | `newc-aarch64-linux` |
| macOS Apple Silicon | `newc-aarch64-macos` |
| Windows x86_64 | `newc-x86_64-windows.exe` |

```bash
# Linux x86_64
curl -fsSL https://github.com/TheHomelessTwig/newc-rs/releases/latest/download/newc-x86_64-linux \
    -o /tmp/newc && chmod +x /tmp/newc && sudo mv /tmp/newc /usr/local/bin/newc
```

---

## Build from source — Requirements

| Tool | Version | Notes |
|---|---|---|
| Rust | stable | Install via [rustup](https://rustup.rs/) |
| gcc or clang | any recent | For compiling C projects newc manages |
| make | any | Required for generated Makefile targets |
| cmake | 3.10+ | Required if scaffolding with `--build-system cmake` |
| git | 2.x | Optional; required for git panel features |
| clang-format | any | Optional; required for "Format" button in editor |

---

## Linux

### Ubuntu / Debian

```bash
sudo apt update
sudo apt install build-essential git clang-format curl \
                 libvulkan-dev vulkan-tools mesa-vulkan-drivers

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

git clone https://github.com/TheHomelessTwig/newc-rs.git
cd newc-rs
cargo build --release
sudo cp target/release/newc /usr/local/bin/newc
```

### Arch Linux

```bash
sudo pacman -S base-devel git clang rustup vulkan-icd-loader mesa

rustup toolchain install stable
rustup default stable

git clone https://github.com/TheHomelessTwig/newc-rs.git
cd newc-rs
cargo build --release
sudo cp target/release/newc /usr/local/bin/newc
```

### Fedora / RHEL

```bash
sudo dnf install gcc gcc-c++ make git clang-tools-extra curl vulkan-loader mesa-vulkan-drivers

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

git clone https://github.com/TheHomelessTwig/newc-rs.git
cd newc-rs
cargo build --release
sudo cp target/release/newc /usr/local/bin/newc
```

---

## WSL2 (Windows Subsystem for Linux)

The GUI auto-detects WSL2 by reading `/proc/version`. When detected, it automatically selects the best available Vulkan ICD:

- **With GPU passthrough** (`/dev/dri` present): prefers `virtio_icd.json` or `gfxstream_vk_icd.json`
- **Without GPU** (most WSL2 setups): uses LLVMpipe software Vulkan (`lvp_icd.json`)

`WAYLAND_DISPLAY` is always unset to avoid WSLg socket instability; the app runs under XWayland/X11.

**No manual configuration is needed** — the ICD selection is fully automatic.

### Required packages

```bash
# Ubuntu/Debian WSL2
sudo apt install mesa-vulkan-drivers libvulkan1 vulkan-tools

# Arch WSL2
sudo pacman -S mesa vulkan-icd-loader vulkan-mesa-layers
```

Build steps are the same as the Linux distribution above.

### Troubleshooting WSL2

If the GUI fails to start, test Vulkan manually:

```bash
vulkaninfo --summary
```

If Vulkan is unavailable, force software GL as a last resort:

```bash
WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1 newc gui
```

---

## macOS

```bash
xcode-select --install

/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install llvm rustup-init
rustup-init
source "$HOME/.cargo/env"

git clone https://github.com/TheHomelessTwig/newc-rs.git
cd newc-rs
cargo build --release
cp target/release/newc /usr/local/bin/newc
```

macOS uses the native Metal backend via iced/wgpu. No additional display server is required.

---

## Windows

### Prerequisites

1. Install [Rust](https://www.rust-lang.org/tools/install) (rustup-init.exe)
2. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (MSVC toolchain)
3. Install [Git for Windows](https://git-scm.com/download/win)
4. Optional: Install [LLVM](https://releases.llvm.org/download.html) for `clang-format`

### Build

```powershell
git clone https://github.com/TheHomelessTwig/newc-rs.git
cd newc-rs
cargo build --release
```

Binary at `target\release\newc.exe`. Add to PATH.

When `newc` opens the GUI on Windows it spawns itself with `CREATE_NO_WINDOW` — no console appears alongside the GUI.

`make` is not included with Windows — install [GnuWin32 make](http://gnuwin32.sourceforge.net/packages/make.htm) or use WSL2.

---

## Troubleshooting

### GUI window does not appear (WSL2 without WSLg)

Set `DISPLAY=:0` and ensure an X server is running (e.g. [VcXsrv](https://sourceforge.net/projects/vcxsrv/) on Windows 10).

### `cargo build` fails with linker errors (Linux)

Install display system development libraries:

```bash
# Ubuntu/Debian
sudo apt install libx11-dev libxrandr-dev libxi-dev libxcursor-dev libxinerama-dev \
                 libwayland-dev libxkbcommon-dev

# Arch
sudo pacman -S libx11 libxrandr libxi libxcursor libxinerama wayland libxkbcommon
```

### `cargo build` fails with linker errors (macOS)

```bash
xcode-select --install
```

### GUI crashes immediately on WSL2

Check which Vulkan ICD is being selected:

```bash
VK_LOADER_DEBUG=all newc gui 2>&1 | grep "ICD"
```

Force LLVMpipe manually if auto-detection fails:

```bash
VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json newc gui
```
