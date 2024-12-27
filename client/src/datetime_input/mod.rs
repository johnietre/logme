use chrono::{Datelike as _, Timelike as _};
use leptos::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::time::Duration;
use wasm_bindgen::JsCast;

mod format_num;
use format_num::*;

type DTL = chrono::DateTime<chrono::Local>;

#[component]
pub fn DateTimeInput(
    datetime: RwSignal<DateTime>,
    #[prop(default = false)]
    has_now: bool,
) -> impl IntoView {
    // Name, leading-zero count
    let holds_focus = RwSignal::<(&'static str, usize)>::new(("", 0));
    let showing_popup = RwSignal::new(false);
    let dt = datetime;

    view! {
        <>
            <DateTimeInputShort
                dt=dt has_now=has_now
                holds_focus=holds_focus showing_popup=showing_popup />
            <Show when=move || showing_popup()>
                <DateTimeInputPopup dt=dt holds_focus=holds_focus showing_popup=showing_popup />
            </Show>
        </>
    }
}

fn on_number_input(
    el: web_sys::HtmlInputElement,
    dt: &mut DateTime,
    field: &str,
    holds_focus: RwSignal<(&'static str, usize)>,
) -> bool {
    let val = el.value();

    let (focus, _) = holds_focus();
    if focus == field {
        holds_focus((focus, val.len()));
    }

    if val.len() == 0 {
        return match field.split_once('-').map(|(f, _)| f).unwrap_or(field) {
            "year" => dt.set_year(None),
            "month" => dt.set_month(None),
            "day" => dt.set_day(None),
            "hour" => dt.set_hour(None),
            "minute" => dt.set_minute(None),
            "second" => dt.set_second(None),
            _ => unreachable!("got unexpected field: {field}"),
        };
    }

    let size = if field.starts_with("year") {
        4
    } else {
        2
    };
    let Ok(num) = val.parse::<i32>() else {
        for (count, (i, c)) in val.char_indices().enumerate() {
            if !c.is_ascii_digit() || count == size {
                el.set_value(&val[..i]);
                return false;
            }
        }
        el.set_value(&val[..val.len().min(size)]);
        return false;
    };
    if val.len() > size {
        let l = val.char_indices().take(4).last().map(|(i, _)| i).unwrap_or(0);
        el.set_value(&val[..l]);
        return false;
    }
    let num = num as u32;
    let res = match field.split_once('-').map(|(f, _)| f).unwrap_or(field) {
        "year" => {
            if num > 2100 || !dt.set_year(Some(num as i32)) {
                Err(val.as_str())
            } else {
                Ok(val.as_str())
            }
        }
        "month" => {
            if num < 1 || num > 12 || !dt.set_month(Some(num)) {
                Err(val.as_str())
            } else {
                Ok(val.as_str())
            }
        }
        "day" => {
            if num < 1 || num > days_in_month_date(dt.date()) || !dt.set_day(Some(num)) {
                Err(val.as_str())
            } else {
                Ok(val.as_str())
            }
        }
        "hour" => {
            if num > 23 || !dt.set_hour(Some(num)) {
                Err(val.as_str())
            } else {
                Ok(val.as_str())
            }
        }
        "minute" => {
            if num > 59 || !dt.set_minute(Some(num)) {
                Err(val.as_str())
            } else {
                Ok(val.as_str())
            }
        }
        "second" => {
            if num > 59 || !dt.set_second(Some(num)) {
                Err(val.as_str())
            } else {
                Ok(val.as_str())
            }
        }
        _ => unreachable!("got unexpected field: {field}"),
    };
    if let Err(val) = res {
        el.set_value(val);
        return false;
    }
    if val.len() == size {
        let mut sib_opt = el.next_element_sibling();
        loop {
            let Some(sib) = sib_opt else {
                break;
            };
            if sib.tag_name() == "INPUT" {
                if let Ok(sib) = sib.dyn_into::<web_sys::HtmlElement>() {
                    let _ = sib.focus();
                }
                break;
            }
            sib_opt = sib.next_element_sibling();
        }
        let _ = el.blur();
    }
    true
}

#[component]
fn DateTimeInputShort(
    dt: RwSignal<DateTime>,
    holds_focus: RwSignal<(&'static str, usize)>,
    showing_popup: RwSignal<bool>,
    has_now: bool,
) -> impl IntoView {
    view! {
        <div class="dtinput">
            <input
                type="number"
                class="dtinput-field dtinput-no-number-arrows"
                placeholder="yyyy"
                size="4" min="2000" max="2100"
                on:focus:target=move |event| {
                    holds_focus(("year", event.target().value().len()))
                }
                on:blur=move |_| holds_focus(("", 0))
                on:keydown=move |event| {
                    if !key_is_valid(event.key().as_str()) {
                        event.prevent_default();
                    }
                }
                prop:value=move || {
                    dt.with(|dt| format_input_value(dt, "year", holds_focus()))
                }
                on:input:target=move |event| dt.update(|dt| {
                    if !on_number_input(event.target(), dt, "year", holds_focus) {
                        event.prevent_default();
                    }
                })
            />
            <span class="dtinput-separator">"/"</span>
            <input
                type="number"
                class="dtinput-field dtinput-no-number-arrows"
                placeholder="mm"
                size="2" min="1" max="12"
                pattern="[0-9]{2}"
                on:focus:target=move |event| {
                    holds_focus(("month", event.target().value().len()))
                }
                on:blur=move |_| holds_focus(("", 0))
                on:keydown=move |event| {
                    if !key_is_valid(event.key().as_str()) {
                        event.prevent_default();
                    }
                }
                prop:value=move || {
                    dt.with(|dt| format_input_value(dt, "month", holds_focus()))
                }
                on:input:target=move |event| dt.update(|dt| {
                    if !on_number_input(event.target(), dt, "month", holds_focus) {
                        event.prevent_default();
                    }
                })
            />
            <span class="dtinput-separator">"/"</span>
            <input
                type="number"
                class="dtinput-field dtinput-no-number-arrows"
                placeholder="dd"
                size="2" min="1"
                prop:max=move || {
                    dt.with(|dt| days_in_month_date(dt.date())).to_string()
                }
                pattern="[0-9]{2}"
                on:focus:target=move |event| {
                    holds_focus(("day", event.target().value().len()))
                }
                on:blur=move |_| holds_focus(("", 0))
                on:keydown=move |event| {
                    if !key_is_valid(event.key().as_str()) {
                        event.prevent_default();
                    }
                }
                prop:value=move || {
                    dt.with(|dt| format_input_value(dt, "day", holds_focus()))
                }
                on:input:target=move |event| dt.update(|dt| {
                    if !on_number_input(event.target(), dt, "day", holds_focus) {
                        event.prevent_default();
                    }
                })
            />
            <span class="dtinput-separator">", "</span>
            <input
                type="number"
                class="dtinput-field dtinput-no-number-arrows"
                placeholder="HH"
                size="2" min="0" max="23"
                pattern="[0-9]{2}"
                on:focus:target=move |event| {
                    holds_focus(("hour", event.target().value().len()))
                }
                on:blur=move |_| holds_focus(("", 0))
                on:keydown=move |event| {
                    if !key_is_valid(event.key().as_str()) {
                        event.prevent_default();
                    }
                }
                prop:value=move || {
                    dt.with(|dt| format_input_value(dt, "hour", holds_focus()))
                }
                on:input:target=move |event| dt.update(|dt| {
                    if !on_number_input(event.target(), dt, "hour", holds_focus) {
                        event.prevent_default();
                    }
                })
            />
            <span class="dtinput-separator">":"</span>
            <input
                type="number"
                class="dtinput-field dtinput-no-number-arrows"
                placeholder="MM"
                size="2" min="0" max="60"
                pattern="[0-9]{2}"
                on:focus:target=move |event| {
                    holds_focus(("minute", event.target().value().len()))
                }
                on:blur=move |_| holds_focus(("", 0))
                on:keydown=move |event| {
                    if !key_is_valid(event.key().as_str()) {
                        event.prevent_default();
                    }
                }
                prop:value=move || {
                    dt.with(|dt| format_input_value(dt, "minute", holds_focus()))
                }
                on:input:target=move |event| dt.update(|dt| {
                    if !on_number_input(event.target(), dt, "minute", holds_focus) {
                        event.prevent_default();
                    }
                })
            />
            <span class="dtinput-separator">"|"</span>
            <button
                type="button" class="dtinput-show-popup-button"
                on:click=move |_| showing_popup(!showing_popup())
                >"📅"
            </button>
            {move || if has_now {
                view! {
                    <>
                        <span class="dtinput-separator">"|"</span>
                        <button
                            class="dtinput-now-button"
                            type="button" on:click=move |_| dt.update(|dt| dt.set_now())
                        >"Now"</button>
                    </>
                }.into_any()
            } else {
                view! { <></> }.into_any()
            }}
        </div>
    }
}

#[component]
fn DateTimeInputPopup(
    dt: RwSignal<DateTime>,
    holds_focus: RwSignal<(&'static str, usize)>,
    showing_popup: RwSignal<bool>,
) -> impl IntoView {
    let month0_init = dt.with(|dt| dt.month0());
    view! {

<Show
    when=move || showing_popup()
    fallback=|| view! { <div hidden=true></div> }
>
    <div class="dtinput-popup flex-column">
        <div class="dtinput-calendar-view flex-column">
            <div>
                <div class="dtinput-time-input">
                    <input
                        type="number" step="1" min="0" max="23"
                        class="dtinput-field"
                        placeholder="HH"
                        on:focus:target=move |event| {
                            holds_focus(("hour-popup", event.target().value().len()))
                        }
                        on:blur=move |_| holds_focus(("", 0))
                        on:keydown=move |event| {
                            if !key_is_valid(event.key().as_str()) {
                                event.prevent_default();
                            }
                        }
                        prop:value=move || {
                            dt.with(|dt| {
                                format_input_value(dt, "hour-popup", holds_focus())
                            })
                        }
                        on:input:target=move |event| dt.update(|dt| {
                            if !on_number_input(
                                event.target(),
                                dt,
                                "hour-popup",
                                holds_focus,
                            ) {
                                event.prevent_default();
                            }
                        })
                    />
                    <span class="dtinput-separator">":"</span>
                    <input
                        type="number" step="1" min="0" max="59"
                        placeholder="MM"
                        class="dtinput-field"
                        on:focus:target=move |event| {
                            holds_focus(("minute-popup", event.target().value().len()))
                        }
                        on:blur=move |_| holds_focus(("", 0))
                        on:keydown=move |event| {
                            if !key_is_valid(event.key().as_str()) {
                                event.prevent_default();
                            }
                        }
                        prop:value=move || {
                            dt.with(|dt| {
                                format_input_value(
                                    dt,
                                    "minute-popup",
                                    holds_focus(),
                                )
                            })
                        }
                        on:input:target=move |event| dt.update(|dt| {
                            if !on_number_input(
                                event.target(),
                                dt,
                                "minute",
                                holds_focus,
                            ) {
                                event.prevent_default();
                            }
                        })
                    />
                    <span class="dtinput-separator">":"</span>
                    <input
                        type="number" step="1" min="0" max="59"
                        placeholder="SS"
                        class="dtinput-field"
                        on:focus:target=move |event| {
                            holds_focus(("second-popup", event.target().value().len()))
                        }
                        on:blur=move |_| holds_focus(("", 0))
                        on:keydown=move |event| {
                            if !key_is_valid(event.key().as_str()) {
                                event.prevent_default();
                            }
                        }
                        prop:value=move || {
                            dt.with(|dt| {
                                format_input_value(
                                    dt,
                                    "second-popup",
                                    holds_focus(),
                                )
                            })
                        }
                        on:input:target=move |event| dt.update(|dt| {
                            if !on_number_input(
                                event.target(),
                                dt,
                                "second-popup",
                                holds_focus,
                            ) {
                                event.prevent_default();
                            }
                        })
                    />
                </div>
                <div class="dtinput-date-input">
                    <input
                        type="number" step="1" min="1"
                        prop:max=move || {
                            dt.with(|dt| days_in_month_date(dt.date())).to_string()
                        }
                        class="dtinput-field"
                        placeholder="dd"
                        on:focus:target=move |event| {
                            holds_focus(("day-popup", event.target().value().len()))
                        }
                        on:blur=move |_| holds_focus(("", 0))
                        on:keydown=move |event| {
                            if !key_is_valid(event.key().as_str()) {
                                event.prevent_default();
                            }
                        }
                        prop:value=move || {
                            dt.with(|dt| {
                                format_input_value(
                                    dt,
                                    "day-popup",
                                    holds_focus(),
                                )
                            })
                        }
                        on:input:target=move |event| dt.update(|dt| {
                            if !on_number_input(
                                event.target(),
                                dt,
                                "day-popup",
                                holds_focus,
                            ) {
                                event.prevent_default();
                            }
                        })
                    />
                    <span class="dtinput-separator">", "</span>
                    <select
                        class="dtinput-month-input"
                        on:change:target=move |ev| {
                            let month0 = if ev.target().value().len() == 0 {
                                None
                            } else {
                                let Ok(month0) = ev.target().value().parse() else {
                                    return;
                                };
                                Some(month0)
                            };
                            dt.update(|dt| {
                                if dt.set_month0(month0) {
                                    dt.generate_days();
                                }
                            })
                        }
                        prop:value=move || {
                            dt.with(|dt| {
                                let s = dt.month0().map(format_num).unwrap_or(Cow::Borrowed(""));
                                s
                            })
                        }
                    >
                        <option value="">"---"</option>
                        <option value="0" prop:selected={month0_init == Some(0)}>"Jan"</option>
                        <option value="1" prop:selected={month0_init == Some(1)}>"Feb"</option>
                        <option value="2" prop:selected={month0_init == Some(2)}>"Mar"</option>
                        <option value="3" prop:selected={month0_init == Some(3)}>"Apr"</option>
                        <option value="4" prop:selected={month0_init == Some(4)}>"May"</option>
                        <option value="5" prop:selected={month0_init == Some(5)}>"Jun"</option>
                        <option value="6" prop:selected={month0_init == Some(6)}>"Jul"</option>
                        <option value="7" prop:selected={month0_init == Some(7)}>"Aug"</option>
                        <option value="8" prop:selected={month0_init == Some(8)}>"Sep"</option>
                        <option value="9" prop:selected={month0_init == Some(9)}>"Oct"</option>
                        <option value="10" prop:selected={month0_init == Some(10)}>"Nov"</option>
                        <option value="11" prop:selected={month0_init == Some(11)}>"Dec"</option>
                    </select>
                    <span class="dtinput-separator">" "</span>
                    <input
                        type="number"
                        class="dtinput-field"
                        step="1" min="2000" max="2100"
                        placeholder="yyyy"
                        on:focus:target=move |event| {
                            holds_focus(("year-popup", event.target().value().len()))
                        }
                        on:blur=move |_| holds_focus(("", 0))
                        on:keydown=move |event| {
                            if !key_is_valid(event.key().as_str()) {
                                event.prevent_default();
                            }
                        }
                        prop:value=move || {
                            dt.with(|dt| {
                                format_input_value(
                                    dt,
                                    "year-popup",
                                    holds_focus(),
                                )
                            })
                        }
                        on:input:target=move |event| dt.update(|dt| {
                            if !on_number_input(
                                event.target(),
                                dt,
                                "year-popup",
                                holds_focus,
                            ) {
                                event.prevent_default();
                            }
                        })
                    />
                </div>
            </div>
            <div class="dtinput-date-prev-next-div">
                <button
                    type="button"
                    on:click=move |_| dt.update(|dt| {
                        dt.decr_year();
                    })
                >"<<"</button>
                <button
                    type="button"
                    on:click=move |_| dt.update(|dt| {
                        dt.decr_month();
                    })
                >"<"</button>
                <button
                    type="button" style="margin:0px 2px;text-align:center;"
                    on:click=move |_| dt.update(|dt| dt.set_now())
                >"Now"</button>
                <button
                    type="button"
                    on:click=move |_| dt.update(|dt| {
                        dt.incr_month();
                    })
                >">"</button>
                <button
                    type="button"
                    on:click=move |_| dt.update(|dt| {
                        dt.incr_year();
                    })
                >">>"</button>
            </div>
            <div>
            </div>
            <DateTimeInputPopupCalendar dt=dt />
        </div>
        <div class="dtinput-time-view">
        </div>
        <div>
            <button
                type="button"
                on:click=move |_| dt.update(|dt| dt.clear())
            >"Clear"</button>
            <span class="dtinput-separator">"  "</span>
            <button
                type="button"
                on:click=move |_| showing_popup(false)
            >"Close"</button>
        </div>
    </div>
</Show>

    }
}

#[component]
fn DateTimeInputPopupCalendar(
    dt: RwSignal<DateTime>,
) -> impl IntoView {
    // For use in setting the date with the calendar from inside the tbody loop.
    let dt2 = dt;
    view! {
<table class="dtinput-calendar">
    <thead>
        <tr>
            <th><div>"Sun"</div></th>
            <th><div>"Mon"</div></th>
            <th><div>"Tue"</div></th>
            <th><div>"Wed"</div></th>
            <th><div>"Thu"</div></th>
            <th><div>"Fri"</div></th>
            <th><div>"Sat"</div></th>
        </tr>
    </thead>
    <tbody>
        // TODO: do better/make more efficient
        /*
        {move || dt.with(|dt| dt.days().chunks(7).map(|c| c.to_vec()).map(|week| view! {
            <tr>{week.into_iter().map(|day| view! {
                <td>
                    <div
                        on:click=move |_| {
                            dt2.update(|dt2| {
                                // NOTE: if clicking outside month, should this change month
                                // instead
                                if day.in_month() {
                                    dt2.set_day(Some(day.date()));
                                }
                            })
                        }
                        style:color=move || if day.in_month() { "black" } else { "gray" }
                        style:background-color=move || {
                            if Some(day.date()) == dt.day() {
                                "aqua"
                            } else {
                                "transparent"
                            }
                        }
                    >
                        {move || day.in_month().then(|| format_num(day.date() as _))}
                    </div>
                </td>
            }).collect_view()}</tr>
        }).collect_view())}
        */
        {move || {
            let dt = dt();
            dt.days().chunks(7).map(|c| c.to_vec()).map(|week| view! {
                <tr>{week.into_iter().map(|day| view! {
                    <td>
                        <div
                            on:click=move |_| {
                                dt2.update(|dt2| {
                                    // NOTE: if clicking outside month, should this change month
                                    // instead
                                    if day.in_month() {
                                        dt2.set_day(Some(day.date()));
                                    }
                                })
                            }
                            style:color=move || if day.in_month() { "black" } else { "gray" }
                            style:background-color=move || {
                                if day.in_month() && Some(day.date()) == dt2.with(|dt2| dt2.day()) {
                                    "aqua"
                                } else {
                                    "transparent"
                                }
                            }
                        >
                            {move || day.in_month().then(|| format_num(day.date() as _))}
                        </div>
                    </td>
                }).collect_view()}</tr>
            }).collect_view()
        }}

        /*
        <For
            each=move || {
                dt.with(|dt| {
                    dt.days().chunks(7).map(|c| c.to_vec()).collect::<Vec<_>>()
                })
            }
            key=|week| {
                let (mut sign, mut prod) = (1, 1);
                for i in 0..week.len() {
                    if !week[i].in_month() {
                        sign = -1;
                    }
                    prod *= week[i].date() as i32;
                }
                prod * sign
            }
            let:week
        >
            <tr>
                <For
                    each=move || week.clone()
                    key=|day| if day.in_month() { day.date() as i32 } else { -(day.date() as i32) }
                    let:day
                >
                    <td>
                        <Show
                            when=move || day.in_month()
                            fallback=|| view! { <div></div> }
                        >
                            <div>{format_num(day.date() as _)}</div>
                        </Show>
                    </td>
                </For>
            </tr>
        </For>
        */
    </tbody>
</table>
    }
}

/// If what is a month, expects normal month (not month0).
fn format_input_value(
    dt: &DateTime,
    what: &str,
    (focused, len): (&str, usize),
) -> Cow<'static, str> {
    // NOTE: the year check is to keep in line with other year checks.
    let val = match what.split_once('-').map(|(f, _)| f).unwrap_or(what) {
        "year" => dt.year(),
        "month" => dt.month().map(|val| val as i32),
        "day" => dt.day().map(|val| val as i32),
        "hour" => dt.hour().map(|val| val as i32),
        "minute" => dt.minute().map(|val| val as i32),
        "second" => dt.second().map(|val| val as i32),
        _ => unreachable!("format_input_value: got unexpected what: {what}"),
    };
    let Some(val) = val else {
        if what == focused && len != 0 {
            return match len {
                1 => Cow::Borrowed("0"),
                2 => Cow::Borrowed("00"),
                3 => Cow::Borrowed("000"),
                4 => Cow::Borrowed("0000"),
                z => Cow::Owned("0".repeat(z as _)),
            };
        }
        return Cow::Borrowed("");
    };
    if what == focused {
        if what.starts_with("year") {
            if len == 0 {
                return static_year(val as _);
            }
            let width = len.min(4);
            return Cow::Owned(format!("{val:0width$}"));
        }
        if len == 0 {
            return static_num(val as _);
        }
        let width = len.min(2);
        return Cow::Owned(format!("{val:0width$}"));
    }
    if what.starts_with("year") {
        format_year_4_zeros(val as _)
    } else {
        format_num_2_zeros(val as _)
    }
}

fn key_is_valid(key: &str) -> bool {
    key.len() != 1 || key.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(true)
}

#[inline]
pub fn days_in_month_date(dt: impl chrono::Datelike) -> u32 {
    days_in_month(dt.month0(), dt.year())
}

#[inline]
/// Expected 0-based month
pub fn days_in_month(month: u32, year: i32) -> u32 {
    if (month % 2 == 0) == (month <= 6) {
        return 31;
    } else if month != 1 {
        return 30;
    }
    if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
        return 29;
    }
    28
}

#[derive(Clone, Copy, PartialEq)]
pub struct Day {
    date: u32,
    in_month: bool,
}

impl Day {
    pub fn new(date: u32, in_month: bool) -> Self {
        Self { date, in_month }
    }

    pub fn date(self) -> u32 {
        self.date
    }

    pub fn in_month(self) -> bool {
        self.in_month
    }
}

#[derive(Clone)]
pub struct DateTime {
    year: Option<i32>,
    month: Option<u32>,
    day: Option<u32>,
    hour: Option<u32>,
    minute: Option<u32>,
    second: Option<u32>,

    days: (DTL, Vec<Day>),
}

impl Default for DateTime {
    fn default() -> Self {
        Self::empty()
    }
}

impl PartialEq for DateTime {
    fn eq(&self, other: &Self) -> bool {
        self.year == other.year &&
            self.month == other.month &&
            self.day == other.day &&
            self.hour == other.hour &&
            self.minute == other.minute &&
            self.second == other.second
    }
}

#[allow(dead_code)]
impl DateTime {
    const FIRST_AD: i64 = -2208988800;

    pub fn from_timestamp(ts: i64) -> Option<Self> {
        chrono::DateTime::from_timestamp(ts, 0)
            .map(|date| date.with_timezone(&chrono::Local))
            .map(|date| Self {
                year: Some(date.year()),
                month: Some(date.month()),
                day: Some(date.day()),
                hour: Some(date.hour()),
                minute: Some(date.minute()),
                second: Some(date.second()),
                days: (date, Vec::new()),
            })
    }

    pub fn empty() -> Self {
        Self {
            year: None,
            month: None,
            day: None,
            hour: None,
            minute: None,
            second: None,
            days: (DTL::MIN_UTC.with_timezone(&chrono::Local), Vec::new()),
        }
    }

    pub fn date(&self) -> DTL {
        self.date_opt()
            .unwrap_or_else(|| {
                chrono::DateTime::from_timestamp(0, 0)
                    .expect("zero timestamp should be valid")
                    .with_timezone(&chrono::Local)
            })
    }

    pub fn date_opt(&self) -> Option<DTL> {
        let dt = chrono::Local::now();
        let dt = dt.with_year(self.year()?)?;
        let dt = dt.with_month(self.month()?)?;
        let dt = dt.with_day(self.day()?)?;
        let dt = dt.with_hour(self.hour()?)?;
        let dt = dt.with_minute(self.minute()?)?;
        let dt = dt.with_second(self.second()?)?;
        Some(dt)
    }

    pub fn timestamp(&self) -> i64 {
        self.date().timestamp()
    }

    pub fn year(&self) -> Option<i32> {
        self.year
    }

    pub fn month(&self) -> Option<u32> {
        self.month
    }

    pub fn month0(&self) -> Option<u32> {
        self.month.map(|month| month - 1)
    }

    pub fn day(&self) -> Option<u32> {
        self.day
    }

    pub fn hour(&self) -> Option<u32> {
        self.hour
    }

    pub fn minute(&self) -> Option<u32> {
        self.minute
    }

    pub fn second(&self) -> Option<u32> {
        self.second
    }

    pub fn set_year(&mut self, year: Option<i32>) -> bool {
        let Some(year) = year else {
            self.year = None;
            return true;
        };
        self.year = Some(year);
        let Some((month, day)) = self.month.zip(self.day) else {
            return true;
        };
        self.day = Some(day.min(days_in_month(month - 1, year)));
        true
    }

    pub fn set_month(&mut self, month: Option<u32>) -> bool {
        let Some(month) = month else {
            self.month = None;
            return true;
        };
        if month > 12 || month == 0 {
            return false;
        }
        self.month = Some(month);
        let Some(day) = self.day else {
            return true;
        };
        let year = self.year.unwrap_or(2001);
        self.day = Some(day.min(days_in_month(month - 1, year)));
        true
    }

    pub fn set_month0(&mut self, month0: Option<u32>) -> bool {
        self.set_month(month0.map(|month0| month0 + 1))
    }

    pub fn set_day(&mut self, day: Option<u32>) -> bool {
        let Some(day) = day else {
            self.day = None;
            return true;
        };
        if day > 31 || day == 0 {
            return false;
        }
        let Some(month) = self.month else {
            self.day = Some(day);
            return true;
        };
        let year = self.year.unwrap_or(2001);
        if day > days_in_month(month - 1, year) {
            return false;
        }
        self.day = Some(day);
        true
    }

    pub fn set_day0(&mut self, day0: Option<u32>) -> bool {
        self.set_day(day0.map(|day0| day0 + 1))
    }

    pub fn set_hour(&mut self, hour: Option<u32>) -> bool {
        if hour > Some(23) {
            return false;
        }
        self.hour = hour;
        true
    }

    pub fn set_minute(&mut self, minute: Option<u32>) -> bool {
        if minute > Some(59) {
            return false;
        }
        self.minute = minute;
        true
    }

    pub fn set_second(&mut self, second: Option<u32>) -> bool {
        if second > Some(59) {
            return false;
        }
        self.second = second;
        true
    }

    /// Sets the DateTime to now and generates new days.
    pub fn set_now(&mut self) {
        let dt = chrono::Local::now();
        self.year = Some(dt.year());
        self.month = Some(dt.month());
        self.day = Some(dt.day());
        self.hour = Some(dt.hour());
        self.minute = Some(dt.minute());
        self.second = Some(dt.second());
        self.generate_days();
    }

    /// Increments the year and generates new days.
    pub fn incr_year(&mut self) -> bool {
        let year = self.year().unwrap_or_else(|| chrono::Local::now().year());
        if !self.set_year(Some(year + 1)) {
            return false;
        }
        self.generate_days();
        true
    }

    /// Decrements the year and generates new days.
    pub fn decr_year(&mut self) -> bool {
        let year = self.year().unwrap_or_else(|| chrono::Local::now().year());
        if !self.set_year(Some(year - 1)) {
            return false;
        }
        self.generate_days();
        true
    }

    /// Increments the month and generates new days.
    pub fn incr_month(&mut self) -> bool {
        let mo = match self.month0().unwrap_or_else(|| chrono::Local::now().month0()) {
            11 => 0,
            mo => mo + 1,
        };
        if !self.set_month0(Some(mo)) {
            return false;
        }
        self.generate_days();
        true
    }

    /// Decrements the month and generates new days.
    pub fn decr_month(&mut self) -> bool {
        let mo = match self.month0().unwrap_or_else(|| chrono::Local::now().month0()) {
            0 => 11,
            mo => mo - 1,
        };
        if !self.set_month0(Some(mo)) {
            return false;
        }
        self.generate_days();
        true
    }

    pub fn generate_days(&mut self) {
        let date = self.date();
        if date.year() == self.days.0.year() &&
            date.month() == self.days.0.month() &&
            date.offset().local_minus_utc() == self.days.0.offset().local_minus_utc() {
            return;
        }
        let Some((year, month)) = self.year.zip(self.month) else {
            return;
        };

        let Some(date) = chrono::Local::now()
            .with_day(1)
            .unwrap()
            .with_month(month)
            .unwrap()
            .with_year(year) else {
            // NOTE: do something?
            return;
        };
        self.days.0 = date;
        let days = &mut self.days.1;
        days.clear();

        let prev_date = date - Duration::from_secs(24 * 60 * 60);
        let (mut i, mut index, day) = (0, 0, (date.weekday() as u32 + 1) % 7);
        let mut d = days_in_month_date(prev_date) - day;
        while i < day {
            d += 1;
            index += 1;
            days.push(Day::new(d, false));

            i += 1;
        }
        let (mut i, dim) = (0, days_in_month_date(date));
        while i < dim {
            index += 1;
            days.push(Day::new(i + 1, true));

            i += 1;
        }
        let mut i = 1;
        while index % 7 != 0 {
            index += 1;
            days.push(Day::new(i, false));

            i += 1;
        }
    }

    pub fn days(&self) -> &[Day] {
        &self.days.1
    }

    pub fn clear(&mut self) {
        *self = Self::empty()
    }

    pub fn year_str(&self) -> Cow<'static, str> {
        self.year().map(format_year_4_zeros).unwrap_or(Cow::Borrowed(""))
    }

    pub fn month_str(&self) -> Cow<'static, str> {
        self.month().map(format_num_2_zeros).unwrap_or(Cow::Borrowed(""))
    }

    pub fn day_str(&self) -> Cow<'static, str> {
        self.day().map(format_num_2_zeros).unwrap_or(Cow::Borrowed(""))
    }

    pub fn hour_str(&self) -> Cow<'static, str> {
        self.hour().map(format_num_2_zeros).unwrap_or(Cow::Borrowed(""))
    }

    pub fn minute_str(&self) -> Cow<'static, str> {
        self.minute().map(format_num_2_zeros).unwrap_or(Cow::Borrowed(""))
    }

    pub fn second_str(&self) -> Cow<'static, str> {
        self.second().map(format_num_2_zeros).unwrap_or(Cow::Borrowed(""))
    }
}

impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.date().timestamp())
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error as _;

        let ts = i64::deserialize(deserializer)?;
        DateTime::from_timestamp(ts)
            .ok_or_else(|| D::Error::custom(format!("invalid timestamp value: {ts}")))
    }
}
