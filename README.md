# bilibili-live-danmaku-cli
This is a command line interface tool for monitoring danmaku and other live messages on Bilibili Live.

## Usage
This application use cookies (SESSDATA) on bilibili.com for authentication. You can find your bilibili SESSDATA in your browser. 
```bash
./bilibili-live-danmaku-cli --sessdata <SESSDATA> --uid <UID> --room-id <ROOM_ID>
```
For better security, the program can also read SESSDATA from standard input.
```bash
cat SESSDATA.txt | ./bilibili-live-danmaku-cli --sessdata - --uid <UID> --room-id <ROOM_ID>
```
You can also join a live room as guest. But notice that some you may not be able to receive full information of messages.
```bash
./bilibili-live-danmaku-cli --room-id <ROOM_ID>
```