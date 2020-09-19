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
        //Utf8(std::string::Error);
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
            "Unknown command {}. Try /flash 0x0 https://.../firmware.bin",
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

    //  Download the firmware
    match download_file(firmware).await {
        Err(_) => {
            api.send(message.text_reply(format!(
                "Unable to download {}", firmware
            )))
            .await ? ;
            Ok(())
        }
        Ok(tmp_dir) => {
            let path = tmp_dir.path();
            println!("path={}", path.to_str().unwrap());
            //  Flash the firmware and reboot PineTime
            flash_firmware(addr, path.to_str().unwrap()).await ? ;
            Ok(())
        }
    }
    //  Upon exit, files in tmp_dir are deleted
}

/// Flash the downloaded firmware to PineTime at the address
async fn flash_firmware(addr: &str, path: &str) -> Result<()> {
    let output = std::process::Command
        ::new("ls")
        .arg("-l")
        .arg(path)
        .output() ? ;

    if !output.status.success() {
        error_chain::bail!("Command executed with failing error code");
    }
    let output = String::from_utf8(output.stdout).unwrap();
    println!("Output: {}", output);

    /*
    String::from_utf8(output.stdout) ?
        .lines()
        .for_each(|x| println!("{:?}", x));    
    */
    Ok(())
}

/// Download the URL. Returns the downloaded pathname.
async fn download_file(url: &str) -> Result<tempfile::TempDir> {
    println!("url to download: '{}'", url);
    let tmp_dir = tempfile::Builder::new().prefix("pinetime").tempdir() ? ;
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
    Ok(tmp_dir)
}