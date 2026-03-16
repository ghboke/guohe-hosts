# GitHub 开源发布说明

本文档用于把当前项目发布到 GitHub 开源仓库时，安全、清晰地处理“在线更新”相关代码。

## 目标

本项目已经把在线更新功能集中到了独立目录，后续可以：

- 只禁用更新功能，但保留代码
- 或者彻底删除更新功能代码，再发布到 GitHub

## 一、代码位置

在线更新相关代码目前集中在以下位置：

### 前端

- `src/features/update/`
  - `config.ts`：前端更新功能开关
  - `store.ts`：检查更新、右下角提示、下载弹窗
  - `types.ts`：更新响应类型

### 后端

- `src-tauri/src/features/update/`
  - `command.rs`：Tauri 更新命令
  - `model.rs`：更新结果模型
  - `service.rs`：请求更新接口、生成环境信息、处理 install_id

### 接入点

- `src/App.vue`
  - 应用启动后自动检查更新
- `src/components/SettingsModal.vue`
  - “关于”页中的检查更新 UI
- `src-tauri/src/lib.rs`
  - 注册 `check_update`

## 二、推荐方式：禁用更新但保留代码

如果你只是要把代码发到 GitHub，但本地或私有版本仍然要保留更新能力，推荐使用环境变量禁用。

### 前端开关

- 变量名：`VITE_ENABLE_UPDATE`
- 取值：
  - `true`：启用在线更新
  - `false`：禁用在线更新

### Rust / Tauri 开关

- 变量名：`APP_ENABLE_UPDATE`
- 取值：
  - `true`：启用在线更新
  - `false`：禁用在线更新

### PowerShell 示例

#### 开源版开发运行

```powershell
$env:VITE_ENABLE_UPDATE = "false"
$env:APP_ENABLE_UPDATE = "false"
bun run tauri dev
```

#### 开源版打包

```powershell
$env:VITE_ENABLE_UPDATE = "false"
$env:APP_ENABLE_UPDATE = "false"
bun run tauri build
```

#### 私有版开发运行

```powershell
$env:VITE_ENABLE_UPDATE = "true"
$env:APP_ENABLE_UPDATE = "true"
bun run tauri dev
```

### 禁用后的效果

- 启动时不会自动检查更新
- “关于”页不再显示检查更新卡片
- “关于”页会显示“当前构建未启用在线更新”
- 后端 `check_update` 会直接返回“未检查、无更新”

## 三、彻底移除更新代码再发布 GitHub

如果你希望 GitHub 开源仓库里完全不包含在线更新实现，可以直接删除以下内容：

### 删除目录

- `src/features/update/`
- `src-tauri/src/features/update/`

### 修改文件

#### 1. `src/App.vue`

删除：

- `useUpdateStore`
- `UPDATE_FEATURE_ENABLED`
- `updateStore.checkForUpdates()`

#### 2. `src/components/SettingsModal.vue`

删除：

- `useUpdateStore`
- `UPDATE_FEATURE_ENABLED`
- 关于页中的更新卡片
- 与更新相关的方法：
  - `handleCheckUpdate`
  - `handleOpenUpdate`
  - `hasUpdate`
  - `latestVersionText`

#### 3. `src-tauri/src/lib.rs`

删除：

- `mod features;`
- `features::update::command::check_update,`

#### 4. `src/i18n/zh-CN.ts`

删除 `update` 相关文案。

#### 5. `src/i18n/en-US.ts`

删除 `update` 相关文案。

## 四、发布到 GitHub 前建议检查

建议在推送到 GitHub 前执行以下检查：

```powershell
bun run build
cd src-tauri
cargo check
```

如果你发布的是开源版，建议在禁用更新的环境变量下也再跑一次：

```powershell
$env:VITE_ENABLE_UPDATE = "false"
$env:APP_ENABLE_UPDATE = "false"
bun run build
cd src-tauri
cargo check
```

## 五、建议的 Git 发布流程

1. 新建一个用于开源发布的分支
2. 按“第二部分”先禁用更新功能验证界面
3. 如果需要彻底移除，再按“第三部分”删除更新代码
4. 运行构建检查
5. 检查是否还包含私有更新接口、API Key、私有下载地址
6. 确认无误后推送到 GitHub

## 六、当前与更新有关的敏感点

如果你打算彻底开源，重点检查这些内容是否还存在：

- 更新接口地址
- `X-Api-Key`
- 私有下载地址
- 与 install_id / 环境收集有关的逻辑

这些逻辑当前都已经集中在：

- `src/features/update/`
- `src-tauri/src/features/update/`

删除时优先处理这两处即可。
