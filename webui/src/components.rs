//! General-use components

use leptos::ev::Event;
use leptos::*;

use crate::error::*;
use crate::lang::*;

/// Generic text input component.
/// Takes the children as the label, and sets name= to every relevant thing.
/// Currently always a required input.
#[component]
pub fn Input<S, FI, FC>(
    name: S,
    children: ChildrenFn,
    #[prop(optional)] password: bool,
    input: FI,
    change: FC,
) -> impl IntoView
where
    S: ToString,
    // Need different types because they're different closures
    FI: Fn(Event) + 'static,
    FC: Fn(Event) + 'static,
{
    let mut name = name.to_string();
    if name.is_empty() {
        if password {
            name = "password".into();
        } else {
            panic!("You must provide a name for non-password inputs!");
        }
    }
    view! {
        <div class="input-group">
            <label for=&name>{children()}</label>
            <input type=if password {"password"} else {"text"} id=&name name=&name
                on:change=change on:input=input required/>
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
