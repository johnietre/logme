use crate::console::log;
use crate::utils::*;
use crate::datetime_input::*;
use leptos::prelude::*;
use leptos::task::spawn_local as leptos_spawn_local;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[component]
pub fn App() -> impl IntoView {
    let app = AppSettings::new();
    let logged_in = app.logged_in;
    let app = RwSignal::new(Arc::new(app));
    provide_context(app);

    leptos_spawn_local(async move {
        let app = app();
        app.get_user().await;
        if (app.logged_in)() {
            app.get_logs().await;
        }
    });

view! {

<div id="app">

<header id="header">
  <h1 id="title">LogMe</h1>
  <Show when=move || logged_in()>
      <div id="logout-button-div">
        <button
            on:click=move |_| {
                let app = app();
                leptos_spawn_local(async move {
                    app.logout().await;
                })
            }
        >Logout</button>
      </div>
  </Show>
</header>

<Show
    when=move || logged_in()
    fallback=|| view! { <Login /> }
>
    <Main />
</Show>

</div>

}
}

#[component]
fn Main() -> impl IntoView {
    let app = use_context::<RwSignal<ArcApp>>().expect("missing app context");
    let showing_sort_filters = RwSignal::new(false);

    view! {

<main id="main" class="flex-column" v-else>
    <NewEditLog />

    <hr style="width:90%" />


    <button
        on:click=move |_| showing_sort_filters(!showing_sort_filters())
    >
        <span>{move || if showing_sort_filters() {
            "Hide Sort/Filters"
        } else {
            "Show Sort/Filters"
        }}</span>
    </button>
    <Show when=move || showing_sort_filters()>
        <SortFiltersForm showing_sort_filters=showing_sort_filters />
    </Show>

    <div>
    <button
        on:click=move |_| {
            app.update(|app| {
                // TODO: check
                let app = app.clone();
                leptos_spawn_local(async move {
                    app.get_logs().await;
                });
            });
        }
        style="margin:2px">"Refresh"</button>
    </div>

    <div id="check-buttons">
        <button on:click=move |_| app.update(|app| app.uncheck_all())>"Uncheck All"</button>
        <button on:click=move |_| app.update(|app| app.check_all())>"Check All"</button>
        <button on:click=move |_| {
            app.update(|app| {
                // TODO: check
                let app = app.clone();
                leptos_spawn_local(async move {
                    app.delete_checked().await;
                });
            });
        }>"Delete Checked"</button>
    </div>

    <Logs />
</main>

}
}

#[component]
fn SortFiltersForm(
    showing_sort_filters: RwSignal<bool>,
) -> impl IntoView {
    let app = use_context::<RwSignal<ArcApp>>().expect("missing app context");
    let sort_filters = app.with(|app| app.new_sort_filters);
    let start_timestamp = RwSignal::new(sort_filters.with(|sf| sf.start));
    let end_timestamp = RwSignal::new(sort_filters.with(|sf| sf.end));
    let sort_by_init = sort_filters.with(|sf| sf.sort_by);

view! {

<form
    id="sort-filters-form" class="flex-column"
    on:submit=move |event| {
        app.update(|app| {
            // TODO: check
            let app = app.clone();
            leptos_spawn_local(async move {
                app.get_logs().await;
            });
        });
        showing_sort_filters(false);
        event.prevent_default();
    }
>
    <div id="sort-div">
        <div>
            <label>Sort By:</label>
            <select
                name="sort"
                on:change:target=move |event| {
                    sort_filters.update(|sf| {
                        sf.sort_by = SortBy::from_str(&event.target().value());
                    })
                }
                prop:value=move || sort_filters.with(|sf| sf.sort_by.as_str())
            >
                <option
                    prop:value={SortBy::Time.as_str()}
                    prop:selected={sort_by_init == SortBy::Time}
                >"Time"</option>
                <option
                    prop:value={SortBy::Added.as_str()}
                    prop:selected={sort_by_init == SortBy::Added}
                >"Added"</option>
            </select>
        </div>

        <div>
            <label>Desc:</label>
                <input
                    type="checkbox" name="desc"
                    prop:checked=move || sort_filters.with(|sf| sf.desc)
                    on:input:target=move |event| {
                        sort_filters.update(|sf| sf.desc = event.target().checked());
                    }
                />
        </div>
    </div>

    <hr style="width:90%" />

    <div id="filter-div">
        <div>
            <label>"Start:"</label>
            {move || {
                let timestamp = start_timestamp();
                view! {
                    <DateTimeInput datetime=timestamp />
                }
            }}
        </div>
        <div>
            <label>"End:"</label>
            {move || {
                let timestamp = end_timestamp();
                view! {
                    <DateTimeInput datetime=timestamp />
                }
            }}
        </div>

        <div>
            <label>"Limit:"</label>
            <input
                type="number" name="limit"
                min="1" step="1" size="6"
                on:input:target=move |event| {
                    let Ok(limit) = event.target().value().parse::<u32>() else {
                        event.prevent_default();
                        return;
                    };
                    sort_filters.update(|sf| sf.limit = limit);
                }
                prop:value=move || sort_filters.with(|sf| sf.limit).to_string()
            />
        </div>
        <div>
            <label>"Offset:"</label>
            <input
                type="number" name="offset"
                min="0" step="1" size="8"
                on:input:target=move |event| {
                    let Ok(offset) = event.target().value().parse::<u32>() else {
                        event.prevent_default();
                        return;
                    };
                    sort_filters.update(|sf| sf.offset = offset);
                }
                prop:value=move || sort_filters.with(|sf| sf.offset).to_string()
            />
        </div>
    </div>

    <hr style="width:90%" />

    <div id="reset-cancel-buttons-div">
        <button
            on:click=move |_| {
                app.with(|app| app.cancel_sort_filters());
                showing_sort_filters(false);
            }
        >"Cancel"</button>
        <button type="submit">"Apply"</button>
        <button
            on:click=move |_| app.with(|app| app.reset_sort_filters())
        >"Reset"</button>
    </div>
</form>

}
}

