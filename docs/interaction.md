# 当前交互规则（键鼠与物理手柄共用）

本文件描述当前实现；verification.md 保留历史变更，旧记录不应覆盖这里的规则。

## 焦点、选中与输入来源

焦点表示下一次确认的目标；选中表示当前页面或当前游戏配置。移动焦点不自动选中页面，鼠标悬停不修改配置。手柄焦点框与持久选中样式相互独立；版本菜单使用背景高亮表示悬停/手柄焦点，白框只表示已选配置。

已移除 F8 键盘模拟手柄及自定义模拟事件入口。物理手柄输入接续实际 DOM 焦点或最近操作的有效控件；鼠标点击、普通键盘按键接管后隐藏手柄焦点框。

普通键盘的焦点框按用户要求隐藏，Tab/Shift+Tab 仍按原生顺序运行；这是明确的产品选择，但会降低纯键盘用户定位当前控件的可见性。手柄长按仅重复方向和滚动，不重复确认、返回、肩键或菜单；键盘 Enter/Space 的重复 keydown 被抑制。文本编辑与输入法组合输入不应被导航截获。

## 页面与区域

- 点击或确认侧栏只改变页面，不把焦点强制送进右侧。
- 游戏和账户分别记住上次页签。侧栏浏览下载页与游戏版本管理页保持独立。
- 主界面分为侧栏（sidebar）、顶部页签（tabs）、页面内容（content）三个区域。
- 侧栏与右侧只允许左右方向跨越；进入右侧聚焦最左侧页签，返回侧栏聚焦当前页面归属入口。
- 右侧页签与内容只允许上下方向跨越；从内容返回顶部聚焦当前页面对应页签，而非距离最近的页签。
- 所有页面均有顶部页签，独立页面使用同名单页签。跨区只移动焦点，不自动确认或切换页面。
- 原生 Tab 不受方向区域栅栏约束。
- 肩键只在当前游戏/账户页签组内切换；焦点在当前页签上时同步到新页签。焦点在侧栏则保留侧栏位置。
- 正常切页保留仍有效的焦点；原控件卸载才回退。Esc/Back 按真实历史返回，恢复历史对应位置，不固定跳 Play。
- 无历史时 Back 不做任何动作；相同页面不重复压入历史。
- 空白占位页也能从侧栏进入顶部页签；内容无可操作控件时，下方向保持原位。

## 下拉菜单

展开后先聚焦当前配置。上下循环移动，确认提交，Back/Esc 取消并恢复触发按钮。左右、肩键、菜单不离开列表。

鼠标点击外部收起且不吞掉点击，其他按钮仍能正常响应；Tab 离开菜单时收起。鼠标悬停仅改变背景，不提交选择。普通鼠标/键盘操作不显示手柄焦点框。手柄模式不叠加静止鼠标指针的悬停高亮。

## 弹窗

打开后焦点进入弹窗、背景 inert。Tab 循环限制在弹窗，方向/确认只访问弹窗控件。Esc/Back 或点击遮罩关闭并恢复触发器。关闭弹窗不同时执行页面历史回退。

## 当前验证范围

Vitest 验证适配器、焦点管理、下拉菜单、模拟器生命周期；浏览器实测下拉内上下移动/左右锁定、鼠标接管点击侧栏、Esc 回退、肩键切页焦点同步。真实硬件、原生 WebKit 交互与 Windows/macOS 未据此宣称通过。

## 方向导航架构

页面层为页签注册 `group="tabs"`，并提供 `sidebarEntryId` 与 `activeTabId`；焦点管理器集中处理区域规则，不在按钮点击处理器里自动跨区。

处理顺序：下拉菜单拦截 → 当前作用域（主界面/弹窗）过滤 → 非方向动作 → 区域栅栏过滤（显式邻居同样受约束）→ 几何候选排序 → 跨区固定落点。侧栏边界优先于右侧上下边界，因此侧栏可以用右键进入顶部。目标不可用时不跳往无关选项。区域内保持空间导航。

栅栏只影响方向动作；鼠标、原生 Tab、Esc/Back、确认、文本编辑及肩键切页沿用既有规则。弹窗只在自身作用域导航，不套用主界面区域规则。

## 本地配置编辑

版本配置页的新建、编辑、删除使用原有弹窗作用域。文本输入控件也注册焦点，文本编辑键不被导航截获；手柄确认进入字段编辑，再确认/返回结束编辑并保留焦点；方向输入可继续导航。Steam 虚拟键盘仅在 Steam 适配生效时调用。保存成功再更新全局配置，失败保留草稿并提示；保存期间防止重复提交。删除需确认，仅删除配置记录。删除最后一项使编辑/删除/版本选择不可用，焦点恢复到新建按钮。新建成功后选中所创建的配置，页面位置不变。

## 文件迁移与任务进度

目录变更需弹窗确认。迁移期间锁定其他操作，Esc/Back/遮罩与窗口关闭不能取消；任务完成或报错后才恢复操作。进度条显示在右侧内容区下方、操作提示栏上方，参与布局，不覆盖内容和滚动条；无进行中任务时卸载。未来下载复用该进度条，普通下载不套用迁移的全局操作锁。

## Steam 可选适配与退出（2026-10-13）

设置只有“适配 Steam 启动”复选框，默认关闭。配置保存于游戏根目录 launcher-settings.json，随目录迁移。通过 SteamGameId / SteamAppId 正整数或 --steam-entry 判断 Steam 启动；gamescope、Steam 正在后台运行本身均不足以触发。仅勾选且来自 Steam 时全屏并启用键盘请求；不调用 Steam Input，也不依赖 Proton。

