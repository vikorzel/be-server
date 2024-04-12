use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct ServerConfig {
    #[clap(long="lhost", default_value = "127.0.0.1", help="Listen Host")]
    pub host: String,
    #[clap(long="lport", default_value = "11110", help="Listen Port")]
    pub port: String,
    #[clap(long="mhost", default_value ="127.0.0.1", help="MQTT Host")]
    pub mqtt_host: String,
    #[clap(long="mport", default_value_t = 1883, help="MQTT Port")]
    pub mqtt_port: u16,
    #[clap(long="mtopic", default_value = "beserver", help="MQTT Root Topic name")]
    pub mqtt_topic: String,
    #[clap(long="muser", default_value = "user", help="MQTT UserName")]
    pub mqtt_user: String,
    #[clap(long="mpassword", default_value="password", help="MQTT Password")]
    pub mqtt_password: String,
    #[clap(long="sport", default_value_t = 0, help="Service Port")]
    pub service_port: u16
}