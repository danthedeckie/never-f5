use std::collections::{HashMap, HashSet};

use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::filewatcher::{SomethingChanged};

#[derive(Clone)]
struct ClientMsgSomethingChanged {
    pub filename: String,
}

impl Message for ClientMsgSomethingChanged { type Result = (); }

struct ClientConnect { pub addr:Recipient<ClientMsgSomethingChanged> }
impl Message for ClientConnect { type Result = (); }
struct ClientDisconnect { pub addr:Recipient<ClientMsgSomethingChanged> }
impl Message for ClientDisconnect { type Result = (); }

pub struct ClientList {
    sessions: Vec<Recipient<ClientMsgSomethingChanged>>,
}

impl Default for ClientList {
    fn default() -> ClientList {
        ClientList { sessions: vec!() }
    }
}

impl Actor for ClientList {
    type Context = Context<Self>;
}

impl Handler<SomethingChanged> for ClientList {
    type Result = ();

    fn handle(&mut self, evt: SomethingChanged, ctx: &mut Self::Context) {
        let client_event = ClientMsgSomethingChanged { filename: evt.filename };
        for client in self.sessions.iter() {
            client.try_send(client_event.clone());
        }
    }
}

impl Handler<ClientConnect> for ClientList {
    type Result = ();
    fn handle(&mut self, msg: ClientConnect, _: &mut Context<Self>) -> Self::Result {
        self.sessions.push(msg.addr);
    }
}

impl Handler<ClientDisconnect> for ClientList {
    type Result = ();
    fn handle(&mut self, msg: ClientDisconnect, _: &mut Context<Self>) -> Self::Result {
        // TODO
    }
}

///////////////////
// And here is the individual session object:

pub struct Client {
    pub clientlist: Addr<ClientList>,
}

impl Actor for Client {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.clientlist.try_send(ClientConnect { addr: ctx.address().recipient() });
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        self.clientlist.try_send(ClientDisconnect { addr: ctx.address().recipient() });
        Running::Stop
    }
}

impl Handler<ClientMsgSomethingChanged> for Client {
    type Result = ();
    fn handle(&mut self, msg: ClientMsgSomethingChanged, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.filename);
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for Client {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        println!("WS Message: {:?}", msg);
    }
}