#[component]
fn Logs() -> impl IntoView {
    let app = use_context::<RwSignal<ArcApp>>().expect("missing app context");
    let editing_log = app.with(|app| app.editing_log);
    let logs = app.with(|app| app.logs);
    let checked_logs = app.with(|app| app.checked_logs);

    view! {
<table
    //width="100%">
    style="width:100%"
>
    <tbody>
    {move || {
        logs().into_iter().enumerate().map(|(index, log)| view! {
            <tr>
                <td style="max-width:70vw">
                    <div style="text-align:center;font-family:monospace">
                        <u>{
                            ts_to_datetime_str(log.timestamp.with(|dt| dt.timestamp()))
                        }</u>
                    </div>
                    <div>{(log.msg)()}</div>
                </td>
                <td style="text-align:center;padding:0;width:7ch">
                    <button
                        on:click=move |_| {
                            if editing_log() == Some(index) {
                                app.update(|app| app.cancel_editing());
                            } else {
                                app.update(|app| app.edit_log(index))
                            }
                        }
                        style="margin:0"
                    >
                    <span>
                    {move || if editing_log() == Some(index) {
                        "Cancel"
                    } else {
                        "Edit"
                    }}
                    </span>
                    </button>
                </td>
                <td style="text-align:center;padding:0">
                    <input
                        type="checkbox"
                        prop:checked=move || {
                            checked_logs.with(|checked| *checked.get(index).unwrap_or(&false))
                        }
                        on:input:target=move |event| {
                            checked_logs.update(|checked| {
                                if index < checked.len() {
                                    checked[index] = event.target().checked();
                                }
                            });
                        }
                        />
                </td>
            </tr>
        }).collect_view()
    }}
    </tbody>
</table>
    }
}

