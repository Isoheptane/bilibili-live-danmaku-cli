# bilibili-live-danmaku-cli
This is a light-weight command line interface tool for monitoring danmaku and other live messages on Bilibili Live.

[中文](./README.md) | **English**

## Usage
This tool use cookies (SESSDATA) on bilibili.com for authentication. You can find your bilibili SESSDATA in your browser. 
```bash
./bilibili-live-danmaku-cli --sessdata <SESSDATA> --uid <UID> --room-id <ROOM_ID>
```

If you are using Firefox, you can specify Firefox cookies database file. This tool will read `SESSDATA` in website `.bilibili.com` from the cookies database file.
```bash
./bilibili-live-danmaku-cli --uid <UID> --room-id <ROOM_ID> --database <PATH_TO_DATABASE_FILE>
```

You can specify your arguments in a config file. Notice this tool will ignore other arguments in command line arguments if a config file is specified.
```bash
./bilibili-live-danmaku-cli --config config.json
```

You can join a live room as guest. But notice that some you may not be able to receive full information of messages.
```bash
./bilibili-live-danmaku-cli --room-id <ROOM_ID>
```

## Configuration File
Example config file:
```json
{
    "roomId": 4793604,
    "uid": 1939036,
    "sessdata": "<YOUR_SESSDATA_COOKIE>",
    "giftCombo": true,
    "comboInterval": 2000,
    "repeatSuperchat": true,
    "repeatSuperchatInterval": 30,
    "pollInterval": 200
}
```

## Options
ptions are available in both config file and command line arguments.

### `--config <FILE_PATH>`
Thisoption is only avaliable in command line argument.

Specifies the config file path. If this argument is specified, other command line arguments will be ignored.

### `roomId` | `--room-id <ROOM_ID>`
Specifies the live room ID. This argument is required.

### `sessdata` | `--sessdata <SESSDATA>`
Specifies the SESSDATA cookie used when sending requests. It is required if you specified your UID.

### `uid` | `--uid <UID>`
Specifies your bilibili user UID. If this argument is not provided, it will try to connect to the live room as guest.

### `giftCombo` | `--gift-combo`
Enable gift combo feature. This feature will combine multiple gift message within a time interval into one message with accumulated gift count. This time interval will not refresh by default, which means the combining interval is only determined by the first gift message.

Only gift messages from same audience with same gift name are combined.

### `comboInterval` | `--combo-interval <INTERVAL_MS>`
Specifies the gift message combining interval in milliseconds. If this argument is not specified, it will default to 2000 ms.

### `repeatSuperchat` | `--repeat-sc`
(Experimental feature, not tested)

Enable repeat superchat feature. This feature will repeat superchats at a predetermined interval. This feature will also repeat superchats when they are expired.

### `repeatSuperchatInterval` | `--repeat-sc-interval <INTERVAL_MS>`
Specifies the interval between two superchat repeats. If this argument is not specified, it will default to 30s.

### `pollInterval` | `--poll-interval <INTERVAL_MS>`
Specifies the message poll interval of live WebSocket stream in milliseconds. If this argument is not specified, it will default to 200 ms.

Since this tool do not utilize multithreading or async frameworks, the poll interval is also the tick interval. At every tick, this tool will check heartbeat and gift message combining. It's recommended to set a short poll interval.

### `firefoxCookiesDatabase` | `--database <DATABASE_PATH>`
Specifies the Firefox cookies database path. If `sessdata` is specified, this option will be ignored, the tool won't read sessdata from cookies database.