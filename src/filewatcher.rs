extern crate actix;
extern crate actix_web;
extern crate notify;

use self::actix::*;
use std::collections::HashSet;

use self::notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use std::sync::mpsc::channel;
use std::time::Duration;

use clientlist;

// --------------------------------
// File Watcher Actor:

#[derive(Message)]
pub struct SomethingChanged();


// TODO.

pub struct FileWatcher {
    pub folder: String,
    pub requested_files: HashSet<String>,
    pub client_list: Option<Addr<clientlist::ClientList>>,
}

impl Actor for FileWatcher {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("hello world (from FileWatcher)");
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

        watcher.watch(".", RecursiveMode::Recursive).unwrap();

        loop {
            match rx.recv() {
                Ok(event) => {
                    println!("{:?}", event);
                    let path = match event {
                        DebouncedEvent::NoticeWrite(p) => Some(p),
                        DebouncedEvent::Write(p) => Some(p),
                        DebouncedEvent::Rename(_, p) => Some(p),
                        _ => None,
                    };

                    match self.client_list.clone() {
                        None => (),
                        Some(client_list) => { client_list.do_send(clientlist::ReloadYall) }
                    }
                },
                Err(e) =>println!("WErr: {:?}", e),
            }
        }

    }
}

impl Default for FileWatcher {
    fn default() -> FileWatcher {
        FileWatcher {
            folder: String::from("."),
            requested_files: HashSet::new(),
            client_list: None,
        }
    }
}


