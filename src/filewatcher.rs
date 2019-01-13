extern crate actix;
extern crate actix_web;
extern crate notify;

use self::actix::*;
use std::collections::HashSet;

use self::notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::env::{current_dir};

use clientlist;

//  Internal 'real' filewatcher.

#[derive(Message)]
struct FileChanged{
    pub filename: String,
}

#[derive(Message)]
struct HereIAm{
    pub addr: Addr<FileWatcher>,
}

#[derive(Debug)]
pub struct UnreachableFileWatcher {
    pub file_watcher: Option<Addr<FileWatcher>>,
}

impl Actor for UnreachableFileWatcher {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
    }
}

impl Handler<HereIAm> for UnreachableFileWatcher {
    type Result = ();

    fn handle(&mut self, here: HereIAm, _: &mut Context<Self>) -> Self::Result {
        self.file_watcher = Some(here.addr);
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

        watcher.watch(".", RecursiveMode::Recursive).unwrap();

        loop {
            match rx.recv() {
                Ok(event) => {
                    // println!("{:?}", event);
                    let path = match event {
                        DebouncedEvent::NoticeWrite(p) => Some(p),
                        DebouncedEvent::Create(p) => Some(p),
                        DebouncedEvent::Write(p) => Some(p),
                        DebouncedEvent::Rename(_, p) => Some(p),
                        _ => None,
                    };
                    if let Some(p) = path {
                        if let Some(pp) = p.to_str() {
                            //ctx.state().file_watcher.do_send(FileChanged{filename: String::from(pp)});
                            if let Some(ref fw) = self.file_watcher {
                                fw.do_send(FileChanged{filename: String::from(pp)});
                            } else {
                                println!("ERR: There is no attached file watcher!");
                            }
                        }
                    }
                },
                Err(e) => println!("WErr: {:?}", e),
            }
        }


    }
}

// --------------------------------
// File Watcher Actor:

#[derive(Message)]
pub struct SomethingChanged();

#[derive(Message)]
pub struct PleaseWatch {
    pub filename: String,
}

#[derive(Debug)]
pub struct FileWatcher {
    pub requested_files: HashSet<String>,
    pub client_list: Option<Addr<clientlist::ClientList>>,
    realwatcher: Addr<UnreachableFileWatcher>,
    cdir: String,
}

impl Actor for FileWatcher {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.realwatcher.do_send(HereIAm{ addr: addr});
    }
}

impl Handler<PleaseWatch> for FileWatcher {
    type Result = ();

    fn handle(&mut self, pls: PleaseWatch, _: &mut Context<Self>) -> Self::Result {
        let mut filename = pls.filename;
        if filename.ends_with("!") {
            filename.pop();
        }
        filename.insert_str(0, &self.cdir);

        self.requested_files.insert(filename);
        println!("watching: {:?}", self.requested_files);
    }
}

impl Handler<FileChanged> for FileWatcher {
    type Result = ();

    fn handle(&mut self, file: FileChanged, _: &mut Context<Self>) -> Self::Result {
        if self.requested_files.contains(&file.filename) {
            if let Some(clientlist) = self.client_list.clone() {
                clientlist.do_send(clientlist::ReloadYall);
            }
        }
    }
}

impl FileWatcher {
    pub fn new(dir: &str, client_list: Addr<clientlist::ClientList>, realwatcher: Addr<UnreachableFileWatcher>) -> FileWatcher {
        FileWatcher {
            requested_files: HashSet::new(),
            client_list: Some(client_list),
            realwatcher: realwatcher,
            cdir: String::from(current_dir().unwrap().to_str().unwrap()),
            }
    }
}

impl UnreachableFileWatcher {
    pub fn new(dir: &str) -> UnreachableFileWatcher {
        UnreachableFileWatcher {
            file_watcher: None,
        }
    }
}
