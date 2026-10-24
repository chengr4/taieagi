# Taieagi

輕量、高效且以 26 鍵標準英文字母為基礎的臺灣本土語言快彈詞盤 Chrome 擴充功能。平時不干擾系統原生注音與英文輸入，隨時隨地於網頁輸入框喚起微型輸入器快速選字。

A lightweight, efficient Chrome extension providing a 26-key Quick Palette for Taiwanese indigenous and native languages without disturbing standard OS input methods.  

## 專案定位與願景 (Vision & Architecture)

**Taieagi** 致力於為臺灣所有非華語之本土語言（臺灣閩南語、臺灣客語、臺灣原住民族各語）打造統一、現代且零干擾的 26 鍵輸入體驗。核心場景鎖定於社交平台（Threads、Facebook、Dcard、X 等）留言或即時通訊中快速鍵入正字與標調拼音。

* 平常不攔截任何全域鍵盤事件；使用者按下非列印快捷鍵（macOS 預設 `Option + T`、其他平台預設 `Alt + T`）或點擊留言框右邊徽章時，隨即於游標旁彈出輕量選字視窗，鍵入拼音選字完成後自動寫入網頁並閉合。
* 目前僅實作臺灣閩南語。
* **希望擴充支援臺灣其他本土語言**，並支援前端即時切換偏好語言。

> 還希望可以支援寫入台灣閩南語歌詞，歡迎大家貢獻 🥹。

**Taieagi** is dedicated to providing a unified, modern, and zero-interference 26-key typing experience for all Taiwanese indigenous and native languages (Taiwanese Hokkien, Hakka, and Indigenous languages). The primary focus is enabling rapid insertion of native orthography and tonal romanization in social media comments (Threads, Facebook, Dcard, X) and instant messaging.

* Never intercepts global keystrokes during standard typing. Pressing the non-printing shortcut (`Option + T` on macOS, `Alt + T` on other platforms) or clicking the right-side badge triggers a lightweight Quick Palette next to the cursor, which commits text and dismisses itself upon selection.
* Currently implements Taiwanese Hokkien.
* **Aims to expand support to other Taiwanese native languages** with instant language profile switching in the UI.

> We also hope to support typing Taiwanese song lyrics—community contributions are warmly welcomed 🥹!

## 本機開發與測試 (Development & Testing)

