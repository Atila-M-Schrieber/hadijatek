//! General-use components

use leptos::ev::Event;
use leptos::html::Input;
use leptos::*;
use leptos_router::*;

use crate::error::*;
use crate::lang::*;

/// Generic text input component.
/// Takes the children as the label, and sets name= to every relevant thing.
/// Currently always a required input.
#[component]
pub fn Input<S>(
    name: S,
    children: ChildrenFn,
    #[prop(optional)] value: String,
    #[prop(optional)] password: bool,
    #[prop(optional)] focus_on_show: bool,
) -> impl IntoView
where
    S: ToString,
{
    let mut name = name.to_string();
    if name.is_empty() {
        if password {
            name = "password".into();
        } else {
            panic!("You must provide a name for non-password inputs!");
        }
    }

    // focus the main input on load
    let input_ref = create_node_ref::<Input>();
    create_effect(move |_| {
        if focus_on_show {
            if let Some(input) = input_ref.get() {
                // So inside, we wait a tick for the browser to mount it, then .focus()
                request_animation_frame(move || {
                    let _ = input.focus();
                });
            }
        }
    });

    view! {
        <div class="input-group">
            <label for=&name>{children()}</label>
            <input type=if password {"password"} else {"text"} id=&name name=&name value=&value
                node_ref=input_ref required/>
        </div>
    }
}

/// EditableInput - button that doubles as input - click to change.
/// When a button, it has a hidden input to pass along its current value
#[component]
pub fn EditableInput<S, F>(
    name: S,
    /// The value of the input by default
    value: String,
    /// Signal that determines the state of the button - RwSignal
    toggle: RwSignal<bool>,
    input: F,
    children: ChildrenFn,
) -> impl IntoView
where
    S: ToString + Clone + IntoAttribute + 'static,
    F: Fn(Event) + Copy + 'static,
{
    let toggled = move || toggle.get();
    let manual_toggle = move || toggle.update(|t| *t = !*t);
    // let toggle = move |_| toggle.update(|t| *t = !*t);

    let (entering, set_entering) = create_signal(true);
    let enter = move |_| set_entering.set_untracked(true);
    let exit = move |_| set_entering.set_untracked(false);

    let toggle = move |_| {
        toggle.update(|t| *t = !*t);
        log!("ent: {}, tog: {}", entering(), toggled());
    };

    create_effect(move |_| log!("ent: {}, tog: {}", entering(), toggled()));

    let (new_value, set_new_value) = create_signal(value.clone());
    let set_new_value = move |ev: Event| set_new_value(event_target_value(&ev));

    let value = store_value(value);
    let value = move || value.get_value();
    let name = store_value(name);
    let name = move || name.get_value();
    let children = store_value(children);

    // If the input is the same as the initial new_value, toggle back to button mode
    create_effect(move |_| {
        if toggled() {
            if new_value() == value() && !entering() {
                manual_toggle();
            }
        }
    });

    let button = move || {
        view! {
            {children.with_value(|c| c())}": "
            <button on:click=move |ev| toggle(ev.clone())/* ; enter(ev) */ >
                {value()}
            </button>
            // <input type="hidden" name=name() value=value()/>
        }
    };

    view! {
        <Show when=toggled fallback=button >
            <Input name=name() value=value()
                on:change=set_new_value on:input=input
                on:focus=enter on:focusout=exit focus_on_show=true >
                {children.with_value(|c| c())}
            </Input>
        </Show>
    }
}

/// Remember Me checkbox. Comes with handy "please remember me".
#[component]
pub fn RememberMe() -> impl IntoView {
    let (checked, set_checked) = create_signal(true);
    let check = move |_ev: Event| {
        set_checked.update(|b| *b = !*b);
    };

    view! {
        <div class="checkbox-group">
            <input type="checkbox" id="remember" name="remember" on:input=check checked/>
            <label for="remember"><Lang hu="Emlékezz rám" en="Remember me"/></label>
        </div>
        <Show when=move||!checked() fallback=||()>
            <Alert header="" warning=true >
                <Lang hu="Nem akarlak elfelejteni 🥺" en="I don't want to forget you 🥺" />
            </Alert>
        </Show>
    }
}

/// Submit component, with a "disable" signal.
/// Takes children as the label
#[component]
pub fn Submit<F: Fn() -> bool + Copy + 'static>(disable: F, children: ChildrenFn) -> impl IntoView {
    view! {
        <div class="input-group">
            <button type="submit" class:disabled=disable disabled=disable >
                {children()}
            </button>
        </div>
    }
}

/// Table component, which takes children as the <th/>,
/// a list of items (stored values at the moment),
/// and a function which turns said items into rows.
#[component]
pub fn Table<T: Clone + 'static, F: Fn(T) -> IV + std::clone::Clone + 'static, IV: IntoView>(
    items: StoredValue<Vec<T>>,
    to_row: F,
    children: ChildrenFn,
) -> impl IntoView {
    let to_row = store_value(to_row);
    view! {
        <Show when=move||!items.with_value(|v| v.is_empty()) fallback=||()>
            <table class="token-table" >
                <thead>
                    <tr>
                     {children()}
                    </tr>
                </thead>
                <tbody>
                    {move||items.get_value().into_iter().map(to_row.get_value()).collect_view()}
                </tbody>
            </table>
        </Show>
    }
}