#[component]
fn NewEditLog() -> impl IntoView {
    let app = use_context::<RwSignal<ArcApp>>().expect("missing app context");
    let logs = app.with(|app| app.logs);
    let editing_log = app.with(|app| app.editing_log);
    let new_log = app.with(|app| app.new_log);
    let new_log_msg = RwSignal::new(new_log.with(|log| log.msg));
    let new_log_timestamp = RwSignal::new(new_log.with(|log| log.timestamp));

    Effect::new(move || {
        if let Some(i) = editing_log() {
            logs.with(|logs| {
                if logs[i].timestamp != new_log_timestamp() {
                    new_log_msg(logs[i].msg);
                    new_log_timestamp(logs[i].timestamp);
                }
            });
        } else {
            new_log.with(|log| {
                if log.timestamp != new_log_timestamp() {
                    new_log_msg(log.msg);
                    new_log_timestamp(log.timestamp);
                }
            });
        }
    });

    view! {
<form
    id="new-log-form"
    class="flex-column"
    on:submit=move |event| {
        app.update(|app| {
            // TODO: check
            let app = app.clone();
            leptos_spawn_local(async move {
                app.submit_log().await;
            });
        });
        event.prevent_default();
    }
>
    <h3>
        <span>
            {move || if editing_log().is_none() {
                "New Log"
            } else {
                "Editing Log"
            }}
        </span>
    </h3>
    <div>
        {move || {
            let msg = new_log_msg();
            view! {
                <input
                    type="text" name="msg"
                    style="width:95vw" placeholder="Message" autocomplete="off"
                    prop:value=move || msg()
                    on:input:target=move |event| {
                        msg(event.target().value());
                    }
                />
            }
        }}
    </div>

    <div>
        {move || {
            let timestamp = new_log_timestamp();
            view! {
                <DateTimeInput datetime=timestamp has_now=true />
            }
        }}
        /*
        {move || new_log.with(|log| {
            view! {
                <DateTimeInput datetime=log.timestamp has_now=true />
            }
        })}
        */
    </div>

    <div>
        {move || if editing_log().is_some() {
            Some(view! {
                <button
                    type="button"
                    on:click=move |_| app.update(|app| app.cancel_editing())
                >"Cancel"</button>
            })
        } else {
            None
        }}
        <button type="submit" style="margin:2px">"Submit"</button>
    </div>
</form>

}
}

#[component]
fn Login() -> impl IntoView {
    let app = use_context::<RwSignal<ArcApp>>().expect("missing app context");
    let registering = RwSignal::new(false);

    view! {

<div id="login">
    <form
        id="login-form"
        class="flex-column"
        on:submit=move |event| {
            let app = app();
            if registering() {
                leptos_spawn_local(async move {
                    app.register().await;
                });
            } else {
                leptos_spawn_local(async move {
                    app.login().await;
                });
            }
            event.prevent_default();
        }
    >
        <div>
            <label for="email">"Email:"</label>
            <input
                type="email"
                name="email"
                placeholder="Email"
                prop:value=move || app.with(|app| (app.email)())
                on:input:target=move |event| {
                    app.update(|app| (app.email)(event.target().value()))
                }
                required=true
            />
        </div>
        <div>
            <label for="password">"Password:"</label>
            <input
                type="password"
                name="password"
                placeholder="Password"
                prop:value=move || app.with(|app| (app.password)())
                on:input:target=move |event| {
                    app.update(|app| (app.password)(event.target().value()))
                }
                required=true
            />
        </div>

        {move || if registering() {
            Some(view! {
                <div>
                    <label for="password-conf">"Password Confirmation:"</label>
                    <input
                        type="password"
                        name="password-conf"
                        placeholder="Password Confirmation"
                        prop:value=move || app.with(|app| (app.password_conf)())
                        on:input:target=move |event| {
                            app.update(|app| (app.password_conf)(event.target().value()))
                        }
                        required=true
                    />
                </div>
            })
        } else {
            None
        }}
        <div>
            <button type="submit">Submit</button>
            {move || if registering() {
                view! {
                    <button on:click=move |_| registering(false)>"Go To Login"</button>
                }.into_any()
            } else {
                view! {
                    <button on:click=move |_| registering(true)>"Go To Registration"</button>
                }.into_any()
            }}
        </div>
    </form>
</div>

}
}

#[derive(Clone, Serialize, Deserialize)]
struct User {
    #[serde(default)]
    id: i64,
    #[serde(default)]
    email: String,
    #[serde(default)]
    deleted: bool,
    #[serde(default)]
    max_logs: u32,
}

