#![allow(dead_code)]

use crate::colors::Colorize;
use crate::config_manager::ConfigManager;
use crate::irc_parser;
use crate::Args;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use tokio_tungstenite::tungstenite::Message;
use std::sync::Arc;
use std::time::Duration;

use futures::{ pin_mut, SinkExt, StreamExt };

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TwitchClientConfig {
    pub server_address: String,
    pub nick: String,
    pub token: String,
    pub channel: String,
    pub log_level: String,
    pub anti_idle: i32,
}

impl ConfigManager for TwitchClientConfig {}

impl Default for TwitchClientConfig {
    fn default() -> Self {
        TwitchClientConfig {
            server_address: "wss://irc-ws.chat.twitch.tv:443".into(),
            nick: "justinfan123".into(),
            token: "oauth:1234567890".into(),
            channel: "icsboyx".into(),
            log_level: "info".into(),
            anti_idle: 180,
        }
    }
}

trait WsMessageHandler {
    fn to_ws_text(&self) -> Message;
}

impl<T: std::fmt::Display> WsMessageHandler for T {
    fn to_ws_text(&self) -> Message {
        println!("{} {}", "[TX]".blue(), self);
        Message::text(self.to_string())
    }
}

pub async fn start(args: Arc<Args>) -> Result<()> {
    let config_file_name = "twitch_client_config.toml";

    // Load twitch Client configuration or use default values and write to config file
    let twitch_client_config = TwitchClientConfig::load_config::<TwitchClientConfig>(
        TwitchClientConfig::default(),
        config_file_name
    ).await?;

    println!("Starting Twitch Client");

    let server_address = twitch_client_config.server_address;
    let user_token = twitch_client_config.token;
    let user_nick = twitch_client_config.nick;
    let user_channel = twitch_client_config.channel;

    let (ws_stream, _response) = tokio_tungstenite::connect_async(server_address).await?;
    let (mut write, mut read) = ws_stream.split();

    println!("[DEBUG] Connected to Twitch, sending auth, nick, and join");
    _ = write.send(format!("PASS oauth:{}", user_token).to_ws_text()).await?;
    _ = write.send(format!("NICK {}", user_nick).to_ws_text()).await?;
    _ = write.send(format!("JOIN #{}", user_channel).to_ws_text()).await?;
    _ = write.send("CAP REQ :twitch.tv/tags".to_ws_text()).await?;

    let ping_interval = tokio::time::interval(Duration::from_secs(180));

    pin_mut!(ping_interval);

    loop {
        tokio::select! {
        _ = ping_interval.tick() => {
            let payload = "PING :tmi.twitch.tv";
            write.send(payload.to_ws_text()).await?;
            }

        Some(line) = read.next() => {
            if let Ok(line) = line {
                let lines = line.to_text().unwrap().trim_end_matches("\r\n").split("\r\n");
                for line in lines {
                    let payload = line;
                    println!("{}{} ","[RX][RAW] ".magenta(), payload);
                    let irc_message = irc_parser::parse_message(&payload.to_string());
                    println!("{}{:?} ", ["[RX]".magenta().as_str(),"[MSG]".green().as_str()].join(""), irc_message);
                    match irc_message.context.command.as_str() {
                        "001" => {
                            println!("[DEBUG] Bot {}, connected to Twitch.", irc_message.context.destination);
                            args.bot_info.set_name(&irc_message.context.destination).await;
                            args.bot_info.set_main_channel(&user_channel).await;
                            println!("[DEBUG] Bot Info: {:?}", args.bot_info);
                        }
                        "PRIVMSG" => {
                                let payload = format!("[{}]: {}", irc_message.context.sender, irc_message.payload);
                                args.ollama.send(payload).await;
                                args.tts_message_queue.send(irc_message.payload).await;
                            //   println!("Sending message to TTS engine.");
                            //   println!("Sending message to TTS engine: {:?}", irc_message);
                            // let tts_message = TTSMessages::new(
                            //     irc_message.context.sender.clone(),
                            //     irc_message.payload.clone(),
                            //     PlayMode::Normal,
                            // );
                            // args.tts_queue.push(tts_message).await;
                            }
                        "PING" => {
                            write.send("PONG :tmi.twitch.tv".to_ws_text()).await?;
                        }
                        _ => {
                            // TODO: Add more commands
                        }     
                        }
            }
        }
    

      // Ok(msg) = my_subscriber.recv::<IrcMessage>()=> {
      //     println!("[TWITCH][TX] {:?}", msg);
      // }
  }

        ret_val = args.twitch_queue.recv() => {
            let payload = ret_val;
            println!("{}{} Sending: {}", "[TX]".green(), "[MSG]".blue(), payload);
            write.send(format!("PRIVMSG #{} :{}", user_channel, payload).to_ws_text()).await?;
        }
  
  }
    }
}