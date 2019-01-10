extern crate actix;
extern crate actix_web;
extern crate notify;

use self::actix::*;
use self::actix_web::{ws};
use self::actix::AsyncContext;

use appstate::*;
use clientlist::*;
use filewatcher::*;

pub struct Ws;

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self, AppState>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println! ( "started!");
        let addr = ctx.address();
        ctx.state()
            .client_list
            .send(NewSession { addr: addr.recipient() })
            .into_actor(self)
            .then(|_res, _act, _ctx| { fut::ok(()) })
            .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        // tell client_list...
        ctx.state().client_list.do_send(EndSession { addr: ctx.address().recipient() });
        Running::Stop
    }
}

impl Handler<SomethingChanged> for Ws {
    type Result = ();

    fn handle(&mut self, _msg: SomethingChanged, ctx: &mut ws::WebsocketContext<Self, AppState>) {
        // self.0.text("something changed!");
        ctx.text("something changed!")
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for Ws {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Binary(bin) => ctx.binary(bin),
            _ => (),
        }
    }
}


