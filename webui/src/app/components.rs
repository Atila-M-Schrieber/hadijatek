//! General-use components

use std::str::FromStr;

use leptos::ev::Event;
use leptos::html::Input;
use leptos::*;
use map_utils::Color;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

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

/// A Color selector component, which takes children as a label,
/// and uses a HTML color selector.
#[component]
pub fn ColorSelector(color: RwSignal<Color>, children: ChildrenFn) -> impl IntoView {
    let color_str = move || color.get().to_string();

    let set_color = move |ev: Event| {
        let color_ = event_target_value(&ev);
        if let Ok(color_) = Color::from_str(&color_) {
            color.set(color_);
        } else {
            log!("{color_:?}");
        }
    };

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
            <input type="color" value=color_str on:input=set_color />
        </div>
    }
}

/// Only renders children when on the client side - useful for leptos_use shenanigans which don't
/// play well with ssr.
#[component]
pub fn ClientOnly(#[allow(unused_variables)] children: Children) -> impl IntoView {
    #[allow(unused_variables)]
    let (children_view, set_children_view) = create_signal(None::<View>);
    #[cfg(any(feature = "csr", feature = "hydrate"))]
    request_animation_frame(move || set_children_view.set(Some(children().into_view())));
    move || children_view.get()
}

async fn copy_to_clipboard_(to_clipboard: String) -> Result<JsValue, JsValue> {
    let window = window(); //.expect("Should have a Window");
    let navigator = window.navigator();
    let clipboard = navigator
        .clipboard()
        .ok_or(JsValue::from_str("No clipboard found"))?;

    let clipboard_promise = clipboard.write_text(&to_clipboard);
    let _written = JsFuture::from(clipboard_promise).await?;
    Ok(JsValue::from_str("Written to clipboard"))
}

pub fn copy_to_clipboard(to_clipboard: String) {
    spawn_local(async move {
        let copied = copy_to_clipboard_(to_clipboard).await;
        if let Err(err) = copied {
            log!("Failed to copy to clipboard: {err:?}")
        }
    })
}
