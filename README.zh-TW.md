# MC Launcher Advanced

面向 Minecraft Java Edition 的第三方啟動器，採用類似官方啟動器的介面。

**[English](README.md)** · **[简体中文](README.zh-CN.md)** · 包名：`mclauncher-advanced`

本專案為獨立桌面啟動器，支援鍵鼠和控制器導航、便攜目錄、遊戲配置隔離及 Modrinth 內容管理。它不是 Minecraft 官方產品，與 Mojang、Microsoft 沒有關聯，也未獲得其認可或背書。

## 釋出狀態

當前版本為 **1.0.5**，提供 Linux 與 Windows x86_64 發行包。各平臺驗證範圍見下文。

本次變更見 [1.0.5 更新公告](docs/release-notes/1.0.5.zh-TW.md)。

1.0.5 新增繁體中文支援，統一中文資源包文案，整理 Linux 應用入口與註冊流程，並修復皮膚頁面的焦點恢復問題。

「有什麼新內容？」依目前介面語言載入對應的啟動器更新公告，缺少翻譯時顯示提示，不回退至其他語言。

## 功能

- 使用玩家明確選擇的 Java 安裝、修復和啟動 Minecraft: Java Edition。
- 建立隔離配置、識別本地版本資料夾，分別管理記憶體、顯示方式和 Java。
- 安裝 Fabric、Quilt、Forge、NeoForge，並在配置原有載入器型別內更換版本。
- 從 Eclipse Adoptium／Temurin 下載 Java，校驗下載檔案並重新檢測本機執行時。下載和選擇均由玩家主動操作。
- 根據當前配置的遊戲版本與載入器，在 Modrinth 瀏覽、搜尋、安裝和更新模組、資源包及光影包。下載頁與配置管理共用配置選擇。
- 掃描本地內容、啟停模組、刪除檔案及檢查更新。資源包與光影包在遊戲內啟用和設定。
- 使用滑鼠、鍵盤或實體控制器導航。可選 Steam 適配在從 Steam 啟動時啟用啟動器全屏，並請求 Steam 虛擬鍵盤。
- 設定頁提供 Xbox、PlayStation、Nintendo Switch Pro 和通用控制器示意圖及操作說明；可從裝置下拉選單切換，收到有效控制器輸入後靜默跟隨當前裝置，不改變頁面與焦點。
- 提供英文、簡體中文和繁體中文介面，首次啟動預設英文。
- 在介面內檢視各配置的遊戲／安裝記錄以及啟動器記錄。
- 首頁顯示五條 Java 版官方新聞；遊戲更新說明按當前配置的 Minecraft 版本載入。
- 在“設定 → 關於”檢視版本、檢查更新、訪問維護者主頁和專案主頁，並通過獨立彈窗閱讀開源許可證和第三方宣告。

皮膚庫支援匯入本地 PNG、預覽正版帳戶皮膚，以及上傳並應用皮膚。選擇經典或纖細手臂模型後，點選“使用皮膚”同步到帳戶。啟動時會自動檢查 GitHub 正式版更新，“設定 → 關於”也可手動檢查、下載校驗並自動重啟安裝。新版採用發行包的檔名並儲存在原目錄，舊版加上 `.previous` 字尾保留；新版成功啟動後清理更新快取。詳見[釋出約定與驗證範圍](docs/launcher-updates.md)。

## 官方新聞與遊戲更新說明

首頁從 Mojang 的公開啟動器資料來源載入五條 Java 版相關新聞，點選卡片在瀏覽器中開啟對應的 Minecraft 官方文章。

“更新說明”顯示當前配置的 Minecraft 遊戲版本對應的官方文章，與左側“有什麼新內容？”中的啟動器更新公告相互獨立。遊戲文章按版本精確匹配，不會用其他版本的說明代替；沒有對應文章或網路請求失敗時會顯示提示。官方新聞與遊戲更新說明統一使用英文原文，不隨介面語言切換；按鈕與來源說明隨介面語言顯示。新聞或文章暫時不可用不會阻止遊戲啟動。

1.0.2 為啟動器視窗、桌面整合及瀏覽器預覽啟用 MCLA 圖示，修復 Wayland 應用身份匹配，並最佳化內建字型、文章排版和英文介面文案。素材宣告與許可證原文統一以英文展示。

## 平臺和首次執行

| 平臺 | 包格式 | 驗證狀態 |
| --- | --- | --- |
| Linux x86_64，glibc | AppImage | 開發過程中已測試遊戲安裝和啟動 |
| Windows 10／11 x86_64 | 便攜 EXE | 此前版本完成有限的 Wine 檢查；1.0.5 已交叉編譯打包，仍需 Windows 實機驗收 |
| macOS ARM64 | 計劃中 | 暫無構建 |

Windows 需要安裝 **Microsoft Edge WebView2 Runtime**。便攜 EXE 不內建該執行時，也不會自動安裝。Wine／Proton 不作為 Windows 驗收環境；Linux 請使用原生 AppImage。目前不面向 Windows 7／8／8.1 或其他處理器架構。

1. 將執行檔放入可寫資料夾，避免使用受系統保護的安裝目錄。
2. Linux 下賦予 AppImage 執行許可權後執行；系統需支援執行 AppImage。
3. Windows 下如缺少 WebView2 Runtime，請先安裝，再執行 EXE。首次執行會在同目錄建立 `launch.bat` 和桌面快捷方式，不註冊開始選單入口。
4. 選擇配置與 Java。Forge／NeoForge 安裝流程要求有效的 Java；其他安裝流程允許明確選擇“暫不選擇 Java”，啟動遊戲時仍需 Java。
5. 登入並啟動。缺失的受管理遊戲檔案通過安裝／修復流程補齊。

