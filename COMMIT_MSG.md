Address code review feedback for i18n string extraction

Fix spelling inconsistencies in execplan documentation:
- Use American English spelling ("-ize" forms per Oxford convention)
- Add comma before "but" in Decision log entry
- Correct `icon_button` tooltip type documentation

Localize the "Plain Text" status bar string in chrome_panels.rs

Refactor file_status_line() in view.rs to use a FileStatus enum with
precedence logic instead of inline if-else chain. The enum variants are
ordered by priority (history error > save error > open error > saved > opened).
