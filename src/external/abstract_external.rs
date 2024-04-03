use async_trait::async_trait;

#[async_trait]
pub trait ChannelSender<D> {
    async fn send(&mut self, device: D) -> Result<(), std::io::Error>;
}