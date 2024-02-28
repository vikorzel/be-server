mod config;
mod device;
mod state;
mod metrics;
use be_server::external::abstract_external;
use clap::Parser;
use log::error;
use state::GlobalState;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use std::io;
use std::error::Error;
use std::thread;

async fn process_socket(socket: TcpStream, state: &GlobalState) -> Result<(), Box<dyn Error>> {
    let mut buf = [0; 1024];
    socket.readable().await?;

    match socket.try_read(&mut buf) {
        Ok(0) => return Ok(()),
        Ok(n) => {
            println!("read {} bytes", n);
            let devices = device::Device::factory(&buf, n);
            match devices {
                Ok(devices) => {
                    for device in devices {
                        state.new_device(device);
                    }
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
        Err(e) => {
            return Err(e.into());
        }
    }
    Ok(())
    
}

#[tokio::main]
async fn main() -> io::Result<()> {
    
    let (metrics_snd_channel, metrics_rcv_channel) = std::sync::mpsc::channel::<device::Device>();
    let state = state::GlobalState::new(metrics_snd_channel, config::ServerConfig::parse());

  
      let _metrics_thread = thread::spawn(move || {
        // let metrics = metrics::Metrics::new(metrics_rcv_channel);
        // metrics.run();
    });

    let listener = TcpListener::bind(state.get_tcp_addr()).await?;
    loop {
        let (socket, _) = listener.accept().await?;
        match process_socket(socket, &state).await {
            Ok(_) => {}
            Err(e) => {
                error!("Error: {}", e);
            }
        }
    }
}
