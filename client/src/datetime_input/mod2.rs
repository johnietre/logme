use chrono::{Datelike as _, Timelike as _};
use crate::console::log;
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
    has_now: bool,
) -> impl IntoView {
    log!("{has_now}");
    let holds_focus = RwSignal::<&'static str>::new("");
    let showing_popup = RwSignal::new(false);
    let dt = datetime;

    view! {
        <>
            <DateTimeInputShort dt=dt has_now=has_now holds_focus=holds_focus showing_popup=showing_popup />
            <DateTimeInputPopup dt=dt holds_focus=holds_focus showing_popup=showing_popup />
        </>
    }
}

fn on_number_input(el: web_sys::HtmlInputElement, dt: &mut DateTime, field: &str) -> bool {
    let val = el.value();

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
            if num > 2100 || !dt.set_year(num as i32) {
                Err(val.as_str())
            } else {
                Ok(val.as_str())
            }
        }
        "month" => {
            if num < 1 || num > 12 || !dt.set_month(num) {
                Err(val.as_str())
            } else {
                Ok(val.as_str())
            }
        }
        "day" => {
            if num < 1 || num > days_in_month_date(dt.date()) || !dt.set_day(num) {
                Err(val.as_str())
            } else {
                Ok(val.as_str())
            }
        }
        "hour" => {
            if num > 23 || !dt.set_hour(num) {
                Err(val.as_str())
            } else {
                Ok(val.as_str())
            }
        }
        "minute" => {
            if num > 59 || !dt.set_minute(num) {
                Err(val.as_str())
            } else {
                Ok(val.as_str())
            }
        }
        "second" => {
            if num > 59 || !dt.set_second(num) {
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
                    let _ = el.blur();
                    let _ = sib.focus();
                }
                break;
            }
            sib_opt = sib.next_element_sibling();
        }
    }
    true
}

