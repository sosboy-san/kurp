use std::env;
use std::fs;
use std::path::PathBuf;

use config::{Config, ConfigError, Environment, File};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(unused)]
pub struct AppConfig {
    pub port: u16,
    pub upstream_url: String,
    pub upscale: bool,
    pub return_format: Format,
    pub size_threshold_enabled: bool,
    pub size_threshold: u32,
    pub size_threshold_png: u32,
    pub upscaler: EnabledUpscaler,
    pub upscale_tag: Option<String>,
    pub allow_config_updates: bool,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Format {
    Png,
    Jpeg,
    WebP,
    Avif,
    Original,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EnabledUpscaler {
    Waifu2x,
    Realcugan,
    Lanczos3,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let config_dir = AppConfig::get_config_directory();

        let mut config = Config::builder();
        if config_dir.join("config.yml").exists() {
            config = config.add_source(File::from(config_dir.join("config.yml")))
        }

        config = config.add_source(Environment::with_prefix("kurp"))
            .set_default("port", "3030")?
            .set_default("upstream_url", "http://localhost:8080")?
            .set_default("upscale", true)?
            .set_default("return_format", "WebP")?
            .set_default("size_threshold_enabled", "true")?
            .set_default("size_threshold", "500")?
            .set_default("size_threshold_png", "1000")?
            .set_default("upscaler", "Lanczos3")?
            .set_default("allow_config_updates", false)?;

        config.build()?.try_deserialize()
    }

    pub fn write_config(config: AppConfig) {
        let yaml = serde_yaml::to_string(&config).unwrap();
        let config_path = AppConfig::get_config_directory().join("config.yml");
        fs::write(config_path, yaml).expect("Unable to write file");
    }

    fn get_config_directory() -> PathBuf {
        let current_dir = env::current_dir().expect("can't read current dir");
        let dir_env = env::var("KURP_CONF_DIR");
        let config_dir: PathBuf = dir_env.map(|path| { PathBuf::from(path) })
            .unwrap_or_else(|_| { current_dir.clone() });

        fs::create_dir_all(&config_dir).expect("can't create config directory");

        config_dir
    }
}
