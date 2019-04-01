use std::sync::RwLock;
use gtk::prelude::*;
use gtk::{
    self, CellRendererText, AboutDialog, CheckMenuItem, IconSize, Image, Label, Menu, MenuBar, MenuItem, Window,
    WindowPosition, WindowType, StatusIcon, ListStore, TreeView, TreeViewColumn, Builder, Grid, Button, Orientation,
    ReliefStyle, Widget, TextView, Fixed, ScrolledWindow, Alignment, ButtonBox, WrapMode, Application
};
use lazy_static::lazy_static;

lazy_static!{
    static ref SEND_TASKS: RwLock<Vec<SendTask>> = RwLock::new(Vec::new());
}

enum SendTask {
    PathTask,
    FileTask,
}

struct PathTaskInfo {

}

struct FileTaskInfo {
    send_bytes: u32,
    file_size: u32,
}

#[derive(Clone)]
pub struct ManagerWindow {
    pub win :Window,
    //pub send_lists_store :ListStore,
}

impl ManagerWindow {
    pub fn new(application: &Application) -> ManagerWindow {
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_position(gtk::WindowPosition::Center);

        application.add_window(&window);

        window.connect_delete_event(clone!(window => move|_, _| {
            window.destroy();
            Inhibit(false)
        }));

        window.show_all();

        ManagerWindow {
            win: window
        }
    }
}