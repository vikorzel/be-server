use crate::device::Device;
use be_server::external::abstract_external;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, atomic::{AtomicBool, Ordering::Relaxed}};
use std::time::Duration;

pub struct Metrics<T: abstract_external::ChannelSender<Device>> {
    reciever_channel: Receiver<Device>,
    channel_sender: T,

}

impl<'a, T: abstract_external::ChannelSender<Device>> Metrics<T>{
    pub fn new(rec: Receiver<Device>, sender: T) -> Metrics<T> {
        Metrics {
            reciever_channel: rec,
            channel_sender: sender,
        }
    }
    pub fn run(&mut self, run : Arc<AtomicBool>) {
        while run.load(Relaxed) {
            match self.reciever_channel.recv_timeout(Duration::from_millis(20)) {
                Ok(device) => {
                    self.channel_sender.send(device);
                }
                Err(_) => {
                    continue;
                }
            }            
        }
    }
}

#[cfg(test)]
mod tests {
    use be_server::external::abstract_external;
    use std::sync::{atomic::AtomicBool, mpsc::channel, Arc};
    use crate::device::Device;
    use std::sync::mpsc::Sender;
    use std::time::Duration;


    struct MockChannelSender {
        sent:Sender<Device>,
    }
    impl abstract_external::ChannelSender<Device> for MockChannelSender {
        fn send(&mut self, device: Device) {
            let _ = self.sent.send(device);
        }
    }
    impl MockChannelSender {
        fn new(sender: Sender<Device>) -> MockChannelSender{
            MockChannelSender {
                sent: sender,
            }
        }
    }

    #[test]
    fn test_sending() {
        let (snd, rcv) = channel::<Device>();
        let (test_snd, test_rcv) = channel::<Device>();
        let mock = MockChannelSender::new(test_snd);
        let mut metrics = super::Metrics::<MockChannelSender>::new(rcv, mock);
        
        //TODO: Make factory for bytes to make device
        let mut buf = [0; 1024];
        buf[0] = 7; //id
        buf[1] = 2; //devices count
        
        //temperature
        buf[2] = f32::to_le_bytes(0.123)[0]; 
        buf[3] = f32::to_le_bytes(0.123)[1];
        buf[4] = f32::to_le_bytes(0.123)[2];
        buf[5] = f32::to_le_bytes(0.123)[3];

        //humidity
        buf[6] = f32::to_le_bytes(0.456)[0];
        buf[7] = f32::to_le_bytes(0.456)[1];
        buf[8] = f32::to_le_bytes(0.456)[2];
        buf[9] = f32::to_le_bytes(0.456)[3];


        let run_metrics_thread = Arc::new((AtomicBool::new(true)));
        let run_metrics_thread_clone = run_metrics_thread.clone();
        std::thread::spawn(move || {
            metrics.run(run_metrics_thread_clone);
        });
        let devices = Device::factory(&buf, 1024).unwrap();
        for device in devices {
            snd.send(device).unwrap();
        }
        std::thread::sleep(Duration::from_millis(50));
        run_metrics_thread.store(false, std::sync::atomic::Ordering::Relaxed);
        let mut devices = Vec::<Device>::new();
        for device in test_rcv.iter() {
            devices.push(device);
        }

        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].get_id(), 701);
        assert_eq!(devices[0].get_temperature(), 0.123);
        assert_eq!(devices[0].get_humidity(), 0.456);

        assert_eq!(devices[1].get_id(), 702);
        assert_eq!(devices[1].get_temperature(), 0.0);
        assert_eq!(devices[1].get_humidity(), 0.0);
        
    }

    
}