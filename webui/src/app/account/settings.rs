use super::*;

/// The settings page - admin / regular
#[component]
pub fn SettingsPage() -> impl IntoView {
    let is_admin = move || with_user(|user| user.role) == Ok(UserRole::Admin);
    let logged_in = move || with_user(|_| ()).is_ok();

    view! {
        <Show when=logged_in fallback=||()>
            <Show when=is_admin fallback=RegularSettingsPage>
                <AdminSettingsPage/>
            </Show>
        </Show>
    }
}

/// The regular settings page
#[component]
pub fn RegularSettingsPage() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <h1>"Welcome to Regular Settings!"</h1>
        <button on:click=on_click><Lang hu="Nyomjá'meg" en="Click Me"/>": " {count}</button>

        <h2><Lang hu="Felhasználói beállítások" en="User settings"/></h2>
        <ErrorBoundary fallback=|_| "Some user error occurred" >
        {move || with_user(|user| view!{
            <UserSettings user=user.clone() />
        })}
        </ErrorBoundary>
    }
}

/// The admin settings page
#[component]
pub fn AdminSettingsPage() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    // User tokens
    let create_user_token = create_server_action::<CreateUserToken>();
    let delete_user_token = create_server_action::<DeleteUserToken>();

    let user_tokens = create_resource(
        move || {
            (
                create_user_token.version().get(),
                delete_user_token.version().get(),
            )
        },
        move |_| get_user_token_info(),
    );

    let list_user_tokens = move || -> Result<View, ServerFnError> {
        let mut tokens = user_tokens
            .get()
            .ok_or(ServerFnError::ServerError("bruh".into()))??;

        let used_tokens =
            if let Some(pos) = tokens.iter().position(|(_, consumer)| consumer.is_some()) {
                tokens.split_off(pos)
            } else {
                Vec::new()
            };

        let tokens_store = store_value(tokens);
        let used_tokens_store = store_value(used_tokens);

        let to_row = move |(token, consumer): UserCreationToken| {
            let consumer = store_value(consumer);
            let consumer = move || consumer.get_value();
            let token = store_value(token);
            let token = move || token.get_value();

            let class = if consumer().is_none() {
                "active"
            } else {
                "consumed"
            };

            let time =
                |t: DateTime<Utc>| t.with_timezone(&Local).format("%d/%m/%Y %H:%M").to_string();

            // copy-on-click
            let copy_token = move |_| copy_to_clipboard(token().token);

            view! {
                <tr>
                    <td class=class title="Copy" on:click=copy_token >
                            {token().token}
                    </td>
                    <td>{time(token().created)}</td>
                    <td>{consumer().map(|(user, _)| user.username)}</td>
                    <td><Show when=move||consumer().is_none()
                            fallback=move||consumer().map(|(_, t)| time(t)) >
                            <ActionForm action=delete_user_token>
                                <input type="hidden" name="token" value=token().token />
                                <Submit disable=||false >
                                    <Lang hu="Törlés"
                                        en="Delete" />
                                </Submit>
                            </ActionForm>
                        </Show>
                    </td>
                </tr>
            }
        };

        Ok(view! {
            <Table items=tokens_store to_row=to_row >
                <th><Lang hu="Token" en="Token" /></th>
                <th><Lang hu="Létrehozás ideje" en="Creation Time" /></th>
                <th><Lang hu="" en="" /></th>
                <th><Lang hu="Törlés" en="Delete" /></th>
            </Table>
            <Table items=used_tokens_store to_row=to_row >
                <th><Lang hu="Token" en="Token" /></th>
                <th><Lang hu="Létrehozás ideje" en="Creation Time" /></th>
                <th><Lang hu="Felhasználó" en="User" /></th>
                <th><Lang hu="Felhasználás ideje" en="Consumption Time" /></th>
            </Table>
        }
        .into_view())
    };

    // Map tokens
    let create_map_token = create_server_action::<CreateMapToken>();
    let delete_map_token = create_server_action::<DeleteMapToken>();

    let map_tokens = create_resource(
        move || {
            (
                create_map_token.version().get(),
                delete_map_token.version().get(),
            )
        },
        move |_| get_map_token_info(),
    );

    let list_map_tokens = move || -> Result<View, ServerFnError> {
        let mut tokens = map_tokens
            .get()
            .ok_or(ServerFnError::ServerError("bruh".into()))??;

        let mut claimed_tokens = if let Some(pos) = tokens
            .iter()
            .position(|(_, _, consumer)| consumer.is_some())
        {
            tokens.split_off(pos)
        } else {
            Vec::new()
        };

        let used_tokens = if let Some(pos) = tokens
            .iter()
            .position(|(_, consumer, _)| consumer.is_some())
        {
            claimed_tokens.split_off(pos)
        } else {
            Vec::new()
        };

        let tokens_store = store_value(tokens);
        let claimed_tokens_store = store_value(claimed_tokens);
        let used_tokens_store = store_value(used_tokens);

        let to_row = move |(token, map_consumer, user_consumer): MapCreationToken| {
            let token = store_value(token);
            let token = move || token.get_value();
            let map_consumer = store_value(map_consumer);
            let map_consumer = move || map_consumer.get_value();
            let user_consumer = store_value(user_consumer);
            let user_consumer = move || user_consumer.get_value();

            let class = if user_consumer().is_none() {
                "active"
            } else {
                "consumed"
            };

            let time =
                |t: DateTime<Utc>| t.with_timezone(&Local).format("%d/%m/%Y %H:%M").to_string();

            // copy-on-click
            let copy_token = move |_| copy_to_clipboard(token().token);

            view! {
                <tr>
                    <td class=class title="Copy" on:click=copy_token >
                            {token().token}
                    </td>
                    <td>{time(token().created)}</td>
                    <td>{map_consumer().map(|(map, _)| map.0)}</td>
                    <td>{map_consumer().map(|(_, t)| time(t))}</td>
                    <td>{user_consumer().map(|(user, _)| user.username)}</td>
                    <td><Show when=move||user_consumer().is_none()
                            fallback=move||user_consumer().map(|(_, t)| time(t)) >
                            <ActionForm action=delete_map_token>
                                <input type="hidden" name="token" value=token().token />
                                <Submit disable=||false >
                                    <Lang hu="Törlés"
                                        en="Delete" />
                                </Submit>
                            </ActionForm>
                        </Show>
                    </td>
                </tr>
            }
        };

        Ok(view! {
            <Table items=tokens_store to_row=to_row >
                <th><Lang hu="Token" en="Token" /></th>
                <th><Lang hu="Létrehozás ideje" en="Creation Time" /></th>
                <th><Lang hu="" en="" /></th>
                <th><Lang hu="" en="" /></th>
                <th><Lang hu="" en="" /></th>
                <th><Lang hu="Törlés" en="Delete" /></th>
            </Table>
            <Table items=claimed_tokens_store to_row=to_row >
                <th><Lang hu="Token" en="Token" /></th>
                <th><Lang hu="Létrehozás ideje" en="Creation Time" /></th>
                <th><Lang hu="" en="" /></th>
                <th><Lang hu="" en="" /></th>
                <th><Lang hu="Felhasználó" en="User" /></th>
                <th><Lang hu="Igénybevétel ideje" en="Claim Time" /></th>
            </Table>
            <Table items=used_tokens_store to_row=to_row >
                <th><Lang hu="Token" en="Token" /></th>
                <th><Lang hu="Létrehozás ideje" en="Creation Time" /></th>
                <th><Lang hu="Térkép" en="Map" /></th>
                <th><Lang hu="Felhasználás ideje" en="Consumption Time" /></th>
                <th><Lang hu="Felhasználó" en="User" /></th>
                <th><Lang hu="Igénybevétel ideje" en="Claim Time" /></th>
            </Table>
        }
        .into_view())
    };

    let (is_user_tokens_expanded, set_is_user_tokens_expanded) = create_signal(false);
    let (is_map_tokens_expanded, set_is_map_tokens_expanded) = create_signal(false);

    let expand_user_tokens = move |_| set_is_user_tokens_expanded.update(|i_e| *i_e = !*i_e);
    let expand_map_tokens = move |_| set_is_map_tokens_expanded.update(|i_e| *i_e = !*i_e);

    view! {
        <h1>"Welcome to Admin Settings!"</h1>
        <button on:click=on_click><Lang hu="Nyomjá'meg" en="Click Me"/>": " {count}</button>
        <div class="panel" class:expanded=is_user_tokens_expanded >
            <div class="panel-heading" on:click=expand_user_tokens >
                <span class="arrow-icon">">"</span>" "
                <Lang hu="Regisztrációs tokenek" en="User creation tokens"/>
            </div>
            <div class="panel-content" >
                <ActionForm action=create_user_token class="create-token-form" >
                    <Submit disable=||false >
                        <Lang hu="Új regisztrációs token létrehozása" en="Create signup token" />
                    </Submit>
                </ActionForm>
                <Transition fallback=move || view! {
                    <p><Lang hu="Tokenek betöltése.." en="Loading tokens..."/></p>
                }>
                    <ErrorBoundary fallback=|_| view!{<p>"Something's gone wrong :("</p>}>
                        {list_user_tokens}
                    </ErrorBoundary>
                </Transition>
            </div>
        </div>
        <div class="panel" class:expanded=is_map_tokens_expanded >
            <div class="panel-heading" on:click=expand_map_tokens >
                <span class="arrow-icon">">"</span>" "
                <Lang hu="Térkép-előállítási tokenek" en="Map creation tokens"/>
            </div>
            <div class="panel-content" >
                <ActionForm action=create_map_token class="create-token-form" >
                    <Submit disable=||false >
                        <Lang hu="Új térkép token létrehozása" en="Create map token" />
                    </Submit>
                </ActionForm>
                <Transition fallback=move || view! {
                    <p><Lang hu="Tokenek betöltése.." en="Loading tokens..."/></p>
                }>
                    <ErrorBoundary fallback=|_| view!{<p>"Something's gone wrong :("</p>}>
                        {list_map_tokens}
                    </ErrorBoundary>
                </Transition>
            </div>
        </div>
    }
}
