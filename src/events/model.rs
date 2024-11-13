use crate::constants::protocol;
use crate::core::download::{ManagerPool, PoolFile};
use crate::core::fileserver::FileServer;
use crate::core::GLOBLE_SENDER;
use crate::models::event::{ModelEvent, UiEvent};
use crate::models::model::{Packet, ShareInfo};
use combine::parser::Parser;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::mpsc::UnboundedSender;

pub fn model_run(socket: UdpSocket, ui_event_sender: UnboundedSender<UiEvent>) {
    let file_pool: Arc<Mutex<Vec<ShareInfo>>> = Arc::new(Mutex::new(Vec::new()));

    let file_server = FileServer::new(file_pool.clone());

    file_server.run();

    let download_pool: Arc<Mutex<HashMap<u32, PoolFile>>> = Arc::new(Mutex::new(HashMap::new()));

    let manager_pool = ManagerPool::new(download_pool);

    send_ipmsg_br_entry();
}

pub fn send_ipmsg_br_entry() {
    thread::spawn(move || {
        let packet = Packet::new(
            protocol::IPMSG_BR_ENTRY | protocol::IPMSG_BROADCASTOPT,
            Some(format!("{}\0\n{}", *protocol::HOST_NAME, *protocol::HOST_NAME)),
        );
        GLOBLE_SENDER.send(ModelEvent::BroadcastEntry(packet)).unwrap();
        thread::sleep(std::time::Duration::from_secs(20));
    });
}
