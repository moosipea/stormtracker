use std::{thread, io::BufReader, sync::mpsc::Sender, time::Duration, net::TcpListener};

pub enum ThreadMessage {
    Error(String),
    Warning(String),
    Info(String),
    PlotPoint(f64),
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
            let listener = match TcpListener::bind(&address) {
                Ok(listener) => listener,
                Err(_) => {
                    sender.send(ThreadMessage::Error(format!("Invalid IP or port ({})", &address))).unwrap();
                    return;
                },
            };

            loop {
                sender.send(ThreadMessage::Info("hello".to_owned())).unwrap();
                thread::sleep(Duration::from_millis(1000));
            }

            // TODO: yea i dont know how threads work ig
            /*println!("Crashy part...");
            for stream in listener.incoming() {
                let mut stream = match stream {
                    Ok(stream) => stream,
                    Err(_) => {
                        sender.send(ThreadMessage::Error("Invalid stream".to_owned())).unwrap();
                        return;
                    },
                };

                let buf_reader = BufReader::new(&mut stream);
                let request = String::from_utf8(buf_reader.buffer().to_vec()).unwrap();
                println!("{}", request);
            }*/
        });
    }
}