extern crate actix;
extern crate actix_web;
extern crate notify;

use actix::*;
use actix_web::{server, App, HttpRequest, fs, HttpResponse, Error, ws};

use std::fs::File;
use std::io::prelude::*;
use std::collections::HashSet;

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
        var ws = new WebSocket('ws://' + document.URL.match('.*://([^/]*)/')[1] + '/ws/');
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

    println!("{}", filename);

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


fn main() {
    let sys = actix::System::new("livedevel");

    let client_list = Arbiter::start(|_| ClientList::default());
    let client_list2 = client_list.clone();
    let file_watcher = Arbiter::start(|_| {
        FileWatcher {
            folder: String::from("."),
            requested_files: HashSet::new(),
            client_list: Some(client_list2),
        }

    });

    server::new(move || {
            println!("new server");
            let state = AppState {
                client_list: client_list.clone(),
                file_watcher: file_watcher.clone(),
            };

            App::with_state(state)
                .resource("/ws/", |r| r.route().f(|req| ws::start(req, Ws)))
                //.default_resource(nonstatic_handler)
                .handler("/", fs::StaticFiles::new(".")
                        .unwrap()
                        .default_handler(nonstatic_handler)
                        .show_files_listing())
    })
        .bind("127.0.0.1:8088")
        .unwrap()
        .start();

        let _ = sys.run();
}
