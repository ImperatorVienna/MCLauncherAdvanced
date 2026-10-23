# MC Launcher Advanced

面向 Minecraft Java Edition 的第三方启动器，采用类似官方启动器的界面。

**[English](README.md)** · **[繁體中文](README.zh-TW.md)** · 包名：`mclauncher-advanced`

本项目为独立桌面启动器，支持键鼠和手柄导航、便携目录、游戏配置隔离及 Modrinth 内容管理。它不是 Minecraft 官方产品，与 Mojang、Microsoft 没有关联，也未获得其认可或背书。

## 发布状态

当前版本为 **1.0.5**，提供 Linux 与 Windows x86_64 发行包。各平台验证范围见下文。

本次变更见 [1.0.5 更新公告](docs/release-notes/1.0.5.zh-CN.md)。

1.0.5 新增繁体中文支持，统一中文资源包文案，整理 Linux 应用入口与注册流程，并修复皮肤页面的焦点恢复问题。

“有什么新内容？”按当前界面语言加载对应的启动器更新公告，缺少翻译时显示提示，不回退到其他语言。

## 功能

- 使用玩家明确选择的 Java 安装、修复和启动 Minecraft: Java Edition。
- 创建隔离配置、识别本地版本文件夹，分别管理内存、显示方式和 Java。
- 安装 Fabric、Quilt、Forge、NeoForge，并在配置原有加载器类型内更换版本。
- 从 Eclipse Adoptium／Temurin 下载 Java，校验下载文件并重新检测本机运行时。下载和选择均由玩家主动操作。
- 根据当前配置的游戏版本与加载器，在 Modrinth 浏览、搜索、安装和更新模组、资源包及光影包。下载页与配置管理共用配置选择。
- 扫描本地内容、启停模组、删除文件及检查更新。资源包与光影包在游戏内启用和设置。
- 使用鼠标、键盘或实体手柄导航。可选 Steam 适配在从 Steam 启动时启用启动器全屏，并请求 Steam 虚拟键盘。
- 设置页提供 Xbox、PlayStation、Nintendo Switch Pro 和通用手柄示意图及操作说明；可从设备下拉菜单切换，收到有效手柄输入后静默跟随当前设备，不改变页面与焦点。
- 提供英文、简体中文和繁体中文界面，首次启动默认英文。
- 在界面内查看各配置的游戏／安装日志以及启动器日志。
- 首页显示五条 Java 版官方新闻；游戏更新说明按当前配置的 Minecraft 版本加载。
- 在“设置 → 关于”查看版本、检查更新、访问维护者主页和项目主页，并通过独立弹窗阅读开源许可证和第三方声明。

皮肤库支持导入本地 PNG、预览正版账户皮肤，以及上传并应用皮肤。选择经典或纤细手臂模型后，点击“使用皮肤”同步到账户。启动时会自动检查 GitHub 正式版更新，“设置 → 关于”也可手动检查、下载校验并自动重启安装。新版采用发行包的文件名并保存在原目录，旧版加上 `.previous` 后缀保留；新版成功启动后清理更新缓存。详见[发布约定与验证范围](docs/launcher-updates.md)。

## 官方新闻与游戏更新说明

首页从 Mojang 的公开启动器数据源加载五条 Java 版相关新闻，点击卡片在浏览器中打开对应的 Minecraft 官方文章。

“更新说明”显示当前配置的 Minecraft 游戏版本对应的官方文章，与左侧“有什么新内容？”中的启动器更新公告相互独立。游戏文章按版本精确匹配，不会用其他版本的说明代替；没有对应文章或网络请求失败时会显示提示。官方新闻与游戏更新说明统一使用英文原文，不随界面语言切换；按钮与来源说明随界面语言显示。新闻或文章暂时不可用不会阻止游戏启动。

1.0.2 为启动器窗口、桌面集成及浏览器预览启用 MCLA 图标，修复 Wayland 应用身份匹配，并优化内置字体、文章排版和英文界面文案。素材声明与许可证原文统一以英文展示。

## 平台和首次运行

| 平台 | 包格式 | 验证状态 |
| --- | --- | --- |
| Linux x86_64，glibc | AppImage | 开发过程中已测试游戏安装和启动 |
| Windows 10／11 x86_64 | 便携 EXE | 此前版本完成有限的 Wine 检查；1.0.5 已交叉编译打包，仍需 Windows 实机验收 |
| macOS ARM64 | 计划中 | 暂无构建 |

Windows 需要安装 **Microsoft Edge WebView2 Runtime**。便携 EXE 不内置该运行时，也不会自动安装。Wine／Proton 不作为 Windows 验收环境；Linux 请使用原生 AppImage。目前不面向 Windows 7／8／8.1 或其他处理器架构。

1. 将可执行文件放入可写文件夹，避免使用受系统保护的安装目录。
2. Linux 下赋予 AppImage 执行权限后运行；系统需支持运行 AppImage。
3. Windows 下如缺少 WebView2 Runtime，请先安装，再运行 EXE。首次运行会在同目录创建 `launch.bat` 和桌面快捷方式，不注册开始菜单入口。
4. 选择配置与 Java。Forge／NeoForge 安装流程要求有效的 Java；其他安装流程允许明确选择“暂不选择 Java”，启动游戏时仍需 Java。
5. 登录并启动。缺失的受管理游戏文件通过安装／修复流程补齐。

