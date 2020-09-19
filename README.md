# remote-pinetime-bot
Telegram Bot to flash and test PineTime firmware remotely

To flash MCUBoot Bootloader 5.0.4 via Telegram...

```
/flash 0x0 https://github.com/lupyuen/pinetime-rust-mynewt/releases/download/v5.0.4/mynewt.elf.bin
```

To flash older MCUBoot Bootloader 4.1.7 via Telegram...

```
/flash 0x0 https://github.com/lupyuen/pinetime-rust-mynewt/releases/download/v4.1.7/mynewt_nosemi.elf.bin
```

To flash Rust on RIOT...

```
/flash 0x0 https://github.com/lupyuen/pinetime-rust-riot/releases/download/v1.0.3/PineTime.bin
```

To run Telegram Bot...

```bash
export TELEGRAM_BOT_TOKEN=???
cd ~/remote-pinetime-bot
for (( ; ; ))
do
    git pull
    cargo run
    echo "---------ERROR--------"
    sleep 30
done
```

To live stream Raspberry Pi camera to YouTube...

```bash
raspivid -o - -t 0 -vf -hf -fps 30 -b 6000000 | \
    ffmpeg -re -ar 44100 -ac 2 \
    -acodec pcm_s16le -f s16le -ac 2 \
    -i /dev/zero -f h264 -i - -vcodec copy -acodec aac -ab 128k -g 50 -strict experimental \
    -f flv rtmp://a.rtmp.youtube.com/live2/YOUR_YOUTUBE_STREAM_KEY
```

Based on https://www.makeuseof.com/tag/live-stream-youtube-raspberry-pi/
