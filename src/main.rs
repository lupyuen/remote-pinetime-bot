//  Telegram Bot to flash and test PineTime firmware remotely
//  Chat with BotFather, create bot "PineTime Bot"
//  Enter "/mybots", select "PineTime Bot"
//  Select "Edit Commands", enter "flash - flash 0x0 https://.../firmware.bin"
#![recursion_limit="256"]
use std::{env, fs::File, string::String};
use std::process::{Stdio};
use std::net::{SocketAddrV4, Ipv4Addr, TcpListener, TcpStream};
use std::io::prelude::*;
use std::io::{Read};
use tokio::io::{BufReader, AsyncBufReadExt};
use tokio::process::Command;
use futures::{
    future::{Fuse, FusedFuture, FutureExt},
    stream::{FusedStream, Stream, StreamExt},   
    pin_mut,
    select,
    try_join,
};
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

/// Listen for Telegram Bot commands and execute them with OpenOCD. 
/// Log Semihosting Debug Messages emitted by OpenOCD to a Telegram Channel.
#[tokio::main]
async fn main() -> Result<()> {
    //  Event loop based on https://rust-lang.github.io/async-book/06_multiple_futures/03_select.html#concurrent-tasks-in-a-select-loop-with-fuse-and-futuresunordered
    //  OpenOCD is not running initially
    let openocd_task = Fuse::terminated();

    //  Init the Telegram API
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let api = Api::new(token);

    //  Fetch new Telegram updates via long poll method
    let mut telegram_stream = api.stream();

    //  No pending flash command
    let mut pending_command = None;

    //  Create a temporary directory. Upon exit, files in tmp_dir are deleted
    let mut tmp_dir = tempfile::Builder::new().prefix("pinetime").tempdir() ? ;
    
    //  Loop forever processing Telegram and OpenOCD events
    pin_mut!(openocd_task);
    loop {
        //  Wait for Telegram Update to be received or OpenOCD Task to complete
        println!("Before Select: OpenOCD Task Terminated is {:?}", openocd_task.is_terminated());
        select! {
            //  If Telegram Update received...
            update = telegram_stream.next().fuse() => {
                //  If the received update contains a new message...
                if let Some(update) = update {
                    let update = update ? ;
                    println!("----- {:?}", update);
                    if let UpdateKind::Message(message) = update.kind {

                        //  If we received a text message...
                        if let MessageKind::Text { ref data, .. } = message.kind {
                            //  Recreate a temporary directory for downloading the firmware files. The previous downloaded firmware files in tmp_dir are deleted
                            tmp_dir = tempfile::Builder::new().prefix("pinetime").tempdir() ? ;
    
                            //  Handle the Telegram Bot command: /flash 0x0 https://.../firmware.bin
                            match handle_command(&api, &message, data, &tmp_dir).await {
                                //  Command failed
                                Err(err) => println!("Error: {}", err),

                                //  If firmware downloaded, prepare to flash
                                Ok(cmd)  => {
                                    //  Remember the flash command
                                    pending_command = cmd;

                                    //  If there is a pending flash command, tell OpenOCD Task to quit
                                    if pending_command.is_some() && !openocd_task.is_terminated() {
                                        println!("Send shutdown command to OpenOCD");
                                        let mut stream = TcpStream::connect("127.0.0.1:4444") ? ;
                                        stream.write(b"shutdown\r") ? ;
                                    }
                            
                                    //  Let the loop wait for OpenOCD task to quit
                                    //  TODO: Timeout if OpenOCD task doesn't quit
                                }
                            }    
    
                        }
                    }
                }
            },

            //  If OpenOCD Task completed...
            openocd_result = openocd_task => {
                println!("OpenOCD task completed: {:?}", openocd_result);
            },

            //  If everything completed, panic since Telegram Task should always be running
            complete => panic!("Telegram task completed unexpectedly"),
        }

        println!("Select OK: OpenOCD Task Terminated is {:?}", openocd_task.is_terminated());
        //  If there is a pending flash command and OpenOCD Task is completed...
        if pending_command.is_some() && openocd_task.is_terminated() {
            //  Start a new OpenOCD Task with the flash command
            let cmd = pending_command.unwrap();
            let task = flash_firmware(
                &api, &message,
                cmd.0,  //  Address e.g. 0x0
                cmd.1   //  Filename e.g. firmware.bin
            );
            //  Let the loop wait for the OpenOCD Task to complete
            openocd_task.set(task.fuse());
            //  No more pending flash command
            pending_command = None;
        }
    }
}

