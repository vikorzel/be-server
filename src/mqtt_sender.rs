use std::io::Error;

use be_server::{device::HardDevice, external::abstract_external::ChannelSender};
use async_trait::async_trait;
use futures::executor::block_on;

extern crate paho_mqtt as mqtt;

use crate::{device::Device, state::MqttConfig};

pub struct MqttSender {
    topic: String,
    client: mqtt::AsyncClient
}

impl MqttSender {
    pub fn new(config: MqttConfig) -> MqttSender {
        let cli = mqtt::CreateOptionsBuilder::new()
            .server_uri(format!("tcp://{}:{}", config.host, config.port))
            .create_client().expect("Creating MQTT Client");
        let options = mqtt::ConnectOptionsBuilder::new()
            .user_name(config.user)
            .password(config.password)
            .finalize();
        block_on(cli.connect(options)).expect("Connection to MQTT Server");
        return MqttSender {
            topic: config.topic,
            client: cli
        };
    }
}

#[async_trait]
impl ChannelSender<HardDevice> for MqttSender {
    async fn send(&mut self, device: HardDevice) -> Result<(), Error> {
        println!("Send new device data: {}", device.get_id());
        let topic = format!("{}",self.topic);
        let msg = mqtt::MessageBuilder::new()
            .topic(topic)
            .payload(device.as_json())
            .qos(0)
            .finalize();
        self.client.publish(msg).await?;
        Ok(())
        
    }
}
