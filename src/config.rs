enum Env {
    Dev,
    Prod,
}

impl Env {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Dev => "dev",
            Self::Prod => "prod",
        }
    }
}

impl TryFrom<String> for Env {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "dev" => Ok(Self::Dev),
            "prod" => Ok(Self::Prod),
            _ => Err(format!(
                "{} is not supported environment, use either `dev` or `prod` >~<",
                value
            )),
        }
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct ApplicationConfig {
    pub host: String,
    pub port: u16,
    pub allowed_ips: Vec<String>,
    pub max_servers: u8,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Config {
    pub application: ApplicationConfig,
}

pub fn get_config() -> Result<Config, config::ConfigError> {
    let base_path = std::env::current_dir().expect("failed to determine current directory");
    let configuration_directory = base_path.join("config");
    let environment: Env = std::env::var("ENV")
        .unwrap_or_else(|_| "dev".into())
        .try_into()
        .expect("failed to parse ENV");

    let builder = config::Config::builder()
        .add_source(config::File::new(
            configuration_directory.join("base").to_str().unwrap(),
            config::FileFormat::Yaml,
        ))
        .add_source(config::File::new(
            configuration_directory
                .join(environment.as_str())
                .to_str()
                .unwrap(),
            config::FileFormat::Yaml,
        ))
        .build()
        .unwrap();

    builder.try_deserialize()
}
