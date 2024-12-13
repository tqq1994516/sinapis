use leptos::{config::get_configuration, task::Executor};
use leptos_wasi::prelude::{Body, WasiExecutor};
use wasi::{exports::http::incoming_handler::Guest, http::{proxy::export, types::{IncomingRequest, ResponseOutparam}}};

use crate::app::{shell, App};

struct LeptosServer;

impl Guest for LeptosServer {
    fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
        // Initiate a single-threaded [`Future`] Executor so we can run the
        // rendering system and take advantage of bodies streaming.
        let executor = WasiExecutor::new(leptos_wasi::executor::Mode::Stalled);
        Executor::init_local_custom_executor(executor.clone()).expect("cannot init future executor");
        executor.run_until(async {
            handle_request(request, response_out).await;
        })
    }
}

async fn handle_request(request: IncomingRequest, response_out: ResponseOutparam) {
    use leptos_wasi::prelude::Handler;
    
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;

    Handler::build(request, response_out)
        .expect("could not create handler")

        // All static assets should be served on /pkg/...
        // when the user request this path, the passed function is called
        .static_files_handler("/pkg", serve_static_files)

        // When you start writing #[server] functions,
        // be sure to register them there.
        //.with_server_fn::<YourServerFunction>()

        // Fetch all available routes from your App.
        .generate_routes(App)

        // Actually process the request and write the response.
        .handle_with_context( move || shell(leptos_options.clone()), || {}).await.expect("could not handle the request");
}

fn serve_static_files(_path: String)
    -> Option<Body>
{
    // This function will be called everytime the user request a static assets.
    // If you want to see an example using wasi:filesystem to fetch the static
    // assets, see:
    // https://github.com/raskyld/leptos-wasmcloud/blob/47fe3dc6ee297e6e9b1a3fe136b62d5d9420af7c/src/server.rs#L42
    todo!("You must write your logic for static assets!");
}

export!(LeptosServer with_types_in wasi);
