use super::*;

/// The login page
#[component]
pub fn LoginPage(login: Action<Login, Result<(), ServerFnError>>) -> impl IntoView {
    let (name, set_name) = create_signal(String::new());
    let (live_password, set_live_password) = create_signal(String::new());

    let set_name = move |ev: Event| set_name(event_target_value(&ev));
    let set_live_pw = move |ev: Event| {
        set_live_password(event_target_value(&ev));
    };

    let disable_submit = move || name().is_empty() || live_password().is_empty();

    view! {
        <div class="login-container">
            <h2><Lang hu="Bejelentkezés" en="Login"/></h2>
            <UserErrorBoundary action=login />
            <ActionForm action=login>
                <Input name="username" on:input=set_name >
                    <Lang hu="Felhasználónév" en="Username"/>
                </Input>
                <Input name="" on:input=set_live_pw password=true >
                    <Lang hu="Jelszó" en="Password"/>
                </Input>
                <RememberMe/>
                <Submit disable=disable_submit >
                        <Lang hu="Bejelentkezés" en="Log in"/>
                </Submit>
            </ActionForm>
            <p class="signup-text"><a href="/signup">
                <Lang hu="Regisztráció" en="Sign up"/>
            </a></p>
        </div>
        <Transition fallback=||()>
        <ErrorBoundary
            fallback=move |err|
                view!{
                        {move || err.get()
                            .into_iter()
                            .map(|(_, e)| view! {<p>{e.to_string()}</p>})
                            .collect_view()
                        }
                }
        >
        <p>{move || with_user(|user| user.username.clone())}</p>
        </ErrorBoundary>
        </Transition>
    }
}

