use std::fs::{File, read_to_string};
use std::io::prelude::*;

use actix_web::{web, http, App, HttpRequest, HttpServer, HttpResponse, Error};
use actix_files as fs;
use actix_web_actors::ws;

use listenfd::ListenFd;
use actix_service::Service;
use futures::future::Future;

use actix::prelude::*;

#[macro_use]
extern crate structopt;

use std::path::PathBuf;
use structopt::StructOpt;

mod filewatcher;
use crate::filewatcher::*;

mod websockets;
use crate::websockets::*;

////////////////////////////////////////////////////////////////////////////////
/// Main Commandline Options:
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, StructOpt, Clone)]
#[structopt(name="serveme", about="Autoreloading web development server")]
pub struct Options {
    /// Address + port
    #[structopt(short="a", long="address", default_value="127.0.0.1:8088")]
    address: String,
    /// Quiet mode (Doesn't do much yet)
    #[structopt(short="q", long="quiet")]
    quiet: bool,
    /// File System Event Debounce Time
    #[structopt(short="d", long="debounce", default_value="10")]
    debouncetime: u64,
    /// JS Injection file
    #[structopt(short="j", long="javascript-file")]
    js: Option<PathBuf>,
}

////////////////////////////////////////////////////////////////////////////////
// Static files adding the websockets injection:
////////////////////////////////////////////////////////////////////////////////

const WS_INJECTION: &str = "
    <script>
        (function() {
        var ws_url = 'ws://' + document.URL.match('.*://([^/]*)/')[1] + '/!!/';
        var ws = new WebSocket(ws_url);
        var reload = function() {
            window._autoreload_save && window._autoreload_save();
            document.location.replace(document.URL);
        };

        ws.onmessage = reload;
        ws.onclose = function(evt) {
            console.log('Server disconnected! Retrying...');
            setInterval(function() {
                var ws = new WebSocket(ws_url);
                ws.onopen = reload;
            }, 1000);
        };
        window._autoreload_load && window._autoreload_load();
        })();
    </script>";

fn nonstatic_handler(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let uri = req.path();

    let filename: &str = if uri.ends_with("!") {
        &uri[1..uri.len()-1]
    } else { "404.html" };

    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    if uri.ends_with("!") {
        if let Some(js_filename) = &data.config.js {
            let js_file = read_to_string(js_filename);
            if let Ok(js_contents) = js_file {
                contents.push_str("<script>");
                contents.push_str(&js_contents);
                contents.push_str("</script>");
            }

        } else {
            contents.push_str(WS_INJECTION);
        }
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
    pub config: Options,
}

pub fn ws_route(req: HttpRequest, stream: web::Payload, srv: web::Data<AppState>) -> Result<HttpResponse, Error> {
    if ! srv.config.quiet {
        println!("client connected.");
    }
    ws::start( websockets::Client { clientlist: srv.clientlist.clone() }, &req, stream)
}


fn start_server(options: Options) {
    let mut listenfd = ListenFd::from_env();

    let _sys = System::new("example");
    let my_clientlist = ClientList::start_default();
    let c2 = my_clientlist.clone();

    let my_watcher = WatcherHandler::new(".", my_clientlist.recipient(), options.debouncetime).start();

    if let Some(js_file) = &options.js {
        if let Some(js_filestring) = js_file.to_str() {
        my_watcher.do_send(PleaseWatch{filename: js_filestring.to_string()});
        }
    }

    let state = web::Data::new(AppState {
        watcher: my_watcher,
        clientlist: c2,
        config: options.clone(),
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
                    state2.watcher.do_send(PleaseWatch{filename: String::from(req.path())});
                    srv.call(req).map(|mut res| {
                        let headers = res.headers_mut();
                        headers.insert(http::header::CACHE_CONTROL,
                                       http::header::HeaderValue::from_static("no-cache"));
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
        server.bind(&options.address).unwrap()
    };

    if ! &options.quiet {
        println!("Listening on {}", &options.address);
    }

    server.run().unwrap();
}

fn main() {
    let options = Options::from_args();
    println!("{:?}", options);
    start_server(options);
}
