# Interface languages

The first run defaults to English (en-US). Settings > Language switches between
English and Simplified Chinese without restarting or remounting the application.
The selection is stored in the webview's launcher.language.v1 preference.
An unavailable or invalid preference falls back to English.

UI keys stay stable and English. src/i18n/en-US.json contains the complete
English UI catalog, including edited copy where the original source key no longer
matches the Chinese meaning. zh-CN.json contains the Chinese UI catalog.
Backend diagnostics originate in English; Chinese diagnostic templates are applied
at the display boundary. User names, paths, game versions, project names, and log
contents are not translated. Modrinth project descriptions retain source content.

Language changes preserve component identity and navigation registration. Icon
labels are getters, so they cannot retain the language used at module initialization.
Language options use their own names to remain recognizable in either language.

The primary launch button uses its original font with localized text: “开始游戏”
in Chinese and “PLAY” in English. Its account/status subtitle is preserved.
English text continues to use the bundled Noto Sans font.
