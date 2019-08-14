use actix::prelude::*;
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

    fn handle(&mut self, evt: SomethingChanged, _ctx: &mut Self::Context) {
        let client_event = ClientMsgSomethingChanged { filename: evt.filename };
        for client in self.sessions.iter() {
            match client.try_send(client_event.clone()) {
                Ok(()) => (),
                Err(e) => eprintln!("Error sending to client. {:?}", e),
            }

        }
    }
}

impl Handler<ClientConnect> for ClientList {
    type Result = ();
    fn handle(&mut self, msg: ClientConnect, _: &mut Context<Self>) -> Self::Result {
        self.sessions.push(msg.addr);
        println!("{} clients connected", self.sessions.len());
    }
}

impl Handler<ClientDisconnect> for ClientList {
    type Result = ();
    fn handle(&mut self, msg: ClientDisconnect, _: &mut Context<Self>) -> Self::Result {
        if let Some(i) = self.sessions.iter().position(|x| x == &msg.addr) {
            self.sessions.swap_remove(i);
        }
        println!("{} clients connected", self.sessions.len());
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
        self.clientlist.do_send(ClientConnect { addr: ctx.address().recipient() });
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        self.clientlist.do_send(ClientDisconnect { addr: ctx.address().recipient() });
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
        match msg {
            ws::Message::Close(_) => {
                ctx.stop();
            },
            _ => {
                println!("WS Message from client: {:?}", msg);
            }
        };
    }
}
