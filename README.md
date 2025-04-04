# bilibili-live-danmaku-cli
一個監視 Bilibili 直播彈幕和其它信息的輕量級命令行 (CLI) 工具。

**中文** | [English](./README.en.md)

## 使用方式
該程式使用 bilibili.com 上存儲的 SESSDATA (cookie) 進行身份驗證。您可以在您的瀏覽器中找到您的 SESSDATA。
```bash
./bilibili-live-danmaku-cli --sessdata <SESSDATA> --uid <UID> --room-id <ROOM_ID>
```

如果您使用 Firfox，您可以指定 Firefox 的 Cookies 資料庫檔案路徑。該工具會從資料庫中讀取 `.bilibili.com` 的 `SESSDATA`。 
```bash
./bilibili-live-danmaku-cli --uid <UID> --room-id <ROOM_ID> --database <PATH_TO_DATABASE_FILE>
```

您可以將參數寫入設定檔中。注意，如果指定從設定檔中讀取設定，該應用程式將會忽略其他命令行參數。
```bash
./bilibili-live-danmaku-cli --config config.json
```

您可以以遊客身份加入直播間，但是請注意，您可能無法收到完整的消息信息。
```bash
./bilibili-live-danmaku-cli --room-id <ROOM_ID>
```

## 設定檔
樣例設定檔：
```json
{
    "roomId": 4793604,
    "uid": 1939036,
    "sessdata": "<YOUR_SESSDATA_COOKIE>",
    "giftCombo": true,
    "comboInterval": 2000,
    "comboRefresh": false,
    "pollInterval": 200,
    "firefoxCookiesDatabase": "<FIREFOX_DATABASE_FILE>"
}
```

## 選項
大多數選項可以由設定檔或命令行參數指定。

### `--config <FILE_PATH>`
該選項只能由命令行參數指定。

指定設定檔的路徑。如果指定了這個參數，該工具將會忽略其它命令行參數。

### `roomId` | `--room-id <ROOM_ID>`
指定直播間 ID。該參數是必需的。

### `sessdata` | `--sessdata <SESSDATA>`
指定發送請求時使用的 SESSDATA cookie。如果您指定了您的 UID，則該參數是必需的。

### `uid`| `--uid <UID>`
指定您的 bilibili 賬號 UID。如果該參數未提供，則該會嘗試以遊客身份進入直播間。

### `giftCombo` | `--gift-combo`
啟用禮物連擊功能。該功能會將一定時間內的多個禮物消息合併為一個累積了禮物數量的禮物消息。這個時間區間預設為固定的，僅取決於第一個禮物消息的時間。

只有同一個觀眾發送的同類型禮物會被合併。

### `comboInterval` | `--combo-interval <INTERVAL_MS>`
指定合併區間的長度（以毫秒計）。如果該參數未提供，則預設為 2000 毫秒。

### `pollInterval` | `--poll-interval <INTERVAL_MS>`
指定從直播 WebSocket 流中拉取消息的時間間隔（以毫秒計）。如果該參數未提供，則預設為 200 毫秒。

由於該工具並未使用多執行緒技術或異步框架，拉取消息的時間間隔也是整個程式的刻間隔。在每一刻，該程式都會檢查心跳包和禮物消息合併。推薦設定一個較小的時間間隔。

### `firefoxCookiesDatabase` | `--database <DATABASE_PATH>`
指定 Firefox 的 Cookies 資料庫檔案路徑。如果指定了 `sessdata`，該選項會將會被忽略，該工具將不會從 Cookies 資料庫中讀取 SESSDATA。