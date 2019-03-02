#![crate_type = "lib"]
#![crate_name = "raudient"]

//#![feature(trace_macros, log_syntax)]

//trace_macros!(true);

extern crate gtk;
extern crate gio;
extern crate glib;
extern crate chrono;
extern crate encoding;
//#[macro_use]
//extern crate generator;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate env_logger;
#[macro_use] extern crate quick_error;


mod constant;
mod model;
mod message;

#[macro_use]
mod util;
mod chat_window;
mod download;
mod events;
mod fileserver;
pub mod app;

