#![allow(dead_code)]
use std::sync::Arc;
use msedge_tts::{
    tts::{ client::connect_async, SpeechConfig },
    voice::{ get_voices_list_async, Voice },
};
use anyhow::Result;
use rand::Rng;

use crate::Args;

#[derive(Debug, Clone)]
pub struct TTSSpeech {
    voice_config: Arc<Voice>,
    speech_config: Arc<SpeechConfig>,
}

#[derive(Debug, Clone, Copy)]
pub enum TTSGender {
    Male,
    Female,
}

impl TTSGender {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..=1) {
            0 => TTSGender::Male,
            _ => TTSGender::Female,
        }
    }
}

// Implement From<&str> for TTSGender
impl From<&str> for TTSGender {
    fn from(value: &str) -> Self {
        match value {
            "Male" => TTSGender::Male,
            "Female" => TTSGender::Female,
            _ => TTSGender::random(),
        }
    }
}

// Implement From<String> for TTSGender
impl From<String> for TTSGender {
    fn from(value: String) -> Self {
        Self::from(value.as_str()) // Reuse the &str implementation
    }
}

impl From<TTSGender> for String {
    fn from(value: TTSGender) -> Self {
        match value {
            TTSGender::Female => "Female".into(),
            TTSGender::Male => "Male".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TTSConfigs {
    tts_configs: Vec<TTSSpeech>,
}

impl TTSConfigs {
    pub async fn new() -> Self {
        let voices = get_voices_list_async().await
            .unwrap()
            .into_iter()
            .map(|voice| {
                TTSSpeech {
                    speech_config: Arc::new(SpeechConfig::from(&voice)),
                    voice_config: Arc::new(voice),
                }
            })
            .collect::<Vec<TTSSpeech>>();
        TTSConfigs {
            tts_configs: voices,
        }
    }

    pub fn filter_gender(&self, gender: TTSGender) -> TTSConfigs {
        let voices = self.tts_configs
            .iter()
            .filter(|voice| voice.voice_config.gender == Some(gender.into()))
            .map(|voice| voice.clone())
            .collect::<Vec<TTSSpeech>>();

        TTSConfigs {
            tts_configs: voices
                .iter()
                .map(|voice| voice.clone())
                .collect::<Vec<TTSSpeech>>(),
        }
    }

    pub fn filter_locale(&self, locale: &str) -> TTSConfigs {
        let voices = self.tts_configs
            .iter()
            .filter(|voice| voice.voice_config.locale == Some(locale.into()))
            .map(|voice| voice.clone())
            .collect::<Vec<TTSSpeech>>();

        TTSConfigs {
            tts_configs: voices
                .iter()
                .map(|voice| voice.clone())
                .collect::<Vec<TTSSpeech>>(),
        }
    }

    pub fn random(&self) -> TTSSpeech {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.tts_configs.len());
        self.tts_configs[index].clone()
    }
}

pub async fn start(args: Arc<Args>) -> Result<()> {
    let voices = TTSConfigs::new().await;

    let voice = voices.filter_gender(TTSGender::Male).filter_locale("it-IT").random();
    println!("{:#?}", voice);

    let mut tts = connect_async().await?;
    loop {
        tokio::select! {

        ret_val = args.tts_message_queue.recv() => {
            let audio = tts.synthesize(&ret_val, &voice.speech_config.clone()).await?;
            println!("Request {:?}", ret_val);
            println!("Response {:?
            }", audio.audio_metadata);
            
        }


    }
    }
}
