# UI 参考与素材记录

用户在本次会话提供官方 Java Edition Play 页截图，左下角标识 `v.3.41.29-2.6.12`，原始分辨率 3839×2070；另提供侧栏 `MINECRAFT: / JAVA EDITION` 字体局部截图。以这两张截图为本次布局与字体风格参考，不混用其他版本。

用户进一步明确：**不复刻官方图片，仅复刻 UI、LOGO 等美术风格；字体沿用截图风格。** 因此生产资源不含官方截图或图片；背景和资讯缩略图使用原创 CSS 图形，文字 LOGO 为 CSS 表现，不提取官方图像。侧栏采用粗体无衬线、全大写、两行排版；中文采用系统无衬线回退。截图无法识别确切字体文件，未声称字体完全一致。

已对齐：约 190px 侧栏、顶置账户、分隔导航行、紧凑页头、下划线页签、主视觉比例、贴近主视觉底边的绿色 Play、左侧版本选择、右侧状态、五列资讯卡。原生标题栏保留平台行为，不仿造截图窗口按钮。

按项目范围省略其他游戏与 Realms、礼品购买；加入 Mods / 资源和原型说明。全部可操作控件为语义 React 按钮，共用焦点管理器。没有把整屏截图当作界面。

已查看浏览器截图验证构图，未完成指定尺寸的逐像素对比或真实 WebKit 字体对照。后续可在相同 viewport、缩放、字体条件下比较。截图仅作为用户会话参考，没有随应用打包。

素材来源：背景、卡片为原创 CSS；窗口图标为原创 32×32 RGBA 绿色方块；字体为系统字体。不附带官方专有图片、字体或启动器程序。MINECRAFT 品牌名称用于说明对应游戏，本项目非官方产品。

## 宣传区域后续接入

用户确认：主视觉宣传位与下方卡片可在后续接入官方或第三方宣传页面，当前仅预留布局。现阶段不引用外部宣传页面、图片、接口或链接；卡片为非交互占位，避免暗示已有内容。未来接入时再定义来源、内容模型及打开方式，并为新增交互接入统一焦点管理。

侧栏边框取色：使用用户提供的 369×249 官方局部原始 PNG，平坦区域主色为普通左上边 `#514d4c`、暗边 `#000000`、选中右下边 `#6b6562`；普通底色 `#312d2c`、选中底色 `#262423`。按用户要求采用 2 CSS px 边框，选中时持久反向，焦点使用独立内框。与截图的显示缩放、抗锯齿差异仍需区分，不宣称整页逐像素一致。

侧栏按压/持久选中状态修正：左上边使用选中主体色 `#262423`，右下边保留 `#6b6562`，不使用黑色左上边。

## 用户指定的主页素材

用户后续明确提供并要求使用 `mchero.png` 与 `mclogo.png`，此要求替代此前主页仅用原创占位的决定。文件原样复制到 `apps/desktop/src/assets/`，分别用于背景和 LOGO；不做外部请求。来源为用户本地 Pictures/Grids & Icons/Minecraft 目录，版权/再分发授权未提供，分享构建前需核实素材授权。宣传卡片仍为未接入占位。

## Java Edition 合成标志

使用内置 image_gen 根据用户提供的 logo 局部参考生成透明 RGBA 图片，保存为 `apps/desktop/src/assets/mclogo-java.png`，替换主页原有图片加普通副标题文字。原素材保留。

生成提示词：Create a transparent PNG logo asset for a Minecraft Java launcher, closely matching the supplied reference. Exact text: MINECRAFT on the top line, JAVA EDITION centered tightly underneath. Preserve the reference's angular block stone letterforms, creeper-shaped A, pale grey faces, black thick stepped outlines, dark extruded bottoms, subtle stone cracks, and smaller block-pixel JAVA EDITION letters with light gray faces and black outline. Remove the orange scenery completely, genuinely transparent alpha background. No new scenery, no extra text, no framing. Wide tightly cropped horizontal logo composition with minimal transparent padding. This is a logo asset only, not a screenshot or UI.

导航职责：侧栏“模组 / 材质 / 光影”为未来浏览下载页；游戏顶部“版本配置”右侧依次为“模组管理 / 材质管理 / 光影管理”，针对当前所选游戏配置。当前均仅建立界面占位，不访问内容服务或实现安装管理。管理页显示与 Play 共用的当前配置，并可显式进入版本配置页。

账户入口头像位：预留 Minecraft 游戏皮肤头部头像（含帽子外层）的渲染结果，使用 SkinAvatar 组件；不使用 Microsoft/Xbox 用户头像。尚未接入皮肤时显示占位符，不发起头像请求。

- [Checkboxes](checkboxes.md): default square checkbox palette, checked fill, focus and pressed-state rules.
- [Pixel arrow buttons](pixel-arrows.md): shared standalone arrows, gray vertical edge, diagonal white highlights, and arrow-only focus feedback.
- [Scrollbars](scrollbars.md): black track and square gray thumb matching the in-game reference.

- [Form spacing](form-spacing.md): shared 16 CSS px spacing for labels, explanatory text, controls, and adjacent settings groups.
