# bilibili-live-danmaku-cli
一個監視 Bilibili 直播彈幕和其它信息的命令行 (CLI) 工具。\

[English](./README.md) | **中文**

## 使用方式
該程式使用 bilibili.com 上存儲的 SESSDATA (cookie) 進行身份驗證。你可以在你的瀏覽器中找到你的 SESSDATA。
```bash
./bilibili-live-danmaku-cli --sessdata <SESSDATA> --uid <UID> --room-id <ROOM_ID>
```
為增強安全性，該應用程式也可以從標準輸入流 (stdin) 讀取 SESSDATA。您可以從一個文本檔中讀取 SESSDATA 並將其作為該程式的輸入。
```bash
cat SESSDATA.txt | ./bilibili-live-danmaku-cli --sessdata - --uid <UID> --room-id <ROOM_ID>
```
您也可以以遊客身份加入直播間，但是請注意，你可能無法收到完整的消息信息。
```bash
./bilibili-live-danmaku-cli --room-id <ROOM_ID>
```

## 參數
### `--room-id <ROOM_ID>`
指定直播間 ID。該參數是必需的。

### `--sessdata <SESSDATA>`
指定發送請求時使用的 SESSDATA cookie。如果您指定了您的 UID，則該參數是必需的。

### `--uid <UID>`
指定您的 bilibili 賬號 UID。如果該參數未提供，則該會嘗試以遊客身份進入直播間。

### `--gift-combo`
啟用禮物連擊功能。該功能會將一定時間內的多個禮物消息合併為一個累積了禮物數量的禮物消息。這個時間區間是固定的，且僅取決於第一個禮物消息的時間。

只有同一個觀眾發送的同類型禮物會被合併。

### `--combo-interval <INTERVAL_MS>`
指定合併區間的長度（以毫秒計）。如果該參數未提供，則預設為 2000 毫秒。

### `--refresh-combo`
啟用連擊刷新功能。當收到新的禮物消息時，合併區間會延長。

當合併開始時，將會顯示一個提醒消息。