#[component]
fn DateTimeInputShort(
    dt: RwSignal<DateTime>,
    holds_focus: RwSignal<&'static str>,
    showing_popup: RwSignal<bool>,
    has_now: bool,
) -> impl IntoView {
    log!("SHORT: {has_now}");
    view! {
        <div class="dtinput">
            <input
                type="number"
                class="dtinput-field dtinput-no-number-arrows"
                placeholder="yyyy"
                size="4" min="2000" max="2100"
                on:focus=move |_| holds_focus("year")
                on:blur=move |_| holds_focus("")
                on:keydown=move |event| {
                    if !key_is_valid(event.key().as_str()) {
                        event.prevent_default();
                    }
                }
                prop:value=move || {
                    dt.with(|dt| format_input_value(dt, "year", holds_focus()))
                }
                on:input:target=move |event| dt.update(|dt| {
                    if !on_number_input(event.target(), dt, "year") {
                        log!("prevent");
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
                on:focus=move |_| holds_focus("month")
                on:blur=move |_| holds_focus("")
                on:keydown=move |event| {
                    if !key_is_valid(event.key().as_str()) {
                        event.prevent_default();
                    }
                }
                prop:value=move || {
                    dt.with(|dt| format_input_value(dt, "month", holds_focus()))
                }
                on:input:target=move |event| dt.update(|dt| {
                    if !on_number_input(event.target(), dt, "month") {
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
                on:focus=move |_| holds_focus("day")
                on:blur=move |_| holds_focus("")
                on:keydown=move |event| {
                    if !key_is_valid(event.key().as_str()) {
                        event.prevent_default();
                    }
                }
                prop:value=move || {
                    dt.with(|dt| format_input_value(dt, "day", holds_focus()))
                }
                on:input:target=move |event| dt.update(|dt| {
                    if !on_number_input(event.target(), dt, "day") {
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
                on:focus=move |_| holds_focus("hour")
                on:blur=move |_| holds_focus("")
                on:keydown=move |event| {
                    if !key_is_valid(event.key().as_str()) {
                        event.prevent_default();
                    }
                }
                prop:value=move || {
                    dt.with(|dt| format_input_value(dt, "hour", holds_focus()))
                }
                on:input:target=move |event| dt.update(|dt| {
                    if !on_number_input(event.target(), dt, "hour") {
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
                on:focus=move |_| holds_focus("minute")
                on:blur=move |_| holds_focus("")
                on:keydown=move |event| {
                    if !key_is_valid(event.key().as_str()) {
                        event.prevent_default();
                    }
                }
                prop:value=move || {
                    dt.with(|dt| format_input_value(dt, "minute", holds_focus()))
                }
                on:input:target=move |event| dt.update(|dt| {
                    if !on_number_input(event.target(), dt, "minute") {
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
    holds_focus: RwSignal<&'static str>,
    showing_popup: RwSignal<bool>,
) -> impl IntoView {
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
                        on:focus=move |_| holds_focus("hour-popup")
                        on:blur=move |_| holds_focus("")
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
                        on:focus=move |_| holds_focus("minute-popup")
                        on:blur=move |_| holds_focus("")
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
                        on:focus=move |_| holds_focus("second-popup")
                        on:blur=move |_| holds_focus("")
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
                                "second-popup"
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
                        on:focus=move |_| holds_focus("day-popup")
                        on:blur=move |_| holds_focus("")
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
                            ) {
                                event.prevent_default();
                            }
                        })
                    />
                    <span class="dtinput-separator">", "</span>
                    <select
                        class="dtinput-month-input"
                        on:change:target=move |ev| {
                            let Ok(month0) = ev.target().value().parse() else {
                                return;
                            };
                            dt.update(|dt| {
                                if dt.set_month0(month0) {
                                    dt.generate_days();
                                }
                            })
                        }
                        prop:value=move || dt.with(|dt| dt.month0().to_string())
                    >
                        <option value="0">"Jan"</option>
                        <option value="1">"Feb"</option>
                        <option value="2">"Mar"</option>
                        <option value="3">"Apr"</option>
                        <option value="4">"May"</option>
                        <option value="5">"Jun"</option>
                        <option value="6">"Jul"</option>
                        <option value="7">"Aug"</option>
                        <option value="8">"Sep"</option>
                        <option value="9">"Oct"</option>
                        <option value="10">"Nov"</option>
                        <option value="11">"Dec"</option>
                    </select>
                    <span class="dtinput-separator">" "</span>
                    <input
                        type="number"
                        class="dtinput-field"
                        step="1" min="2000" max="2100"
                        placeholder="yyyy"
                        on:focus=move |_| holds_focus("year-popup")
                        on:blur=move |_| holds_focus("")
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
                    type="button" style="margin:0px 2px"
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
        {move || dt.with(|dt| dt.days().chunks(7).map(|c| c.to_vec()).map(|week| view! {
            <tr>{week.into_iter().map(|day| view! {
                <td>
                    <div>
                        {move || day.in_month().then(|| format_num(day.date() as _))}
                    </div>
                </td>
            }).collect_view()}</tr>
        }).collect_view())}
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
fn format_input_value(dt: &DateTime, what: &str, focused: &str) -> Cow<'static, str> {
    // NOTE: the year check is to keep in line with other year checks.
    let val = match what.split_once('-').map(|(f, _)| f).unwrap_or(what) {
        "year" => dt.year(),
        "month" => dt.month() as i32,
        "day" => dt.day() as i32,
        "hour" => dt.hour() as i32,
        "minute" => dt.minute() as i32,
        "second" => dt.second() as i32,
        _ => unreachable!("format_input_value: got unexpected what: {what}"),
    };
    if what == focused {
        if what.starts_with("year") {
            return static_year(val as _);
        }
        return static_num(val as _);
    }
    if what.starts_with("year") {
        format_year_4_zeros(val as _)
    } else {
        format_num_2_zeros(val as _)
    }
}

fn key_is_valid(key: &str) -> bool {
    key.len() == 1 && key.chars().next().unwrap().is_ascii_digit()
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
    date: DTL,
    days: (DTL, Vec<Day>),
}

#[allow(dead_code)]
impl DateTime {
    const FIRST_AD: i64 = -2208988800;

    pub fn from_timestamp(ts: i64) -> Option<Self> {
        chrono::DateTime::from_timestamp(ts, 0)
            .map(|date| date.with_timezone(&chrono::Local))
            .map(|date| Self {
                date,
                days: (date, Vec::new()),
            })
    }

    pub fn empty() -> Self {
        /*
        let date = chrono::DateTime::from_timestamp(0, 0)
            .expect("0 timestamp should be valid")
            .with_timezone(&chrono::Local);
        */
        //let date = chrono::DateTime::MIN.with_timezone(&chrono::Local);
        let date = chrono::DateTime::from_timestamp(Self::FIRST_AD, 0)
            .expect("0 timestamp should be valid")
            .with_timezone(&chrono::Local);
        Self {
            date,
            days: (date, Vec::new()),
        }
    }

    pub fn date(&self) -> DTL {
        self.date
    }

    pub fn timestamp(&self) -> i64 {
        self.date.timestamp()
    }

    pub fn year(&self) -> i32 {
        self.date.year()
    }

    pub fn month(&self) -> u32 {
        self.date.month()
    }

    pub fn month0(&self) -> u32 {
        self.date.month0()
    }

    pub fn weekday(&self) -> chrono::Weekday {
        self.date.weekday()
    }

    pub fn day(&self) -> u32 {
        self.date.day()
    }

    pub fn hour(&self) -> u32 {
        self.date.hour()
    }

    pub fn minute(&self) -> u32 {
        self.date.minute()
    }

    pub fn second(&self) -> u32 {
        self.date.second()
    }

    pub fn set_year(&mut self, year: i32) -> bool {
        let date = self.date;
        let (month,  year) = (date.month0(), year);
        let day = date.day().min(days_in_month(month,  year));
        if let Some(dt) = date.with_day(day).expect("day should be valid").with_year(year) {
            self.date = dt;
            true
        } else {
            false
        }
    }

    pub fn set_month(&mut self, month: u32) -> bool {
        if month == 0 {
            return false;
        }
        self.set_month0(month - 1)
    }

    pub fn set_month0(&mut self, month0: u32) -> bool {
        let date = self.date;
        let year = date.year();
        let day = date.day().min(days_in_month(month0,  year));
        if let Some(dt) = date.with_day(day).expect("day should be valid").with_month0(month0) {
            self.date = dt;
            true
        } else {
            false
        }
    }

    pub fn set_day(&mut self, day: u32) -> bool {
        if let Some(dt) = self.date.with_day(day) {
            self.date = dt;
            true
        } else {
            false
        }
    }

    pub fn set_day0(&mut self, day0: u32) -> bool {
        if let Some(dt) = self.date.with_day0(day0) {
            self.date = dt;
            true
        } else {
            false
        }
    }

    pub fn set_hour(&mut self, hour: u32) -> bool {
        if let Some(dt) = self.date.with_hour(hour) {
            self.date = dt;
            true
        } else {
            false
        }
    }

    pub fn set_minute(&mut self, minute: u32) -> bool {
        if let Some(dt) = self.date.with_minute(minute) {
            self.date = dt;
            true
        } else {
            false
        }
    }

    pub fn set_second(&mut self, second: u32) -> bool {
        if let Some(dt) = self.date.with_second(second) {
            self.date = dt;
            true
        } else {
            false
        }
    }

    /// Sets the DateTime to now and generates new days.
    pub fn set_now(&mut self) {
        self.date = chrono::Local::now();
        self.generate_days();
    }

    /// Increments the year and generates new days.
    pub fn incr_year(&mut self) -> bool {
        if !self.set_year(self.year() + 1) {
            return false;
        }
        self.generate_days();
        true
    }

    /// Decrements the year and generates new days.
    pub fn decr_year(&mut self) -> bool {
        if !self.set_year(self.year() - 1) {
            return false;
        }
        self.generate_days();
        true
    }

    /// Increments the month and generates new days.
    pub fn incr_month(&mut self) -> bool {
        let mo = match self.month0() {
            11 => 0,
            mo => mo + 1,
        };
        if !self.set_month(mo) {
            return false;
        }
        self.generate_days();
        true
    }

    /// Decrements the month and generates new days.
    pub fn decr_month(&mut self) -> bool {
        let mo = match self.month0() {
            0 => 11,
            mo => mo - 1,
        };
        if !self.set_month0(mo) {
            return false;
        }
        self.generate_days();
        true
    }

    pub fn generate_days(&mut self) {
        if self.date.year() == self.days.0.year() &&
            self.date.month() == self.days.0.month() &&
            self.date.offset().local_minus_utc() == self.days.0.offset().local_minus_utc() {
            return;
        }

        let date = chrono::Local::now()
            .with_day(1)
            .unwrap()
            .with_month(self.month())
            .unwrap()
            .with_year(self.year())
            .unwrap();
        self.days.0 = self.date;
        let days = &mut self.days.1;
        days.clear();

        let prev_date = date - Duration::from_secs(24 * 60 * 60);
        let (mut i, mut index, day) = (0, 0, date.weekday() as u32);
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
        format_year_4_zeros(self.year())
    }

    pub fn month_str(&self) -> Cow<'static, str> {
        format_num_2_zeros(self.month())
    }

    pub fn day_str(&self) -> Cow<'static, str> {
        format_num_2_zeros(self.day())
    }

    pub fn hour_str(&self) -> Cow<'static, str> {
        format_num_2_zeros(self.hour())
    }

    pub fn minute_str(&self) -> Cow<'static, str> {
        format_num_2_zeros(self.minute())
    }

    pub fn second_str(&self) -> Cow<'static, str> {
        format_num_2_zeros(self.second())
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
                    /*
                    {dt.with(|dt| {
                        // TODO
                        (0..dt.days().len())
                            .map(|week| {
                                view! {
                                    <tr>
                                    {(&dt.days()[week*7..week*7+7])
                                        .into_iter()
                                        .map(|day| {
                                            view! {
                                                <td>
                                                {if day.in_month() {
                                                    view! {
                                                        <div
                                                            class="dtinput-popup-calendar-day"
                                                            style:background-color=move || {
                                                                // TODO
                                                            }
                                                            on:click=move |_| {
                                                                dt.update(|dt| {
                                                                    dt.set_day(day as _);
                                                                })
                                                            }
                                                        >
                                                        </div>
                                                    }
                                                } else {
                                                    view! { <div></div> }
                                                }}
                                                </td>
                                            }
                                        })
                                        .collect_view()}
                                    </tr>
                                }
                            })
                        }}

                    <For
                        each=move || dt.with(|dt| dt.days().to_vec())
                        let:child
                    >
                        <tr>
                            <For
                                each=move || 
                            >
                            </For>
                        </tr>
                    </For>

                    <tr v-for="(week, wi) in (dtInput.days.length / 7)" :key="wi">
                        <td
                        v-for="(day, di) of dtInput.days.slice((week - 1) * 7, week * 7)"
                        :key="di"
                        on:click=""
                        >
                        <div v-if="!day.inMonth"></div>
                        <div
                        v-else
                        on:click="dtInput.day=day.date"
                        :style="{
                        'color': (day.inMonth) ? 'black' : 'gray',
                        'backgroundColor': (day.date == dtInput.day) ? 'aqua' : 'transparent'
                        }"
                        >{day.date}</div>
                        </td>
                    </tr>
                    */
