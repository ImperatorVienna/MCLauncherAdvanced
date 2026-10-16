# 普通按钮规范（默认规则）

用户指定：今后未单独强调样式的普通按钮遵照本规范；“白色按钮”“绿色按钮”“红色按钮”“黄色按钮”分别对应 `white / green / red / yellow`。已有专门设计的侧栏、主 Play、版本选择器、顶部页签不被覆盖。

来源：用户本次提供的 8 张官方按钮原始 PNG，按顺序前四张普通、后四张按压。原图保存在 [buttons](buttons/) 目录，仅作设计参考，不作为运行时 UI 图片。使用 Pillow 读取原图 RGB，统计大块纯色并检查横纵像素段，排除文字抗锯齿和边角混色。以下为截图实测色值，不是近似命名色。

## 普通状态

| 调用名称 | 主体 | 左上边 | 右下边 | 底部厚阴影 | 文字 |
|---|---|---|---|---|---|
| 白色按钮 | `#D0C5C0` | `#ECE8E6` | `#E3DCD9` | `#ABA09C` | `#262423` |
| 绿色按钮 | `#3C8527` | `#639D52` | `#50913D` | `#2A641C` | `#FFFFFF` |
| 红色按钮 | `#CA3636` | `#D55E5E` | `#CF4A4A` | `#AD1D1D` | `#FFFFFF` |
| 黄色按钮 | `#E2B923` | `#EED57B` | `#E8C74F` | `#B87918` | `#000000` |

## 按压状态

| 调用名称 | 主体 | 左上边 | 右下边 | 底部薄阴影 | 外框 | 文字 |
|---|---|---|---|---|---|---|
| 白色按钮 | `#ABA09C` | `#DDD9D7` | `#CDC6C4` | `#ABA09C` | `#FFFFFF` | `#262423` |
| 绿色按钮 | `#2A641C` | `#558349` | `#407433` | `#2A641C` | `#FFFFFF` | `#FFFFFF` |
| 红色按钮 | `#AD1D1D` | `#BD4A4A` | `#B53434` | `#AD1D1D` | `#FFFFFF` | `#FFFFFF` |
| 黄色按钮 | `#B87918` | `#D4AF74` | `#C69446` | `#B87918` | `#FFFFFF` | `#000000` |

## 形状与交互

矩形直角、左上和右下分色，普通态底部有厚度；按压时主体变深、底部厚度缩小，并显示白色外框。原图边缘为 4 图像像素，普通底厚 12 像素，按压底厚 4 像素、白框 6 像素。组件按约 2× 参考缩放使用 2px 边缘、6px 普通底厚、2px 按压底厚、3px 白框；这是实现尺寸约定，不声称已知截图系统缩放倍率。

用户将后四张定义为按压态，按此实现；白框属于按压反馈，不是普通键鼠焦点框。默认按压随释放结束，不能自动沿用侧栏“选中后持续按压”的专门规则。悬停和禁用的专用截图尚未提供，不臆测额外官方色值；目前悬停保留主体色、禁用沿用公共控件行为。

文字继承项目字体，不使用截图英文文本替换中文；宽度按内容和留白适配。所有普通按钮通过 Control 注册，继续支持键鼠与手柄统一导航。

## 代码调用

```tsx
import { StandardButton } from "./components/StandardButton";

<StandardButton id="save" tone="green" onClick={save}>保存</StandardButton>
```

`tone` 默认 `white`。颜色变量在 `apps/desktop/src/styles/buttons.css`，组件在 `apps/desktop/src/components/StandardButton.tsx`。本次建立规范与组件，未批量改变已有专门样式的按钮。

## 与专用导航控件的边界

“黑色侧栏按钮”与“版本下拉菜单”分别见 [专用导航控件规范](navigation-controls.md)。它们不是第五种普通按钮颜色；持续选中、展开状态、内嵌焦点框与普通彩色按钮的按压白框不可混用。

Log viewing convention: all View logs / View launcher logs / View this launch log
and Open log file actions use the yellow StandardButton tone. Refresh, Copy,
Close and Open instance directory remain ordinary neutral buttons.