/// Spawn OpenOCD to flash the downloaded firmware to PineTime at the address.
/// Transmit the Semihosting Log from OpenOCD to Telegram Channel. Based on https://docs.rs/tokio/0.2.22/tokio/process/index.html
async fn flash_firmware(api: &Api, message: &Message, addr: String, path: String) -> Result<()> {
    //  For Raspberry Pi:
    //  cd $HOME/pinetime-updater
    //  openocd-spi/bin/openocd \
    //    -c ' set filename "firmware.bin" ' \
    //    -c ' set address  "0x0" ' \
    //    -f scripts/swd-pi.ocd \
    //    -f scripts/flash-log.ocd
    #[cfg(target_arch = "arm")]  //  For Raspberry Pi
    let updater_path = "/pinetime-updater";

    //  For Mac with ST-Link:
    //  cd $HOME/pinetime/pinetime-updater
    //  xpack-openocd/bin/openocd \
    //    -c ' set filename "firmware.bin" ' \
    //    -c ' set address  "0x0" ' \
    //    -f scripts/swd-stlink.ocd \
    //    -f scripts/flash-log.ocd
    #[cfg(target_arch = "x86_64")]  //  For Mac with ST-Link
    let updater_path = "/pinetime/pinetime-updater";

    //  Get the path of PineTime Updater. Remember to run "./run.sh" to download xPack OpenOCD or OpenOCD SPI
    let updater_path = env::var("HOME").expect("HOME not set") + &updater_path;

    //  Spawn OpenOCD as a background process
    let mut cmd = Command
        //  ::new(updater_path.clone() + "/openocd-spi/bin/openocd");  //  Raspberry Pi SPI
        ::new(updater_path.clone() + "/xpack-openocd/bin/openocd");    //  ST-Link
    let cmd = cmd
        .current_dir(updater_path)
        .arg("-c")
        .arg("set filename \"".to_string() + &path + "\"")
        .arg("-c")
        .arg("set address \"".to_string() + &addr + "\"")
        .arg("-f")
        //  .arg("scripts/swd-pi.ocd")  //  Raspberry Pi SPI
        .arg("scripts/swd-stlink.ocd")  //  ST-Link
        .arg("-f")
        .arg("scripts/flash-log.ocd");

    //  let mut cmd = Command::new("cargo");
    //  let cmd = cmd.arg("test").arg("--").arg("--nocapture");
    //  let mut cmd = Command::new("bash");
    //  let cmd = cmd.arg(script);

    //  Specify that we want the command's standard output and standard error piped back to us.
    //  By default, standard input/output/error will be inherited from the current process 
    //  (for example, this means that standard input will come from the keyboard and 
    //  standard output/error will go directly to the terminal if this process is invoked from the command line).
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()
        .expect("failed to spawn command");

    let stdout = child.stdout.take()
        .expect("child did not have a handle to stdout");

    //  TODO: In case of error, return the error log

    //  TODO: In the background, read the OpenOCD output line by line
    let mut reader = BufReader::new(stdout).lines();

    // Ensure the child process is spawned in the runtime so it can
    // make progress on its own while we await for any output.
    tokio::spawn(async {
        let status = child.await
            .expect("child process encountered an error");

        println!("child status was: {}", status);
    });

    //  TODO: Transmit each line of OpenOCD output to the Telegram Channel
    while let Some(line) = reader.next_line().await? {
        println!("Line: {}", line);
        if line.len() > 0 {
            api.send(
                line
            )    
            .await ? ;
        }
    }

    //  TODO: Wait for "*** Done" and return the message, while continuing OpenOCD output processing in the background
    //  See https://rust-lang-nursery.github.io/rust-cookbook/concurrency/threads.html#maintain-global-mutable-state
    Ok(())
}

/* Remote PineTime Log Channel:
----- Update { id: 761638748, kind: ChannelPost(
    ChannelPost { id: MessageId(45), date: 1601533862, chat: 
        Channel { id: ChannelId(-1001221686801), title: "Remote PineTime Log", username: Some("remotepinetimelog"), invite_link: None }, 
        forward: None, reply_to_message: None, edit_date: None, kind: NewChatTitle { data: "Remote PineTime Log" } }) }
*/

/*
//  TODO: Send message to Telegram channel
if let UpdateKind::ChannelPost(post) = update.kind {            
    if let MessageKind::Text { ref data, .. } = post.kind {
        // Print received text message to stdout.
        println!("<{}>: {}", "???", data);

        // Answer message with "Hi".
        api.send(post.text_reply(format!(
            "Hi, {}! You just wrote '{}'",
            "???", data
        )))
        .await?;
    }
}
*/

/// Handle a command e.g. "/flash 0x0 https://.../firmware.bin". Return (address, filename).
async fn handle_command(api: &Api, message: &Message, cmd: &str, tmp_dir: &tempfile::TempDir) -> Result<Option<(String, String)>> {
    //  Show received message
    println!("-- <{}>: {}", &message.from.first_name, cmd);

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
        return Ok(None);  //  Nothing to flash
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
    match download_file(firmware, &tmp_dir).await {
        Err(_) => {  //  Unable to download
            api.send(message.text_reply(format!(
                "Unable to download {}", firmware
            )))
            .await ? ;
            Ok(None)  //  Nothing to flash
        }
        Ok(path) => {  //  Download OK
            //  Flash the firmware and reboot PineTime
            api.send(message.text_reply(format!(
                "Downloaded {}", firmware
            )))
            .await ? ;
            println!("path={}", path);
            Ok(Some((addr.to_string(), path)))  //  Flash the firmware at the address
        }
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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use std::net::{SocketAddrV4, Ipv4Addr, TcpListener};
    use std::io::{Read};

    /// Simulate an OpenOCD server that listens for commands on port 4444
    #[test]
    fn test_server() -> Result<()> {
        let loopback = Ipv4Addr::new(127, 0, 0, 1);
        let socket = SocketAddrV4::new(loopback, 4444);
        let listener = TcpListener::bind(socket)?;
        let port = listener.local_addr()?;
        println!("Listening on {}, access this port to end the program", port);
        let (mut tcp_stream, addr) = listener.accept()?; //block  until requested
        println!("Connection received! {:?} is sending data.", addr);
        let mut input = String::new();
        let _ = tcp_stream.read_to_string(&mut input)?;
        println!("{:?} says {}", addr, input);
        Ok(())
    }
}