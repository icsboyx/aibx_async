#![allow(dead_code)]
use std::sync::Arc;
use msedge_tts::{
    tts::{ client::connect_async, SpeechConfig },
    voice::{ get_voices_list_async, Voice, VoiceTag },
};
use anyhow::Result;
use rand::Rng;

use crate::Args;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct TTSVoice {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "ShortName")]
    pub short_name: Option<String>,
    #[serde(rename = "Gender")]
    pub gender: Option<String>,
    #[serde(rename = "Locale")]
    pub locale: Option<String>,
    #[serde(rename = "SuggestedCodec")]
    pub suggested_codec: Option<String>,
    #[serde(rename = "FriendlyName")]
    pub friendly_name: Option<String>,
    #[serde(rename = "Status")]
    pub status: Option<String>,
    #[serde(rename = "VoiceTag")]
    pub voice_tag: Option<TTSVoiceTag>,
}

impl From<&Voice> for TTSVoice {
    fn from(voice: &Voice) -> Self {
        TTSVoice {
            name: voice.name.clone(),
            short_name: voice.short_name.clone(),
            gender: voice.gender.clone(),
            locale: voice.locale.clone(),
            suggested_codec: voice.suggested_codec.clone(),
            friendly_name: voice.friendly_name.clone(),
            status: voice.status.clone(),
            voice_tag: voice.voice_tag.as_ref().map(|tag| tag.into()),
        }
    }
}

impl From<&TTSVoice> for Voice {
    fn from(voice: &TTSVoice) -> Self {
        Voice {
            name: voice.name.clone(),
            short_name: voice.short_name.clone(),
            gender: voice.gender.clone(),
            locale: voice.locale.clone(),
            suggested_codec: voice.suggested_codec.clone(),
            friendly_name: voice.friendly_name.clone(),
            status: voice.status.clone(),
            voice_tag: voice.voice_tag.as_ref().map(|tag| tag.into()),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct TTSVoiceTag {
    #[serde(rename = "ContentCategories")]
    pub content_categories: Option<Vec<String>>,
    #[serde(rename = "VoicePersonalities")]
    pub voice_personalities: Option<Vec<String>>,
}

impl From<&VoiceTag> for TTSVoiceTag {
    fn from(tag: &VoiceTag) -> Self {
        TTSVoiceTag {
            content_categories: tag.content_categories.clone(),
            voice_personalities: tag.voice_personalities.clone(),
        }
    }
}

impl From<&TTSVoiceTag> for VoiceTag {
    fn from(tag: &TTSVoiceTag) -> Self {
        VoiceTag {
            content_categories: tag.content_categories.clone(),
            voice_personalities: tag.voice_personalities.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TTSSpeechConfig {
    pub voice_name: String,
    pub audio_format: String,
    pub pitch: i32,
    pub rate: i32,
    pub volume: i32,
}

impl From<SpeechConfig> for TTSSpeechConfig {
    fn from(config: SpeechConfig) -> Self {
        TTSSpeechConfig {
            voice_name: config.voice_name.clone(),
            audio_format: config.audio_format.clone(),
            pitch: config.pitch,
            rate: config.rate,
            volume: config.volume,
        }
    }
}

impl From<TTSSpeechConfig> for SpeechConfig {
    fn from(config: TTSSpeechConfig) -> Self {
        SpeechConfig {
            voice_name: config.voice_name.clone(),
            audio_format: config.audio_format.clone(),
            pitch: config.pitch,
            rate: config.rate,
            volume: config.volume,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TTSSpeech {
    voice_config: TTSVoice,
    speech_config: TTSSpeechConfig,
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

#[derive(Debug)]
pub struct TTSConfigs {
    tts_configs: Vec<TTSSpeech>,
}

impl TTSConfigs {
    pub async fn new() -> Self {
        let voices = get_voices_list_async().await
            .unwrap()
            .into_iter()
            .map(|voice| TTSSpeech {
                voice_config: TTSVoice::from(&voice),
                speech_config: SpeechConfig::from(&voice).into(),
            })
            .collect::<Vec<TTSSpeech>>();

        TTSConfigs {
            tts_configs: voices,
        }
    }

    pub fn filter_gender(&self, gender: TTSGender) -> Self {
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

    pub fn filter_locale(&self, locale: &str) -> Self {
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
            let audio = tts.synthesize(&ret_val, &voice.speech_config.clone().into()).await?;
            println!("Request {:?}", ret_val);
            println!("Response {:?
            }", audio.audio_metadata);
            
        }


    }
    }
}
