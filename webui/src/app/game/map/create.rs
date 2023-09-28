use super::display::DisplayPreMap;
use crate::app::*;
use js_sys::Uint8Array;
use map_utils::{Color, Goodness, Label, PreProcessed, PreRegion};
use petgraph::{csr::Csr, visit::IntoNodeReferences, Undirected};
use std::collections::HashMap;
use web_sys::{File, SubmitEvent};

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

    // set true once the map display is done processing with its initial sorting
    let (can_contract, set_can_contract) = create_signal(false);

    let file_upload = create_server_action::<ProcessFile>();
    // let processed_file = create_resource(move || file_upload.value(), |processed| processed);
    let processed_file = file_upload.value();

    //let file_done = move || processed_file.with(|p| p.as_ref().map(|_| true)) == Some(true);
    let file_is_processed =
        create_memo(move |_| processed_file.with(|res| res.as_ref().map(|_| true)) == Some(true));
    // let file_err = move || None::<String>;
    let file_err = move || {
        processed_file.with(|e| {
            e.as_ref().and_then(|e| {
                if let Err(e) = e {
                    Some(e.clone())
                } else {
                    None
                }
            })
        })
    };

    let pre_regions: Signal<Csr<PreRegion, (), Undirected>> = Signal::derive(move || {
        if file_is_processed.get_untracked() {
            processed_file().clone().unwrap().unwrap().0
        } else {
            Csr::new()
        }
    });
    let goodnesses: Signal<HashMap<u32, Goodness>> = Signal::derive(move || {
        if file_is_processed.get_untracked() {
            processed_file().clone().unwrap().unwrap().1
        } else {
            HashMap::new()
        }
    });
    let initial_labels: Signal<HashMap<u32, Label>> = Signal::derive(move || {
        if file_is_processed.get_untracked() {
            processed_file().clone().unwrap().unwrap().2
        } else {
            HashMap::new()
        }
    });

    // the selected PreRegion - when not needed anymore, set to none
    let selects = create_rw_signal(None);
    let (_selected, select) = selects.split();

    // create_effect(move |_| selected().map(|pr: PreRegion| log!("{}", pr.name)));

    // Picking colors for water and teams by clicking
    let lock = create_rw_signal(false); // When locked, no other colors may be selected
    provide_context(LockSignal(lock));
    let colors = create_memo(move |_| {
        let mut colors: HashMap<(Color, Color), usize> = HashMap::new();
        if file_is_processed() {
            for (_, pr) in pre_regions().node_references() {
                colors
                    .entry((pr.stroke, pr.color))
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
            }
        };
        colors
    });

    // Water color, strokes
    let water_color = create_rw_signal(Color::new(0, 0, 255));
    let water_stroke = create_rw_signal(Color::new(0, 0, 255));
    let land_stroke = create_rw_signal(Color::new(187, 187, 187));
    let initial_colors_set = create_rw_signal(false);
    create_effect(move |_| {
        if !initial_colors_set.get() {
            let key = |((stroke, color), c): &((Color, Color), usize)| {
                let (r, g, b) = color.get();
                let (sr, sg, sb) = stroke.get();
                // no bases, favor higher blue values, fill > stroke
                let color_score = (stroke != &Color::black()) as isize
                    * (b as isize + (sb / 2) as isize
                        - (r / 2) as isize
                        - (sr / 6) as isize
                        - (g / 3) as isize
                        - (sg / 9) as isize);
                color_score * *c as isize
            };
            let mut colors: Vec<((Color, Color), usize)> = colors().into_iter().collect();
            colors.sort_by_key(key);
            let land = colors.get(0);
            let water = colors.last();
            if let (
                Some(((most_likely_land_stroke, _), _)),
                Some(((most_likely_water_stroke, most_likely_water_color), _)),
            ) = (land, water)
            {
                water_color.set(*most_likely_water_color);
                water_stroke.set(*most_likely_water_stroke);
                land_stroke.set(*most_likely_land_stroke);
                initial_colors_set.set(true);
            }
        }
    });

    // Teams
    let teams = create_rw_signal(Vec::new());

    view! {
        <h1>"Welcome to the CreateMap Page!"</h1>
        <Transition fallback=||"Loading..." >
        <ErrorBoundary fallback=|_|"Encountered error" >
        <Show when=have_token fallback=move||view!{
                <ClaimToken claim_token=claim_token/>
        }>
            <h2><Lang hu="Térkép készítés" en="Map creation" /></h2>
            <UploadInkscapeSVG file=file_upload contract=can_contract />
            <Show when=move||file_err().is_some() fallback=||() >
                <Alert header="ERROR" >
                    <Lang hu="Hiba történt:" en="Error: " />
                    {file_err().map(|e| format!("{e}"))}
                </Alert>
            </Show>
            <Show when=file_is_processed
                fallback=||()>
                <DisplayPreMap pre_regions=pre_regions goodnesses=goodnesses
                    teams=teams select=select
                    water_color=water_color water_stroke=water_stroke
                    land_stroke=land_stroke done=set_can_contract />
                <div  >
                <ClickColor color=water_color select=selects >
                    <Lang hu="Válaszd ki a tengerek színét:" en="Select the sea regions' color:" />
                </ClickColor>
                <ColorSelector color=water_stroke >
                    <Lang hu="Válaszd ki a tengeri mezők körvonalainak színét:"
                        en="Choose the storke color of the sea regions:"/>
                </ColorSelector>
                <ColorSelector color=land_stroke >
                    <Lang hu="Válaszd ki a szárazföldi mezők körvonalainak színét:"
                        en="Choose the storke color of the land regions:"/>
                </ColorSelector>
                <AssignTeams teams=teams select=selects pre_regions=pre_regions />
                </div>
            </Show>
        </Show>
        </ErrorBoundary>
        </Transition>
    }
}

