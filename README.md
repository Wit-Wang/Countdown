# Countdown

轻量级桌面倒计时与待办应用，基于 Vue 3 + TypeScript 前端，使用 Tauri 打包为原生应用。

## 功能简介

- 新建/编辑待办，支持精确到分钟的 `deadline`。
- 支持“重复”选项（以 DateTime-interval 表示），完成有重复的待办会自动把 `deadline` 向后推进；无重复的完成即为删除。
- 本地持久化（保存在平台数据目录），支持 Windows 安装包(NSIS/MSI)等平台发行包。

## 快速开始

### 先决条件

- Node.js（建议 >=16/18）
- pnpm（或 npm/yarn）
- Rust + cargo（用于 Tauri 打包）
- 平台打包工具（仅在生成安装包时需要）：
	- Windows: MSVC 工具链（Visual Studio Build Tools）、WiX Toolset（用于 MSI）、NSIS
	- macOS: Xcode（用于签名/打包）

### 开发（本地运行）

1. 安装依赖：

```bash
pnpm install
```

2. 启动开发模式（前端热重载 + Tauri 开发窗口）：

```bash
pnpm tauri dev
```

> 说明：`pnpm tauri dev` 会根据 `src-tauri/tauri.conf.json` 的 `beforeDevCommand` 自动启动前端 dev server（本项目设为 `pnpm dev`）。

## 构建与打包（发布）

1. 先构建前端产物：

```bash
pnpm build
```

2. 打包为原生安装程序（在当前平台生成安装文件）：

```bash
pnpm tauri build
```

打包产物位置：

- Windows/macOS/Linux 对应的发行包会输出到：`src-tauri/target/release/bundle`。

## 开发约定与实现要点

- 时间类型：本项目在前端/后端统一使用 `DateTime` 结构表示日期时间（字段化表示 year/month/day/hour/minute/second），并在工具函数里提供 `dateTimeToTs`、`tsToDateTime`、`addIntervalToDateTime` 等转换与加法操作。
- `repeat` 字段表示重复间隔（DateTime-interval），完成操作：若 `repeat` 不为空，则 `deadline = addIntervalToDateTime(deadline, repeat)` 并 `update_todo`；若为空则删除（`remove_todo`）。

## 代码质量与工具

- 类型检查：`pnpm build` 会先执行 `vue-tsc --noEmit` 做 TypeScript 类型检查。
- 格式化：使用 Prettier；命令 `pnpm format`。
- 静态检查：ESLint 已配置（Vue3 + TypeScript），命令 `pnpm run lint`、`pnpm run lint:fix`。

## 常见问题与排查

- 如果 `pnpm run lint` 报错，尝试先安装依赖并自动修复：

```bash
pnpm install
pnpm run lint:fix
pnpm format
```

- 如果 `pnpm tauri build` 在 Windows 上失败，检查是否安装了 WiX Toolset 与 NSIS，以及是否安装了 Visual Studio 的 MSVC 工具链。

## 贡献

1. Fork 本仓库并创建 feature 分支。
2. 安装依赖并确保格式化/静态检查通过：

```bash
pnpm install
pnpm run lint:fix
pnpm format
pnpm build
```

3. 提交代码并发起 Pull Request，CI 会运行构建与 lint 检查。

## 许可证

本项目采用 Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International
(CC BY-NC-SA 4.0) 许可，禁止商业用途。详细条款见仓库根目录的 `LICENSE` 文件或
https://creativecommons.org/licenses/by-nc-sa/4.0/。

---

更多信息或遇到平台相关构建问题，请在 issue 中描述你的平台（OS、Node、Rust 版本）与错误日志。

# future updates:
- [ ] 添加单元测试（Jest + Vue Test Utils）
- [ ] 添加 CI/CD 工作流（GitHub Actions）
- [ ] 增加推迟功能（snooze）
- [ ] 使应用始终位于桌面层
- [ ] 添加设置界面（如默认重复间隔、主题、各个紧急度区分等）
- [ ] 减少内存占用（如使用虚拟列表等）