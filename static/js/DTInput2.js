export function daysInMonth(date) {
  const mo = date.getMonth();
  if ((mo % 2 == 0) == mo <= 6) {
    return 31;
  } else if (mo != 1) {
    return 30;
  }
  const yr = date.getYear();
  if ((yr % 4 == 0 && yr % 100 != 0) || yr % 400 == 0) {
    return 29;
  }
  return 28;
}

export function daysInMonthYr(month, year, zeroBased) {
  if (zeroBased) {
    month++;
  }
  switch (month) {
    case 1:
    case 3:
    case 5:
    case 7:
    case 8:
    case 10:
    case 12:
      return 31;
    case 4:
    case 6:
    case 9:
    case 11:
      return 30;
    case 2:
      return (year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)) ? 29 : 28;
    default:
        throw new Error(`invalid month: ${month}`);
  }
}

class Day {
  constructor(date, inMonth) {
    this.date = date ?? 0, this.inMonth = inMonth ?? false;
  }
};

export class DateTimeInput {
  constructor() {
    /*
    const now = new Date();
    this.month = now.getMonth();
    this.year = now.getFullYear();
    this.day = now.getDate();
    this.hour = now.getHours();
    this.minute = now.getMinutes();
    this.second = now.getSeconds();
    this.days = [];
    this.generateDays();
    */
    this.makeNow();
  }
  generateDays() {
    const days = [];
    const date = new Date();
    date.setFullYear(this.year, this.month, 1);
    const prevDate = new Date(date - 24 * 60 * 60 * 1000);
    let index = 0;
    for (var i = 0, day = date.getDay(), d = daysInMonth(prevDate) - day; i < day; i++) {
      d++;
      index++;
      days.push(new Day(d, false));
    }
    for (var i = 0, dim = daysInMonth(date); i < dim; i++) {
      index++;
      days.push(new Day(i + 1, true));
    }
    for (var i = 1; index % 7 != 0; i++) {
      index++;
      days.push(new Day(i, false));
    }
    this.days = days;
  }
  makeNow() {
    const now = new Date();
    this.month = now.getMonth();
    this.year = now.getFullYear();
    this.day = now.getDate();
    this.hour = now.getHours();
    this.minute = now.getMinutes();
    this.second = now.getSeconds();
    this.generateDays();
  }
  // Used to adjust the day and generate.
  adjustDay() {
    const zeroBased = true;
    this.day = Math.min(this.day, daysInMonthYr(this.month, this.year, zeroBased));
    this.generateDays();
  }
  incrYear() {
    const zeroBased = true;
    this.year++;
    this.day = Math.min(this.day, daysInMonthYr(this.month, this.year, zeroBased));
    this.generateDays();
  }
  decrYear() {
    const zeroBased = true;
    this.year--;
    this.day = Math.min(this.day, daysInMonthYr(this.month, this.year, zeroBased));
    this.generateDays();
  }
  incrMonth() {
    const zeroBased = true;
    if (zeroBased) {
      if (this.month == 11) {
        this.month = 0;
        this.year++;
      } else {
        this.month++;
      }
    } else {
      if (this.month == 12) {
        this.month = 1;
        this.year++;
      } else {
        this.month++;
      }
    }
    this.day = Math.min(this.day, daysInMonthYr(this.month, this.year, zeroBased));
    this.generateDays();
  }
  decrMonth() {
    const zeroBased = true;
    if (zeroBased) {
      if (this.month == 0) {
        this.month = 11;
        this.year--;
      } else {
        this.month--;
      }
    } else {
      if (this.month == 1) {
        this.month = 12;
        this.year--;
      } else {
        this.month--;
      }
    }
    this.day = Math.min(this.day, daysInMonthYr(this.month, this.year, zeroBased));
    this.generateDays();
  }
  toDate() {
    const d = new Date();
    d.setDate(this.day);
    d.setMonth(this.month);
    d.setFullYear(this.year);
    d.setHours(this.hour);
    d.setMinutes(this.minute);
    d.setSeconds(this.second);
    return d;
  }
  toTimestamp() {
    return Math.round(this.toDate().getTime() / 1000);
  }
  toTimestampMillis() {
    return this.toDate().getTime();
  }
};

