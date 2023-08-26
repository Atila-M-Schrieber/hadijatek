// Separate things into files without it seeming separate

pub use chrono::offset::Local;
pub use chrono::offset::Utc;
pub use chrono::DateTime;
pub use leptos::ev::Event;
pub use leptos::*;
pub use leptos_router::*;

pub use crate::app::components::*;
pub use crate::auth::{
    token::{map::*, user::*},
    *,
};
pub use crate::error::*;
pub use crate::lang::*;

mod login;
pub use login::*;
mod signup;
pub use signup::*;
mod settings;
pub use settings::*;

/// The login button
#[component]
pub fn UserButton(logout: Action<Logout, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <Transition fallback=move||
            view!{
                <Lang hu="Bejelentkezés..." en="Loging in..."/>
            }
        >
        <ErrorBoundary
            fallback=move |_err| {
                view!{
                    <a href="/login"><Lang hu="Bejelentkezés" en="Log in"/></a>}
            }
        >
        {
            move || { with_user(|user| {
                view!{
                    <a href="/settings">
                        <Lang hu="Beállítások" en="Settings"/>" ("{user.username.clone()}")"
                    </a>
                    <div class="dropdown-content">
                        <a href="#" on:click=move|_|logout.dispatch(Logout {})>
                            <Lang hu="Kijelentkezés" en="Log out"/>
                        </a>
                    </div>
                }
            })}
        }
        </ErrorBoundary>
        </Transition>
    }
}

#[component]
pub fn UserSettings(
    user: crate::auth::User,
    #[prop(default = false)] as_admin: bool,
) -> impl IntoView {
    let user_settings =
        expect_context::<Action<crate::auth::ChangeUserInfo, Result<(), ServerFnError>>>();

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
        len > live_password.len() || live_password[..len] != live_password_confirm
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
                <button type="button" class="change-username" on:click=change_username >
                    <Lang hu="Felhasználónév módosítása" en="Change username"/>
                </button>
            }>
                <button type="button" class="change-username" on:click=change_username >
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
                <button type="button" class="change-password" on:click=change_password >
                    <Lang hu="Jelszó megváltoztatása" en="Change password" />
                </button>
            }>
                <button type="button" class="change-password" on:click=change_password >
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
