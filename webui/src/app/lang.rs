use leptos::*;

#[derive(Debug, Clone, Copy)]
pub enum Language {
    Hungarian,
    English,
}

#[component]
pub fn Lang<S>(cx: Scope, hu: S, en: S) -> impl IntoView
where
    S: ToString + 'static,
{
    use Language::*;
    let lang = expect_context::<ReadSignal<Language>>(cx);

    // must be a closure
    move || match lang() {
        Hungarian => hu.to_string(),
        English => en.to_string(),
    }
}
