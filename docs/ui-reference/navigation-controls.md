# 专用导航控件：侧栏深色按钮与版本下拉选择器

这两类控件独立于 [白/绿/红/黄普通按钮](buttons.md)，不得作为 StandardButton 的颜色变体处理。以下记录当前已确认设计和实际 CSS 生效值；未重新改动界面。

| 类型 | 调用组件 | 主要作用 | 持续状态 |
|---|---|---|---|
| 侧栏深色导航按钮（黑色按钮） | `SidebarButton` | 切换页面区域 | 当前页面一直保持选中 |
| 版本下拉选择器 | `SelectMenu` | 选择同一份游戏配置 | 展开期间按钮保持按下；已选配置独立标记 |
| 普通彩色按钮 | `StandardButton` | 执行动作 | 按压通常随释放结束 |

## 侧栏深色导航按钮

| 部位 | 普通状态 | 按压 / 持久选中 |
|---|---|---|
| 主体 | `#312D2C` | `#262423` |
| 左、上边 | `#514D4C` | `#262423`，与主体同色 |
| 右、下边 | `#000000` | `#6B6562` |
| 外边宽度 | 2px | 2px |
| 选中左侧标记 | 无 | `#EEEEEE`，4×23px |
| 手柄焦点框 | `#AAA6A2`，2px，outline-offset -9px | 相同，独立于页面选中 |
| 主文字 | `#F4F4F4` | 相同 |

普通态没有底部厚阴影；不能套用彩色普通按钮的白色按压外框。切换到别的侧栏页面后才释放持久选中状态。焦点移动不等于选择页面。

字体：Segoe UI / Arial / Noto Sans CJK SC / Microsoft YaHei / sans-serif，14px、700、行高 1.45，选中前后保持一致。账户状态文字沿用辅助灰色 `#AAA6A2`；Java 第二行白色标签保持原层级。

最小高度 60px；账户行高度 76px；内边距 10px 13px；图文间距 10px；图标列宽 30px；文字左对齐。选中标记定位 left 0 / top 18px。全部侧栏入口共用该组件，不恢复旧账户箭头或独立右对齐规则。

```tsx
<SidebarButton id="nav-模组" icon={null} selected={page === "模组"}
  onClick={() => changePage("模组")}>模组</SidebarButton>
```

账户可用 `account` 保留行高差异，仍为页面导航。账户头像来源预留为 Minecraft 皮肤头部，不是 Xbox 头像。

## 版本下拉：触发按钮

| 部位 | 收起 | 展开 / 按压 |
|---|---|---|
| 主体 | `#302D2B` | `#262423` |
| 左、上边 | `#514D49` | `#262423` |
| 右、下边 | `#151413` | `#6B6562` |
| 边框 | 2px | 2px |
| 阴影 | inset 0 1px `#FFFFFF06` | 无 |
| 手柄焦点 | `#86817C`，2px，outline-offset -6px | 同样规则 |

宽度 min(300px,100%)；最小高度 49px；内边距 5px 12px；文字 12px。展开按钮的配色与侧栏相近但数值并非完全相同，应保持独立，不以“统一黑色”覆盖。

## 版本下拉：菜单与项目

| 部位 / 状态 | 配置 |
|---|---|
| 菜单及普通项背景 | `#3D3938` |
| 鼠标悬停 / 手柄焦点项背景 | `#6B6562` |
| 主、副文字 | `#EDE5E2` |
| 已选配置框 | inset 0 0 0 2px `#FFFFFF` |
| 菜单外边 | 2px `#000000`，无白色外框 |
| 阴影 | 0 6px 16px `#00000088` |
| 单独焦点描边 | 无；焦点用背景色区分 |

菜单紧接按钮下方 3px，与按钮同宽，z-index 20，最大高度 230px，溢出纵向滚动；外部 padding 0。每项最小高度 56px、padding 10px 12px、图文间距 12px、字号 12px。白框只表示已选择的配置，绝不能随焦点或悬停移动。

开始游戏页与版本配置页调用同一组件、同一版本状态和图标映射。两页互斥挂载，共用稳定 focus ID；如果未来要同时显示多个选择器，必须先参数化 ID，不能直接复制多个实例。

```tsx
<SelectMenu options={versions} value={version}
  onChange={setVersion} iconForOption={iconForVersion} />
```

## 交互约束

- 手柄/模拟手柄信号才显示焦点反馈；鼠标、普通键盘不显示框。原生键盘编辑不受影响。
- 下拉展开时上下循环，确认选择，Back/Esc 取消；左右、肩键、菜单不能逃离列表。
- 鼠标点击外部、键盘 Tab 离开可收起；外部点击不被吞掉。选择/返回后恢复触发器焦点。
- 静止鼠标不在手柄模式中叠加菜单悬停色。
- 侧栏与右侧内容的上下方向导航不跨区；左右可跨区，返回侧栏指向当前页面对应入口。
- 完整规则见 [交互说明](../interaction.md)。

实现来源：`apps/desktop/src/styles/app.css`、`components/SidebarButton.tsx`、`components/SelectMenu.tsx`。侧栏颜色取自用户的官方侧栏截图；菜单颜色取自官方账户菜单参考。尺寸和焦点样式包含用户在本项目中确认的定制，不应称为整套官方原样样式。

账户管理页使用 16px 内容内边距及组间距，300px 账户选择控件；编辑、删除在选择控件右侧。说明和错误与控件左缘对齐，错误不额外缩进且保持红色。页面不套用 placeholder 布局；顶部选项卡显示“登入”，侧栏及页面标题为“账户管理”。

Account controls now show only the signed-in account selector, Delete, and Sign in. Offline creation, rename, account-type labels, and preview identities are removed. The account tab is Sign in.
