# 果核Hosts (Guohe Hosts)

[English](README-en.md) | 中文

一款基于 Tauri 2 + Vue 3 + Rust 构建的现代化 Windows hosts 文件管理工具。

轻松管理多个 hosts 分组，一键切换，告别手动编辑系统 hosts 文件。

## 功能特性

- **分组管理** — 将 hosts 条目按分组管理，每个分组独立存储为 `.hosts` 文件
- **单选激活** — 单选模式：一次只激活一个分组，即时写入系统 hosts 文件
- **启动同步** — 启动时对比系统 hosts 与本地副本，外部修改会被保留
- **双视图编辑** — 表格视图与 CodeMirror 文本编辑器自由切换
- **拖拽排序** — 通过拖拽手柄调整分组和条目的顺序
- **搜索高亮** — 关键词过滤并高亮匹配内容
- **备份恢复** — 一键创建带时间戳的 hosts 备份，随时恢复
- **导入导出** — 粘贴文本导入，一键复制导出
- **DNS 刷新** — 一键执行 `ipconfig /flushdns`，操作结果即时提示
- **系统托盘** — 关闭窗口最小化到托盘，右键菜单快捷操作
- **暗色模式** — 浅色 / 深色 / 跟随系统，基于 Arco Design
- **双语支持** — 中文 (zh-CN) 和英文 (en-US)
- **便携运行** — 配置和数据存储在程序目录下（`config.json`、`data/`），无需安装

## 截图

| 浅色模式 | 深色模式 |
|---------|---------|
| ![浅色模式](docs/software-light.png) | ![深色模式](docs/software-dark.png) |

## 技术栈

| 层级 | 技术 |
|------|-----|
| 框架 | [Tauri 2.x](https://v2.tauri.app/) |
| 后端 | Rust |
| 前端 | Vue 3 + TypeScript + Vite |
| UI 库 | [Arco Design Vue](https://arco.design/vue) |
| 状态管理 | Pinia 3 |
| 编辑器 | CodeMirror 6 |
| 拖拽排序 | vue-draggable-plus |
| 包管理 | bun |

## 环境要求

- [Rust](https://rustup.rs/) >= 1.77
- [Node.js](https://nodejs.org/) >= 18
- [bun](https://bun.sh/) >= 1.0
- Windows 10/11（写入 hosts 文件需要管理员权限）

## 构建

```bash
# 安装前端依赖
bun install

# 开发模式（启动 Tauri 开发窗口）
bun run tauri dev

# 生产构建（输出到 src-tauri/target/release/bundle/）
bun run tauri build
```

> **注意：** `bun run tauri dev` 需要在管理员终端中运行，因为程序需要写入 `C:\Windows\System32\drivers\etc\hosts`。

## 项目结构

```
├── src/                        # Vue 3 前端
│   ├── components/             # UI 组件
│   │   └── layout/             # AppHeader, AppToolbar, AppSidebar, AppStatusBar
│   ├── stores/                 # Pinia 状态管理 (hosts, config)
│   ├── composables/            # useHosts 组合式函数
│   ├── types/                  # TypeScript 类型定义
│   └── i18n/                   # 中英文翻译文件
├── src-tauri/                  # Rust 后端
│   └── src/
│       ├── commands/hosts.rs   # Tauri IPC 命令
│       ├── models/host.rs      # 数据结构
│       ├── parser/             # Hosts 文件解析与序列化
│       └── services/           # 文件读写、配置、分组存储
├── package.json
├── vite.config.ts
└── src-tauri/tauri.conf.json
```

## 数据存储

```
{程序目录}/
├── data/
│   ├── manifest.json    # 分组元数据（顺序、启用状态、时间戳）
│   ├── System.hosts     # 每个分组一个标准 hosts 文件
│   ├── Work.hosts
│   └── 游戏加速.hosts
├── backups/
│   └── hosts_*.bak      # hosts 文件备份
├── config.json          # 应用配置
└── guohe-hosts.exe
```

## 工作原理

1. **分组存储** — 每个分组是 `{程序目录}/data/` 下的独立 `.hosts` 文件，元数据保存在 `manifest.json` 中。
2. **单选模式** — 同一时间只有一个分组处于激活状态。激活分组时，其条目会写入系统 hosts 文件并自动刷新 DNS。
3. **纯净输出** — 生成的系统 hosts 文件格式干净，只包含标准的 `IP 域名` 行和头部注释，无额外标记。
4. **启动检查** — 启动时对比系统 hosts 文件与激活分组的本地副本。如果不一致（如用户手动编辑了 hosts），以系统版本为准。

## 开源协议

[MIT](LICENSE)
