# Privacy Policy for Taieagi (隱私權政策)

**Last Updated / 最後更新日期**：October 23, 2026

**Taieagi 臺灣本土語言快彈詞盤**（以下簡稱「本擴充功能」或 "Taieagi"）致力於保護使用者的隱私。本隱私權政策旨在說明本擴充功能如何處理使用者資訊。

Taieagi ("the Extension") is committed to protecting your privacy. This Privacy Policy explains our practices regarding user information.

---

## 1. Zero Data Collection / 零資料蒐集政策

### English
* **No Personal Data Collected**: Taieagi does not collect, record, transmit, store, or sell any personal data, sensitive personal information, or user identity.
* **No Keystroke Logging**: All typing, composition, tone conversion, and candidate lookups are executed 100% locally in-memory using an embedded WebAssembly (Rust) module. No keystrokes are recorded to disk or transmitted across the network.
* **No Browsing History or Page Content Harvesting**: Taieagi never inspects, harvests, or analyzes the content of web pages you visit, nor does it track your browsing activity or URLs.

### 中文
* **不蒐集任何個人資訊**：Taieagi 絕不蒐集、記錄、傳輸、儲存或販售任何個人身分資料、敏感資訊或使用者識別碼。
* **絕不記錄按鍵紀錄（No Keystroke Logging）**：所有的拼音輸入、組字狀態機、聲調轉換與詞庫檢索，全數透過內建之 WebAssembly（Rust）模組在使用者本機記憶體中即時運算完成。絕不將任何按鍵內容存入磁碟或透過網路發送。
* **不擷取網頁內容與瀏覽紀錄**：Taieagi 絕不讀取、分析或蒐集您瀏覽的網頁內容，亦不追蹤您的瀏覽紀錄或造訪網址。

---

## 2. Use of Storage Permissions / 儲存權限使用說明

### English
* **User Preferences Only**: Taieagi utilizes Chrome's `chrome.storage.sync` API solely to store and synchronize non-identifiable user preferences (specifically, the chosen `outputMode`: Pure Hanzi, Hanzi with tone marks, or Hanzi with tone numbers) across your browser sessions.
* **No Sensitive Data in Storage**: No personal or identifiable data is ever written to storage.

### 中文
* **僅儲存非識別性偏好設定**：Taieagi 僅使用瀏覽器內建的 `chrome.storage.sync` API 儲存並同步使用者的非識別性介面偏好（具體僅為選字面板的輸出模式 `outputMode`：純漢字、漢字加調符、或漢字加數字調）。
* **無敏感資料**：儲存區內絕不寫入任何個人或敏感資料。

---

## 3. Third-Party Services & Analytics / 第三方服務與分析

### English
* Taieagi contains zero third-party trackers, zero advertising SDKs, and zero telemetry analytics.
* The extension operates completely offline and makes zero external API requests.

### 中文
* Taieagi 絕不包含任何第三方追蹤器、廣告 SDK 或外部分析遙測工具。
* 擴充功能完全具備本機離線運作能力，絕不主動發起任何外部網路 API 請求。

---

## 4. Open Source & Transparency / 開源與透明度

### English
Taieagi is an open-source project dual-licensed under the MIT and Apache-2.0 licenses. Our complete source code is public and fully auditable by anyone at:
https://github.com/chengr4/taieagi

### 中文
Taieagi 為標準開源專案，遵循 MIT 與 Apache-2.0 雙授權條款。所有原始碼公開透明，任何使用者與審查員皆可在 GitHub 上檢驗原始碼：
https://github.com/chengr4/taieagi

---

## 5. Contact Us / 聯絡方式

If you have questions about this Privacy Policy, please contact:  
若對本隱私權政策有任何疑問，歡迎聯絡維護者：

* **Author / 維護者**：R4 Cheng
* **Email**：karaburi2023@gmail.com
* **GitHub Issues**：https://github.com/chengr4/taieagi/issues