// TODO: passing number as prop doesn't work
export const DTInput = {
  props: ["modelValue"],
  setup(props) {
    return {
      modelValue: props.modelValue,
    };
  },
  data() {
    let dtInput;
    if (typeof this.modelValue == "number") {
      dtInput = new DateTimeInput();
    } else {
      dtInput = this.modelValue;
    }
    return {
      dtInput: dtInput,
      showingPopup: false,
    };
  },
  watch: {
    dtInput: {
      handler() {
        if (typeof this.modelValue === "number") {
          this.modelValue = this.dtInput.toTimestamp();
        }
      },
      deep: true
    }
  },
  mounted() {
    for (const el of document.querySelectorAll("input.dtinput-field")) {
      if (el.value != "" && !isNaN(parseInt(el.value))) {
        el.value = el.value.padStart(el.size, '0');
      }
    }
  },
  methods: {
    incrYear() {
      this.dtInput.incrYear();
    },
    decrYear() {
      this.dtInput.decrYear();
    },
    incrMonth() {
      this.dtInput.incrMonth();
    },
    decrMonth() {
      this.dtInput.decrMonth();
    },
    adjustDateTimeInput() {
      this.dtInput.adjustDay();
    },
    onNumberInput(event, field) {
      const num = parseInt(event.target.value);
      if (isNaN(num)) {
        event.preventDefault();
        return;
      }
      switch (field) {
        case "year":
          if (num < 2000 || num > 2100) {
            event.preventDefault();
            return;
          }
          this.dtInput.year = num;
          break;
        case "month":
          if (num < 1 || num > 12) {
            event.preventDefault();
            return;
          }
          this.dtInput.month = num - 1;
          break;
        case "day":
          if (num < 0 || num > daysInMonth(this.dtInput.toDate())) {
            event.preventDefault();
            return;
          }
          this.dtInput.day = num;
          break;
        case "hour":
          if (num < 0 || num > 23) {
            event.preventDefault();
            return;
          }
          this.dtInput.hour = num;
          break;
        case "minute":
          if (num < 0 || num > 59) {
            event.preventDefault();
            return;
          }
          this.dtInput.minute = num;
          break;
        case "second":
          if (num < 0 || num > 59) {
            event.preventDefault();
            return;
          }
          this.dtInput.second = num;
          break;
      }
      const dim = daysInMonth(this.dtInput.toDate());
      if (this.dtInput.day > dim) {
        this.dtInput.day = dim;
      }
      event.target.value = event.target.value.padStart(event.target.size, "0");
      console.log("Value:", event.target.value);
    },
    daysInMonth(date) {
      return daysInMonth(date);
    }
  },
  // TODO: initial leading zeros in number inputs
  template: `
<div class="dtinput">
  <input
    type="number"
    class="dtinput-field dtinput-no-number-arrows"
    placeholder="yyyy"
    size=4 min="2000" max="2100"
    :value="dtInput.year"
    @input="event => onNumberInput(event, 'year')"
    >
  <span class="dtinput-separator">/</span>
  <input
    type="number"
    class="dtinput-field dtinput-no-number-arrows"
    placeholder="mm"
    size=2 min="1" max="12"
    pattern="[0-9]{2}"
    :value="dtInput.month + 1"
    @input="event => onNumberInput(event, 'month')"
    >
  <span class="dtinput-separator">/</span>
  <input
    type="text"
    class="dtinput-field dtinput-no-number-arrows"
    placeholder="dd"
    size=2 min="0" max="31"
    pattern="[0-9]{2}"
    @focus="console.log('focus')"
    @blur="console.log('blur')"
    :value="dtInput.day"
    @input="event => onNumberInput(event, 'day')"
    >
  <span class="dtinput-separator">,&nbsp;</span>
  <input
    type="number"
    class="dtinput-field dtinput-no-number-arrows"
    placeholder="HH"
    size=2 min="0" max="23"
    pattern="[0-9]{2}"
    :value="dtInput.hour"
    @input="event => onNumberInput(event, 'hour')"
    >
  <span class="dtinput-separator">:</span>
  <input
    type="number"
    class="dtinput-field dtinput-no-number-arrows"
    placeholder="MM"
    size=2 min="0" max="60"
    pattern="[0-9]{2}"
    :value="dtInput.minute"
    @input="event => onNumberInput(event, 'minute')"
    >
  <button type="button" class="dtinput-show-popup-button" @click="showingPopup=!showingPopup">
  +
  <!--
    <svg viewBox="0 0 100 100" class="dtinput-calendar-icon">
      <polygon
        points="10,10 10,90 90,90 90,10 70,10 70,0 70,10 30,10 30,0 30,10 10,10"
        fill="none" stroke="black" stroke-width="5"
      />
      <circle cx="18" cy="23" r="5" />
      <circle cx="34" cy="23" r="5" />
      <circle cx="50" cy="23" r="5" />
      <circle cx="66" cy="23" r="5" />
      <circle cx="82" cy="23" r="5" />

      <circle cx="18" cy="40" r="5" />
      <circle cx="34" cy="40" r="5" />
      <circle cx="50" cy="40" r="5" />
      <circle cx="66" cy="40" r="5" />
      <circle cx="82" cy="40" r="5" />

      <circle cx="18" cy="57" r="5" />
      <circle cx="34" cy="57" r="5" />
      <circle cx="50" cy="57" r="5" />
      <circle cx="66" cy="57" r="5" />
      <circle cx="82" cy="57" r="5" />

      <circle cx="18" cy="75" r="5" />
      <circle cx="34" cy="75" r="5" />
      <circle cx="50" cy="75" r="5" />
      <circle cx="66" cy="75" r="5" />
      <circle cx="82" cy="75" r="5" />
    </svg>
  -->
  </button>
</div>

<div v-if="showingPopup" class="dtinput-popup flex-column">
  <div class="dtinput-calendar-view flex-column">
    <div>
    <div class="dtinput-time-input">
      <input
        type="number" step="1" min="0" max="23"
        :value="dtInput.hour"
        @input="event => onNumberInput(event, 'hour')"
        >
      <span>:</span>
      <input
        type="number" step="1" min="0" max="59"
        :value="dtInput.minute"
        @input="event => onNumberInput(event, 'minute')"
        >
      <span>:</span>
      <input
        type="number" step="1" min="0" max="59"
        :value="dtInput.second"
        @input="event => onNumberInput(event, 'second')"
        >
    </div>
    <div class="dtinput-date-input">
      <input
        type="number"
        step="1" min="1" :max="daysInMonth(dtInput.toDate())"
        placeholder="Day"
        v-model="dtInput.day">
      <input
        type="number" step="1" min="1" :max="daysInMonth(dtInput.toDate())"
        placeholder="Day"
        :value="dtInput.day"
        @input="event => onNumberInput(event, 'day')"
        >
      <span>,&nbsp</span>
      <select
        class="dtinput-month-input"
        @change="adjustDateTimeInput()"
        v-model="dtInput.month">
        <option value="0">Jan</option>
        <option value="1">Feb</option>
        <option value="2">Mar</option>
        <option value="3">Apr</option>
        <option value="4">May</option>
        <option value="5">Jun</option>
        <option value="6">Jul</option>
        <option value="7">Aug</option>
        <option value="8">Sep</option>
        <option value="9">Oct</option>
        <option value="10">Nov</option>
        <option value="11">Dec</option>
      </select>
      <input
        class="dtinput-year-input" type="number"
        step="1" min="2000" max="2100"
        placeholder="year" @change="adjustDateTimeInput()"
        :value="dtInput.year"
        @input="event => onNumberInput(event, 'year')"
      >
    </div>
    </div>
    <div class="dtinput-date-prev-next-div">
      <button type="button" @click="decrYear()">&lt;&lt;</button>
      <button type="button" @click="decrMonth()">&lt;</button>
      <button type="button" style="margin: 0px 2px" @click="dtInput.makeNow()">Now</button>
      <button type="button" @click="incrMonth()">&gt;</button>
      <button type="button" @click="incrYear()">&gt;&gt;</button>
    </div>
    <div>
    </div>
    <table class="dtinput-calendar">
      <thead>
        <tr>
          <th><div>Sun</div></th>
          <th><div>Mon</div></th>
          <th><div>Tue</div></th>
          <th><div>Wed</div></th>
          <th><div>Thu</div></th>
          <th><div>Fri</div></th>
          <th><div>Sat</div></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(week, wi) in (dtInput.days.length / 7)" :key="wi">
          <td
            v-for="(day, di) of dtInput.days.slice((week - 1) * 7, week * 7)"
            :key="di"
            @click=""
            >
            <div v-if="!day.inMonth"></div>
            <div
              v-else
              @click="dtInput.day=day.date"
              :style="{
              'color': (day.inMonth) ? 'black' : 'gray',
              'backgroundColor': (day.date == dtInput.day) ? 'aqua' : 'transparent'
              }"
              >{{day.date}}</div>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
  <div class="dtinput-time-view">
  </div>
  <div>
    <button type="button" @click="showingPopup=false">Close</button>
  </div>
</div>
`
};
//export default { DTInput, DateTimeInput };
