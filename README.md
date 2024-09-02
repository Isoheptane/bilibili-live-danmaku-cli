# bilibili-live-danmaku-cli
This is a light-weight command line interface tool for monitoring danmaku and other live messages on Bilibili Live.

**English** | [中文](./README-zh.md)

## Usage
This tool use cookies (SESSDATA) on bilibili.com for authentication. You can find your bilibili SESSDATA in your browser. 
```bash
./bilibili-live-danmaku-cli --sessdata <SESSDATA> --uid <UID> --room-id <ROOM_ID>
```

For better security, this tool can also read SESSDATA from standard input. You can read SESSDATA from a text file and use it as the input of this tool.
```bash
cat SESSDATA.txt | ./bilibili-live-danmaku-cli --sessdata - --uid <UID> --room-id <ROOM_ID>
```

Your can specify your arguments in a config file. Notice this tool will ignore other arguments in shell command if a config file is specified.
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
    "comboRefresh": false,
    "pollInterval": 200
}
```

## Arguments
### `--room-id <ROOM_ID>`
Specifies the live room ID. This argument is required.

### `--sessdata <SESSDATA>`
Specifies the SESSDATA cookie used when sending requests. It is required if you specified your UID.

### `--uid <UID>`
Specifies your bilibili user UID. If this argument is not provided, it will try to connect to the live room as guest.

### `--gift-combo`
Enable gift combo feature. This feature will combine multiple gift message within a time interval into one message with accumulated gift count. This time interval will not refresh by default, which means the combining interval is only determined by the first gift message.

Only gift messages from same audience with same gift name are combined.

### `--combo-interval <INTERVAL_MS>`
Specifies the gift message combining interval in milliseconds. If this argument is not specified, it will default to 2000 ms.

### `--poll-interval <INTERVAL_MS>`
Specifies the message poll interval of live WebSocket stream in milliseconds. If this argument is not specified, it will default to 200 ms.

Since this tool do not utilize multithreading or async frameworks, the poll interval is also the tick interval. At every tick, this tool will check heartbeat and gift message combining. It's recommended to set a short poll interval.

### `--config <FILE_PATH>`
Specifies the configuration file path. If this argument is specified, this tool will ignore other shell arguments. 