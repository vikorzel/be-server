mod config;
mod device;
mod metrics;
mod mqtt_sender;
mod state;
use async_std::io as aio;
use async_std::task::block_on;
use be_server::external::abstract_external;
use be_server::service_server::ServiceServer;
use clap::Parser;
use futures::future::FutureExt;
use futures::join;
use log::error;
use mqtt_sender::MqttSender;
use state::GlobalState;
use std::error::Error;
use std::io;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicIsize;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

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
                        let dev_json = device.as_json();
                        state.new_device(device);
                        println!("{}", dev_json);
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

async fn listener_routine(
    process_running: Arc<AtomicBool>,
    addr: String,
    state: GlobalState,
    service_counter: Arc<AtomicUsize>,
) -> io::Result<()> {
    println!("Start listen on: {}", addr);
    let listener = TcpListener::bind(addr).await?;
    println!("Listener started");
    service_counter.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    loop {
        if process_running.load(std::sync::atomic::Ordering::Relaxed) == false {
            return Ok(());
        }
        let socket = aio::timeout(Duration::from_secs(1), async {
            let (socket, _) = listener.accept().await?;
            Ok(socket)
        })
        .await;
        if socket.is_err() {
            continue;
        }
        let socket: TcpStream = socket.unwrap();

        match process_socket(socket, &state).await {
            Ok(_) => {}
            Err(e) => {
                error!("Error: {}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("HERE");
    let (metrics_snd_channel, metrics_rcv_channel) = std::sync::mpsc::channel::<device::Device>();
    let state = state::GlobalState::new(metrics_snd_channel, config::ServerConfig::parse());
    let service_counter = Arc::new(AtomicUsize::new(0));

    println!("Starting BE Server");
    let channel_sender = MqttSender::new(state.get_mqtt_config());

    let programm_is_run = Arc::new(AtomicBool::new(true));
    let programm_is_run_metrics_copy = programm_is_run.clone();
    println!("Init Metrics thread");

    service_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let metrics_service_counter = service_counter.clone();
    let _metrics_thread = thread::spawn(move || {
        let mut metrics = metrics::Metrics::<MqttSender>::new(metrics_rcv_channel, channel_sender);
        metrics.run(programm_is_run_metrics_copy, metrics_service_counter);
    });

    let programm_is_run_listener_copy = programm_is_run.clone();
    let listener_state_clone = state.clone();
    println!("Init Listener thread");

    service_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let listener_service_counter = service_counter.clone();
    let _listener_thread = thread::spawn(move || {
        println!("Start Listener...");
        let _ = block_on(listener_routine(
            programm_is_run_listener_copy,
            listener_state_clone.get_tcp_addr(),
            listener_state_clone,
            listener_service_counter,
        ));
    });

    match state.get_service_port() {
        Some(port) => {
            thread::spawn(move || {
                let service_server = ServiceServer::new(service_counter, port);
                service_server.run_listener();
            });
        }
        None => {}
    }

    let _ = ctrlc::set_handler(move || {
        println!("Signal to close programm");
        programm_is_run.store(false, std::sync::atomic::Ordering::Relaxed);
    });
    let _ = _listener_thread.join().expect("Can't join listener thread");
    _metrics_thread.join().expect("Can't join metrics thread");
    Ok(())
}
