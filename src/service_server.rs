use std::{
    io::{Read, Write},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use std::net::TcpListener;

struct ServiceServer {
    services_warmup_counter: Arc<AtomicUsize>,
    service_port: u16,
}

impl ServiceServer {
    pub fn new(counter: Arc<AtomicUsize>, port: u16) -> ServiceServer {
        return ServiceServer {
            services_warmup_counter: counter,
            service_port: port,
        };
    }
    pub fn is_ready(&self) -> bool {
        self.services_warmup_counter.load(Ordering::Relaxed) == 0
    }

    pub fn run_listener(&self) {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.service_port))
            .expect("Running Service Server ");
        let ok_response = "OK";
        let nok_response = "NOK";
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let now: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
                    let formated_date = now.format("%a, %d %b %Y %T GMT");
                    let mut buf = [0u8; 1024];
                    if stream.read(&mut buf).is_err() {
                        continue;
                    }
                    let data = String::from_utf8_lossy(&buf);

                    let by_lines: Vec<&str> = data.split('\n').collect();
                    let first_line: Vec<&str> = by_lines[0].split(" ").collect();
                    let method = first_line[0];
                    let path = first_line[1];

                    if method == "GET" && (path == "/status") {
                        //Mon, 08 Apr 2024 19:21:49 GMT
                        let mut response = nok_response;
                        if self.is_ready() {
                            response = ok_response;
                        }

                        let data_as_text = format!(
                            "HTTP/1.1 200 OK\nDate: {}\nServer: BE Server\nContent-Type: text/html\nContent-Length: {}\n\n{}",
                            formated_date,
                            response.len(),
                            response
                        );
                        stream.write(data_as_text.as_bytes()).unwrap();
                        continue;
                    }
                    stream
                        .write(
                            format!(
                                "HTTP/1.1 404\nDate: {}\nServer: BE Server\n\n",
                                formated_date
                            )
                            .as_bytes(),
                        )
                        .unwrap();
                }
                Err(err) => {
                    panic!("{}", err);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use async_std::task::block_on;
    use reqwest::StatusCode;
    use std::{thread, time::Duration};

    use super::*;
    #[test]
    fn simple_check() {
        let counter = Arc::new(AtomicUsize::new(8));
        let service_server = ServiceServer::new(counter.clone(), 123);

        assert!(!service_server.is_ready());
        counter.store(0, Ordering::Relaxed);
        assert!(service_server.is_ready());
    }

    #[test]
    fn multi_thread_check() {
        let counter = Arc::new(AtomicUsize::new(0));
        let threads_count = 3;
        let mut threads_collection = Vec::new();
        for _i in 0..threads_count {
            let counter_copy = counter.clone();
            counter_copy.fetch_add(1, Ordering::Relaxed);
            threads_collection.push(thread::spawn(move || {
                thread::sleep(Duration::from_millis(100));
                counter_copy.fetch_sub(1, Ordering::Relaxed);
            }))
        }
        let service_server = ServiceServer::new(counter.clone(), 1234);
        assert!(!service_server.is_ready());

        for thread_instance in threads_collection {
            let _ = thread_instance.join();
        }

        assert!(service_server.is_ready());
    }

    #[test]
    fn handler_test() {
        let counter = Arc::new(AtomicUsize::new(1));
        let counter_clone = counter.clone();
        thread::spawn(move || {
            let service_server = ServiceServer::new(counter_clone, 32143);
            service_server.run_listener();
        });
        thread::sleep(Duration::from_millis(100));
        let response = block_on(reqwest::get("http://localhost:32143/status"));

        counter.store(1, Ordering::Relaxed);
        match response {
            Err(_) => {
                assert!(false)
            }
            Ok(resp) => {
                let text = block_on(resp.text()).expect("Response test");
                assert_eq!(text, "NOK");
            }
        }

        counter.store(0, Ordering::Relaxed);
        let response2 = block_on(reqwest::get("http://localhost:32143/status"));
        match response2 {
            Err(_) => {
                assert!(false)
            }
            Ok(resp) => {
                let text = block_on(resp.text()).expect("Response test");
                assert_eq!(text, "OK");
            }
        }
        let response3 = block_on(reqwest::get("http://localhost:32143/unknown_handler")).unwrap();
        assert_eq!(response3.status(), StatusCode::NOT_FOUND);
    }
}
