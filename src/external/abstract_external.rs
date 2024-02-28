pub trait ChannelSender<D> {
    fn send(&mut self, device: D);
}