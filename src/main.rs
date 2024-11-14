#![allow(dead_code)]
use std::sync::Arc;

use com::MessageQueue;
use tokio::sync::RwLock;

mod config_manager;
mod twitch_client;
mod irc_parser;
mod colors;
mod ollama;
mod com;
mod tts;

#[derive(Debug, Clone, Default)]
pub struct BOTInfo {
    name: Arc<RwLock<String>>,
    main_channel: Arc<RwLock<String>>,
}

impl BOTInfo {
    pub async fn set_name(&self, name: &str) {
        *self.name.write().await = name.to_string();
    }

    pub async fn set_main_channel(&self, main_channel: &str) {
        *self.main_channel.write().await = main_channel.to_string();
    }

    pub async fn get_name(&self) -> String {
        self.name.read().await.clone()
    }

    pub async fn get_main_channel(&self) -> String {
        self.main_channel.read().await.clone()
    }
}

struct Args {
    bot_info: BOTInfo,
    twitch_queue: MessageQueue<String>,
    ollama: MessageQueue<String>,
    tts_message_queue: MessageQueue<String>,
}

#[tokio::main]
async fn main() {
    let bot_info = BOTInfo::default();

    let args = Arc::new(Args {
        bot_info,
        ollama: MessageQueue::new(),
        twitch_queue: MessageQueue::new(),
        tts_message_queue: MessageQueue::new(),
    });

    let tasks = vec![
        // tokio::spawn(twitch_client::start(args.clone())),
        // tokio::spawn(ollama::start(args.clone())),
        tokio::spawn(tts::start(args.clone()))
    ];

    let mut tokio_handles = Vec::new();
    tokio_handles.push(tokio::spawn(twitch_client::start(args.clone())));
    // tokio_handles.push(tokio::spawn(ollama::start(args.clone())));

    for task in tasks {
        tokio_handles.push(task);
    }

    for handle in tokio_handles {
        handle.await.unwrap().unwrap();
    }
}
