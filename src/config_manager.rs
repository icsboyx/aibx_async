#![allow(dead_code)]
use std::fmt::Debug;
use anyhow::Result;
use serde::{ Deserialize, Serialize };

pub trait ConfigManager {
    async fn load_config<T>(
        default_config: impl Serialize + Debug,
        config_file_name: &str
    ) -> Result<T>
        where T: for<'de> Deserialize<'de> + Serialize + Default + Debug
    {
        // check if twitch toml file exists
        // if not, use anonymous default config for twitch

        println!("Loading config from {}", config_file_name);
        let config_content = std::fs::read_to_string(config_file_name);
        match config_content {
            Ok(config) => {
                match toml::from_str(&config) {
                    Ok(config) => {
                        return Ok(config);
                    }
                    Err(err) => {
                        println!(
                            "Failed to parse config file: {:#?}. Using default config",
                            err.message()
                        );
                        println!(
                            "Please check the config file: {config_file_name}, delete it and restart the program, or fix the error in the file"
                        );
                        return Ok(serde_json::from_value(serde_json::to_value(default_config)?)?);
                    }
                }
            }
            Err(_) => {
                println!(
                    "No {config_file_name} config file found. Using default config. Saving default config to {config_file_name}"
                );
                {
                    Self::save_config(&default_config, config_file_name).await?;
                }
                Ok(serde_json::from_value(serde_json::to_value(default_config)?)?)
            }
        }
    }
    async fn save_config(config: impl Serialize, config_file_name: &str) -> Result<()> {
        let config_toml = toml::to_string(&config)?;
        std::fs::write(config_file_name, config_toml)?;
        Ok(())
    }
}
