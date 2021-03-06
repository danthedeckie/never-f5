use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use std::fs::canonicalize;
use std::time::Duration;

use actix::prelude::*;
use actix::Arbiter;
use actix;

use crossbeam_channel::{unbounded, Sender, Receiver, RecvError};
use notify::{RecommendedWatcher, RecursiveMode, Result as NResult, Watcher, Event};

// Messages:

pub struct PleaseWatch {
    pub filename: String,
}

impl Message for PleaseWatch { type Result = Result<bool, io::Error>; }

#[derive(Debug)]
pub struct SomethingChanged {
    pub filename: String,
}

impl Message for SomethingChanged { type Result = (); }


// The actor:
pub struct WatcherHandler{
    watchdir: PathBuf,
    watching: HashSet<String>,
    arbiter: Arbiter,
    channel: (Sender<NResult<Event>>, Receiver<NResult<Event>>),
    clientlist: Recipient<SomethingChanged>,
    watcher: RecommendedWatcher,
}

impl Actor for WatcherHandler {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        self.start_watching(ctx.address().recipient()) ;//&self.recipient());
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        self.arbiter.stop();
    }
}

impl WatcherHandler {
    pub fn new(watchdir: &str, clientlist: Recipient<SomethingChanged>, debouncetime: u64) -> WatcherHandler {
        let (tx, rx) = unbounded();
        let mut watcher: RecommendedWatcher = Watcher::new(tx.clone(), Duration::from_millis(debouncetime)).unwrap();

        let a = Arbiter::new();

        let full_pathname = canonicalize(watchdir).unwrap();

        watcher.watch(&full_pathname, RecursiveMode::Recursive).unwrap();

        WatcherHandler {
            watchdir: full_pathname,
            watching: HashSet::new(),
            channel: (tx, rx),
            arbiter: a,
            clientlist: clientlist,
            watcher: watcher,
        }
    }

    fn start_watching(&mut self, addr: Recipient<SomethingChanged>) {
        let me = addr.clone();
        let (_, rx2) = &self.channel; //.clone();
        let rx = rx2.clone();

        self.arbiter.exec_fn(move || {
            loop {
                match rx.recv() {
                    Ok(Ok(event)) => {
                        for path in event.paths {

                            let result = me.try_send(SomethingChanged {filename: String::from(path.to_str().unwrap()) });
                            match result {
                                Ok(()) => (),
                                Err(e) => {
                                    eprintln!("Error telling main thread: {:?}", e);
                                    break
                                },
                            }
                        }
                    },
                    Ok(Err(err)) => eprintln!("recieved an error? {:?}", err),
                    Err(RecvError) => {
                        // Channel Disconnected. Goodbye!
                        break
                    },
                    Err(err) => {
                        eprintln!("watch error... {:?}", err)
                    },
                };
            }
        });
    }
}

impl Handler<SomethingChanged> for WatcherHandler {
    type Result = ();

    fn handle(&mut self, evt: SomethingChanged, _ctx: &mut Self::Context) {
        if self.watching.contains(&evt.filename) {
            match self.clientlist.try_send(evt) {
                Ok(()) => (),
                Err(e) => eprintln!("error sending to clientlist! {:?}", e),
            }
        }
    }

}


impl Handler<PleaseWatch> for WatcherHandler {
    type Result = Result<bool, io::Error>;

    fn handle(&mut self, msg: PleaseWatch, _ctx: &mut Context<Self>) -> Self::Result {
        if let Some(fullname) = self.watchdir.join(msg.filename.trim_matches('/').trim_end_matches('!')).to_str() {
            self.watching.insert(fullname.to_string());
        } else {
            eprintln!("couldn't add path...");
        }
        println!("Watching: {:?}", self.watching);
        Ok(true)
    }
}
