use std::{thread, sync::mpsc::Sender, time::Duration, net::TcpListener};

use rand::Rng;

pub enum ThreadMessage {
    Error(String),
    Warning(String),
    Info(String),
    PlotPoint(f64),
    PlotOnLine(String, f64)
}
pub struct Server {
    running: bool,
    ip: String,
    port: String,
}

impl Server {
    pub fn new() -> Self {
        Self {
            running: false,
            ip: "127.0.0.1".to_owned(),
            port: "6969".to_owned(),
        }
    }

    pub fn start(&mut self, sender: Sender<ThreadMessage>) {
        if self.running {
            return;
        }

        self.running = true;

        let address = format!("{}:{}", self.ip, self.port);
        
        println!("Starting server on {}", address);

        thread::spawn(move || {
            println!("In the thread...");
            let _listener = match TcpListener::bind(&address) {
                Ok(listener) => listener,
                Err(_) => {
                    sender.send(ThreadMessage::Error(format!("Invalid IP or port ({})", &address))).unwrap();
                    return;
                },
            };

            let mut rng = rand::thread_rng();

            loop {
                sender.send(ThreadMessage::PlotOnLine("mKmvITs70dbf9X2o".to_owned(), rng.gen_range(-16.0..16.0))).unwrap();
                thread::sleep(Duration::from_millis(100));
            }
        });
    }
}