impl User {
    fn new() -> Self {
        Self {
            id: 0,
            email: String::new(),
            deleted: false,
            max_logs: 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Response<T> {
    content: Option<T>,
    error: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Log {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub timestamp: RwSignal<DateTime>,
    #[serde(default)]
    pub msg: RwSignal<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(rename="counterpartId")]
    pub counterpart_id: Option<i64>,
}

impl Log {
    pub fn new() -> Self {
        Self {
            id: 0,
            timestamp: RwSignal::new(DateTime::empty()),
            msg: RwSignal::new(String::new()),
            tags: Vec::new(),
            counterpart_id: None,
        }
    }
}

/*
#[derive(Clone, Serialize, Deserialize)]
struct LogDiff {
    pub id: i64,
    pub timestamp: Option<DateTime>,
    pub msg: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(rename="counterpartId")]
    pub counterpart_id: Option<i64>,
}
*/

type ArcApp = Arc<AppSettings>;

pub struct AppSettings {
    client: Arc<reqwest::Client>,

    user: RwSignal<User>,
    email: RwSignal<String>,
    password: RwSignal<String>,
    password_conf: RwSignal<String>,
    logged_in: RwSignal<bool>,

    tok_str: RwSignal<String>,

    new_log: RwSignal<Log>,
    editing_log: RwSignal<Option<usize>>,
    logs: RwSignal<Vec<Log>>,
    checked_logs: RwSignal<Vec<bool>>,

    sort_filters: RwSignal<SortFilters>,
    new_sort_filters: RwSignal<SortFilters>,
}

impl AppSettings {
    fn new() -> Self {
        Self {
            client: Arc::new(reqwest::Client::new()),

            user: RwSignal::new(User::new()),
            email: RwSignal::new(String::new()),
            password: RwSignal::new(String::new()),
            password_conf: RwSignal::new(String::new()),
            logged_in: RwSignal::new(false),

            tok_str: RwSignal::new(String::new()),

            new_log: RwSignal::new(Log::new()),
            editing_log: RwSignal::new(None),
            logs: RwSignal::new(Vec::new()),
            checked_logs: RwSignal::new(Vec::new()),

            sort_filters: RwSignal::new(SortFilters::default()),
            new_sort_filters: RwSignal::new(SortFilters::default()),
        }
    }

    async fn get_user(&self) {
        let resp = self.new_req(reqwest::Method::GET, "/user").send().await;
        let resp = match resp {
            Ok(resp) => resp,
            Err(e) => {
                log!("error sending get user request: {e}");
                alert("Error getting user");
                return;
            }
        };
        let status = resp.status();
        if !status.is_success() {
            let text = Self::get_err(resp).await;
            if status == reqwest::StatusCode::UNAUTHORIZED {
                self.logout().await;
                log!("logged out with error: {text}");
                alert("Logged out");
            } else {
                log!("error getting user: {text}");
                alert(&format!("Error getting user: {text}"));
            }
            return;
        }
        let mut resp = match resp.json::<Response<User>>().await {
            Ok(resp) => resp,
            Err(e) => {
                log!("error reading JSON: {e}");
                alert("Error getting user");
                return;
            }
        };
        if let Some(err) = resp.error.as_ref() {
            log!("received error getting user: {err}");
            alert("Received error getting user");
        }
        if let Some(content) = resp.content.take() {
            (self.user)(content);
            (self.logged_in)(true);
        } else if resp.error.is_none() {
            log!("no user data received even though no error received");
        }
    }

    async fn get_logs(&self) {
        let sort_filters = (self.new_sort_filters)();
        (self.sort_filters)(sort_filters.clone());
        let mut req = self.new_req(reqwest::Method::GET, "/logs")
            .query(&[("sort", sort_filters.sort_by.as_str())])
            .query(&[("desc", sort_filters.desc)])
            .query(&[("limit", sort_filters.limit)])
            .query(&[("offset", sort_filters.offset)]);
        let start = (sort_filters.start)().timestamp();
        if start > 0 {
            req = req.query(&[("start", start)]);
        }
        let end = (sort_filters.end)().timestamp();
        if end > 0 {
            req = req.query(&[("end", end)]);
        }
        let resp = req.send().await;
        let resp = match resp {
            Ok(resp) => resp,
            Err(e) => {
                log!("error sending get logs request: {e}");
                alert("Error getting logs");
                return;
            }
        };
        let status = resp.status();
        if !status.is_success() {
            let text = Self::get_err(resp).await;
            if status == reqwest::StatusCode::UNAUTHORIZED {
                self.logout().await;
                log!("logged out with error: {text}");
                alert("Logged out");
            } else {
                log!("error getting logs: {text}");
                alert(&format!("Error getting logs: {text}"));
            }
            return;
        }
        let mut resp = match resp.json::<Response<Vec<Log>>>().await {
            Ok(resp) => resp,
            Err(e) => {
                log!("error reading JSON: {e}");
                alert("Error getting logs");
                return;
            }
        };
        if let Some(err) = resp.error.as_ref() {
            log!("received partial error getting logs: {err}");
            alert("Partial error getting logs");
        }
        if let Some(content) = resp.content.take() {
            (self.checked_logs)(vec![false; content.len()]);
            (self.logs)(content);
        }
    }

    async fn submit_log(&self) {
        let (log, method, editing) = match (self.editing_log)() {
            Some(i) => (self.logs.with(|logs| logs[i].clone()), reqwest::Method::PUT, true),
            None => ((self.new_log)(), reqwest::Method::POST, false),
        };
        let resp = self.new_req(method, "/logs")
            .query(&[("id", log.id)])
            .json(&log)
            .send()
            .await;
        let resp = match resp {
            Ok(resp) => resp,
            Err(e) => {
                log!("error sending submit log request: {e}");
                alert("Error submitting log");
                return;
            }
        };
        let status = resp.status();
        if !status.is_success() {
            let text = Self::get_err(resp).await;
            if status == reqwest::StatusCode::UNAUTHORIZED {
                self.logout().await;
                log!("logged out with error: {text}");
                alert("Logged out");
            } else {
                log!("error submitting log: {text}");
                alert(&format!("Error submitting log: {text}"));
            }
            return;
        }
        let mut resp = match resp.json::<Response<User>>().await {
            Ok(resp) => resp,
            Err(e) => {
                log!("error reading JSON: {e}");
                alert("Error submitting log");
                return;
            }
        };
        if let Some(err) = resp.error.as_ref() {
            log!("received error submitting log: {err}");
            alert("Received error submitting log");
        }
        if let Some(_content) = resp.content.take() {
        }
        if editing {
            (self.editing_log)(None);
        } else {
            (self.new_log)(Log::new());
        }
        self.get_logs().await;
    }

    async fn delete_checked(&self) {
        let mut ids = Vec::new();
        self.logs.with(|logs| {
            self.checked_logs.with(|checked_logs| {
                for i in 0..logs.len() {
                    if checked_logs[i] {
                        ids.push(logs[i].id.to_string());
                    }
                }
            });
        });
        if ids.len() == 0 {
            return;
        } else if !crate::utils::confirm("Are you sure?") {
            return;
        }
        let resp = self.new_req(reqwest::Method::DELETE, "/logs")
            .query(&[("ids", ids.join(","))])
            .send()
            .await;
        let resp = match resp {
            Ok(resp) => resp,
            Err(e) => {
                log!("error sending delete logs request: {e}");
                alert("Error deleting log(s)");
                return;
            }
        };
        let status = resp.status();
        if !status.is_success() {
            let text = Self::get_err(resp).await;
            if status == reqwest::StatusCode::UNAUTHORIZED {
                self.logout().await;
                log!("logged out with error: {text}");
                alert("Logged out");
            } else {
                log!("error deleting log(s): {text}");
                alert(&format!("Error deleting log(s): {text}"));
            }
            return;
        }
        let resp = match resp.json::<Response<User>>().await {
            Ok(resp) => resp,
            Err(e) => {
                log!("error reading JSON: {e}");
                alert("Error deleting log(s)");
                return;
            }
        };
        if let Some(err) = resp.error.as_ref() {
            log!("received error deleting log(s): {err}");
            alert("Received error deleting log(s)");
        } else {
            self.get_logs().await;
        }
    }

    fn edit_log(&self, index: usize) {
        if index >= self.logs.with(|logs| logs.len()) {
            return;
        }
        (self.editing_log)(Some(index));
    }

    fn cancel_editing(&self) {
        (self.editing_log)(None);
    }

    fn check_all(&self) {
        self.checked_logs.update(|checked| {
            for i in 0..checked.len() {
                checked[i] = true;
            }
        });
    }

    fn uncheck_all(&self) {
        self.checked_logs.update(|checked| {
            for i in 0..checked.len() {
                checked[i] = false;
            }
        });
    }

    async fn logout(&self) {
        let _ = self.new_req(reqwest::Method::DELETE, "/token").send().await;
        (self.email)((self.user)().email);
        (self.user)(User::new());
        (self.logged_in)(false);
    }

    async fn login(&self) {
        let resp = self.new_req(reqwest::Method::POST, "/token")
            .header(
                "Authorization",
                format!(
                    "Basic {}",
                    crate::utils::btoa(&format!("{}:{}", (self.email)(), (self.password)())),
                ),
            )
            .send()
            .await;
        let resp = match resp {
            Ok(resp) => resp,
            Err(e) => {
                log!("error sending login request: {e}");
                alert("Error logging in");
                return;
            }
        };
        let status = resp.status();
        if !status.is_success() {
            let text = Self::get_err(resp).await;
            log!("error logging in: {text}");
            alert(&format!("Error logging in: {text}"));
            return;
        }
        let mut resp = match resp.json::<Response<String>>().await {
            Ok(resp) => resp,
            Err(e) => {
                log!("error reading JSON: {e}");
                alert("Error logging in");
                return;
            }
        };
        if let Some(err) = resp.error.take() {
            log!("received error logging in: {err}");
            alert("Received error logging in");
        }
        if let Some(content) = resp.content.take() {
            (self.tok_str)(content);
            (self.logged_in)(true);
        }
        self.get_user().await;
        self.get_logs().await;
    }

    async fn register(&self) {
        let pwd = (self.password)();
        if pwd != (self.password_conf)() {
            alert("Passwords must match");
            return;
        }
        let email = (self.email)();
        let resp = self.new_req(reqwest::Method::POST, "/user")
            .header(
                "Authorization",
                format!("Basic {}", crate::utils::btoa(&format!("{email}:{pwd}"))),
            )
            .json(&serde_json::json!({ "email": email }))
            .send()
            .await;
        let resp = match resp {
            Ok(resp) => resp,
            Err(e) => {
                log!("error sending register request: {e}");
                alert("Error registering");
                return;
            }
        };
        let status = resp.status();
        if !status.is_success() {
            let text = Self::get_err(resp).await;
            log!("error registering: {text}");
            alert(&format!("Error registering: {text}"));
            return;
        }
        let mut resp = match resp.json::<Response<User>>().await {
            Ok(resp) => resp,
            Err(e) => {
                log!("error reading JSON: {e}");
                alert("Error registering");
                return;
            }
        };
        if let Some(err) = resp.error.take() {
            log!("received error registering: {err}");
            alert("Received error registering");
        }
        if let Some(_content) = resp.content.take() {
        }
        (self.password_conf)(String::new());
        self.login().await;
    }

    fn new_req(
        &self,
        method: reqwest::Method,
        path: &str,
    ) -> reqwest::RequestBuilder {
        let loc = document().location().expect("document missing location");
        let href = loc.href().expect("document location href error");
        let mut url = reqwest::Url::parse(&href).expect("invalid document location href URL");
        url.set_path(path);
        self.client
            .request(method, url)
            .header("Authorization", format!("Bearer {}", (self.tok_str)()))
    }

    fn reset_sort_filters(&self) {
        (self.new_sort_filters)(SortFilters::default());
    }

    fn cancel_sort_filters(&self) {
        (self.new_sort_filters)((self.sort_filters)());
    }

    async fn get_err(resp: reqwest::Response) -> String {
        let status = resp.status();
        let text = resp.text().await.unwrap_or(String::new());
        if text.len() == 0 {
            return status.as_str().to_string();
        }
        match serde_json::from_str::<Response<()>>(&text) {
            Ok(js_resp) => match js_resp.error {
                Some(err) => err,
                // NOTE: return text or status?
                None => status.as_str().to_string(),
            }
            Err(_) => text,
        }
    }
}

#[derive(Clone)]
struct SortFilters {
    sort_by: SortBy,
    desc: bool,
    start: RwSignal<DateTime>,
    end: RwSignal<DateTime>,
    limit: u32,
    offset: u32,
}

impl Default for SortFilters {
    fn default() -> Self {
        Self {
            sort_by: SortBy::default(),
            desc: true,
            start: RwSignal::new(DateTime::empty()),
            end: RwSignal::new(DateTime::empty()),
            limit: 50,
            offset: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
enum SortBy {
    #[default]
    Time,
    Added,
}

impl SortBy {
    //const VALUES: &[Self] = &[SortBy::Time, SortBy::Added];

    fn from_str(s: &str) -> Self {
        match s {
            "time" => SortBy::Time,
            "added" => SortBy::Added,
            _ => Self::default(),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            SortBy::Time => "timestamp",
            SortBy::Added => "id",
        }
    }
}

fn ts_to_datetime_str(ts: i64) -> String {
    let Some(dt) = chrono::DateTime::from_timestamp(ts, 0) else {
        return "INVALID DATE".into();
    };
    format!("{}", dt.format("%Y/%m/%d %H:%M:%S"))
}
