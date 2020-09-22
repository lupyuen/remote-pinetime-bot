![Telegram Bot to flash and test PineTime firmware remotely](https://lupyuen.github.io/images/remote-pinetime-arch.jpg)

# Remote PineTime: Flash and Test a PineTime Smart Watch remotely, from anywhere in the world

Remote PineTime is a [PineTime Smart Watch](https://wiki.pine64.org/index.php/PineTime) in my bedroom (in Singapore) that's configured to allow anyone in the world to flash and test firmware remotely.

The Remote PineTime Bot (created in Rust) watches a Telegram group for flashing commands and flashes the firmware to PineTime. 

The display on PineTime is streamed live to YouTube, so you can watch your firmware running live on my PineTime.

To flash your own firmware and test the PineTime in my bedroom, join the "Remote PineTime" Telegram group...

https://t.me/remotepinetime

And watch the "Remote PineTime" live stream on YouTube...

https://youtu.be/G3hjdxgiz0k

The live stream URL will change whenever I reboot my Raspberry Pi. Please check this page for the updated live stream URL.

![Remote PineTime Live Stream](https://lupyuen.github.io/images/remote-pinetime-youtube.png)

## Telegram Commands

To flash a firmware binary file `https://.../firmware.bin` to PineTime at address `0x0`...

```
/flash 0x0 https://.../firmware.bin
```

This works for any URL that is not login protected.

Don't pass URLs for artifacts created by GitHub Actions. They require login and the Telegram Bot will be blocked.

Instead, copy the artifacts and upload them under "Releases", which is not protected by login.

Some flavours of PineTime firmware require a Bootloader, like MCUBoot or SoftDevice. Flash the Bootloader to address `0x0` first, then flash the firmware.

MCUBoot-Compatible Firmware should be flashed to address `0x8000`

## Sample Firmware

To flash MCUBoot Bootloader 5.0.4...

```
/flash 0x0 https://github.com/lupyuen/pinetime-rust-mynewt/releases/download/v5.0.4/mynewt.elf.bin
```

Sometimes PineTime will get locked up due to firmware errors. Flashing the above MCUBoot Booloader should fix the locking.

To flash a modified "MIND BLOWN" InfiniTime firmware that never sleeps: flash the above MCUBoot Bootloader, then flash this...

```
/flash 0x8000 https://github.com/AntonMadness/Pinetime/releases/download/v0.1.1/pinetime-mcuboot-app-img.bin
```

This was modified by editing [`src/DisplayApp/DisplayApp.cpp`](https://github.com/AntonMadness/Pinetime/blob/master/src/DisplayApp/DisplayApp.cpp) to remove all calls to `case Messages::GoToSleep:`

To flash Rust on RIOT...

```
/flash 0x0 https://github.com/lupyuen/pinetime-rust-riot/releases/download/v1.0.3/PineTime.bin
```

To flash older MCUBoot Bootloader 4.1.7...

```
/flash 0x0 https://github.com/lupyuen/pinetime-rust-mynewt/releases/download/v4.1.7/mynewt_nosemi.elf.bin
```

To build your own firmware in a web browser (without installing any IDE or toolchain) check out the articles...

1. [Build PineTime Firmware in the Cloud with GitHub Actions](https://lupyuen.github.io/pinetime-rust-mynewt/articles/cloud)

1. [Preview PineTime Watch Faces in your Web Browser with WebAssembly](https://lupyuen.github.io/pinetime-rust-mynewt/articles/simulator)

How the flashing looks in Telegram...

![Flashing Remote PineTime with Telegram](https://lupyuen.github.io/images/remote-pinetime.png)

Got questions on PineTime? Chat with the PineTime Community on Matrix / Discord / Telegram / IRC...

https://wiki.pine64.org/index.php/PineTime#Community

## Start Telegram Bot

To run your own Telegram Bot: Clone this repo and run...

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

The Telegram Bot calls [PineTime Updater](https://github.com/lupyuen/pinetime-updater/blob/master/README.md) and [xPack OpenOCD](https://xpack.github.io/openocd/install/) to flash firmware to PineTime via SWD.

To download xPack OpenOCD (for Mac) or OpenOCD SPI (for Raspberry Pi), look at [`pinetime-updater/run.sh`](https://github.com/lupyuen/pinetime-updater/blob/master/run.sh)

The Telegram Bot is currently running on Raspberry Pi with xPack OpenOCD (instead of OpenOCD SPI). xPack OpenOCD for 32-bit Raspbian was [downloaded from here](https://github.com/xpack-dev-tools/openocd-xpack/releases/download/v0.10.0-14/xpack-openocd-0.10.0-14-linux-arm.tar.gz)

The USB driver for ST-Link was configured on Raspbian like so...

```bash
#  For Linux Only: Install UDEV Rules according to https://xpack.github.io/openocd/install/#udev
sudo cp xpack-openocd/contrib/60-openocd.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
```

## Live Video Stream

To live stream your Raspberry Pi camera to YouTube...

```bash
for (( ; ; ))
do
    raspivid -n -o - -t 0 -vf -hf -fps 30 -b 6000000 | \
        ffmpeg -re -ar 44100 -ac 2 \
        -acodec pcm_s16le -f s16le -ac 2 \
        -i /dev/zero -f h264 -i - -vcodec copy -acodec aac -ab 128k -g 50 -strict experimental \
        -f flv rtmp://a.rtmp.youtube.com/live2/YOUR_YOUTUBE_STREAM_KEY
    sleep 1
done
```

Based on https://www.makeuseof.com/tag/live-stream-youtube-raspberry-pi/

Here is the live streaming setup with (left to right) Raspberry Pi 4, Raspberry Pi v2 Camera Module (8 MP), Two Magnifying Glasses, PineTime with Pogo Pins (sharp tip) and ST-Link v2...

![Raspberry Pi Live Stream](https://lupyuen.github.io/images/remote-pinetime-stream.jpg)

## TODO

1. Write Semihosting Debug Log to a separate Telegram Channel

1. Throttle the number of Semihosting messages that will be logged to the Telegram Channel

1. Allow Semihosting Debug Log and Firmware Flashing to coexist (they both use OpenOCD)
