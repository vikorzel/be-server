use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct ServerConfig {
    #[clap(short, long, default_value = "127.0.0.1")]
    pub host: String,
    #[clap(short, long, default_value = "11110")]
    pub port: String,
}