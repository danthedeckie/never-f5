extern crate actix;
extern crate actix_web;
extern crate notify;

use std::fmt;

use self::actix::*;

use filewatcher::*;
// --------------------------------
// ClientList Actor:

// Messages from the clients:
#[derive(Message)]
pub struct NewSession {
    pub addr: Recipient<SomethingChanged>,
}

#[derive(Message)]
pub struct EndSession {
    pub addr: Recipient<SomethingChanged>,
}

#[derive(Message)]
pub struct ReloadYall;

pub struct ClientList {
    sessions: Vec<Recipient<SomethingChanged>>,
}

impl fmt::Debug for ClientList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ClientList")
    }
}

impl Actor for ClientList {
    type Context = Context<Self>;
}

impl Handler<NewSession> for ClientList {
    type Result = ();

    fn handle(&mut self, msg: NewSession, _: &mut Context<Self>) -> Self::Result {
        self.sessions.push(msg.addr);
    }
}

impl Handler<EndSession> for ClientList {
    type Result = ();

    fn handle(&mut self, msg: EndSession, _: &mut Context<Self>) -> Self::Result {
        let addr = msg.addr.clone();
        let i = self.sessions.iter().position(|ref a| a == &&addr).unwrap();
        self.sessions.remove(i);
    }
}

impl Handler<ReloadYall> for ClientList {
    type Result = ();

    fn handle(&mut self, _msg: ReloadYall, _ctx: &mut Context<Self>) -> Self::Result {
        self.tell_everyone_to_reload();
    }
}


impl Default for ClientList {
    fn default() -> ClientList {
        ClientList {
            sessions: Vec::new(),
        }
    }
}

impl ClientList {
    fn tell_everyone_to_reload(&self) {
        for addr in self.sessions.iter() {
            addr.do_send(SomethingChanged());
        }
    }
}


