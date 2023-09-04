use std::collections::HashMap;

use super::display::DisplayPreMap;
use crate::app::*;
use js_sys::Uint8Array;
use leptos::html::Div;
use leptos_use::*;
use map_utils::{Color, PreRegion};
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

    // File processing
    let file = create_rw_signal(None);

    let file_string = create_local_resource(file, |file| get_file_string(file));
    let processed_file = create_resource(
        move || file_string.get().and_then(Result::ok),
        |maybefile| process_file(maybefile),
    );

    // the selected PreRegion - when not needed anymore, set to none
    let selects = create_rw_signal(None);
    let (selected, select) = selects.split();

    create_effect(move |_| selected().map(|pr: PreRegion| log!("{}", pr.name)));

    // Picking colors for water and teams by clicking
    let lock = create_rw_signal(false); // When locked, no other colors may be selected
    let colors = create_memo(move |_| {
        let mut colors: HashMap<Color, usize> = HashMap::new();
        processed_file.map(|pf| {
            if let Ok(prs) = pf {
                for pr in prs.iter() {
                    colors.entry(pr.color).and_modify(|c| *c += 1).or_insert(1);
                }
            }
        });
        colors
    });

    // Water color
    let water_color = create_rw_signal(Color::new(0, 0, 255));
    let initial_water_color_set = create_rw_signal(false);
    create_effect(move |_| {
        if !initial_water_color_set.get() {
            if let Some((most_likely_water_color, _)) = colors().iter().max_by_key(|(_, c)| **c) {
                log!("{:?}", colors());
                water_color.set(*most_likely_water_color);
                initial_water_color_set.set(true);
            }
        }
    });

    // just for testing
    let team1_color = create_rw_signal(Color::new(84, 44, 2));

    view! {
        <h1>"Welcome to the CreateMap Page!"</h1>
        <Transition fallback=||"Loading..." >
        <ErrorBoundary fallback=|_|"Encountered error" >
        <Show when=have_token fallback=move||view!{<ClaimToken claim_token=claim_token/>} >
            <Lang hu="térképkészítés" en="map creation" />
            <ClientOnly>
                <UploadInkscapeSVG file=file />
            </ClientOnly>
            <Suspense fallback=||view!{<Lang hu="Feldolgozás..." en="Processing..."/>}>
                {move || processed_file.map(|prs| view!{
                    <DisplayPreMap pre_regions=prs.clone() select=select />
                })}
                <ClickColor color=water_color select=selects lock=lock >
                    <Lang hu="Válaszd ki a víz színét:" en="Select the sea regions' color:" />
                </ClickColor>
                <ClickColor color=team1_color select=selects lock=lock >
                    <Lang hu="Válaszd ki az egyik csapat színét:"
                     en="Select a team's home region's color:" />
                </ClickColor>
            </Suspense>
        </Show>
        </ErrorBoundary>
        </Transition>
    }
}

#[component]
fn ClickColor(
    color: RwSignal<Color>,
    select: RwSignal<Option<PreRegion>>,
    lock: RwSignal<bool>,
    children: ChildrenFn,
) -> impl IntoView {
    let started = create_rw_signal(false);
    let on_click = move |_| {
        if !started.get() && !lock.get() {
            select.set(None);
            lock.set(true);
            started.set(true); // specific to this component
        } else if started.get() {
            started.set(false);
            lock.set(false);
        }
    };

    // need to do this better w/o effect
    create_effect(move |_| {
        if started.get() {
            if let Some(pr) = select.get() {
                color.set(pr.color);
                lock.set(false);
                started.set(false);
            }
        }
    });

    let color_str = move || color.get().to_string();

    view! {
        <div class="color-picker" >
            {children}
            <div class="color-shower" >
                <svg viewBox="0 0 1 1" >
                    <rect x=0 y=0 width=1 height=1 fill=color_str >
                        <title>{color_str}</title>
                    </rect>
                </svg>
            </div>
            <button type="button" on:click=on_click
                class:disabled={move||lock.get() && !started.get()} >
                <Show when=move||!started.get()
                    fallback=||view!{<Lang hu="Mégsem" en="Cancel" />}>
                    <Lang hu="Válassz színt" en="Select color" />
                </Show>
            </button>
        </div>
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
async fn process_file(
    file_string: Option<String>,
) -> Result<Vec<map_utils::PreRegion>, ServerFnError> {
    if let Some(file_string) = file_string {
        if file_string.find("<script").is_some() {
            return Err(ServerFnError::ServerError("NO SCRIPT TAGS ALLOWED!".into()));
        }
        Ok(map_utils::pre_process_svg(file_string)
            .map_err(|err| ServerFnError::ServerError(err.to_string()))?)
    } else {
        // Err(ServerFnError::ServerError("no file string".into()))
        Ok(Vec::new())
    }
}

#[component]
fn UploadInkscapeSVG(file: RwSignal<Option<File>>) -> impl IntoView {
    let drop_zone = create_node_ref::<Div>();

    let set_file = move |file_: Option<File>| file.set(file_);

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
        files: _,
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
