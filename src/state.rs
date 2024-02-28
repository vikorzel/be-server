use crate::device;
use crate::config;
use std::sync::mpsc::Sender;


#[derive(Clone)]
pub struct GlobalState {
    metrics_sender: Sender<device::Device>,
    config: config::ServerConfig,
}

impl GlobalState {
    pub fn new(metrics_sender: Sender<device::Device>, config: config::ServerConfig) -> GlobalState {
        GlobalState {
            metrics_sender,
            config,
        }
    }
    pub fn new_device(&self, device: device::Device) {
        self.metrics_sender.send(device).unwrap();
    }
    pub fn get_tcp_addr(&self) -> String {
        format!("{}:{}", self.config.host, self.config.port)
    }
}