使用 Steam 时，将生成的启动器入口／脚本添加为非 Steam 游戏。Windows 快捷方式指向 `launch.bat`，EXE 改名后脚本仍可识别。移动整个文件夹后，先直接运行一次可执行文件以刷新桌面入口，再重新添加到 Steam。

## 便携目录

存储位置以启动器可执行文件所在目录为准，不使用启动命令时的工作目录：

```text
<启动器文件夹>/
  <启动器可执行文件>
  launch.sh 或 launch.bat
  launcher-library.json
  launcher-settings.json
  accounts.json
  webview/
  runtimes/java/
  logs/
  .minecraft/
    assets/
    libraries/
    versions/<配置文件夹>/
      mods/
      resourcepacks/
      shaderpacks/
      saves/
      logs/
```

文件夹按需创建。存档、模组、资源包、光影包和游戏设置按配置隔离，libraries／assets 共用。启动器不会迁移其他启动器的数据，也不会继承其公共游戏目录中的模组和资源包。本地版本识别不保证兼容所有第三方启动器的目录格式。

托管 Java 使用相对启动器目录的路径保存。换电脑后，系统 Java 可能需要重新选择；重新检测会检查当前电脑的运行时。

`accounts.json` 包含认证凭据，使用操作系统文件权限保护，并非加密导出文件。**不要上传、附在问题报告中，或在将启动器复制给他人时分享该文件。**

## 内容更新规则

- 兼容性筛选跟随当前配置；不支持检索的游戏版本可能需要手动安装内容。
- 通过文件哈希匹配 Modrinth 能识别的内容。未识别的文件仍按文件名管理，但无法自动匹配更新。
- 模组、资源包和光影包统一使用上游新版本的文件名。更新后如有需要，请在游戏内重新启用资源包并调整设置；启动器不修改游戏的资源包选择配置。
- 缺少的必需依赖会下载，已识别的现有依赖会保留。更新目标文件不代表已验证所有模组之间的完整兼容性。
- 删除文件为永久删除，不创建备份。
- 游戏内使用手柄需要 Controlify 等手柄模组；使用光影包需要兼容的光影模组。启动器的导航功能不会自动让游戏具备这些能力。

## 开发

项目使用 Tauri 2／Rust 和 React／TypeScript／Vite。请安装兼容的 Rust 工具链、Node.js 22.12+、`package.json` 指定的 pnpm 版本，以及 [Tauri 平台依赖](https://v2.tauri.app/start/prerequisites/)。

```sh
pnpm install
pnpm dev                       # 浏览器预览：http://127.0.0.1:1420
pnpm test                      # 前端回归测试
pnpm build                     # 类型检查和前端构建
pnpm tauri dev                 # 原生桌面开发
pnpm tauri build --bundles appimage  # Linux AppImage
python3 scripts/check-source-language.py
```

浏览器预览使用独立的本地数据，不执行原生游戏安装或启动。F8 手柄模拟已移除；手柄导航通过原生输入适配器接入。

严格遵守架构分层：前端负责显示、焦点和导航；核心负责通用模型与持久化；领域服务负责游戏和内容流程；平台适配器负责操作系统交互。参见[平台边界](docs/platform-boundary.md)、[目录规则](docs/storage-and-updates.md)、[安装流程](docs/installation.md)和[表单间距](docs/ui-reference/form-spacing.md)。

## 隐私与本地数据

[隐私与本地数据](PRIVACY.zh-CN.md)

## 维护者与联系方式

MC Launcher Advanced 由 [ImperatorVienna](https://github.com/ImperatorVienna) 以个人名义维护和发布，项目责任主体为维护者本人，不代表公司或组织。

联系邮箱：[imperator.vienna@outlook.com](mailto:imperator.vienna@outlook.com)。

## 问题报告与致谢

报告问题时请提供版本分支、系统／架构、游戏与加载器版本、复现步骤及脱敏日志。不要附上访问令牌、刷新令牌、设备登录代码或账户存储文件。

依赖和打包素材各有独立的声明与许可证，见 `apps/desktop/src-tauri/licenses/`、内置字体的 OFL 声明及[素材来源](docs/ui-reference/README.md)。本 README 不授予第三方美术素材或 Minecraft 资源的再分发权。Minecraft 及相关名称属于各自权利人。

## 许可证

项目自有代码采用 [GNU GPL 3.0 许可证](LICENSE)（`GPL-3.0-only`）。
第三方库、字体与美术素材保留各自许可及权利归属，此许可证不对 Minecraft 素材重新授权。

## 界面设计与素材声明

界面参考 Minecraft 官方启动器的视觉风格，由本项目独立实现，并针对键鼠与手柄操作进行优化。本项目未使用通过逆向工程取得的官方启动器程序代码。

原创代码按 GPL-3.0-only 发布；引用的 Minecraft 标志、纹理、皮肤、背景及官方新闻内容不在该授权范围内。MCLA 图标的原创字母和几何排布与引用的草方块纹理分别声明权利。来源说明不等于使用授权，未确认的素材再分发许可均如实记录。详见[素材、设计与第三方权利声明](ASSET-NOTICES.md)。
