# 配置图标

使用本机已安装的 Minecraft Java Edition 26.3 客户端中的原始材质。用户已要求直接采用这些素材；不将其标注为开源或原创素材。素材权利属于 Mojang/Microsoft。

当前共 30 项，菜单按建筑方块 → 功能方块 → 材料 → 工具武器排列；食物归材料，桶、鞍、不死图腾归工具装备。

9 个方块采用官方贴图生成 SVG；铁砧按官方 template_anvil 模型的分段几何与 UV 渲染，工作台和熔炉分别使用顶面、正面、侧面材质。其余 21 个物品为原始 PNG。

本轮移除书、下界合金锭、黏液球、钻石镐、水桶、烟花火箭、金块、石头、泥土、弩、末影之眼，新增铁砧、工作台、熔炉、木斧、鞍。移除的图片不再随项目引用；读取旧配置时将已退役图标回退为草方块，不改动其他设置，也不在只读加载时写回文件。

源文件路径、逐项 SHA-256 和客户端 JAR SHA-256 记录在 apps/desktop/src/assets/installation-icons/sources.json。生成脚本：scripts/import-game-icons.py，参数为本地客户端 JAR；只读游戏文件，输出到项目素材目录。

InstallationIcon 使用 img 标签，透明背景、object-fit: contain、image-rendering: pixelated；侧栏、当前配置和下拉菜单共用。不依赖玩家运行时已安装某一版本。
