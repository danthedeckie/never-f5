extern crate actix;
use self::actix::*;


use clientlist;
use filewatcher;

#[derive(Debug)]
pub struct AppState {
    pub client_list: Addr<clientlist::ClientList>,
    pub file_watcher: Addr<filewatcher::FileWatcher>,
}

