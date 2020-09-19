//  Telegram Bot to flash and test PineTime firmware remotely
//  Chat with BotFather, create bot "PineTime Bot"
//  Enter "/mybots", select "PineTime Bot"
//  Select "Edit Commands", enter "flash - flash 0x0 https://.../firmware.bin"
use std::{env, string::String};
use futures::StreamExt;
use telegram_bot::*;
use error_chain::error_chain;

//  Define the error types
error_chain!{
    foreign_links {
        Io(std::io::Error);
        Reqwest(reqwest::Error);
        Telegram(telegram_bot::Error);
    }
}

/// Listen for commands and handle them
#[tokio::main]
async fn main() -> Result<()> {
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let api = Api::new(token);

    // Fetch new updates via long poll method
    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        // If the received update contains a new message...
        let update = update ? ;
        println!("----- {:?}", update);
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                //  Show received message
                println!("-- <{}>: {}", &message.from.first_name, data);

                //  Handle the command
                handle_command(&api, &message, data)
                .await ? ;
            }
        }
    }
    Ok(())
}

/// Handle a command e.g. "flash - flash 0x0 https://.../firmware.bin"
async fn handle_command(api: &Api, message: &Message, cmd: &str) -> Result<()> {
    //  Remove leading and trailing spaces. Replace multiple spaces by 1.
    let cmd = cmd
        .trim_start()
        .trim_end()
        .replace("  ", " ")
        .replace("  ", " ")
        .replace("  ", " ")
        .replace("  ", " ")
        .replace("  ", " ");
    let split: Vec<&str> = cmd.split(' ').collect();
    let cmd = split[0];

    if cmd != "/flash" || split.len() != 3 {
        //  Unknown command
        api.send(message.text_reply(format!(
            "Unknown command {}. Try /flash 0x0 https://github.com/JF002/Pinetime/releases/download/0.8.1-develop/pinetime-app-0.8.1-develop.bin",
            cmd
        )))
        .await ? ;
        return Ok(());
    }

    //  Handle flash command
    let addr = split[1];      //  e.g. 0x0
    let firmware = split[2];  //  e.g. https://.../firmware.bin
    api.send(message.text_reply(format!(
        "Flashing {} to PineTime at address {}...",
        firmware, addr
    )))
    .await ? ;

    //  Create a temporary directory
    let tmp_dir = tempfile::Builder::new().prefix("pinetime").tempdir() ? ;

    //  Download the firmware
    match download_file(firmware, &tmp_dir).await {
        Err(_) => {  //  Unable to download
            api.send(message.text_reply(format!(
                "Unable to download {}", firmware
            )))
            .await ? ;
            Ok(())
        }
        Ok(path) => {  //  Download OK
            //  Flash the firmware and reboot PineTime
            api.send(message.text_reply(format!(
                "Downloaded {}", firmware
            )))
            .await ? ;
            println!("path={}", path);
            match flash_firmware(addr, &path).await {
                Err(err) => {  //  Flash failed
                    println!("Error: {:?}", err);
                    api.send(message.text_reply(format!(
                        "Error: {}", err
                    )))
                    .await ? ;
                    Ok(())        
                }
                Ok(output) => {  //  Flash OK
                    //  Show the output
                    api.send(message.text_reply("Output: ".to_string() + &output)).await ? ;
                    Ok(())
                }
            }
        }
    }
    //  Upon exit, files in tmp_dir are deleted
}

/// Flash the downloaded firmware to PineTime at the address
async fn flash_firmware(addr: &str, path: &str) -> Result<String> {
    //  For Pi:
    //  cd $HOME/pinetime-updater
    //  openocd-spi/bin/openocd 
    //  -c ' set filename "/tmp/mynewt_nosemi.elf.bin" ' 
    //  -c ' set address  "0x0" ' 
    //  -f scripts/swd-pi.ocd 
    //  -f scripts/flash-program.ocd

    //  For ST-Link:
    //  cd $HOME/pinetime-updater
    //  xpack-openocd/bin/openocd
    //  -c ' set filename "/tmp/mynewt_nosemi.elf.bin" ' 
    //  -c ' set address  "0x0" ' 
    //  -f $HOME/pinetime-updater/scripts/swd-stlink.ocd 
    //  -f $HOME/pinetime-updater/scripts/flash-program.ocd
    let updater_path = env::var("HOME").expect("HOME not set") + "/pinetime-updater";
    //  let updater_path = env::var("HOME").expect("HOME not set") + "/pinetime/pinetime-updater";
    let output = std::process::Command
        ::new(updater_path.clone() + "/openocd-spi/bin/openocd")  //  Pi
        //  ::new(updater_path.clone() + "/xpack-openocd/bin/openocd")  //  ST-Link
        .current_dir(updater_path)
        .arg("-c")
        .arg("set filename \"".to_string() + path + "\"")
        .arg("-c")
        .arg("set address \"".to_string() + addr + "\"")
        .arg("-f")
        .arg("scripts/swd-pi.ocd")  //  Pi
        //  .arg("scripts/swd-stlink.ocd")  //  ST-Link
        .arg("-f")
        .arg("scripts/flash-program.ocd")
        .output() ? ;
    /*
    let output = std::process::Command
        ::new("ls")
        .arg("-l")
        .arg(path)
        .output() ? ;
    */    
    if !output.status.success() {
        println!("Output: {:?}", output);
        let error = String::from_utf8(output.stderr).unwrap();
        error_chain::bail!(error);
    }
    println!("Output: {:?}", output);
    let output = 
        String::from_utf8(output.stdout).unwrap() + "\n" +
        &String::from_utf8(output.stderr).unwrap();
    println!("Output: {}", output);
    Ok(output)
}

/// Download the URL to the temporary directory. Returns the downloaded pathname.
async fn download_file(url: &str, tmp_dir: &tempfile::TempDir) -> Result<String> {
    println!("url to download: '{}'", url);
    let response = reqwest::get(url).await ? ;

    let fname = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("firmware.bin");

    let fname = tmp_dir.path().join(fname);
    println!("will be located under: '{:?}'", fname);
    let mut dest = std::fs::File::create(fname.clone()) ? ;
    let content = response.text().await ? ;
    std::io::copy(&mut content.as_bytes(), &mut dest) ? ;
    Ok(fname.to_str().unwrap().to_string())
}