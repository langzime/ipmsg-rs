mod constant;
mod model;
mod message;

#[macro_use]
mod util;
mod chat_window;
mod download;
mod events;
mod fileserver;
mod main_win;
// mod chat_window_new;
pub mod app;

use crossbeam_channel::unbounded;
use once_cell::sync::Lazy;
use crate::events::model::ModelEvent;


///
/// 全局队列
pub static GLOBAL_CHANNEL: Lazy<(crossbeam_channel::Sender<ModelEvent>, crossbeam_channel::Receiver<ModelEvent>)> = Lazy::new(|| {
    let (model_sender, model_receiver): (crossbeam_channel::Sender<ModelEvent>, crossbeam_channel::Receiver<ModelEvent>) = unbounded();
    return (model_sender, model_receiver);
});

///
/// 全局队列发送
pub static GLOBLE_SENDER: Lazy<crossbeam_channel::Sender<ModelEvent>> = Lazy::new(|| {
    return GLOBAL_CHANNEL.0.clone();
});

///
/// 全局队列接收
pub static GLOBLE_RECEIVER: Lazy<crossbeam_channel::Receiver<ModelEvent>> = Lazy::new(|| {
    return GLOBAL_CHANNEL.1.clone();
});