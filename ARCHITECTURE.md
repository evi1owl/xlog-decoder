# xlog-decoder 架构文档

## 项目概览

xlog-decoder 是一个桌面应用，用于解密 Mars-Xlog 格式的 `.xlog` 日志文件。用户拖入文件 → 自动解密 → 点击可打开输出目录。

**技术栈：** Tauri 2（Rust 后端）+ Vue 3（TypeScript 前端）+ Vite（构建）+ Less（样式）

**窗口：** 320×360，不可调整大小

---

## 架构分层

```
┌──────────────────────────────────────────┐
│              前端 (Vue 3)                 │
│  main.ts → App.vue                       │
│              ├── ListView.vue             │
│              │     └── Item.vue × N       │
│              ├── BottomView.vue           │
│              └── Preference.vue           │
├──────────────────────────────────────────┤
│          Tauri 桥接层 (@tauri-apps/api)    │
│  invoke() 调用 Rust 命令                  │
│  onDragDropEvent() 监听系统拖放            │
│  localStorage 存储用户设置                 │
├──────────────────────────────────────────┤
│              Rust 后端 (src-tauri)         │
│  main.rs                                  │
│    ├── decode()       调用解密二进制       │
│    └── show_in_folder()  打开文件管理器    │
├──────────────────────────────────────────┤
│         外部二进制 (binary/)               │
│  decode_log_file / decode_log_file.exe    │
│  由微信 Mars-Xlog 提供的解密工具           │
└──────────────────────────────────────────┘
```

---

## 文件逐个说明

### 根目录配置

#### `index.html`
应用入口 HTML。只做一件事：放一个 `<div id="app">` 占位，然后加载 `/src/main.ts`。

#### `package.json`
Node 依赖声明。关键依赖：
- `vue` — 前端框架
- `@tauri-apps/api` — Tauri 前端 API（调用 Rust 命令、监听拖放事件）
- `@tauri-apps/plugin-dialog` — 系统原生文件选择对话框
- `@tauri-apps/plugin-fs` — 文件系统读写
- `less` — CSS 预处理器
- `vite` — 构建工具

#### `vite.config.ts`
Vite 配置。要点：
- 固定端口 `1420`（Tauri 开发模式要求）
- `@` 路径别名 → `./src`
- 忽略 `src-tauri/` 目录的文件监听（Rust 代码变动不需要 Vite 热更新）

#### `tsconfig.json` / `tsconfig.node.json`
TypeScript 编译配置，没什么特别的。

---

### 前端 (src/)

#### `src/main.ts` — 应用启动
```ts
createApp(App).mount('#app')
```
Vue 3 的标准启动方式。创建 App 根组件，挂载到 `index.html` 的 `#app` div 上。同时引入全局样式 `styles.css`。

#### `src/styles.css` — 全局样式
- 让 `html, body, #app` 全屏固定，无滚动条
- 定义全局字体、颜色、暗色模式适配
- 禁止文字选中（`user-select: none`），适合桌面应用的体验

#### `src/App.vue` — 根组件（核心入口）

**职责：**
1. 维护 `paths` 数组，存放所有拖入的文件路径
2. 注册 Tauri 拖放事件监听
3. 控制设置面板的显示/隐藏
4. 组装子组件：ListView + BottomView + Preference

**拖放处理逻辑：**
```
onDragDropEvent → 监听系统拖放事件
  └── type === "drop" → 遍历 p.paths（所有拖入的文件）
       └── 去重检查 → 加入 paths 数组
```

**模板结构：**
```html
<div class="container">           ← 蓝色渐变背景，全屏
  <list-view :paths="..." />      ← 上：文件列表（占满剩余空间）
  <bottom-view />                 ← 下：底部栏 48px
  <preference :show="..." />      ← 浮层：设置面板
</div>
```

**关键细节：** `paths.slice().reverse()` 传入 ListView，最新拖入的文件显示在最上面。

---

#### `src/components/ListView.vue` — 文件列表

**职责：** 根据 `paths` 是否为空，显示两种状态：

