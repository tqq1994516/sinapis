#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{Router, middleware};
    use leptos::{logging::log, prelude::*};
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use regex::Regex;

    // use idgen::{IdGeneratorOptions, YitIdHelper};
    use layer::{auth::auth_middleware, middleware::{sea_orm_connect_extension, redis_connect_extension}};
    use entity::entities::state::OpenApiState;
    use semen_sinapis::app::*;

    let project_dir = std::env::current_dir().unwrap();
    dotenv::from_path(project_dir.join("entity").join(".env")).unwrap();

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    // let mut options = IdGeneratorOptions::New(1);
    // options.WorkerIdBitLength = 6;
    // options.SeqBitLength = 10;
    // YitIdHelper::SetIdGenerator(options);

    let open_api_state = OpenApiState {
        openapi: vec![Regex::new(r"^/$").unwrap(), Regex::new(r"^/api/login").unwrap(), Regex::new(r"^/api/access").unwrap()],
    };

    // 依赖基础extension加在业务中间件后面
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .layer(middleware::from_fn_with_state(open_api_state, auth_middleware))
        .layer(redis_connect_extension().await)
        .layer(sea_orm_connect_extension().await)
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