### 前置需求 (Prerequisites)
* [Rust](https://rustup.rs/) (with `wasm32-unknown-unknown` target)
* [Bun](https://bun.sh/) (JavaScript / TypeScript runtime and bundler)
* [wasm-bindgen-cli](https://crates.io/crates/wasm-bindgen-cli)

### 建置與測試步驟 (Build & Test)

```bash
# 1. Add WebAssembly target
rustup target add wasm32-unknown-unknown

# 2. Compile dictionary binary index (compiles assets/*.csv into assets/*.compact)
cargo run --bin prepare_dict

# 3. Build Chrome extension (compiles WASM core and bundles extension to extension/)
bun run build

# 4. Run pure unit tests and WASM runtime assertions
cargo test --quiet && bun test
```

## 資料來源與鳴謝 (Data Sources & Credits)

本專案之台語詞庫與標準音讀資料，深度仰賴並感謝以下開放資料與社群先行者之無私奉獻：

* **教育部《臺灣閩南語常用詞辭典》**
  * 由中華民國教育部編纂整理，辭典成果依「[創用 CC 姓名標示-禁止改作 3.0 臺灣](https://creativecommons.org/licenses/by-nd/3.0/tw/)」授權條款釋出。
  * 辭典線上檢索系統：[https://sutian.moe.edu.tw/](https://sutian.moe.edu.tw/)
* **g0v 零時政府「萌典」專案團隊**
  * 感謝 g0v 社群與專案維護者篳路藍縷，將教育部原始辭典資料有系統地轉換、清理並開源為機器易讀格式：[`g0v/moedict-data-twblg`](https://github.com/g0v/moedict-data-twblg)。
  * 本專案主要詞庫音讀與詞目資料集來源為該專案之 [`uni/詞目總檔.csv`](https://github.com/g0v/moedict-data-twblg/blob/master/uni/%E8%A9%9E%E7%9B%AE%E7%B8%BD%E6%AA%94.csv)（收錄逾 27,200 筆標準詞目與教育部臺羅調符）。

**在此向所有為台灣語言文化數位化與本土語言教育付出心力的專家、學者與開源貢獻者致上最深敬意。**

The lexicon and standard pronunciations in this project rely on and acknowledge the generous contributions of the following open data initiatives:

* **MOE Dictionary of Frequently-Used Taiwan Minnan**
  * Compiled by the Ministry of Education, Taiwan, licensed under [CC BY-ND 3.0 TW](https://creativecommons.org/licenses/by-nd/3.0/tw/).
  * Online Dictionary: [https://sutian.moe.edu.tw/](https://sutian.moe.edu.tw/)
* **g0v "Moedict" Project Team**
  * Deep gratitude to the g0v open-source community for systematically converting and maintaining the MOE dictionary in machine-readable formats: [`g0v/moedict-data-twblg`](https://github.com/g0v/moedict-data-twblg).
  * Our core dictionary dataset originates from [`uni/詞目總檔.csv`](https://github.com/g0v/moedict-data-twblg/blob/master/uni/%E8%A9%9E%E7%9B%AE%E7%B8%BD%E6%AA%94.csv) (over 27,200 headwords with Tâi-lô diacritics).

**We extend our highest respect to all researchers, educators, and open-source contributors dedicated to preserving and digitalizing Taiwan's native languages.**

## 社群資料集貢獻 (Contributing Lexicon & Hints)

若您在使用過程中發現未收錄詞彙、經典台語歌詞或常見誤拼，歡迎開立 Issue 回報或直接發起 Pull Request：

* **日常補充詞庫**（[`assets/supplement.csv`](assets/supplement.csv)）：
  * 格式：`詞目,音讀`（範例：`這个,tsit ê`）。
  * 適用於日常高頻但教育部未獨立列目之詞彙；提交前請嚴格確認發音與臺羅標調。
* **經典台語歌詞**（[`assets/lyrics.csv`](assets/lyrics.csv)）：
  * 格式：`詞目,音讀`（範例：`我合你上好就到遮,guá kap lí siōng-hó tiō kàu tsia`）。
  * 適用於知名台語歌曲之標準用字歌詞，便於社群留言時快速輸入整句。
* **口語誤拼正音提示**（[`assets/hints.csv`](assets/hints.csv) 與 [`assets/lyrics_hints.csv`](assets/lyrics_hints.csv)）：
  * 格式：`詞目,正確臺羅,常見誤拼`（範例：`這个,tsit-ê,jire/jile`）。
  * 常見誤拼請直接依照使用者自己的直覺拼字感覺（多組誤拼以斜線 `/` 分隔）；當鍵入該拼法時，面板將即時提供正音引導。

> 本機可執行 `cargo run --bin prepare_dict` 重新編譯二進位索引（`.compact`）進行測試；提交 PR 後 CI 自動化管線亦會自動編譯與驗證產物，無須手動提交二進位檔案。

If you encounter missing vocabulary, classic lyrics, or common phonetic misspellings, feel free to open an Issue or submit a Pull Request:

* **Everyday Supplement Lexicon** ([`assets/supplement.csv`](assets/supplement.csv)):
  * Format: `headword,romanization` (e.g. `這个,tsit ê`).
  * For frequent colloquial words not individually cataloged by the MOE dictionary; please verify standard Tâi-lô romanization and tone diacritics before submitting.
* **Classic Taiwanese Lyrics** ([`assets/lyrics.csv`](assets/lyrics.csv)):
  * Format: `headword,romanization` (e.g. `我合你上好就到遮,guá kap lí siōng-hó tiō kàu tsia` from Mayday's *Peter & Mary*).
  * Standard orthography lyrics of well-known Taiwanese songs for quick lyric phrase typing in social threads.
* **Colloquial Typo & Orthography Hints** ([`assets/hints.csv`](assets/hints.csv) & [`assets/lyrics_hints.csv`](assets/lyrics_hints.csv)):
  * Format: `headword,correct_tailo,common_typos` (e.g. `這个,tsit-ê,jire/jile`).
  * Typos should reflect intuitive, colloquial phonetic guesses (separated by `/`); typing these intuitively displays standard orthography recommendations in the palette.

> You can run `cargo run --bin prepare_dict` locally to recompile binary indexes (`.compact`) for testing. When PRs are merged, the CI automation pipeline automatically recompiles and validates binary artifacts—there is no need to manually commit binary files.

## 授權條款 (License)

* **程式碼 (Code)**：本專案所有原創原始碼遵循 [MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE) 雙授權（Dual-licensed under MIT OR Apache-2.0）。
* **詞庫資料 (Data)**：教育部臺灣閩南語常用詞辭典本文遵循 [創用 CC 姓名標示-禁止改作 3.0 臺灣 (CC BY-ND 3.0 TW)](https://creativecommons.org/licenses/by-nd/3.0/tw/) 授權。