| 状态 | 显示内容 |
|------|---------|
| 无文件 (`paths.length === 0`) | 空状态图标 + "Drag and drop .xlog files here!" |
| 有文件 | 遍历 paths，每个路径渲染一个 `<Item>` 组件 |

```html
<!-- 空状态 -->
<div class="empty">
  <img src="ic_empty.png" />
  <p>Drag and drop .xlog files here!</p>
</div>

<!-- 有文件 -->
<div class="list">
  <item v-for="item in paths" :key="item" :path="item" />
</div>
```

**关键细节：** `v-for` 的 `:key="item"` 用文件路径作为 key，因为路径唯一，Vue 可以精确追踪每个文件对应的 DOM 节点。

---

#### `src/components/Item.vue` — 单个文件条目

**职责：** 每个拖入的文件对应一个 Item，独立完成解密流程。

**解密流程：**
```
watch(path)  ← 监听到路径变化
  ├── basename(path)  → 提取文件名，如 "20240501.xlog"
  └── decode()
        ├── 检查后缀是否为 .xlog → 否则标记失败
        ├── 读取 localStorage 中用户设置的自定义输出目录
        ├── 如果自定义目录不存在 → mkdir 创建
        ├── 拼接输出路径（自定义目录 或 原文件同目录 + .log 后缀）
        └── invoke("decode", { name, privateKey, dist })
              └── 调用 Rust 后端解密
```

**三种状态显示：**

| status | 含义 | 图标 |
|--------|------|------|
| `-1` | 解密中（初始值） | 无图标 |
| `0` | 解密成功 | 绿色 ✓ |
| 其他 | 解密失败 | 红色 ✗ |

**点击行为：** 点击 Item → `showInFolder()` → 调用 Rust 后端在文件管理器中打开输出文件所在目录。

**关键细节：** `watch` 加了 `{ immediate: true }`，组件一创建立刻执行解密。`user-select: none` 防止误选中文件名文字。

---

#### `src/components/BottomView.vue` — 底部栏

**职责：** 固定在窗口底部 48px 的操作栏。

| 元素 | 功能 | 调用 |
|------|------|------|
| `© baosong` | 版权文字 | 无 |
| 文件夹图标 | 打开输出目录 | `invoke("show_in_folder", { opening: true })` |
| 齿轮图标 | 打开设置面板 | `emit("openPreference")` → App.vue 显示 Preference |

**关键细节：** 文件夹图标调用 `show_in_folder` 时 `opening: true`，表示**打开目录本身**（而非定位到某个文件）。

---

#### `src/components/Preference.vue` — 设置面板

**职责：** 浮层式设置面板，两个配置项。

| 配置项 | 存储 Key | 说明 |
|--------|----------|------|
| Save To（输出目录） | `dist` | 点击输入框弹出系统文件夹选择器 |
| Private Key（解密私钥） | `key` | 手动输入，点击 ✓ 保存 |

**交互细节：**
- 点击输出目录输入框 → `@tauri-apps/plugin-dialog` 的 `open({ directory: true })` → 系统原生文件夹选择器
- 点击 ✓ → `localStorage.setItem()` 保存 → `emit("done")` 通知父组件关闭面板
- `v-show="show"` 控制显隐（不是 `v-if`，DOM 始终存在，避免重复创建）

---

### Rust 后端 (src-tauri/)

#### `src-tauri/Cargo.toml`
Rust 依赖：`tauri`、`tauri-plugin-dialog`、`tauri-plugin-fs`。

#### `src-tauri/tauri.conf.json` — Tauri 配置

关键配置项：
| 配置 | 值 | 说明 |
|------|-----|------|
| `dragDropEnabled` | `true` | **启用原生拖放，这是整个拖入功能的前提** |
| `width × height` | 320 × 360 | 固定窗口大小 |
| `resizable` | `false` | 不可拉伸 |
| `resources` | `["binary"]` | 打包时包含 `binary/` 目录（解密二进制文件） |
| `devUrl` | `http://localhost:1420` | 开发时加载 Vite dev server |
| `frontendDist` | `../dist` | 生产时加载 Vite 构建产物 |

