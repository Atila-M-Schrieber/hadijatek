use std::fmt::Display;

use crate::lang::*;
use cfg_if::cfg_if;
use http::status::StatusCode;
use leptos::error::Error;
use leptos::*;
use thiserror::Error;

#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;

#[derive(Clone, Debug, Error)]
pub enum AppError {
    #[error("Not Found")]
    NotFound,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

// A basic function to display errors served by the error boundaries.
// Feel free to do more complicated things here than just displaying the error.
#[component]
pub fn ErrorTemplate(
    #[prop(optional)] outside_errors: Option<Errors>,
    #[prop(optional)] errors: Option<RwSignal<Errors>>,
) -> impl IntoView {
    let errors = match outside_errors {
        Some(e) => create_rw_signal(e),
        None => match errors {
            Some(e) => e,
            None => panic!("No Errors found and we expected errors!"),
        },
    };
    // Get Errors from Signal
    let errors = errors.get();

    // Downcast lets us take a type that implements `std::error::Error`
    let errors: Vec<AppError> = errors
        .into_iter()
        .filter_map(|(_k, v)| v.downcast_ref::<AppError>().cloned())
        .collect();
    println!("Errors: {errors:#?}");

    // Only the response code for the first error is actually sent from the server
    // this may be customized by the specific application
    cfg_if! { if #[cfg(feature="ssr")] {
        let response = use_context::<ResponseOptions>();
        if let Some(response) = response {
            response.set_status(errors[0].status_code());
        }
    }}

    view! {
        <h1>{if errors.len() > 1 {"Errors"} else {"Error"}}</h1>
        <For
            // a function that returns the items we're iterating over; a signal is fine
            each= move || {errors.clone().into_iter().enumerate()}
            // a unique key for each item as a reference
            key=|(index, _error)| *index
            // renders each item to a view
            view= move |error| {
                let error_string = error.1.to_string();
                let error_code= error.1.status_code();
                view! {

                    <h2>{error_code.to_string()}</h2>
                    <p>"Error: " {error_string}</p>
                }
            }
        />
    }
}

/// Alert for errors and warnings, provide either content or children
#[component]
pub fn Alert<S>(
    header: S,
    #[prop(optional)] warning: bool,
    #[prop(optional)] content: Option<String>,
    #[prop(optional)] children: Option<ChildrenFn>,
) -> impl IntoView
where
    S: ToString + 'static,
{
    let class: String = "alert ".to_string() + if !warning { "error" } else { "warning" };
    let header = header.to_string();
    view! {
        <div class={class}>
        {if !header.is_empty() {
            view!{<h4>{header}</h4>}.into_view()
        } else {().into_view()}}
        <div class="alert-content">
        {children}{content}
        </div>
        </div>
    }
}

/// User-related errors
#[derive(Debug, Clone, PartialEq)]
pub enum UserError {
    GuestUser,
    NoneErr,
    OtherServerError(ServerFnError),
    NoUser,
    // includes signup's pw stuff - those are useless in the frontend
    BadPassword,
    TakenName,
    BadToken,
    UsedToken,
}

impl From<ServerFnError> for UserError {
    fn from(err: ServerFnError) -> Self {
        use UserError::*;
        if let ServerFnError::ServerError(err) = &err {
            if let Some(err) = err.split_once(": ").map(|e| e.0) {
                match err {
                    "NO_USER" => return NoUser,
                    "BAD_PASSWORD"
                    | "SHORT_PASSWORD"
                    | "INVALID_PASSWORD"
                    | "UNCONFIRMED_PASSWORD" => return BadPassword,
                    "TAKEN_NAME" => return TakenName,
                    "BAD_TOKEN" => return BadToken,
                    "USED_TOKEN" => return UsedToken,
                    _ => {}
                };
            }
        }
        OtherServerError(err)
    }
}

impl Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserError::OtherServerError(err) => write!(f, "{err}"),
            err => write!(f, "{err:?}"),
        }
    }
}

impl std::error::Error for UserError {}

/// Takes an action, and returns a Result, which can then be used in login/signup failures
pub fn user_error<A>(action: Action<A, Result<(), ServerFnError>>) -> Result<(), UserError> {
    if let Some(Err(err)) = action.value().get() {
        Err(err.into())
    } else {
        // If all fine, or not yet called, no need for an error
        Ok(())
    }
}

#[component]
pub fn UserErrorBoundary<A: 'static>(
    action: Action<A, Result<(), ServerFnError>>,
) -> impl IntoView {
    use UserError::*;

    let err_text = move |err: &Error| {
        // let to = |err: UserError| Error::from(err);
        let out = match err.downcast_ref::<UserError>().unwrap() {
            GuestUser => ("Vendégként tilos!", "Guests forbidden!"),
            NoneErr => (
                "Még nem töltődött be a felhasználó...",
                "User still loading...",
            ),
            NoUser => ("Ismeretlen felhasználónév", "Unknown username"),
            BadPassword => ("Hibás jelszó", "Incorrect password"),
            TakenName => (
                "Ez a felhasználónév már foglalt!",
                "This username is taken!",
            ),
            BadToken => ("Ismeretlen token!", "Unknown token!"),
            UsedToken => (
                "Ez a token már fel lett használva!",
                "This token has been used!",
            ),
            OtherServerError(err) => {
                log!("OtherServerError encoundered: {err}");
                (
                    "Ismeretlen hiba, add meg az adminisztrátornak, a hiba körülményeit, és hogy pontosan mikor történt!",
                    "Unknown error, inform the administrator about the circumstances of the error, and the exact time!",
                )
            }
        };
        (out.0, out.1)
    };

    let err = move |err: RwSignal<Errors>| {
        let err_text = err
            .get()
            .iter()
            .next()
            .map(|(_, err)| err_text(err))
            .unwrap_or(("", ""));
        view! {
            <Show when=||!err_text.0.is_empty() fallback=||()>
                <Alert header="">
                    <Lang hu=err_text.0 en=err_text.1 />
                </Alert>
            </Show>
        }
    };

    view! {
        <ErrorBoundary fallback=err>
        {move || user_error(action)}
        </ErrorBoundary>
    }
}
