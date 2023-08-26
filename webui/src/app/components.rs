//! General-use components

use leptos::html::Input;
use leptos::*;

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