#### `src-tauri/capabilities/default.json` — 权限声明

Tauri 2 的安全模型需要显式声明权限：
- `core:default` — 基础窗口权限
- `dialog:default` — 文件选择对话框权限
- `fs:read-all` / `fs:write-all` — 文件系统读写
- `fs:scope` → `**` — 允许访问任意路径

#### `src-tauri/src/main.rs` — Rust 后端入口

**两个 Tauri 命令：**

##### `decode(name, privateKey, dist) → i32`
根据当前操作系统和架构，找到打包的 `decode_log_file` 二进制，传入三个参数执行：
```
decode_log_file <privateKey> <xlog文件路径> [输出路径]
```
返回退出码（0 = 成功）。

**二进制路径规则：** `binary/{os}/{arch}/decode_log_file[.exe]`
例如 Linux x86_64 → `binary/linux/x86_64/decode_log_file`

##### `show_in_folder(path, opening)`
跨平台打开文件管理器：

| 平台 | opening=true | opening=false |
|------|-------------|---------------|
| Windows | `explorer <path>` 打开目录 | `explorer /select, <path>` 定位文件 |
| Linux | `xdg-open <path>` | `xdg-open <父目录>` |
| macOS | `open <path>` | `open -R <path>` 在 Finder 中定位 |

**`main()` 函数：** 注册插件（dialog、fs）和命令（decode、show_in_folder），然后启动应用。

---

## 页面渲染流程

```
1. 应用启动
   index.html 加载 main.ts
   └── main.ts 创建 Vue App，挂载到 #app
       └── App.vue 渲染

2. 首次渲染
   ┌─────────────────────────────┐
   │  ListView (占满剩余空间)      │
   │  ┌───────────────────────┐  │
   │  │  空状态图标             │  │
   │  │  "拖入 .xlog 文件"     │  │  ← paths 为空，显示 Empty View
   │  └───────────────────────┘  │
   ├─────────────────────────────┤
   │  BottomView (48px)          │
   │  © baosong    [📁] [⚙]     │
   └─────────────────────────────┘

3. 用户拖入文件
   Tauri 窗口捕获系统拖放事件
   └── onDragDropEvent 回调
       └── paths.value.push(文件路径)
           └── Vue 响应式更新 → 重新渲染

4. 拖入文件后的渲染
   ┌─────────────────────────────┐
   │  ListView (可滚动)           │
   │  ┌───────────────────────┐  │
   │  │  Item: file2.xlog  ✓  │  │  ← 最新拖入的在最上面
   │  ├───────────────────────┤  │
   │  │  Item: file1.xlog  ✓  │  │
   │  └───────────────────────┘  │
   ├─────────────────────────────┤
   │  BottomView                 │
   └─────────────────────────────┘

   每个 Item 一挂载就立即调用 decode() 开始解密

5. 用户点击设置
   ┌─────────────────────────────┐
   │  ListView                   │
   │  ...                        │
   ├─────────────────────────────┤
   │  Preference (浮层覆盖底部)    │
   │  Save To:    [/path/to/out]│
   │  Private Key:[******    ] ✓│
   ├─────────────────────────────┤
   │  BottomView                 │
   └─────────────────────────────┘
```

---

## 数据流总结

```
                  用户拖入文件
                      │
                      ▼
              App.vue (paths[])
                 │
                 │ 传入 paths
                 ▼
            ListView.vue
                 │
                 │ v-for 每个 path
                 ▼
            Item.vue × N
                 │
                 │ invoke("decode", ...)
                 ▼
           Rust main.rs
                 │
                 │ 调用外部二进制
                 ▼
          decode_log_file
                 │
                 │ 返回退出码
                 ▼
            Item 更新 status
                 │
          ┌──────┴──────┐
      成功(✓)        失败(✗)
      点击→打开目录    无操作
```

**状态存储：**
- `paths` — Vue 响应式变量（内存），应用关闭即丢失
- `dist` / `key` — `localStorage`（磁盘），持久化保存
