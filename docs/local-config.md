# 本地配置闭环

本轮进入第二阶段的本地配置部分，不包含认证、下载、安装和 JVM 启动。旧示例配置已退出应用数据源，旧浏览器偏好键保留但不迁移为已安装游戏。

## 数据与职责

- `crates/launcher-core`：独立 Rust 库，无 Tauri 或网络依赖。数据校验、JSON 文件读写、版本冲突检查。
- Tauri：读取启动器同目录的游戏根目录 `<launcher>/.minecraft`，不读取目录定位文件，配置文件为启动器目录下的 `launcher-library.json`，管理 Store，暴露 `load_library` / `save_library`。
- `state/library.ts`：类型、存储适配和单一 React 状态。桌面调用 Tauri；浏览器预览独立使用 `launcher.local-library.v1`。原生调用失败不会退回浏览器存储。
- 页面只提交配置更改，保存成功后更新共享状态。Play/版本配置复用下拉，侧栏图标及三个管理页读取同一当前配置。

每个配置含稳定 ID、名称、Minecraft 版本字符串、图标 ID。名称唯一且最多 64 个 Unicode 字符。版本字符串目前只做格式校验，不承诺该版本存在或兼容。新配置随机分配一个临时内置图标，用户可更换；图标 ID 随配置持久化，后续可扩展为图标库。

`selectedId` 为当前选择；`lastLaunchedId` 仅代表成功启动。启动时优先恢复后者，其次恢复选择。编辑/选择不能生成启动记录；当前尚无写入成功启动记录的业务入口。删除相应配置时清理失效引用；删至空列表后允许重新创建。

## 保存与错误

文件保存前先校验数据；使用进程内互斥锁及 OS 文件锁串行化原生写入，修订号拒绝旧快照覆盖。临时文件写入并同步后 rename 替换正式文件。错误会显示给用户，保留编辑草稿；损坏文件不自动重置、不覆盖。未来若修改 schema，应提供显式迁移。

浏览器存储仅用于预览，不承诺多标签页原子事务。浏览器与桌面不自动导入彼此的数据。

本轮删除仅删除配置记录，不触碰游戏文件。保存过程中禁止重复提交及关闭编辑弹窗；保存完成或报错后恢复操作。加载/保存失败可重新加载应用重试。

## 输入与界面

普通操作使用既有白/绿/红按钮。编辑器的文本字段注册到集中焦点管理器，手柄可导航到字段；文字录入仍使用键盘，虚拟键盘后续实现。输入框内方向、字母与输入法组合输入保留原生行为。Esc、取消或遮罩关闭草稿；关闭后返回触发器，删除最后一项则返回新建按钮。不会改变侧栏/顶部/内容的区域栅栏。

## 检查命令

```sh
pnpm test
pnpm build
cargo test --manifest-path crates/launcher-core/Cargo.toml
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
cargo build --manifest-path apps/desktop/src-tauri/Cargo.toml
```

测试覆盖文件往返、修订冲突、损坏文件保留、无效数据/伪造启动记录拒绝、Tauri IPC 到文件存储闭环、浏览器失败处理及输入焦点回归。Linux 实际窗口启动不等于 Windows/macOS 验证，也不等于原生 WebView 每项交互均已实测。

固定保存目录、AppImage 自动迁入与启动脚本、后续更新规则及统一进度条见 [storage-and-updates.md](storage-and-updates.md)。

## Launch settings (r14)

Instance `launchSettings` contains `maxMemoryMib` (null for automatic), `displayMode` (inherit/windowed/fullscreen), `width`, `height`, and `rememberMaximized`. Defaults are automatic memory, inherited display mode, 1280 × 720 and no maximized-state retention. New optional settings use a single default model; no legacy schema reader is added.

The management page owns editing and automatic saving of valid changes to these settings. Frontend `state/launchSettings.ts` defines the typed model and presets; `state/useAvailableMemory.ts` owns memory polling. The Rust core owns structural validation; desktop configuration saving rechecks Linux MemAvailable before accepting changed manual-memory settings. Stored preferences survive restart and remain independent per instance. Failed validation leaves the existing file untouched.

Actual game process launch, automatic heap selection at launch, and observing/restoring a game window's maximized state are not implemented. `rememberMaximized` currently persists the preference only; it does not track the launcher's own window. A future launch operation must revalidate memory immediately before spawning Java. Do not present saved preferences as proof that game launch behavior has been implemented.

## Accounts

Accounts are created only by successful Microsoft authorization. Local account
creation and rename controls, preview identities, and unverified game launch paths
have been removed. See microsoft-accounts.md for persistence and verification.

## Vanilla game launch (Linux x86_64)

The desktop `launch_game` command reads the instance and selected authenticated identity
from native storage, probes the explicitly selected Java, and validates available
memory again. Automatic heap size uses half of currently available memory, capped
at 4096 MiB. It never silently installs or changes Java.

`launch_plan.rs` resolves vanilla JVM/game arguments, ordered OS rules, custom
resolution features, shared libraries/assets, instance natives, logging config,
and legacy `minecraftArguments`. Unknown placeholders and inherited/modded
metadata fail explicitly. Arguments are passed directly to Java without a shell.
Windowed/fullscreen modes update only `fullscreen` in the instance's options.txt;
inherit leaves game preferences untouched. Remembering window-manager maximization
is still a stored preference, not implemented in this launch path.

The game runs with the instance directory as cwd. Output and errors are written to
`versions/<directory>/logs/launcher-<timestamp>.log`. Nonzero exit codes appear in
the UI with the log path. Only one game/file task runs at once; installation,
repair and deletion are locked while playing. Closing the launcher while a game
runs hides its window and keeps the file/session lock until the game exits, then
exits the launcher. This also preserves Steam process tracking.

Verification includes modern/legacy argument fixtures, OS/feature filtering,
missing files, option preservation, and an explicitly run real-JDK integration
test with a temporary Java program (normal and nonzero exits). It is not a claim
that every historical Minecraft version has been tested in-game.
