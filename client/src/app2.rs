use crate::console::log;
use crate::datetime_input::*;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize)]
pub struct Log {
    pub id: i64,
    pub timestamp: RwSignal<DateTime>,
    pub msg: String,
    pub tags: Vec<String>,
    #[serde(rename="counterpartId")]
    pub counterpart_id: Option<i64>,
}

impl Log {
    pub fn new() -> Self {
        Self {
            id: 0,
            timestamp: RwSignal::new(DateTime::empty()),
            msg: String::new(),
            tags: Vec::new(),
            counterpart_id: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct LogDiff {
    pub id: i64,
    pub timestamp: Option<DateTime>,
    pub msg: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(rename="counterpartId")]
    pub counterpart_id: Option<i64>,
}

type ArcApp = Arc<AppSettings>;

pub struct AppSettings {
    //user: User,
    email: RwSignal<String>,
    password: RwSignal<String>,
    password_conf: RwSignal<String>,
    logged_in: RwSignal<bool>,

    new_log: RwSignal<Log>,
    editing_log: RwSignal<Option<usize>>,
    logs: RwSignal<Vec<Log>>,
    checked_logs: RwSignal<Vec<bool>>,
}

impl AppSettings {
    fn new() -> Self {
        Self {
            email: RwSignal::new(String::new()),
            password: RwSignal::new(String::new()),
            password_conf: RwSignal::new(String::new()),
            logged_in: RwSignal::new(true),

            new_log: RwSignal::new(Log::new()),
            editing_log: RwSignal::new(None),
            logs: RwSignal::new(Vec::new()),
            checked_logs: RwSignal::new(Vec::new()),
        }
    }

    fn get_logs(&self) {
        todo!()
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

    fn submit_log(&self) {
        (self.new_log)(Log::new());
        //todo!()
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

    fn delete_checked(&self) {
        todo!()
    }

    fn logout(&self) {
        (self.logged_in)(false);
    }

    fn login(&self) {
        (self.logged_in)(true);
    }
}

#[component]
pub fn App() -> impl IntoView {
    let app = AppSettings::new();
    let logged_in = app.logged_in;
    let app = RwSignal::new(Arc::new(app));
    provide_context(app);

view! {

<div id="app">

<header id="header">
  <h1 id="title">LogMe</h1>
  <Show when=move || logged_in()>
      <div id="logout-button-div">
        <button on:click=move |_| app.update(|app| app.logout())>Logout</button>
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

    view! {

<main id="main" class="flex-column" v-else>
    <NewEditLog />

    <hr style="width:90%" />

  /*
  <button @click="showingSortFilters = !showingSortFilters">
    <span v-if="showingSortFilters">Hide</span><span v-else>Show</span> Sort/Filters
  </button>

  <form
    id="sort-filters-form" class="flex-column"
    @submit.prevent="getLogs" v-if="showingSortFilters">
    <div id="sort-div">
      <div>
        <label for="sort">Sort By:</label>
        <select name="sort" v-model="newSortFilters.sort">
          <option value="timestamp" selected>Time</option>
          <option value="id">Added</option>
        </select>
      </div>

      <div>
        <label for="desc">Desc:</label>
        <input type="checkbox" name="desc" v-model="newSortFilters.desc" >
      </div>
    </div>
    <hr style="width:90%" >

    <div id="filter-div">
      <div>
        <label for="start">Start:</label>
        <DTInput v-model="newSortFilters" valname="start"></DTInput>
      </div>
      <div>
        <label for="end">End:</label>
        <DTInput v-model="newSortFilters" valname="end"></DTInput>
      </div>

      <div>
        <label for="limit">Limit:</label>
        <input
        type="number" name="limit"
        min="1" step="1" size="6"
        v-model="newSortFilters.limit" >
      </div>
      <div>
        <label for="offset">Offset:</label>
        <input
        type="number" name="offset"
        min="0" step="1" size="8"
        v-model="newSortFilters.offset" >
      </div>
    </div>
    <hr style="width:90%" >

    <div id="reset-cancel-buttons-div">
      <button @click="cancelSortFilters()">Cancel</button>
      <button type="submit">Apply</button>
      <button onclick="resetSortFilters()">Reset</button>
    </div>
  </form>
  */

    <div>
    <button
        on:click=move |_| app.update(|app| app.get_logs())
        style="margin:2px">"Refresh"</button>
    </div>

    <div id="check-buttons">
        <button on:click=move |_| app.update(|app| app.uncheck_all())>"Uncheck All"</button>
        <button on:click=move |_| app.update(|app| app.check_all())>"Check All"</button>
        <button on:click=move |_| app.update(|app| app.delete_checked())>"Delete Checked"</button>
    </div>

    <Logs />
</main>

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
    style="width:100%">
    <For
        each=move || logs.with(|logs| (0..logs.len()))
        key=move |&i| logs.with(|logs| logs[i].id)
        let:index
    >
        <tr>
            <td style="max-width:70vw">
                <div style="text-align:center;font-family:monospace">
                    <u>{
                        ts_to_datetime_str(logs.with(|logs| {
                            logs[index].timestamp.with(|dt| dt.timestamp())
                        }))
                    }</u>
                </div>
                <div>{logs.with(|logs| logs[index].msg.clone())}</div>
            </td>
            <td style="text-align:center;padding:0;width:7ch">
                <button
                    on:click=move |_| app.update(|app| app.edit_log(index))
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
                    prop:checked=move || checked_logs.with(|checked| checked[index])
                    on:input=move |_| {
                        checked_logs.update(|checked| checked[index] = !checked[index])
                    }
                    />
            </td>
        </tr>
    </For>
    /*
    <For
        each=move || logs.with(|logs| logs.as_slice().iter().enumerate())
        key=|(_, log)| log.id
        let:child
    >
        <tr>
            <td style="max-width:70vw">
                <div style="text-align:center;font-family:monospace">
                  <u>{ts_to_datetime_str(child.1.timestamp.with(|dt| dt.timestamp()))}</u>
                </div>
                <div>{child.1.msg}</div>
            </td>
            <td style="text-align:center;padding:0;width:7ch">
                <button
                    on:click=move |_| app.update(|app| app.edit_log(child.0))
                    style="margin:0"
                >
                <span>
                {move || if app.with(|app| editing_log() == Some(child.0)) {
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
                    prop:checked=move || checked_logs.with(|checked| checked[child.0])
                    on:input=move |_| {
                        checked_logs.update(|checked| checked[child.0] = !checked[child.0])
                    }
                    />
            </td>
        </tr>
    </For>
    */
</table>
    }
}

#[component]
fn NewEditLog() -> impl IntoView {
    let app = use_context::<RwSignal<ArcApp>>().expect("missing app context");
    let editing_log = app.with(|app| app.editing_log);
    let new_log = app.with(|app| app.new_log);
    let new_log_timestamp = RwSignal::new(new_log.with(|log| log.timestamp));

    Effect::new(move || {
        new_log.with(|log| {
            if log.timestamp != new_log_timestamp() {
                new_log_timestamp(log.timestamp);
            }
        })
    });

    view! {
<form
    id="new-log-form"
    class="flex-column"
    on:submit=move |event| {
        app.update(|app| app.submit_log());
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
        <input
            type="text" name="msg"
            style="width:95vw" placeholder="Message" autocomplete="off"
            prop:value=move || new_log.with(|log| log.msg.clone())
            on:input:target=move |event| new_log.update(|log| log.msg = event.target().value())
        />
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
            app.update(|app| app.login());
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

fn ts_to_datetime_str(ts: i64) -> String {
    let Some(dt) = chrono::DateTime::from_timestamp(ts, 0) else {
        return "INVALID DATE".into();
    };
    format!("{}", dt.format("%Y/%m/%d %H:%M:%S"))
}