#[derive(Clone)]
struct LockSignal(RwSignal<bool>);

#[derive(PartialEq, Clone, Debug)]
enum TeamError {
    Homeless,
    Cringe(Vec<PreRegion>),
    NonyoTeam(String),
    Zilch,
}

type TeamInfo = (usize, RwSignal<Color>, RwSignal<String>);

#[component]
fn AssignTeams(
    teams: RwSignal<Vec<TeamInfo>>,
    select: RwSignal<Option<PreRegion>>,
    pre_regions: Signal<Csr<PreRegion, (), Undirected>>,
) -> impl IntoView {
    let team_lock = create_rw_signal(false);

    let add_team = move |_| {
        let index = teams
            .get()
            .iter()
            .max_by_key(|(i, _, _)| i)
            .map(|(i, _, _)| *i)
            .unwrap_or_default()
            + 1;

        let color = create_rw_signal(Color::black());

        let name = create_rw_signal(String::new());

        teams.update(|teams| teams.push((index, color, name)));
    };

    let delete_team = move |index| {
        teams.update(|teams| {
            if let Some(pos) = teams.iter().position(|(i, _, _)| i == &index) {
                teams.remove(pos);
            }
        })
    };

    let render_team = move |(index, team_color, team_name): TeamInfo| {
        use TeamError as TE;

        let home_bases = create_memo(move |_| {
            let tn = team_name();
            let team_with_same_name = teams()
                .into_iter()
                .find(|(i, _, t_n)| i != &index && t_n() == tn && !tn.is_empty());
            if let Some((_, _, t_n)) = team_with_same_name {
                return Err(TE::NonyoTeam(t_n()));
            }
            let tc = team_color();
            if tc != Color::black() {
                let team_with_same_color = teams()
                    .into_iter()
                    .find(|(i, t_c, _)| i != &index && t_c() == tc);
                if let Some((_, _, t_n)) = team_with_same_color {
                    return Err(TE::NonyoTeam(t_n()));
                }
                let home_bases: Vec<PreRegion> = pre_regions.with(|prs| {
                    prs.node_references()
                        .filter(|(_, pr)| pr.color == tc)
                        .map(|(_, pr)| pr.clone())
                        .collect()
                });
                if home_bases.is_empty() {
                    return Err(TE::Homeless);
                }
                // cringe: the opposite of based (region without base)
                let cringe_home_bases: Vec<PreRegion> = home_bases
                    .iter()
                    .filter(|pr| !pr.has_base)
                    .cloned()
                    .collect();
                if !cringe_home_bases.is_empty() {
                    return Err(TE::Cringe(cringe_home_bases));
                }
                Ok(home_bases)
            } else {
                Err(TE::Zilch)
            }
        });
        let no_home_bases = move || home_bases.with(|hb| hb != &Err(TeamError::Zilch));

        // let no_home_bases = || false;

        let set_team_name = move |ev: Event| {
            team_name.set(event_target_value(&ev));
        };

        let list_region_names = move |prs: &Vec<PreRegion>| {
            prs.iter()
                .map(|pr| pr.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        };

        view! {
            <div class="team-selector" >
                <div class="team-selector-header" >
                <Input name=format!{"team{index}"} focus_on_show=true on:input=set_team_name >
                    <Lang hu="Csapatnév:" en="Team name:" />
                </Input>
                <ClickColor color=team_color select=select lock=team_lock >
                    <Show when=move || team_color.get() == Color::black() fallback=move||view!{
                        <Lang hu="Másik anyabázis kiválasztása" en="Select different home base"/>
                    }>
                        <Lang hu="Válaszd ki az egyik anyabázist:" en="Select a home base" />
                    </Show>
                </ClickColor>
                <button type="button" class="delete-button" on:click=move|_|delete_team(index) >
                    <Lang hu="Törlés" en="Delete" />
                </button>
                </div>
                <Show when=no_home_bases fallback=||()>
                <div class="team-selector-footer" >
                {move || {
                    let bases = home_bases.get();
                    if let Err(te) = home_bases.get() {
                        let (hu, en) = match te {
                            TE::Homeless => ("Nem tudom hogyan, de sikerült nemlétező mezőt választani".into(),
                                "I don't know how, but you managed to select a non-existent region".into()),
                            TE::Zilch => ("Nem kellene itt lenned! 😈🔪".into(), "You're not supposed to be here! 😈🔪".into()),
                            TE::Cringe(regions) => (
                                format!("Ezek a mezők a csapat kiválasztott színével rendelkeznek, de nem bázisok: {}", list_region_names(&regions)),
                                format!("These regions have the team's chosen color, but are not bases: {}", list_region_names(&regions)),
                            ),
                            TE::NonyoTeam(team) => {
                                let (hu, en) = if team.is_empty() {
                                    ("ISMERETLEN", "UNKNOWN")
                                } else { (team.as_str(), team.as_str()) };
                                (
                                    format!("Ez a csapat már létezik! (Csapatnév: {})", hu),
                                    format!("This team already exists! (Team name: {})", en)
                                )
                            }
                        };
                        view!{
                            <Alert header="" ><Lang hu=hu.clone() en=en.clone()/></Alert>
                        }.into_view()
                    } else {
                        let bases = bases.expect("to have just checked for errors");
                        let maybe_warn = if bases.len() != 3 {
                            view!{
                                <Alert header="" warning=true >
                                    <Lang hu="Ajánlatos 3 anyabázist adni egy csapatnak"
                                        en="You might want to provide 3 home bases to this team"/>
                                </Alert>
                            }
                        } else {
                            ().into_view()
                        };
                        view! {
                            <p>
                                <Lang hu="Anyabázisok" en="Home bases"/>:
                                {list_region_names(&bases)}
                            </p>
                            {maybe_warn}
                        }.into_view()
                    }
                }}
                </div>
                </Show>
            </div>
        }
    };

    view! {
        <div>
            <div class="new-team" >
            <button type="button" on:click=add_team >
                <Lang hu="Új csapat" en="New team" />
            </button>
            </div>
            <For each=teams key=|(i, _, _)|*i view=render_team />
        </div>
    }
}

#[component]
fn ClickColor(
    color: RwSignal<Color>,
    select: RwSignal<Option<PreRegion>>,
    #[prop(optional)] lock: Option<RwSignal<bool>>,
    children: ChildrenFn,
) -> impl IntoView {
    // can specify lock if needed for outside context, or use the 'global' lock context
    let global_lock = expect_context::<LockSignal>().0;
    let locked = move || {
        if let Some(lock) = lock {
            lock.get() || global_lock.get()
        } else {
            global_lock.get()
        }
    };

    let set_locks = move |b| {
        if let Some(lock) = lock {
            lock.set(b);
            global_lock.set(b)
        } else {
            global_lock.set(b)
        }
    };

    let started = create_rw_signal(false);
    let on_click = move |_| {
        if !started.get() && !locked() {
            select.set(None);
            set_locks(true);
            started.set(true); // specific to this component
        } else if started.get() {
            started.set(false);
            set_locks(false);
        }
    };

    // need to do this better w/o effect
    create_effect(move |_| {
        if started.get() {
            if let Some(pr) = select.get() {
                color.set(pr.color);
                set_locks(false);
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
                class:disabled={move||locked() && !started.get()} >
                <Show when=move||!started.get()
                    fallback=||view!{<Lang hu="Mégsem" en="Cancel" />}>
                    <Lang hu="Válassz mezőt" en="Select region" />
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
    let string = String::from_utf8(data).map_err(|err| format!("{err}"))?;

    if string.find("<script").is_some() {
        return Err("NO SCRIPT TAGS ALLOWED!".into());
    }

    Ok(string)
}

#[server(ProcessFile, "/api")]
pub async fn process_file(file: String) -> Result<PreProcessed, ServerFnError> {
    Ok(map_utils::pre_process_svg(file)
        .map_err(|err| ServerFnError::ServerError(err.to_string()))?)
}

#[component]
fn UploadInkscapeSVG(
    file: Action<ProcessFile, Result<PreProcessed, ServerFnError>>,
    #[prop(into)] contract: Signal<bool>,
) -> impl IntoView {
    let file_picker: NodeRef<leptos::html::Input> = create_node_ref();

    let (file_err, set_file_err) = create_signal(None);
    let (submitted, set_submitted) = create_signal(false);

    let file_string = create_action(move |_| async move {
        let file = file_picker
            .get_untracked()
            .and_then(|fs| fs.files().and_then(|fs| fs.item(0)));
        get_file_string(file).await
    });

    let get_fs = move |_| {
        set_file_err(None);
        file_string.dispatch(())
    };

    let upload = move |ev: SubmitEvent| {
        if submitted() {
            return ();
        } else {
            ev.prevent_default();
        }

        let file_string = file_string.value().get_untracked();
        if let Some(Ok(file_string)) = file_string {
            set_file_err(None);
            set_submitted(true);
            file.dispatch(ProcessFile { file: file_string });
        } else if let Some(Err(err)) = file_string {
            set_file_err(Some(err));
        }
    };

    let (hover, set_hover) = create_signal(false);
    let set_hover = move |_| set_hover.update(|h| *h = !*h);

    let disabled = move || file_string.value().with(|v| v.is_none());

    view! {
        <div class="upload-container" >
            <form on:submit=upload >
                <div class="file-upload-wrapper" class:dropped=contract
                    on:mouseenter=set_hover on:mouseleave=set_hover >
                    <input _ref=file_picker on:input=get_fs type="file" name="file" id="file"
                        accept=".svg" />
                    <label for="file" class:hovered=hover class:dropped=contract >
                        <Show when=disabled fallback=||view!{
                            <Lang hu="Térkép kiválasztva" en="Map selected"/>
                        }>
                            <Lang hu="EJTSD IDE A TÉRKÉPET" en="DROP MAP HERE" />
                        </Show>
                    </label>
                </div>
                <Show when=move||file_err().is_some() fallback=||()>
                    <Alert header="ERROR" >
                        <Lang hu="Hiba: " en="Error: " />{file_err()}
                    </Alert>
                </Show>
                <Submit disable=disabled >
                    <Show when=submitted fallback=||view!{<Lang hu="Feltöltés" en="Upload" />}>
                        <Lang hu="Újrakezdés" en="Restart" />
                    </Show>
                </Submit>
            </form>
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
