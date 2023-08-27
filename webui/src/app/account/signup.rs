use super::*;

/// The signup page
#[component]
pub fn SignupPage(signup: Action<Signup, Result<(), ServerFnError>>) -> impl IntoView {
    // Bool to see if it's been changed
    let (token, set_token) = create_signal((String::new(), false)); // prevents empty token warning
    let (live_token, set_live_token) = create_signal(String::new());

    let activate_token = move || set_token.update(|(_, active)| *active = true); // enable warning
    let set_token = move |ev: Event| set_token((event_target_value(&ev), true));
    let set_live_token = move |ev: Event| set_live_token(event_target_value(&ev));

    let (name, set_name) = create_signal(String::new());

    let set_name = move |ev: Event| {
        activate_token();
        set_name(event_target_value(&ev))
    };

    let MIN_PW_LEN: usize = 6;

    let (password, set_password) = create_signal(String::new());
    let (live_password, set_live_password) = create_signal(String::new());
    let (live_password_confirm, set_live_password_confirm) = create_signal(String::new());

    let set_pw = move |ev: Event| {
        set_password(event_target_value(&ev));
    };
    let set_live_pw = move |ev: Event| {
        activate_token();
        set_live_password(event_target_value(&ev));
    };
    let set_live_pw_cnf = move |ev: Event| {
        activate_token();
        set_live_password_confirm(event_target_value(&ev));
    };

    let empty_token = move || token().0.is_empty() && token().1;
    let bad_length_token = move || token().0.len() != 20 && token().1;
    let bad_live_length_token = move || live_token().len() != 20;
    let invalid_token = move || {
        live_token()
            .chars()
            .any(|c| c.is_ascii_lowercase() == c.is_ascii_digit())
    };
    let invalid_name = move || name() != name().trim();
    let valid_pw_len = move || live_password().len() >= MIN_PW_LEN;
    let valid_pw_chars = move || {
        !live_password()
            .chars()
            .any(|c| c.is_whitespace() || !c.is_ascii())
    };
    let valid_pw = move || valid_pw_len() && valid_pw_chars();
    let matching_pw = move || live_password() == live_password_confirm();
    let wont_be_matching_pw = move || {
        let live_password_confirm = live_password_confirm();
        let len = live_password_confirm.len();
        let live_password = live_password();
        len > live_password.len() || live_password[..len] != live_password_confirm
    };

    let pw_strength = move || {
        let unit = match live_password().len() {
            0..=4 => ("Hajó", "Ship"),
            5..=7 => ("Tank", "Tank"),
            8..=10 => ("Repülő", "Plane"),
            11..=13 => ("Szupertank", "Supertank"),
            14..=16 => ("Tengeralattjáró", "Submarine"),
            _ => (
                "Tüzérség, \"A Csata Királynője\" (vagy a jelszavaké..)",
                "Artillery, \"The Queen of Battle\" (or of passwords..)",
            ),
        };
        view! {
            <Lang hu="Jelszó erőssége" en="Password strength" />": "<Lang hu=unit.0 en=unit.1 />
        }
    };

    let disable_submit = move || {
        empty_token()
            || bad_live_length_token()
            || invalid_token()
            || invalid_name()
            || !valid_pw()
            || !matching_pw()
            || name().is_empty()
            || password().is_empty()
    };

    let token_problems = move || {
        view! {
            <Show when=empty_token fallback=||()>
                <Alert header="" warning=true >
                    <Lang hu="Regisztrációhoz kell token! Ha nincs, kérj az adminisztrátortól!"
                        en="You may only sign up with a user creation token! You may ask the administrator to issue you a token."/>
                </Alert>
            </Show>
            <Show when=move || !empty_token() && bad_length_token() fallback=||()>
                <Alert header="">
                    <Lang hu="A token hossza 20 karakter!"
                        en="The token's length must be 20 characters!"/>
                </Alert>
            </Show>
            <Show when=invalid_token fallback=||()>
                <Alert header="">
                    <Lang hu="A token csak a-z közötti ékezet nélküli karaktereket, és 0-9 közötti karaktereket tartalmazhat!"
                        en="The token may only contain characters a-z or 0-9 (ASCII - no accents)!"/>
                </Alert>
            </Show>
        }
    };

    let name_problems = move || {
        view! {
            <Show when=invalid_name fallback=||()>
                <Alert header="">
                    <Lang hu="Név nem kezdődhet vagy végződhet szóközzel!"
                        en="Name must not start or end with whitespace!"/>
                </Alert>
            </Show>
        }
    };

    let pw_problems = move || {
        view! {
                <Show
                    when={move || !password().is_empty() && !valid_pw_len()}
                    fallback=||()
                >
                    <Alert header="">
                        <Lang hu="Túl rövid a jelszó!"
                            en="Password is too short!"/>
                    </Alert>
                </Show>
                <Show
                    when={move || !valid_pw_chars()}
                    fallback=||()
                >
                    <Alert header="">
                        <Lang hu="A jelszó nem tartalmazhat szóközt, illetve ASCII-n kívüli karaktert! (Pl. ékezetet)"
                            en="The password may not contain whitespace, or non-ASCII characters! (Eg. accents)"/>
                    </Alert>
                </Show>
        }
    };

    let pw_cnf_problems = move || {
        view! {
                <Show
                    when={move || !live_password_confirm().is_empty() && wont_be_matching_pw()}
                    fallback=||()
                >
                    <Alert header="">
                        <Lang hu="Nem egyeznek meg a jelszavak!"
                            en="Passwords do not match!"/>
                    </Alert>
                </Show>
        }
    };

    view! {
        <div class="login-container">
            <h2><Lang hu="Regisztráció" en="Signup"/></h2>
            <UserErrorBoundary action=signup />
            <ActionForm action=signup>
                <Input name="user_creation_token" on:change=set_token on:input=set_live_token >
                    <Lang hu="Regisztrációs token" en="User creation token"/>
                </Input>
                {token_problems}
                <Input name="username" on:input=set_name >
                    <Lang hu="Felhasználónév" en="Username"/>
                </Input>
                {name_problems}
                <Input name="" on:change=set_pw on:input=set_live_pw password=true >
                    <Lang hu="Jelszó" en="Password"/>
                </Input>
                {pw_problems}
                <div class="pw-strength">{pw_strength}</div>
                <Input name="password_confirmation"
                    on:input=set_live_pw_cnf password=true >
                    <Lang hu="Jelszó újra" en="Password again"/>
                </Input>
                {pw_cnf_problems}
                <RememberMe/>
                <Submit disable=disable_submit>
                    <Lang hu="Regisztráció" en="Sign up"/>
                </Submit>
            </ActionForm>
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