使用 Steam 時，將生成的啟動器入口／指令碼新增為非 Steam 遊戲。Windows 快捷方式指向 `launch.bat`，EXE 改名後腳本仍可識別。移動整個資料夾後，先直接執行一次執行檔以重新整理桌面入口，再重新新增到 Steam。

## 便攜目錄

儲存位置以啟動器執行檔所在目錄為準，不使用啟動命令時的工作目錄：

```text
<啟動器資料夾>/
  <啟動器執行檔>
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
    versions/<配置資料夾>/
      mods/
      resourcepacks/
      shaderpacks/
      saves/
      logs/
```

資料夾按需建立。存檔、模組、資源包、光影包和遊戲設定按配置隔離，libraries／assets 共用。啟動器不會遷移其他啟動器的資料，也不會繼承其公共遊戲目錄中的模組和資源包。本地版本識別不保證相容所有第三方啟動器的目錄格式。

託管 Java 使用相對啟動器目錄的路徑儲存。換電腦後，系統 Java 可能需要重新選擇；重新檢測會檢查當前電腦的執行時。

`accounts.json` 包含認證憑據，使用作業系統檔案許可權保護，並非加密匯出檔案。**不要上傳、附在問題報告中，或在將啟動器複製給他人時分享該檔案。**

## 內容更新規則

- 相容性篩選跟隨當前配置；不支援檢索的遊戲版本可能需要手動安裝內容。
- 通過檔案雜湊匹配 Modrinth 能識別的內容。未識別的檔案仍按檔名管理，但無法自動匹配更新。
- 模組、資源包和光影包統一使用上游新版本的檔名。更新後如有需要，請在遊戲內重新啟用資源包並調整設定；啟動器不修改遊戲的資源包選擇配置。
- 缺少的必需依賴會下載，已識別的現有依賴會保留。更新目標檔案不代表已驗證所有模組之間的完整相容性。
- 刪除檔案為永久刪除，不建立備份。
- 遊戲內使用控制器需要 Controlify 等控制器模組；使用光影包需要相容的光影模組。啟動器的導航功能不會自動讓遊戲具備這些能力。

## 開發

專案使用 Tauri 2／Rust 和 React／TypeScript／Vite。請安裝相容的 Rust 工具鏈、Node.js 22.12+、`package.json` 指定的 pnpm 版本，以及 [Tauri 平臺依賴](https://v2.tauri.app/start/prerequisites/)。

```sh
pnpm install
pnpm dev                       # 瀏覽器預覽：http://127.0.0.1:1420
pnpm test                      # 前端迴歸測試
pnpm build                     # 型別檢查和前端構建
pnpm tauri dev                 # 原生桌面開發
pnpm tauri build --bundles appimage  # Linux AppImage
python3 scripts/check-source-language.py
```

瀏覽器預覽使用獨立的本地資料，不執行原生遊戲安裝或啟動。F8 控制器模擬已移除；控制器導航通過原生輸入介面卡接入。

嚴格遵守架構分層：前端負責顯示、焦點和導航；核心負責通用模型與持久化；領域服務負責遊戲和內容流程；平臺介面卡負責作業系統互動。參見[平臺邊界](docs/platform-boundary.md)、[目錄規則](docs/storage-and-updates.md)、[安裝流程](docs/installation.md)和[表單間距](docs/ui-reference/form-spacing.md)。

## 隱私與本地資料

[隱私與本地資料](PRIVACY.zh-CN.md)

## 維護者與聯絡方式

MC Launcher Advanced 由 [ImperatorVienna](https://github.com/ImperatorVienna) 以個人名義維護和釋出，專案責任主體為維護者本人，不代表公司或組織。

聯絡郵箱：[imperator.vienna@outlook.com](mailto:imperator.vienna@outlook.com)。

## 問題報告與致謝

報告問題時請提供版本分支、系統／架構、遊戲與載入器版本、復現步驟及脫敏記錄。不要附上訪問令牌、重新整理令牌、裝置登入程式碼或帳戶儲存檔案。

依賴和打包素材各有獨立的宣告與許可證，見 `apps/desktop/src-tauri/licenses/`、內建字型的 OFL 宣告及[素材來源](docs/ui-reference/README.md)。本 README 不授予第三方美術素材或 Minecraft 資源的再分發權。Minecraft 及相關名稱屬於各自權利人。

## 許可證

專案自有程式碼採用 [GNU GPL 3.0 許可證](LICENSE)（`GPL-3.0-only`）。
第三方庫、字型與美術素材保留各自許可及權利歸屬，此許可證不對 Minecraft 素材重新授權。

## 介面設計與素材宣告

介面參考 Minecraft 官方啟動器的視覺風格，由本專案獨立實現，並針對鍵鼠與控制器操作進行最佳化。本專案未使用通過逆向工程取得的官方啟動器程式程式碼。

原創程式碼按 GPL-3.0-only 釋出；引用的 Minecraft 標誌、紋理、皮膚、背景及官方新聞內容不在該授權範圍內。MCLA 圖示的原創字母和幾何排布與引用的草方塊紋理分別宣告權利。來源說明不等於使用授權，未確認的素材再分發許可均如實記錄。詳見[素材、設計與第三方權利宣告](ASSET-NOTICES.md)。
