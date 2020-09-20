# remote-pinetime-bot
Telegram Bot to flash and test PineTime firmware remotely

This bot watches a Telegram group for flashing commands and flashes the firmware to PineTime. The display on PineTime is streamed live to YouTube, so you can use the bot to test PineTime firmware remotely.

Join the "Remote PineTime" Telegram group...

https://t.me/remotepinetime

Watch the live stream...

https://youtu.be/1V_eLd3G_AA

## Telegram Commands

To flash `https://.../firmware.bin` to PineTime at address `0x0`...

```
/flash 0x0 https://.../firmware.bin
```

This works for any URL that is not login protected.

Don't pass URLs for artifacts created by GitHub Actions. They require login and the Telegram Bot will be blocked.

Instead, copy the artifacts and upload them under "Releases", which is not protected by login.

Some flavours of PineTime firmware require a Bootloader, like MCUBoot or SoftDevice. Flash the Bootloader to address `0x0` first, then flash the firmware.

## Sample Firmware

To flash MCUBoot Bootloader 5.0.4...

```
/flash 0x0 https://github.com/lupyuen/pinetime-rust-mynewt/releases/download/v5.0.4/mynewt.elf.bin
```

Sometimes PineTime will get locked up due to firmware errors. Flashing the above MCUBoot Booloader should fix the locking.

To flash older MCUBoot Bootloader 4.1.7...

```
/flash 0x0 https://github.com/lupyuen/pinetime-rust-mynewt/releases/download/v4.1.7/mynewt_nosemi.elf.bin
```

To flash Rust on RIOT...

```
/flash 0x0 https://github.com/lupyuen/pinetime-rust-riot/releases/download/v1.0.3/PineTime.bin
```

How the flashing looks in Telegram...

![Flashing Remote PineTime with Telegram](https://lupyuen.github.io/images/remote-pinetime.png)

## Start Telegram Bot

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

## Live Video Stream

To live stream Raspberry Pi camera to YouTube...

```bash
raspivid -o - -t 0 -vf -hf -fps 30 -b 6000000 | \
    ffmpeg -re -ar 44100 -ac 2 \
    -acodec pcm_s16le -f s16le -ac 2 \
    -i /dev/zero -f h264 -i - -vcodec copy -acodec aac -ab 128k -g 50 -strict experimental \
    -f flv rtmp://a.rtmp.youtube.com/live2/YOUR_YOUTUBE_STREAM_KEY
```

Based on https://www.makeuseof.com/tag/live-stream-youtube-raspberry-pi/
