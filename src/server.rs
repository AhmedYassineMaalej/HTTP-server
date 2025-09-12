use std::net::{SocketAddr, TcpListener};

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn serve(port: u16) -> Self {
        Self {
            listener: TcpListener::bind(("127.0.0.1", port)).unwrap(),
        }
    }

    pub fn close() {
        todo!()
    }

    pub fn listen() {
        todo!()
    }

    pub fn handle() {
        todo!()
    }
}
