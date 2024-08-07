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
    pub service_port: u16,
    
    #[clap(long="plogin", default_value="posgtgress", help="Postgress Login")]
    pub sql_login : String,
    #[clap(long="ppassword", default_value="password", help="Postgress password")]
    pub slq_password: String,
    #[clap(long="phost", default_value="127.0.0.1", help="Postgres hostname")]
    pub sql_host: String,
    #[clap(long="pport", default_value_t = 5432, help = "Postgres port")]
    pub sql_port: u16,
    #[clap(long="pdbname", default_value="devices", help="Postgress database name")]
    pub sql_dbname: String
}