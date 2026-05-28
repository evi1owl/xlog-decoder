# xlog-decoder 编译指南

## 你需要装什么

三个东西，所有平台都需要：

- **Node.js** ≥ 18（编译前端用）
- **Rust** ≥ 1.77（编译后端用）
- **系统库**（Tauri 框架依赖，各平台不一样）

---

## 1. 装 Node.js

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
nvm install 18
nvm use 18
```

---

## 2. 装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

重启终端后确认版本：

```bash
rustc --version   # 应该 ≥ 1.77
```

---

## 3. 装系统依赖（按平台选一节）

### Linux

```bash
sudo apt install -y \
  build-essential \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libssl-dev \
  libjavascriptcoregtk-4.1-dev \
  libsoup-3.0-dev \
  libxdo-dev
```

### macOS

```bash
xcode-select --install
```

### Windows

装两个东西：

1. [Microsoft Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist)（运行时）
2. [Visual Studio 2022 Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)，安装时勾选 **"Desktop development with C++"** 工作负载

或者用 winget 一步搞定：

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools ^
  --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

WebView2 通常系统已自带，如果没有的话 [点这里下载](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)。

---

## 4. 编译

```bash
# 克隆
git clone git@github.com:evi1owl/xlog-decoder.git
cd xlog-decoder

# 装前端依赖
npm install

# 编译
npm run tauri build
```

最后一步会自动做三件事：

1. Vite 打包前端 → `dist/`
2. Cargo 编译 Rust 后端 → `src-tauri/target/release/`
3. 打包成安装程序

---

## 5. 产物在哪

都在 `src-tauri/target/release/bundle/` 下：

| 平台 | 产物 |
|------|------|
| Linux | `appimage/xlog-decoder_*.AppImage`、`deb/`、`rpm/` |
| macOS | `dmg/xlog-decoder_*.dmg`、`macos/xlog-decoder.app` |
| Windows | `msi/xlog-decoder_*.msi`、`nsis/xlog-decoder_*.exe` |

---

## 开发模式

不想每次都完整编译，只改前端的话：

```bash
npm run tauri dev
```

Vite 热更新 + Tauri 窗口，改代码自动刷新。

---

## 关于解密二进制

`src-tauri/binary/` 目录下是微信 Mars-Xlog 的解密工具 `decode_log_file`，已按平台和架构放好：

```
binary/
├── linux/     x86_64  aarch64
├── macos/     x86_64  aarch64
└── windows/   x86_64  aarch64
```

编译时 Tauri 会自动打包对应平台的版本到安装包里，不需要你手动处理。

---

## 常见报错

| 报错 | 解决 |
|------|------|
| `libwebkit2gtk-4.1 not found` | 回到第 3 步装 Linux 系统依赖 |
| `linker 'cc' not found` | `apt install build-essential` |
| macOS 打开提示已损坏 | `xattr -cr xlog-decoder.app`（未签名开发版） |
| Windows 缺 `vcruntime140.dll` | 装 Visual C++ Redistributable |

---

## CI 三平台编译（GitHub Actions）

```yaml
name: Build
on:
  push:
    tags: ["v*"]

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-22.04
          - os: macos-latest
          - os: windows-latest
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 18
      - uses: dtolnay/rust-toolchain@stable

      - name: Linux 依赖
        if: runner.os == 'Linux'
        run: |
          sudo apt update
          sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
            libayatana-appindicator3-dev librsvg2-dev libssl-dev \
            libjavascriptcoregtk-4.1-dev libsoup-3.0-dev libxdo-dev

      - run: npm install
      - run: npm run tauri build

      - uses: actions/upload-artifact@v4
        with:
          name: xlog-decoder-${{ runner.os }}
          path: src-tauri/target/release/bundle/
```
