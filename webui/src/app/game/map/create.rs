use crate::app::*;

// put claim token in show, need current_token resource
//
#[component]
pub fn CreateMapPage() -> impl IntoView {
    let claim_token = create_server_action::<ClaimMapToken>();

    // Map token which is claimed but not consumed by the user - only 1 should exist
    let map_tokens = create_resource(
        move || claim_token.version().get(),
        move |_| get_map_token_info(),
    );

    let my_map_token = move || {
        map_tokens.read().map(|tokens| {
            tokens.map(|tokens| {
                tokens
                    .into_iter()
                    .filter(|(_, map, user)| {
                        map.is_none()
                            && Ok(true)
                                == with_user(|u| {
                                    Some(true) == user.as_ref().map(|(user, _)| user == u)
                                })
                    })
                    .next()
                    .map(|(token, _, _)| token)
            })
        })
    };

    let have_token = move || my_map_token().map(|t| t.map(|t| t.is_some())) == Some(Ok(true));

    view! {
        <h1>"Welcome to the CreateMap Page!"</h1>
        <Transition fallback=||"Loading..." >
        <ErrorBoundary fallback=|_|"Encountered error" >
        <Show when=have_token fallback=move||view!{<ClaimToken claim_token=claim_token/>} >
        <Lang hu="térképkészítés" en="map creation" />
        </Show>
        </ErrorBoundary>
        </Transition>
    }
}

#[component]
fn ClaimToken(claim_token: Action<ClaimMapToken, Result<(), ServerFnError>>) -> impl IntoView {
    // Bool to see if it's been changed
    let (token, set_token) = create_signal((String::new(), false)); // prevents empty token warning
    let (live_token, set_live_token) = create_signal(String::new());

    // let activate_token = move || set_token.update(|(_, active)| *active = true); // enable warning
    let set_token = move |ev: Event| set_token((event_target_value(&ev), true));
    let set_live_token = move |ev: Event| set_live_token(event_target_value(&ev));

    let empty_token = move || token().0.is_empty() && token().1;
    let bad_length_token = move || token().0.len() != 20 && token().1;
    let bad_live_length_token = move || live_token().len() != 20;
    let invalid_token = move || {
        live_token()
            .chars()
            .any(|c| c.is_ascii_lowercase() == c.is_ascii_digit())
    };

    let disable_submit = move || empty_token() || bad_live_length_token() || invalid_token();

    let token_problems = move || {
        view! {
            <Show when=empty_token fallback=||()>
                <Alert header="" warning=true >
                    <Lang hu="Térkép előállításához token kell! Ha nincs, kérj az adminisztrátortól!"
                        en="You may only create a map with a map creation token! You may ask the administrator to issue you a token."/>
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

    view! {
        <div class="login-container">
            <h2><Lang hu="Token" en="Token"/></h2>
            <UserErrorBoundary action=claim_token />
            <ActionForm action=claim_token>
                <Input name="token" on:change=set_token on:input=set_live_token >
                    <Lang hu="Térkép token" en="Map creation token"/>
                </Input>
                {token_problems}
                <Submit disable=disable_submit>
                    <Lang hu="Token igénybevétele" en="Claim token"/>
                </Submit>
            </ActionForm>
        </div>
    }
}