#[component]
pub fn UserSettings(
    user: crate::auth::User,
    #[prop(default = false)] as_admin: bool,
) -> impl IntoView {
    let user_settings = create_server_action::<crate::auth::ChangeUserInfo>();

    let username = store_value(user.username);
    let username = move || username.get_value();

    let (username_change, set_username_change) = create_signal(false);
    let change_username = move |_| set_username_change.update(|unc| *unc = !*unc);

    let (password_change, set_password_change) = create_signal(false);
    let change_password = move |_| set_password_change.update(|pwc| *pwc = !*pwc);

    let (name, set_name) = create_signal(String::new());

    let set_name = move |ev: Event| set_name(event_target_value(&ev));

    let MIN_PW_LEN: usize = 6;

    let (password, set_password) = create_signal(String::new());
    let (new_password, set_new_password) = create_signal(String::new());
    let (live_new_password, set_live_new_password) = create_signal(String::new());
    let (live_password_confirm, set_live_password_confirm) = create_signal(String::new());

    let set_pw = move |ev: Event| {
        set_password(event_target_value(&ev));
    };
    let set_new_pw = move |ev: Event| {
        set_new_password(event_target_value(&ev));
    };
    let set_live_new_pw = move |ev: Event| {
        set_live_new_password(event_target_value(&ev));
    };
    let set_live_pw_cnf = move |ev: Event| {
        set_live_password_confirm(event_target_value(&ev));
    };

    let invalid_name = move || name() != name().trim();
    let valid_pw_len = move || live_new_password().len() >= MIN_PW_LEN;
    let valid_pw_chars = move || {
        !live_new_password()
            .chars()
            .any(|c| c.is_whitespace() || !c.is_ascii())
    };
    let valid_pw = move || valid_pw_len() && valid_pw_chars();
    let matching_pw = move || live_new_password() == live_password_confirm();
    let wont_be_matching_pw = move || {
        let live_password_confirm = live_password_confirm();
        let len = live_password_confirm.len();
        let live_password = live_new_password();
        len > live_password.len() || &live_password[..len] != &live_password_confirm
    };

    let pw_strength = move || {
        let unit = match live_new_password().len() {
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
        invalid_name()
            || name().is_empty()
            || name() == username()
            // only if changing pw
            || (password_change() && (!valid_pw() || !matching_pw() || new_password().is_empty()))
            || password().is_empty()
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
                    when={move || !new_password().is_empty() && !valid_pw_len()}
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
        <div class="user-settings-container">
        <UserErrorBoundary action=user_settings/>
        <ActionForm action=user_settings>
            <input type="hidden" name="as_admin" prop:value=as_admin />
            <input type="hidden" name="username" prop:value=username() />
            <Show when=username_change fallback=move || view!{
                <button class="change-username" on:click=change_username >
                    <Lang hu="Felhasználónév módosítása" en="Change username"/>
                </button>
            }>
                <button class="change-username" on:click=change_username >
                    <Lang hu="Felhasználónév módosításának elhagyása" en="Cancel username change"/>
                </button>
                <Input name="new_username" on:input=set_name >
                    <Lang hu="Felhasználónév (jelenleg: " en="Username (currently: "/>
                    {username()}
                    ")"
                </Input>
                {name_problems}
            </Show>
            <Show when=password_change fallback=move || view!{
                <button class="change-password" on:click=change_password >
                    <Lang hu="Jelszó megváltoztatása" en="Change password" />
                </button>
            }>
                <button class="change-password" on:click=change_password >
                    <Lang hu="Jelszó megváltoztatásának elhagyása" en="Cancel password change" />
                </button>
                <Input name="new_password" password=true
                    on:change=set_new_pw on:input=set_live_new_pw >
                    <Lang hu="Új jelszó" en="New password" />
                </Input>
                {pw_problems}
                <div class="pw-strength">{pw_strength}</div>
                <Input name="password_confirmation" password=true
                    on:input=set_live_pw_cnf >
                    <Lang hu="Új jelszó újra" en="New password again" />
                </Input>
                {pw_cnf_problems}
            </Show>
            <Show when={move || username_change() || password_change()} fallback=||()>
                <div class="current-password" >
                    <Input name="password" on:input=set_pw password=true >
                        <Lang hu="Jóváhagyás (Jelszó)" en="Confirm changes (Password)" />
                    </Input>
                </div>
                <Submit disable=disable_submit >
                    <Lang hu="" en="Change"/>
                    {move || match (username_change(), password_change()) {
                        (false, false) => view!{<Lang hu="" en=""/>},
                        (true, false) => view!{<Lang hu="Felhasználónév" en="username"/>},
                        (false, true) => view!{<Lang hu="Jelszó" en="password"/>},
                        (true, true) => view!{
                            <Lang hu="Felhasználónév és jelszó" en="username and password"/>
                        }
                    }}
                    <Lang hu=" megváltoztatása" en=""/>
                </Submit>
            </Show>
        </ActionForm>
        </div>
    }
}
