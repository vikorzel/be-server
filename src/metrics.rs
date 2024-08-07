use crate::device::Device;
use async_std::task::block_on;
use be_server::device::HardDevice;
use be_server::external::abstract_external;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, atomic::{AtomicBool, Ordering::Relaxed, AtomicUsize}};
use std::time::Duration;

pub struct Metrics<T: abstract_external::ChannelSender<HardDevice>> {
    reciever_channel: Receiver<HardDevice>,
    channel_sender: T,

}

impl<'a, T: abstract_external::ChannelSender<HardDevice>> Metrics<T>{
    pub fn new(rec: Receiver<HardDevice>, sender: T) -> Metrics<T> {
        Metrics {
            reciever_channel: rec,
            channel_sender: sender,
        }
    }
    pub fn run(&mut self, run : Arc<AtomicBool>, service_counter: Arc<AtomicUsize> ) {
        service_counter.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        while run.load(Relaxed) {
            match self.reciever_channel.recv_timeout(Duration::from_millis(20)) {
                Ok(device) => {
                    println!("New device in channel: {}", device.get_id());
                    block_on(self.channel_sender.send(device)).unwrap();
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
    use crate::device::{HardDevice, Device};
    use std::sync::mpsc::Sender;
    use std::time::Duration;
    use async_trait::async_trait;


    struct MockChannelSender {
        sent:Sender<HardDevice>,
    }
    
    #[async_trait]
    impl abstract_external::ChannelSender<HardDevice> for MockChannelSender {
        async fn send(&mut self, device: HardDevice) -> Result<(), std::io::Error> {
            let _ = self.sent.send(device);
            Ok(())
        }
    }
    impl MockChannelSender {
        fn new(sender: Sender<HardDevice>) -> MockChannelSender{
            MockChannelSender {
                sent: sender,
            }
        }
    }

    #[test]
    fn test_sending() {
        let (snd, rcv) = channel::<HardDevice>();
        let (test_snd, test_rcv) = channel::<HardDevice>();
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


        let run_metrics_thread = Arc::new(AtomicBool::new(true));
        
        let run_metrics_thread_clone = run_metrics_thread.clone();
        let service_counter = Arc::new(std::sync::atomic::AtomicUsize::new(1));
        std::thread::spawn(move || {
            metrics.run(run_metrics_thread_clone, service_counter);
        });
        let devices = HardDevice::factory(&buf, 1024).unwrap();
        for device in devices {
            snd.send(device).unwrap();
        }
        std::thread::sleep(Duration::from_millis(50));
        run_metrics_thread.store(false, std::sync::atomic::Ordering::Relaxed);
        let mut devices = Vec::<HardDevice>::new();
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