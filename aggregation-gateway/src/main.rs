#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use idgen::{IdGeneratorOptions, YitIdHelper};
    use layer::auth::AuthLayer;

    use super::app::*;

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let mut options = IdGeneratorOptions::New(1);
    options.WorkerIdBitLength = 6;
    options.SeqBitLength = 10;
    YitIdHelper::SetIdGenerator(options);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let app = Router::new().layer(AuthLayer {
        auth_service: app,
        unauth_service: app,
        open_apis: vec!["/1", "/b"],
    });

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}


// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let address = std::env::var("ADDR")?.parse()?;
//     axum::Server::bind(&address)
//         .serve(register_route().await.into_make_service())
//         .await?;

//     Ok(())
// }

