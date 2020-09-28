//  Telegram Bot to flash and test PineTime firmware remotely
//  Chat with BotFather, create bot "PineTime Bot"
//  Enter "/mybots", select "PineTime Bot"
//  Select "Edit Commands", enter "flash - flash 0x0 https://.../firmware.bin"
use std::{env, fs::File, string::String};
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
                match handle_command(&api, &message, data).await {
                    //  Command failed
                    Err(err) => println!("Error: {}", err),
                    //  Command succeeded
                    Ok(_)    => {}
                }
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
            "Unknown command {}. Try /flash 0x0 https://github.com/lupyuen/pinetime-rust-mynewt/releases/download/v5.0.4/mynewt.elf.bin",
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
    //  For Raspberry Pi:
    //  cd $HOME/pinetime-updater
    //  openocd-spi/bin/openocd 
    //  -c ' set filename "firmware.bin" ' 
    //  -c ' set address  "0x0" ' 
    //  -f scripts/swd-pi.ocd 
    //  -f scripts/flash-program.ocd
    #[cfg(target_arch = "arm")]  //  For Raspberry Pi
    let updater_path = "/pinetime-updater";

    //  For Mac with ST-Link:
    //  cd $HOME/pinetime/pinetime-updater
    //  xpack-openocd/bin/openocd
    //  -c ' set filename "firmware.bin" ' 
    //  -c ' set address  "0x0" ' 
    //  -f scripts/swd-stlink.ocd 
    //  -f scripts/flash-program.ocd
    #[cfg(target_arch = "x86_64")]  //  For Mac with ST-Link
    let updater_path = "/pinetime/pinetime-updater";

    //  Get the path of PineTime Updater. Remember to run "./run.sh" to download xPack OpenOCD or OpenOCD SPI
    let updater_path = env::var("HOME").expect("HOME not set") + &updater_path;

    //  Run the command and wait for output
    let output = std::process::Command
        //  ::new(updater_path.clone() + "/openocd-spi/bin/openocd")  //  Raspberry Pi SPI
        ::new(updater_path.clone() + "/xpack-openocd/bin/openocd")    //  ST-Link
        .current_dir(updater_path)
        .arg("-c")
        .arg("set filename \"".to_string() + path + "\"")
        .arg("-c")
        .arg("set address \"".to_string() + addr + "\"")
        .arg("-f")
        //  .arg("scripts/swd-pi.ocd")  //  Raspberry Pi SPI
        .arg("scripts/swd-stlink.ocd")  //  ST-Link
        .arg("-f")
        .arg("scripts/flash-program.ocd")
        .output() ? ;

    //  If command failed, dump stderr
    if !output.status.success() {
        println!("Output: {:?}", output);
        let error = String::from_utf8(output.stderr).unwrap();
        error_chain::bail!(error);
    }

    //  If command succeeded, dump stdout and stderr
    println!("Output: {:?}", output);
    let output = 
        String::from_utf8(output.stdout).unwrap() + "\n" +
        &String::from_utf8(output.stderr).unwrap();
    println!("Output: {}", output);
    Ok(output)
}

/// Transmit the 
async fn transmit_log() {
    let mut cmd = Command::new("cat");

    // Specify that we want the command's standard output piped back to us.
    // By default, standard input/output/error will be inherited from the
    // current process (for example, this means that standard input will
    // come from the keyboard and standard output/error will go directly to
    // the terminal if this process is invoked from the command line).
    cmd.stdout(Stdio::piped());

    let mut child = cmd.spawn()
        .expect("failed to spawn command");

    let stdout = child.stdout.take()
        .expect("child did not have a handle to stdout");

    let mut reader = BufReader::new(stdout).lines();

    // Ensure the child process is spawned in the runtime so it can
    // make progress on its own while we await for any output.
    tokio::spawn(async {
        let status = child.await
            .expect("child process encountered an error");

        println!("child status was: {}", status);
    });

    while let Some(line) = reader.next_line().await? {
        println!("Line: {}", line);
    }
}

/// Download the URL to the temporary directory. Returns the downloaded pathname.
async fn download_file(url: &str, tmp_dir: &tempfile::TempDir) -> Result<String> {
    //  Download the file and wait for the download to be completed
    println!("url to download: '{}'", url);
    let response = reqwest::get(url).await ? ;

    //  Get the last part of the URL as filename, or firmware.bin
    let fname = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("firmware.bin");

    //  Create the temporary pathname
    let fname = tmp_dir.path().join(fname);
    println!("will be located under: '{:?}'", fname);
    let mut dest = File::create(fname.clone()) ? ;

    //  Copy the downloaded data to the temporary pathname
    let content = response.bytes().await ? ;
    std::io::copy(&mut content.as_ref(), &mut dest) ? ;
    Ok(fname.to_str().unwrap().to_string())
}