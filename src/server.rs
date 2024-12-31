use std::task::Poll;
use bytes::Bytes;
use futures::stream;
use leptos::{config::get_configuration, error::Error, task::Executor};
use leptos_wasi::prelude::{Body, WasiExecutor};
use wasi::{exports::http::incoming_handler::Guest, http::{proxy::export, types::{IncomingRequest, ResponseOutparam}}};
mod bindings {
    wit_bindgen::generate!({ generate_all });
}

use crate::app::{shell, App, Login, Access};
use bindings::wasi::blobstore::{blobstore::{container_exists, create_container, get_container}, types::IncomingValue};
use bindings::wasi::logging::logging::{log, Level};

const CONTAINER_NAME: &str = "pkg";

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
        .static_files_handler(&format!("/{}", CONTAINER_NAME), serve_static_files)

        // When you start writing #[server] functions,
        // be sure to register them there.
        .with_server_fn::<Login>()
        .with_server_fn::<Access>()

        // Fetch all available routes from your App.
        .generate_routes(App)

        // Actually process the request and write the response.
        .handle_with_context( move || shell(leptos_options.clone()), || {}).await.expect("could not handle the request");
}

fn serve_static_files(path: String)
    -> Option<Body>
{
    log(Level::Info, "", &format!("access file path:{}", path));
    let container = if container_exists(&CONTAINER_NAME.to_string()).unwrap() {
        log(Level::Debug, "", "Container already exists, fetching ...");
        get_container(&CONTAINER_NAME.to_string()).expect("container to exist")
    } else {
        log(Level::Error, "", "Container already exists, fetching ...");
        create_container(&CONTAINER_NAME.to_string()).expect("to be able to create container")
    };
    let object_name = path.strip_prefix("/").unwrap_or(&path).to_string();
    if let Ok(result) = container.has_object(&object_name) {
        if result {
            let metadata = container.object_info(&object_name).unwrap();
            let object = container.get_data(&object_name, 0, metadata.size).unwrap();
            if let Ok(body) = IncomingValue::incoming_value_consume_async(object) {
                let mut read_bytes: u64 = 0;
                Some(
                    Body::Async(
                        Box::pin(stream::poll_fn(move |_| -> Poll<Option<Result<Bytes, Error>>> {
                            if read_bytes >= metadata.size {
                                return Poll::Ready(None)
                            }

                            match body.blocking_read(256) {
                                Err(err) => Poll::Ready(Some(Err(err.into()))),
                                Ok(data) => {
                                    read_bytes += data.len() as u64;
                                    Poll::Ready(Some(Ok(Bytes::from(data))))
                                }
                            }
                        }))
                    )
                )
            } else {
                log(Level::Error, "", "Failed to convert object to bytes");
                None
            }
        } else {
            log(Level::Error, "", "Failed to find object");
            None
        }
    } else {
        log(Level::Error, "", "Failed to check if object exists");
        None
    }
}

export!(LeptosServer with_types_in wasi);
