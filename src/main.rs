extern crate actix;
extern crate actix_web;
extern crate notify;

use actix::*;
use actix_web::{server, App, HttpRequest, fs, HttpResponse, Error, ws};
use actix_web::middleware::{Middleware, Started, Response};
use actix_web::http::{header};

use std::fs::File;
use std::io::prelude::*;

mod filewatcher;
use filewatcher::*;

mod clientlist;
use clientlist::*;

mod websocket;
use websocket::*;

mod appstate;
use appstate::*;

const WS_INJECTION: &str = "
    <script>
        var ws = new WebSocket('ws://' + document.URL.match('.*://([^/]*)/')[1] + '/!!/');
        ws.onmessage = function (evt) {
            document.location.replace(document.URL);
            // document.location.reload(true);
        }
    </script>";

fn nonstatic_handler(_req: &HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    let uri = _req.path();

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

// Middleware

struct CatchFilepath;
impl Middleware<AppState> for CatchFilepath {
    fn start(&self, req: &HttpRequest<AppState>) -> Result<Started, Error> {
        let ref watcher = req.state().file_watcher;
        if watcher.connected() {
            let m = watcher.do_send( PleaseWatch { filename: String::from(req.path()) });
        } else {
            println!("not connected...");
        }
        Ok(Started::Done)
    }

    // Disabe Caching... (TODO should be a separate middleware)
    fn response(&self, req: &HttpRequest<AppState>, mut resp: HttpResponse)
        -> Result<Response, Error>
    {
        resp.headers_mut().insert(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("no-cache"));
        Ok(Response::Done(resp))
    }
}


// Main:
fn main() {
    let sys = actix::System::new("livedevel");

    let client_list = Arbiter::start(|_| ClientList::default());
    let client_list2 = client_list.clone(); // address, so we can send it to
                                            // the other thread in the closure.
    let real_file_watcher = Arbiter::start(|_| :: UnreachableFileWatcher::new("."));

    let file_watcher = Arbiter::start(|_| FileWatcher::new(".", client_list2, real_file_watcher));

    server::new(move || {
            let state = AppState {
                client_list: client_list.clone(),
                file_watcher: file_watcher.clone(),
            };

            App::with_state(state)
                .resource("/!!/", |r| r.route().f(|req| ws::start(req, Ws)))
                //.default_resource(nonstatic_handler)
                .middleware(CatchFilepath)
                .handler("/", fs::StaticFiles::new(".")
                        .unwrap()
                        .default_handler(nonstatic_handler)
                        .show_files_listing())
    })
        .bind("127.0.0.1:8088")
        .unwrap()
        .start();

        println!("Listening on 127.0.0.1:8088");

        let _ = sys.run();
}
