use crate::device;
use crate::config;
use std::sync::mpsc::Sender;


#[derive(Clone)]
pub struct GlobalState {
    metrics_sender: Sender<device::Device>,
    config: config::ServerConfig,
}

#[derive(Clone)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub topic: String,
    pub user: String,
    pub password: String
}

impl GlobalState {
    pub fn new(metrics_sender: Sender<device::Device>, config: config::ServerConfig) -> GlobalState {
        GlobalState {
            metrics_sender,
            config,
        }
    }
    pub fn new_device(&self, device: device::Device) {
        println!("Send new device data");
        self.metrics_sender.send(device).unwrap();
    }
    pub fn get_tcp_addr(&self) -> String {
        format!("{}:{}", self.config.host, self.config.port)
    }

    pub fn get_mqtt_config(&self) -> MqttConfig {
        let host = self.config.mqtt_host.clone();
        let port = self.config.mqtt_port.clone();
        let topic = self.config.mqtt_topic.clone();
        let user = self.config.mqtt_user.clone();
        let password = self.config.mqtt_password.clone();

        MqttConfig{
            host,
            port,
            topic,
            user,
            password
        }
    }
}