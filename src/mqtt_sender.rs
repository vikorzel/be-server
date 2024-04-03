use std::io::Error;

use be_server::external::abstract_external::ChannelSender;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use async_trait::async_trait;
const MQTT_SENDER_ID: &str = "SignalConsumer";
const MQTT_CAPABILITY: usize = 65535;

use crate::{device::Device, state::MqttConfig};

pub struct MqttSender {
    client: AsyncClient,
    topic: String,
    event_loop: rumqttc::EventLoop
}

impl MqttSender {
    pub fn new(config: MqttConfig) -> MqttSender {
        let mut mqtt_options = MqttOptions::new(MQTT_SENDER_ID, config.host, config.port);
        mqtt_options.set_credentials(config.user, config.password);
        let (mqtt_client, event_loop) = AsyncClient::new(mqtt_options, MQTT_CAPABILITY);
        return MqttSender {
            client: mqtt_client,
            topic: config.topic,
            event_loop: event_loop
        };
    }
}

#[async_trait]
impl ChannelSender<Device> for MqttSender {
    async fn send(&mut self, device: Device) -> Result<(), Error> {
        println!("Send new device data: {}", device.get_id());
        let topic = format!("{}",self.topic);
        match self.client.publish(topic, QoS::AtLeastOnce, false, device.as_json()).await  {
            Err(e) => {
                println!("Publish error: {}", e);
            },
            Ok(_) => {
                println!("Data sent correctly");
            }
        };
        Ok(())
        
    }
}
