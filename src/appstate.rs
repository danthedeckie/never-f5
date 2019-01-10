extern crate actix;
use self::actix::*;

use clientlist;
use filewatcher;

pub struct AppState {
    pub client_list: Addr<clientlist::ClientList>,
    pub file_watcher: Addr<filewatcher::FileWatcher>,
}


