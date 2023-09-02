use crate::app::{account::token::Token, *};
use js_sys::Uint8Array;
use leptos::html::Div;
use leptos_use::*;
use web_sys::File;

// put claim token in show, need current_token resource
#[component]
pub fn CreateMapPage() -> impl IntoView {
    let claim_token = create_server_action::<ClaimMapToken>();

    // Map token which is claimed but not consumed by the user - only 1 should exist
    let map_tokens = create_resource(
        move || claim_token.version().get(),
        move |_| get_map_token_info(),
    );

    // The user's currently claimed & not consumed token (Option<Token>)
    let my_map_token = move || {
        map_tokens.get().and_then(|tokens| {
            tokens.ok().and_then(|tokens| {
                tokens
                    .into_iter()
                    .find(|(_, map, user)| {
                        map.is_none()
                            && Ok(true)
                                == with_user(|u| {
                                    Some(true) == user.as_ref().map(|(user, _)| user == u)
                                })
                    })
                    .map(|(token, _, _)| token)
            })
        })
    };

    let have_token = move || my_map_token().is_some();

    // file upload: leptos-use's use_drop_zone
    // create server action/event? that sends svg when "upload" is pressed
    // maybe do some pre-processing on the svg - ie. valid or not, display it first?
    view! {
        <h1>"Welcome to the CreateMap Page!"</h1>
        <Transition fallback=||"Loading..." >
        <ErrorBoundary fallback=|_|"Encountered error" >
        <Show when=have_token fallback=move||view!{<ClaimToken claim_token=claim_token/>} >
            <Lang hu="térképkészítés" en="map creation" />
            <ClientOnly>
                <UploadInkscapeSVG token=Signal::derive(my_map_token) />
            </ClientOnly>
            "\nother stuff..."
        </Show>
        </ErrorBoundary>
        </Transition>
    }
}

async fn get_file_string(file: Option<File>) -> Result<String, String> {
    let file = if let Some(file) = file {
        file
    } else {
        return Err("no file".into());
    };

    let name = file.name();
    if !name.ends_with(".svg") {
        return Err("Not an svg".into());
    }

    let js_future = wasm_bindgen_futures::JsFuture::from(file.array_buffer());
    let jsval = js_future.await.unwrap();
    let arr: Uint8Array = Uint8Array::new(&jsval);
    let data: Vec<u8> = arr.to_vec();

    // the errror should eventually be an alert
    String::from_utf8(data).map_err(|err| format!("{err}"))
}

#[server(ProcessFile, "/api")]
async fn process_file(file_string: Option<String>) -> Result<String, ServerFnError> {
    if let Some(file_string) = file_string {
        if file_string.find("<script").is_some() {
            return Err(ServerFnError::ServerError("NO SCRIPT TAGS ALLOWED!".into()));
        }
        Ok(file_string)
    } else {
        // Err(ServerFnError::ServerError("no file string".into()))
        Ok(String::new())
    }
}

#[component]
fn UploadInkscapeSVG(token: Signal<Option<Token>>) -> impl IntoView {
    let drop_zone = create_node_ref::<Div>();

    let (file, set_file) = create_signal(None);

    let file_string = create_local_resource(file, |file| get_file_string(file));
    let processed_file = create_resource(
        move || file_string.get().and_then(Result::ok),
        |maybefile| process_file(maybefile),
    );

    let on_drop = move |event: UseDropZoneEvent| {
        let file = if !event.files.is_empty() {
            Some(event.files[0].clone())
        } else {
            None
        };
        set_file(file);
    };

    let UseDropZoneReturn {
        is_over_drop_zone,
        files,
    } = use_drop_zone_with_options(drop_zone, UseDropZoneOptions::default().on_drop(on_drop));

    view! {
        <div class="parent-container" >
        <div node_ref=drop_zone class="drop-zone" class:dropped=move||file.with(|f| f.is_some())
            class:active=is_over_drop_zone >
            <Lang hu="EJTSD IDE A TÉRKÉPET (jelenleg a select nem működik)"
                en="DROP HERE (for now selecting in the menu when you click doesn't work)" />
            {/*<input type="file" node_ref=file_select accept=".svg" style="display:none;" />*/}
        </div>
        </div>

        <Suspense fallback=||view!{"No file string yet"}>
        {move || processed_file.map(|fs| match fs {
            Ok(s) => view!{<div class="svg-container"/>}.inner_html(s.clone()),
            _ => view!{<div/>}
        })}
        </Suspense>
        <p>{move || if is_over_drop_zone.get() {
                "over drop zone"
            } else {
                "not over drop zone"
            }
        }</p>
        <p>"files:" {move || format!("{:?}", files.get())}</p>
        <Suspense fallback=||view!{"No file string yet"}>
        <ErrorBoundary fallback=move|_|view!{ "something wrong" }>
        <p>"file string:" {move || format!("{:?}", file_string.get())}</p>
        </ErrorBoundary>
        </Suspense>
        <p>"\ntodo\n" {move||token().map(|t| t.token)}</p>
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