点击或手柄确认文本输入框时通过 /usr/bin/steam -ifrunning 发送固定的虚拟键盘请求，不上传字段文字。不在普通 focus 事件中重复呼出。Enter/Esc 或手柄确认/返回结束本次编辑，保留输入框焦点，不提交表单、不关闭外层弹窗；接着方向输入可导航。移到其他控件时请求收起键盘。请求串行，子进程有超时；失败保留实体键盘输入。Steam URL 请求没有可靠关闭回调；覆盖层自行收起是否发送 Enter/Esc、焦点与 gamescope 下实际显示仍须实机验证。

Alt+F4、Ctrl+Q（macOS 可用 Meta+Q）不受 Steam 开关限制；调用与系统 CloseRequested 共用的迁移保护，不直接强杀进程。没有新增退出按钮，Esc/返回不退出启动器。浏览器预览不关闭浏览器窗口。

设备映射原则：底部面键确认、右侧面键返回，Nintendo B/A 对应 Xbox A/B；以后提示用位置高亮，不按品牌字母。物理输入已通过 gilrs 接入共用动作适配器；架构与验证边界见 [native-input.md](native-input.md)。


## 固定目录修订（取代上述自定义迁移说明）

游戏目录固定 ~/.minecraft；自定义目录控件和迁移弹窗/IPC 已删除。Steam 设置继续存入此目录。AppImage 启动时自动迁入固定目录，验证通过后替换同名目标并更新启动脚本。旧 root-location 不再读取或写入。通用任务进度组件保留，当前没有运行中的下载功能。

## 本轮导航审查

保留侧栏/右侧水平跨区、页签/内容垂直跨区、从侧栏进入首个页签、从内容回当前页签、模态与下拉范围限制。修正下拉因父页面重渲染跳回已选项的问题，重渲染保留仍有效的菜单焦点。输入切换接续当前焦点；物理和模拟信号不共享按住状态。按键冲突消解与文本编辑通过自动化回归验证。

## Single editing entry per setting

- Keep one primary editing surface for each setting; do not duplicate controls across the management page, editor, and installation flow.
- Instance management owns Java selection/download/rescan and game launch settings (memory, display mode, resolution).
- The instance editor changes only the display name and icon. Renaming does not relocate the existing instance directory.
- Installing a new instance selects the Minecraft version, loader, name, icon and an explicit Java choice through the shared JavaSettings component. Vanilla/Fabric/Quilt may explicitly select no Java; Forge/NeoForge require a runtime. Changing game version or loader family resets confirmation. The management page retains Java controls for later changes.
- Launch settings are saved per instance; actual game launch and game-window state capture are still pending.
- Reuse existing components and input navigation rules when adding settings, and check whether an existing entry already covers the task before adding another.

- All dropdown triggers use the shared 300px width, capped at the available container width. Do not stretch dropdowns to fill a page or dialog; avoid per-page width overrides unless explicitly requested.

- Text inputs follow the same 300px width limit as dropdowns. Related fields such as window width and height sit side by side and wrap when space is insufficient; do not stretch them across the page.

- Future checkboxes follow [the shared checkbox design](ui-reference/checkboxes.md), with green fill and square-only hover/focus feedback. Keep this separate from ordinary button and dropdown styles.

## Combined input audit — 2026-10-14

- Menu dismissal by an outside pointer remembers the trigger without stealing the pointer's new target; controller takeover can resume locally after a blank click.
- Empty or disabled menus close and release their action interceptor. Cleanup clears only the interceptor owned by that menu.
- Focus registration cleanup cannot remove a newer registration with the same ID.
- Existing sidebar/tab/content fences, modal scope, input editing, menu interception and primary-action entry rules remain in force.
- Autosave regression covers rapid checkbox changes while a previous request is pending and avoids endless retries after failure.
- Validation: 60 frontend tests and production build passed. Browser exercise confirmed simulated-controller menu navigation, Escape closing the menu first, downward entry into the green primary action, and Escape restoring the editor trigger. Physical hardware was not exercised in this audit; adapter tests cover its shared routing logic.

Text-edit cancellation: mouse and controller editing use the same Back/Escape path,
independently of Steam adaptation. The first Back ends text entry and retains focus.
Manual-save fields restore their value at the start of that edit; immediate fields
(such as heap size) preserve their current value and existing autosave behavior.
An inline account editor consumes the next Back to cancel the form before page history.

- Future standalone directional arrows follow [the shared pixel arrow design](ui-reference/pixel-arrows.md). Hover and keyboard/controller focus highlight the arrow itself without rectangular chrome.

## Vertical navigation rows

Up/down first selects the nearest visual row containing an enabled, visible control,
then chooses the closest horizontal target within that row. Vertically overlapping
controls belong to the same row; height differences must not cause sideways moves.
Disabled-only rows are skipped. Explicit neighbors and primary action preferences
cannot jump past an intervening interactive row. Horizontal navigation, sidebar/tab
region boundaries and open-dropdown interception remain unchanged.

Regression validation: offset small buttons between wide selectors, up/down symmetry,
disabled rows, mixed-height controls and nearest target within a row. 90 frontend tests
and production build passed; browser verified instance selector → View logs → loader
selector through the shared directional navigation path.
