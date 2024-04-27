use async_trait::async_trait;

#[async_trait]
pub trait ChannelSender<D> {
    async fn send(&mut self, device: D) -> Result<(), std::io::Error>;
}

#[async_trait]
pub trait ExternalDatabase {
    async fn get_device_config(&mut self, key: &String) -> Result<String, std::io::Error>;
}