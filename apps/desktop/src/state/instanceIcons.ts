import { t } from "../i18n";
// Stable IDs backed by textures imported from the official local client.
export const iconCatalog = ["grass", "sand", "cobblestone", "oak", "brick", "obsidian", "anvil", "crafting_table", "furnace", "diamond", "emerald", "redstone", "coal", "iron_ingot", "gold_ingot", "lapis_lazuli", "blaze_rod", "nether_star", "ender_pearl", "enchanted_book", "apple", "golden_apple", "wooden_axe", "iron_pickaxe", "diamond_sword", "bow", "fishing_rod", "lava_bucket", "saddle", "totem_of_undying"] as const;
export type IconId = typeof iconCatalog[number];
export const iconNames: Record<IconId, string> = { get grass() { return t("Grass block"); }, get sand() { return t("Sand"); }, get cobblestone() { return t("Cobblestone"); }, get oak() { return t("Oak log"); }, get brick() { return t("Bricks"); }, get diamond() { return t("Diamond"); }, get emerald() { return t("Emerald"); }, get redstone() { return t("Redstone"); }, get obsidian() { return t("Obsidian"); }, get diamond_sword() { return t("Diamond sword"); }, get iron_pickaxe() { return t("Iron pickaxe"); }, get bow() { return t("Bow"); }, get apple() { return t("Apple"); }, get golden_apple() { return t("Golden apple"); }, get ender_pearl() { return t("Ender pearl"); }, get enchanted_book() { return t("Enchanted book"); }, get coal() { return t("Coal"); }, get iron_ingot() { return t("Iron ingot"); }, get gold_ingot() { return t("Gold ingot"); }, get lapis_lazuli() { return t("Lapis lazuli"); }, get totem_of_undying() { return t("Totem of undying"); }, get lava_bucket() { return t("Lava bucket"); }, get fishing_rod() { return t("Fishing rod"); }, get blaze_rod() { return t("Blaze rod"); }, get nether_star() { return t("Nether star"); }, get anvil() { return t("Anvil"); }, get crafting_table() { return t("Crafting table"); }, get furnace() { return t("Furnace"); }, get wooden_axe() { return t("Wooden axe"); }, get saddle() { return t("Saddle"); } };
export const iconImages: Record<IconId, string> = {
    grass: new URL("../assets/installation-icons/grass.svg", import.meta.url).href,
    sand: new URL("../assets/installation-icons/sand.svg", import.meta.url).href,
    cobblestone: new URL("../assets/installation-icons/cobblestone.svg", import.meta.url).href,
    oak: new URL("../assets/installation-icons/oak.svg", import.meta.url).href,
    brick: new URL("../assets/installation-icons/brick.svg", import.meta.url).href,
    diamond: new URL("../assets/installation-icons/diamond.png", import.meta.url).href,
    emerald: new URL("../assets/installation-icons/emerald.png", import.meta.url).href,
    redstone: new URL("../assets/installation-icons/redstone.png", import.meta.url).href,
    obsidian: new URL("../assets/installation-icons/obsidian.svg", import.meta.url).href,
    diamond_sword: new URL("../assets/installation-icons/diamond_sword.png", import.meta.url).href,
    iron_pickaxe: new URL("../assets/installation-icons/iron_pickaxe.png", import.meta.url).href,
    bow: new URL("../assets/installation-icons/bow.png", import.meta.url).href,
    apple: new URL("../assets/installation-icons/apple.png", import.meta.url).href,
    golden_apple: new URL("../assets/installation-icons/golden_apple.png", import.meta.url).href,
    ender_pearl: new URL("../assets/installation-icons/ender_pearl.png", import.meta.url).href,
    enchanted_book: new URL("../assets/installation-icons/enchanted_book.png", import.meta.url).href,
    coal: new URL("../assets/installation-icons/coal.png", import.meta.url).href,
    iron_ingot: new URL("../assets/installation-icons/iron_ingot.png", import.meta.url).href,
    gold_ingot: new URL("../assets/installation-icons/gold_ingot.png", import.meta.url).href,
    lapis_lazuli: new URL("../assets/installation-icons/lapis_lazuli.png", import.meta.url).href,
    totem_of_undying: new URL("../assets/installation-icons/totem_of_undying.png", import.meta.url).href,
    lava_bucket: new URL("../assets/installation-icons/lava_bucket.png", import.meta.url).href,
    fishing_rod: new URL("../assets/installation-icons/fishing_rod.png", import.meta.url).href,
    blaze_rod: new URL("../assets/installation-icons/blaze_rod.png", import.meta.url).href,
    nether_star: new URL("../assets/installation-icons/nether_star.png", import.meta.url).href,
    anvil: new URL("../assets/installation-icons/anvil.svg", import.meta.url).href,
    crafting_table: new URL("../assets/installation-icons/crafting_table.svg", import.meta.url).href,
    furnace: new URL("../assets/installation-icons/furnace.svg", import.meta.url).href,
    wooden_axe: new URL("../assets/installation-icons/wooden_axe.png", import.meta.url).href,
    saddle: new URL("../assets/installation-icons/saddle.png", import.meta.url).href,
};
