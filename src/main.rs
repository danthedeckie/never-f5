use std::collections::HashSet;
use std::sync::Mutex;
use std::fs::File;
use std::io::prelude::*;

use actix_web::{web, http, App, HttpRequest, HttpServer, HttpResponse, Error};
use actix_files as fs;
use actix_web_actors::ws;

use listenfd::ListenFd;
use actix_service::Service;
use futures::future::Future;

use actix::dev::{MessageResponse, ResponseChannel};
use actix::prelude::*;

mod filewatcher;
use crate::filewatcher::*;

mod websockets;
use crate::websockets::*;

////////////////////////////////////////////////////////////////////////////////
// Static files adding the websockets injection:
////////////////////////////////////////////////////////////////////////////////

const WS_INJECTION: &str = "
    <script>
        var ws = new WebSocket('ws://' + document.URL.match('.*://([^/]*)/')[1] + '/!!/');
        ws.onmessage = function (evt) {
            document.location.replace(document.URL);
            // document.location.reload(true);
        }
    </script>";

fn nonstatic_handler(req: HttpRequest) -> Result<HttpResponse, Error> {
    let uri = req.path();

    let filename: &str = if uri.ends_with("!") {
        &uri[1..uri.len()-1]
    } else { "404.html" };

    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    if uri.ends_with("!") {
        contents.push_str(WS_INJECTION);
    }
    Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(contents))
}



////////////////////////////////////////////////////////////////////////////////
// The actual server stuff:
////////////////////////////////////////////////////////////////////////////////
pub struct AppState {
    pub watcher: Addr<WatcherHandler>,
    pub clientlist: Addr<websockets::ClientList>,
}

pub fn ws_route(req: HttpRequest, stream: web::Payload, srv: web::Data<AppState>) -> Result<HttpResponse, Error> {
    println!("client connected.");
    ws::start( websockets::Client { clientlist: srv.clientlist.clone() }, &req, stream)
}

fn main() {
    let mut listenfd = ListenFd::from_env();

    let sys = System::new("example");
    let my_clientlist = ClientList::start_default();
    let c2 = my_clientlist.clone();

    let my_watcher = WatcherHandler::new(".", my_clientlist.recipient()).start();

    let state = web::Data::new(AppState {
        watcher: my_watcher,
        clientlist: c2,
    });


    let mut server = HttpServer::new(
        move || {
            let state2 = state.clone();
            App::new()
                .register_data(state.clone())
                .data(state.clone())
                // websocket handler:
                .service(web::resource("/!!/").route(web::get().to(ws_route)))
                // Middleware to store all requested filenames in state.requested_files
                .wrap_fn(move |req, srv| {
                    state2.watcher.try_send(PleaseWatch{filename: String::from(req.path())});
                    // TODO: add no-cache headers!
                    srv.call(req).map(|mut res| {
                        let mut headers = res.headers_mut();
                        headers.insert(http::header::CACHE_CONTROL, http::header::HeaderValue::from_static("no-cache"));
                        res
                    })
                })
                // And register our routes:
                .route("/{path:.*}!", web::get().to(nonstatic_handler))
                // Handle all static files:
                .service(fs::Files::new("", ".").show_files_listing())
            });

    // Start server with reloading magic:

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l).unwrap()
    } else {
        server.bind("127.0.0.1:3000").unwrap()
    };

    server.run().unwrap();
}
