use leptos::{prelude::*, task::spawn_local};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use leptos_use::storage::{use_local_storage_with_options, UseStorageOptions};
use codee::string::FromToStringCodec;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options=options islands=true/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/semen-sinapis.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <h1>"Welcome to Leptos!"</h1>
        <GetId/>
    }
}

#[island]
fn GetId() -> impl IntoView {
    // Creates a reactive value to update the button
    let (user_id, set_user_id, _) = use_local_storage_with_options::<i64, FromToStringCodec>(
        "user_id",
        UseStorageOptions::default().delay_during_hydration(true),
    );
    let login_action = ServerAction::<Login>::new();

    // let access = Resource::new(move || login_action.value().get(), move |u| access(u.map_or(0, |iu| iu.unwrap_or(0))));

    view! {
        <button on:click=move |_| {
            spawn_local(async {
                let res = login().await;
                if let Ok(res) = res {
                    println!("{}", res);
                    // let (_, set_user_id, _) = use_local_storage_with_options::<i64, FromToStringCodec>(
                    //     "user_id",
                    //     UseStorageOptions::default().delay_during_hydration(true),
                    // );
                    // set_user_id(res);
                    let _ = access(res).await;
                };
            });
        }>login</button>
    }
}

#[server]
pub async fn login() -> Result<i64, ServerFnError> {
    use axum::{Extension, http::StatusCode};
    use leptos_axum::{extract, redirect, ResponseOptions};
    use bb8::Pool;

    use entity::user_property;
    use pool::grpc::person_center::PersonCenterGrpcClientManager;

    redirect("/");
    let person_center_pool: Extension<Pool<PersonCenterGrpcClientManager>> = extract().await?;
    let person_center_client = person_center_pool.get().await.unwrap();

    match user_property::Entity::find_by_id(1).one(&pg_conn.0).await? {
        Some(user) => {
            let response = expect_context::<ResponseOptions>();

            // set the HTTP status code
            response.set_status(StatusCode::ACCEPTED);
            Ok(user.id)
        },
        None => Ok(0),
    }
}

#[server]
pub async fn access(user_id: i64) -> Result<(), ServerFnError> {
    println!("{}", user_id);

    Ok(())
}
