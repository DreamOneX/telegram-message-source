use serde_derive::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

lazy_static! {
    pub static ref CONFIG: Config = {
        let path = Path::new("config.toml");
        let config = read_or_create_config(path).unwrap();
        config.transfer();
        config
    };
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
pub struct Config {
    pub enabled: bool,
    pub forwarding: ForwardingConfig,
    pub telegram: TelegramConfig,
    pub proxy: ProxyConfig,

    #[serde(skip)]
    pub target_address_mapper: DashMap<i64, &'static str>,

    #[serde(alias = "target_address_mapper")]
    target_address_mapper_storage: DashMap<&'static str, &'static str>,

    #[serde(skip, default = "default_config_path")]
    config_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForwardingConfig {
    // pattern: "nats://{host}:{port}"
    pub address: &'static str
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    // pattern: "http://{username}:{password}@{host}:{port}"
    pub address: &'static str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub token: &'static str,
    pub bot_name: &'static str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookConfig {}

impl Config {
    pub fn default() -> Config {
        Config {
            enabled: false,
            forwarding: ForwardingConfig::default(),
            telegram: TelegramConfig::default(),
            proxy: ProxyConfig::default(),
            target_address_mapper: DashMap::new(),
            target_address_mapper_storage: DashMap::new(),
            config_path: default_config_path(),
        }
    }
    pub fn default_string() -> Result<String, Error> {
        let result = toml::to_string_pretty(&Config::default())
            .map_err(|_| Error::SerializationError)?;
        Ok(result)
    }
}

impl Default for ForwardingConfig {
    fn default() -> ForwardingConfig {
        ForwardingConfig {
            address: "nats://itsusinn.site:4222"
        }
    }
}

impl Default for TelegramConfig {
    fn default() -> TelegramConfig {
        TelegramConfig {
            token: "BOT_TOKEN".into(),
            bot_name: "BOT_NAME".into(),
        }
    }
}

impl Default for ProxyConfig {
    fn default() -> ProxyConfig {
        ProxyConfig {
            enabled: false,
            address: "http://127.0.0.1:7890".into(),
        }
    }
}

impl Config {
    pub fn save(&self) {
        for pair in self.target_address_mapper.iter(){
            let key = pair.key();
            let val = pair.value();
            self.target_address_mapper_storage.insert(
                Box::leak(key.to_string().into_boxed_str()),
                val
            );
        }

        let ser = toml::ser::to_string_pretty(self).unwrap();
        log::info!("Configuration file was saved");
        fs::write(self.config_path.as_path(), ser).unwrap();
    }
    fn transfer(&self){
        for pair in self.target_address_mapper_storage.iter() {
            let key = pair.key();
            let val = pair.value();
            self.target_address_mapper.insert(key.parse().unwrap(), val);
        }
        self.target_address_mapper_storage.clear();
        self.target_address_mapper_storage.shrink_to_fit();

    }
}

fn default_config_path() -> PathBuf {
    return Path::new("config.toml").to_owned();
}

fn read_or_create_config(path: &Path) -> Result<Config, Error> {
    if !path.exists() {
        fs::create_dir_all(path.parent().unwrap_or(Path::new("./")))
            .map_err(|_| Error::CannotCreateParentDirectory)?;
        fs::write(path, Config::default_string()?)?;
    };
    let data = fs::read(path).map_err(|_| Error::FailedToReadFile)?;
    let result: Result<Config, toml::de::Error> = toml::from_slice(Box::leak(data.into_boxed_slice()));
    let mut result = match result {
        Ok(val) => val,
        Err(_) => {
            let default_string = Config::default_string()?;
            fs::write(path, default_string)?;
            Config::default()
        }
    };
    result.config_path = path.to_owned();
    Ok(result)
}

use dashmap::DashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to create dir all")]
    CannotCreateParentDirectory,
    #[error("无法读取配置文件")]
    FailedToReadFile,
    #[error("无法序列化")]
    SerializationError,
    #[error("I/O error")]
    IO(#[from] io::Error),
    #[error("无法反序列化")]
    TomlDe(#[from] toml::de::Error),
}