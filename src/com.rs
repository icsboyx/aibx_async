#![allow(dead_code)]
use std::{ collections::VecDeque, sync::Arc };

use tokio::sync::{ Notify, RwLock };

#[derive(Debug)]
pub struct MessageQueue<T> {
    msg: Arc<RwLock<VecDeque<T>>>,
    notifier: Notify,
}

impl<T> MessageQueue<T> {
    pub fn new() -> Self {
        MessageQueue {
            msg: Arc::new(RwLock::new(VecDeque::new())),
            notifier: Notify::new(),
        }
    }

    pub async fn send(&self, message: T) {
        self.msg.write().await.push_back(message);
        self.notifier.notify_waiters();
    }

    pub async fn recv(&self) -> T {
        loop {
            if !self.is_empty().await {
                return self.msg.write().await.pop_front().unwrap();
            }
            self.notifier.notified().await;
        }
    }

    async fn is_empty(&self) -> bool {
        self.msg.read().await.is_empty()
    }

    async fn len(&self) -> usize {
        self.msg.read().await.len()
    }

    async fn clear(&self) {
        self.msg.write().await.clear();
    }
}
