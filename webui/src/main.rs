use cfg_if::cfg_if;

// boilerplate to run in different modes
cfg_if! {
if #[cfg(feature = "ssr")] {
    use axum::{
        response::{Response, IntoResponse},
        routing::get,
        extract::{Path, State, RawQuery},
        http::{Request, header::HeaderMap},
        body::Body as AxumBody,
        Router,
    };
    use webui::app::*;
    use webui::auth::*;
    use webui::AppState;
    use webui::fileserv::file_and_error_handler;
    use leptos_axum::{generate_route_list, LeptosRoutes, handle_server_fns_with_context};
    use leptos::{log, view, provide_context, get_configuration};
    use surrealdb::Surreal;
    // use surrealdb::engine::remote::http::Client;
    // use surrealdb::engine::remote::http::Http;
    use surrealdb::engine::remote::ws::Client;
    use surrealdb::engine::remote::ws::Ws;
    use surrealdb::opt::auth::Namespace;
    use axum_session::{SessionConfig, SessionLayer, SessionStore};
    use axum_session_auth::{AuthSessionLayer, AuthConfig, SessionSurrealPool};

    async fn server_fn_handler(
        State(app_state): State<AppState>,
        auth_session: AuthSession,
        path: Path<String>,
        headers: HeaderMap,
        raw_query: RawQuery,
        request: Request<AxumBody>
    ) -> impl IntoResponse {

        log!("{:?}", path);

        handle_server_fns_with_context(path, headers, raw_query, move || {
            provide_context(auth_session.clone());
            provide_context(app_state.db.clone());
        }, request).await
    }

    async fn leptos_routes_handler(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
        req: Request<AxumBody>
    ) -> Response {
            let handler =
                leptos_axum::render_app_to_stream_with_context(app_state.leptos_options.clone(),
            move || {
                provide_context(auth_session.clone());
                provide_context(app_state.db.clone());
            },
            || view! { <App/> }
        );
        handler(req).await.into_response()
    }

    #[tokio::main]
    async fn main() {
        simple_logger::init_with_level(log::Level::Info).expect("couldn't initialize logging");

        let db = Surreal::new::<Ws>("127.0.0.0:8080").await.expect("Cannot connect to DB");
        db.signin(Namespace {
            namespace:"hadijatek",
            username:"hadijatek",
            password:"hadijatek",
        }).await.expect("Cannot sign in to hadijatek");
        db.use_ns("hadijatek").use_db("auth").await.expect("Could not use auth DB");

        // Auth section
        let session_config = SessionConfig::default().with_table_name("axum_sessions");
        let auth_config = AuthConfig::<String>::default();
        let session_store = SessionStore::<SessionSurrealPool<Client>>::new(
            Some(db.clone().into()),
            session_config
        ).await.unwrap();
        session_store.initiate().await.unwrap();

        // Setting this to None means we'll be using cargo-leptos and its env vars
        let conf = get_configuration(None).await.unwrap();
        let leptos_options = conf.leptos_options;
        let addr = leptos_options.site_addr;
        let routes = generate_route_list(|| view! { <App/> }).await;

        let app_state = AppState{
            leptos_options,
            db: db.clone(),
        };

        // build our application with a route
        let app = Router::new()
        .route("/api/*fn_name", get(server_fn_handler).post(server_fn_handler))
        .leptos_routes_with_handler(routes, get(leptos_routes_handler) )
        .fallback(file_and_error_handler)
        .layer(AuthSessionLayer::<User, String, SessionSurrealPool<Client>, Surreal<Client>>::new(Some(db))
        .with_config(auth_config))
        .layer(SessionLayer::new(session_store))
        .with_state(app_state);

        // run our app with hyper
        // `axum::Server` is a re-export of `hyper::Server`
        log!("listening on http://{}", &addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}

    // client-only stuff for Trunk
    else {
        pub fn main() {
            // This example cannot be built as a trunk standalone CSR-only app.
            // Only the server may directly connect to the database.
        }
    }
}
