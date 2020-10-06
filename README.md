![Telegram Bot to flash and test PineTime firmware remotely](https://lupyuen.github.io/images/remote-pinetime-arch.jpg)

# Remote PineTime: Flash and Test a PineTime Smart Watch remotely, from anywhere in the world

[Watch the Demo Video on YouTube](https://youtu.be/mMLWgzJSAGI)

Remote PineTime is a [PineTime Smart Watch](https://wiki.pine64.org/index.php/PineTime) in my bedroom (in Singapore) that's configured to allow anyone in the world to flash and test firmware remotely.

The Remote PineTime Bot (created in Rust) watches a Telegram group for flashing commands and flashes the firmware to PineTime. 

The display on PineTime is streamed live to YouTube, so you can watch your firmware running live on my PineTime.

To flash your own firmware and test the PineTime in my bedroom, join the "Remote PineTime" Telegram group...

https://t.me/remotepinetime

View the flashing log (from OpenOCD) and debug message log (from Arm Semihosting) in the "Remote PineTime Log" Telegram Channel here...

https://t.me/remotepinetimelog

And watch the "Remote PineTime" live stream on YouTube...

https://youtu.be/WfuW5-TPjZM

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

To flash [Breakout on PineTime](https://github.com/TT-392/TT-time)...

```
/flash 0x0 http://tt-392.space/breakout.hex
```

[Demo Video of Breakout on PineTime](https://www.youtube.com/watch?v=5rt6C1FeglM)

To flash a modified "MIND BLOWN" [InfiniTime Firmware](https://github.com/JF002/Pinetime) that never sleeps...

```
/flash 0x0 https://github.com/lupyuen/pinetime-rust-mynewt/releases/download/v5.0.4/mynewt.elf.bin

/flash 0x8000 https://github.com/AntonMadness/Pinetime/releases/download/v0.1.1/pinetime-mcuboot-app-img.bin
```

This was modified by editing [`src/DisplayApp/DisplayApp.cpp`](https://github.com/AntonMadness/Pinetime/blob/master/src/DisplayApp/DisplayApp.cpp) to remove all calls to `case Messages::GoToSleep:`

To flash [Rust on Mynewt Firmware](https://github.com/lupyuen/pinetime-rust-mynewt) that emits Semihosting Debug Messages...

```
/flash 0x0 https://github.com/lupyuen/pinetime-rust-mynewt/releases/download/v5.0.4/mynewt.elf.bin

/flash 0x8000 https://github.com/lupyuen/pinetime-rust-mynewt/releases/download/v5.0.7/my_sensor_app.img
```

To flash [Rust on RIOT](https://github.com/lupyuen/pinetime-rust-riot)...

```
/flash 0x0 https://github.com/lupyuen/pinetime-rust-riot/releases/download/v1.0.3/PineTime.bin
```

To flash [MCUBoot Bootloader](https://lupyuen.github.io/pinetime-rust-mynewt/articles/mcuboot) 5.0.4...

```
/flash 0x0 https://github.com/lupyuen/pinetime-rust-mynewt/releases/download/v5.0.4/mynewt.elf.bin
```

Sometimes PineTime will get locked up due to firmware errors. Flashing the above MCUBoot Booloader should fix the locking.

To build your own firmware in a web browser (without installing any IDE or toolchain) check out the articles...

1. [Build PineTime Firmware in the Cloud with GitHub Actions](https://lupyuen.github.io/pinetime-rust-mynewt/articles/cloud)

1. [Preview PineTime Watch Faces in your Web Browser with WebAssembly](https://lupyuen.github.io/pinetime-rust-mynewt/articles/simulator)

How the flashing looks in Telegram...

[Watch the Demo Video on YouTube](https://youtu.be/mMLWgzJSAGI)

![Flashing Remote PineTime with Telegram](https://lupyuen.github.io/images/remote-pinetime.png)

Got questions on PineTime? Chat with the PineTime Community on Matrix / Discord / Telegram / IRC...

https://wiki.pine64.org/index.php/PineTime#Community

[Check out my PineTime articles](https://lupyuen.github.io)

## Why was Remote PineTime created?

Because it's difficult and expensive to ship real hardware around the world during the pandemic... And remote firmware testing could be the solution. Check out my video presentation...

[RIOT Summit 2020 - Safer & Simpler Embedded Programs with Rust on RIOT](https://youtu.be/LvfCSnOM1Hs)

## What is Arm Semihosting?

[Arm Semihosting](https://www.keil.com/support/man/docs/armcc/armcc_pge1358787046598.htm) enables our firmware to emit debugging messages by invoking the Arm Cortex-M Instruction `bkpt`.

Check out this implementation of Arm Semihosting from [`pinetime-rust-mynewt`](https://github.com/lupyuen/pinetime-rust-mynewt/blob/master/libs/semihosting_console/src/semihosting_console.c#L52-L73)...

```c
/// Send an ARM Semihosting command to the debugger, e.g. to print a message.
/// To see the message you need to run opencd:
/// openocd -f interface/stlink-v2.cfg -f target/stm32f1x.cfg -f scripts/debug.ocd
static int __semihost(int command, void* message) {
    //  Warning: This code will trigger a breakpoint and hang unless a debugger is connected.
    //  That's how ARM Semihosting sends a command to the debugger to print a message.
    //  This code MUST be disabled on production devices.
    __asm( 
        "mov r0, %[cmd] \n"
        "mov r1, %[msg] \n" 
        "bkpt #0xAB \n"
	:  //  Output operand list: (nothing)
	:  //  Input operand list:
        [cmd] "r" (command), 
        [msg] "r" (message)
	:  //  Clobbered register list:
        "r0", "r1", "memory"
	);
	return 0;
}
```

We call `__semihost()` like so: [`semihosting_console.c`](https://github.com/lupyuen/pinetime-rust-mynewt/blob/master/libs/semihosting_console/src/semihosting_console.c#L77-L113)

```c
/// ARM Semihosting Command
#define SYS_WRITE  (0x5)

/// Write "length" number of bytes from "buffer" to the debugger's file handle fh.
/// We set fh=2 to write to the debugger's stderr output.
static int semihost_write(uint32_t fh, const unsigned char *buffer, unsigned int length) {
    //  If debugger is not connected, quit.
    if (!debugger_connected()) { return 0; }
    if (length == 0) { return 0; }
    uint32_t args[3];
    args[0] = (uint32_t) fh;
    args[1] = (uint32_t) buffer;
    args[2] = (uint32_t) length;
    return __semihost(SYS_WRITE, args);
}

/// Return non-zero if debugger is connected. From repos/apache-mynewt-core/hw/mcu/ambiq/apollo2/src/hal_system.c
static int debugger_connected(void) {
    return CoreDebug->DHCSR & CoreDebug_DHCSR_C_DEBUGEN_Msk;
}
```

When we call...

```c
/// Write "hello\n" (6 characters) to the debugger's stderr output.
#define SEMIHOST_HANDLE 2
semihost_write(SEMIHOST_HANDLE, (const unsigned char *) "hello\n", 6);
```

We'll see the the message `hello` appear in OpenOCD and the Remote PineTime Log. (Messages must end with a newline or they won't appear)

Arm Semihosting needs to be enabled in OpenOCD. Here's how Remote PineTime enables Arm Semihosting: [`flash-log.ocd`](https://github.com/lupyuen/pinetime-updater/blob/master/scripts/flash-log.ocd)

```
# Arm Semihosting is used to show debug console output and may only be enabled after the init event.
# We wait for the event and enable Arm Semihosting.
$_TARGETNAME configure -event reset-init {
    echo "Enabled ARM Semihosting to show debug output"
    arm semihosting enable
}
```

Arm Semihosting can be slow... The entire microcontroller freezes while the debug message is transmitted character by character to OpenOCD via the SWD port.

We recommend using a static array to buffer the outgoing messages in memory.

In the [`pinetime-rust-mynewt`](https://github.com/lupyuen/pinetime-rust-mynewt/blob/master/libs/semihosting_console/src/semihosting_console.c#L137-L155) implementation of Arm Semihosting, we use [Mynewt Mbufs](https://mynewt.apache.org/latest/os/core_os/mbuf/mbuf.html) to buffer the messages efficiently.

Don't use Arm Semihosting when Bluetooth LE processing is ongoing... Messages will be dropped and Bluetooth LE clients will automatically disconnect.

Arm Semihosting should be disabled in production firmware. Also, Arm Semihosting may hang when a JLink debugger is connected. For `pinetime-rust-mynewt` we disable Arm Semihosting with the GCC flag `-DDISABLE_SEMIHOSTING` in [`targets/nrf52_boot/pkg.yml`](https://github.com/lupyuen/pinetime-rust-mynewt/blob/master/targets/nrf52_boot/pkg.yml) (for the MCUBoot Bootloader) and in [`targets/nrf52_my_sensor/pkg.yml`](https://github.com/lupyuen/pinetime-rust-mynewt/blob/master/targets/nrf52_my_sensor/pkg.yml) (for the Application Firmware).

## Start Telegram Bot

To create your own Telegram Bot...

1. Chat with BotFather, create a bot named `PineTime Bot`

1. Enter `/mybots`, select `PineTime Bot`

1. Select `Edit Commands`, enter `flash - flash 0x0 https://.../firmware.bin`

To run your own Telegram Bot: Clone this repo and run this in a shell script...

```bash
#  Set your Telegram Bot Token
export TELEGRAM_BOT_TOKEN=???
#  This is needed to fix the h2 / indexmap build error "ids: IndexMap<StreamId, SlabIndex> expected 3 type arguments"
export CARGO_FEATURE_STD=1
#  Show Rust stack trace
export RUST_BACKTRACE=1

cd ~/remote-pinetime-bot
for (( ; ; ))
do
    git pull
    pkill openocd
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

To live stream your Raspberry Pi camera to YouTube: Run this in a shell script...

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

Here is the live streaming setup with (left to right) Raspberry Pi 4, Raspberry Pi v2 Camera Module (8 MP), Two Magnifying Glasses, PineTime with [Pogo Pins (sharp tip)](https://youtu.be/K5GgUlv-1SI) and [ST-Link v2](https://www.aliexpress.com/wholesale?catId=0&initiative_id=SB_20180924134644&SearchText=st-link+v2&switch_new_app=y)...

![Raspberry Pi Live Stream](https://lupyuen.github.io/images/remote-pinetime-stream.jpg)

Cover with a Papier-Mâché Enclosure to block the reflection on the Magnifying Glasses (like a telescope)...

![Remote PineTime Enclosure](https://lupyuen.github.io/images/remote-pinetime-enclosure.jpg)

How to make the Papier-Mâché Enclosure...

1. Position the Raspberry Pi, Camera Module, Two Magnifying Glasses and PineTime

1. Build a Scaffold by resting an Envelope on the Raspberry Pi, Camera Module and Magnifying Glasses

1. Complete the Scaffold by laying a folded piece of A4-size paper

1. Paste strips of Paper Towel on the Scaffold, be sure to cover Raspbery Pi. I created the paste by mixing half-cup of Flour with half-cup of Water.

1. Let the Papier-Mâché dry overnight to form the shape of the enclosure. Trim the Papier-Mâché with scissors. Microwave the Papier-Mâché to dry it.

1. Paste a second layer of Paper Towel strips, in an orderly fashion.

1. Microwave the Papier-Mâché for one minute, let it cool. Use overturned bowls to support the structure. Repeat 3 times until the Papier-Mâché is dry.

1. Trim the Papier-Mâché Enclosure with scissors.

![Making the Remote PineTime Enclosure](https://lupyuen.github.io/images/remote-pinetime-enclosure2.jpg)

## Security Issues

Are there any security issues exposing a Telegram Bot to the world for flashing and testing?

We mitigate the security risks as much as possible...

1. Our Telegram Bot is built with Rust, a secure systems programming language

1. No remote access to the host is allowed. The Telegram Bot polls for `/flash` commands and executes them.

1. Temporary files are automatically deleted after use with the [`tempfile`](https://crates.io/crates/tempfile) library. So we reduce the exposure of files with malware.

But there is one concern... Our PineTime may be flashed with malware that attacks other Bluetooth devices nearby.

For the sake of IoT Education... I'll allow it! :-) 

I'm fully aware of the risks when I operate this free service. And if you choose to operate your own Remote PineTime, you should be aware of the risks too.

## Completed Features

1. Write Semihosting Debug Log to a separate Telegram Channel

1. Throttle the number of Semihosting messages that will be logged to the Telegram Channel (2 messages per second)

1. Allow Semihosting Debug Log and Firmware Flashing to coexist (they both use OpenOCD)
