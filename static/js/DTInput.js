function isDigit(c) {
  const code = c.charCodeAt(0);
  // Less than '0', greather than '9'
  return (code >= 48 && code <= 57);
}

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

export class DateTime {
  static fromTimestamp(ts) {
    return DateTime.fromTimestampMillis(ts * 1000);
  }
  static fromTimestampMillis(mts) {
    if (mts === 0) {
      return DateTime.empty();
    }
    const dti = new DateTime();
    dti.setTimestampMillis(mts);
    return dti;
  }
  static fromDate(date) {
    const dti = new DateTime();
    dti.setDate(date);
    return dti;
  }
  static empty() {
    const dti = new DateTime();
    dti.month = -1;
    dti.year = -1;
    dti.day = -1;
    dti.hour = -1;
    dti.minute = -1;
    dti.second = -1;
    return dti;
  }

  constructor(date) {
    if (date === undefined) {
      this.makeNow();
    } else if (typeof date === number) {
      this.setTimestamp(date);
    } else {
      this.setDate(date);
    }
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
    this.setDate(now);
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
  setTimestamp(ts) {
    this.setTimestampMillis(ts * 1000);
  }
  setTimestampMillis(mts) {
    const dt = new Date(mts);
    this.setDate(dt);
  }
  setDate(date) {
    this.month = date.getMonth();
    this.year = date.getFullYear();
    this.day = date.getDate();
    this.hour = date.getHours();
    this.minute = date.getMinutes();
    this.second = date.getSeconds();
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
  clear() {
    this.month = -1;
    this.year = -1;
    this.day = -1;
    this.hour = -1;
    this.minute = -1;
    this.second = -1;
  }
};

class ModelType {
  static DTInput = "DTInput";
  static Number = "number";
  static Ref = "ref";
  static Obj = "Obj";
  static NamedObj = "NamedObj";
  static Function = "function";
  static Unknown = "unknown";

  static from(val) {
    if (typeof val === "number") {
      return this.Number;
    } else if (typeof val === "function") {
      return this.Function;
    } else if (val instanceof DateTime) {
      return this.DTInput;
    } else if (typeof val === "object") {
      if ("value" in val) {
        return this.Ref;
      }
      if ("name" in val && "object" in val) {
        if (typeof val.name === "string" && typeof val.object === "object") {
          if (val.name in val.object) {
            return this.NamedObj;
          }
        }
      }
      return this.Obj;
    }
    return this.Unknown;
  }

  static setVal(val, dti, valName, depth) {
    const mt = ModelType.from(val);
    switch (mt) {
      case ModelType.DTInput:
        Object.assign(val, dti);
        break;
      case ModelType.Number:
        break;
      case ModelType.Ref:
        var rt = ModelType.from(val.value);
        if (rt == ModelType.Ref) {
          if (depth === 100) {
            // FIXME: what to do when depth is too high.
            val.value = dti;
          } else {
            depth = depth ?? 1;
            ModelType.setVal(val.value, dti, valName, depth + 1);
          }
        } else if (rt == ModelType.Number) {
          val.value = dti.toTimestamp();
        } else {
          ModelType.setVal(val.value, dti, valName);
        }
        break;
      case ModelType.Obj:
        var rt = ModelType.from(val[valName])
        if (rt == ModelType.DTInput) {
          Object.assign(val[valName], dti);
        } else {
          val[valName] = dti.toTimestamp();
        }
        break;
      case ModelType.NamedObj:
        var rt = ModelType.from(val.object[val.name]);
        if (rt == ModelType.DTInput) {
          Object.assign(val.object[val.name], dti);
        } else {
          val.object[val.name] = dti.toTimestamp();
        }
        break;
      case ModelType.Function:
        val(dti);
        break;
      case ModelType.Unknown:
        break;
    }
  }

  static namedObjType(val, valName) {
    const mt = ModelType.from(val);
    if (mt == ModelType.Obj && valName !== "") {
      return ModelType.from(val[valName]);
    }
    if (mt != ModelType.NamedObj) {
      return ModelType.Unknown;
    }
    return ModelType.from(val.object[val.name]);
  }

  static namedObjValue(val, valName) {
    const mt = ModelType.from(val);
    if (mt == ModelType.Obj && valName !== "") {
      return val[valName];
    }
    if (mt != ModelType.NamedObj) {
      return null;
    }
    return val.object[val.name];
  }

  static mvToDTInput(modelValue, valName) {
    const mv = modelValue;
    const mt = ModelType.from(mv);
    let dtInput;
    if (mt == ModelType.Number) {
      dtInput = DateTime.fromTimestamp(mv);
    } else if (mt == ModelType.Ref && ModelType.from(mv.value) == ModelType.Number) {
      dtInput = DateTime.fromTimestamp(mv.value);
    } else if (ModelType.namedObjType(mv, valName) == ModelType.Number) {
      dtInput = DateTime.fromTimestamp(ModelType.namedObjValue(mv, valName));
    } else if (mt == ModelType.Function) {
      dtInput = DateTime.empty();
    } else if (mt == ModelType.DTInput) {
      dtInput = mv;
    } else {
      dtInput = DateTime.empty();
    }
    return dtInput;
  }
};

// TODO: passing number as prop doesn't work
export const DTInput = {
  props: ["modelValue", "hasnow", "valname"],
  setup(props) {
    return {
      modelValue: props.modelValue,
      hasNow: props.hasnow ?? false,
      valName: props.valname ?? "",
    };
  },
  data() {
    const dtInput = ModelType.mvToDTInput(this.modelValue, this.valName);
    return {
      dtInput: dtInput,
      showingPopup: false,
      holdsFocus: "",
      updated: 0,
    };
  },
  watch: {
    dtInput: {
      handler() {
        const now = new Date();
        const time = now.getTime();
        if (time - this.updated < 10) {
          return;
        }
        this.updated = time;
        const mt = ModelType.from(this.modelValue);
        if (mt != ModelType.DTInput) {
          ModelType.setVal(this.modelValue, this.dtInput, this.valName);
        }
      },
      deep: true
    },
    modelValue: {
      handler() {
        console.log("change");
        if (ModelType.from(this.modelValue) == ModelType.DTInput) {
          return;
        }
        const now = new Date();
        const time = now.getTime();
        if (time - this.updated < 10) {
          return;
        }
        this.updated = time;
        this.dtInput = ModelType.mvToDTInput(this.modelValue, this.valName);
      },
      deep: true,
    }
  },
  mounted() {
    /*
    for (const el of document.querySelectorAll("input.dtinput-field")) {
      if (el.value != "" && !isNaN(parseInt(el.value))) {
        el.value = el.value.padStart(el.size, '0');
      }
    }
    */
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
    adjustDateTime() {
      this.dtInput.adjustDay();
    },
    onNumberInput(event, field) {
      const size = event.target.size;
      const num = parseInt(event.target.value);
      if (isNaN(num)) {
        let count = 0;
        for (const c of event.target.value) {
          if (!isDigit(c)) {
            break;
          }
          count += 1;
          if (count == size) {
            break;
          }
        }
        event.target.value = event.target.value.slice(0, count);
        event.preventDefault();
        return;
      }
      if (event.target.value.length > size) {
        event.target.value = event.target.value.slice(0, size);
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
      if (event.target.value.length == event.target.size) {
        let focused = false;
        for (var el = event.target.nextElementSibling; el; el = el.nextElementSibling) {
          if (el.tagName === "INPUT") {
            focused = true;
            el.focus();
            break;
          }
        }
        if (!focused) {
          event.target.blur();
        }
      }
      event.target.value = event.target.value.padStart(event.target.size, "0");
    },
    daysInMonth(date) {
      return daysInMonth(date);
    },
    formatInputValue(what, val, size) {
      if (val < 0) {
        return "";
      }
      if (what == "month") {
        val++;
      }
      if (this.holdsFocus == what) {
        return val.toString();
      }
      size = size || 2;
      return val.toString().padStart(size, "0");
    },
    clearInput() {
      this.dtInput.clear();
    },
    validateKey(event) {
      const c = event.key;
      if (c.length == 1 && !isDigit(c)) {
        event.preventDefault();
        return;
      }
    },
  },
  // TODO: initial leading zeros in number inputs
  template: `
<div class="dtinput">
  <input
    type="number"
    class="dtinput-field dtinput-no-number-arrows"
    placeholder="yyyy"
    size=4 min="2000" max="2100"
    @focus="holdsFocus = 'year'"
    @blur="holdsFocus = ''"
    @keydown="event => validateKey(event)"
    :value="formatInputValue('year', dtInput.year, 4)"
    @input="event => onNumberInput(event, 'year')"
    >
  <span class="dtinput-separator">/</span>
  <input
    type="number"
    class="dtinput-field dtinput-no-number-arrows"
    placeholder="mm"
    size=2 min="1" max="12"
    pattern="[0-9]{2}"
    @focus="holdsFocus = 'month'"
    @blur="holdsFocus = ''"
    @keydown="event => validateKey(event)"
    :value="formatInputValue('month', dtInput.month)"
    @input="event => onNumberInput(event, 'month')"
    >
  <span class="dtinput-separator">/</span>
  <input
    type="number"
    class="dtinput-field dtinput-no-number-arrows"
    placeholder="dd"
    size=2 min="0" max="31"
    pattern="[0-9]{2}"
    @focus="holdsFocus = 'day'"
    @blur="holdsFocus = ''"
    @keydown="event => validateKey(event)"
    :value="formatInputValue('day', dtInput.day)"
    @input="event => onNumberInput(event, 'day')"
    >
  <span class="dtinput-separator">,&nbsp;</span>
  <input
    type="number"
    class="dtinput-field dtinput-no-number-arrows"
    placeholder="HH"
    size=2 min="0" max="23"
    pattern="[0-9]{2}"
    @focus="holdsFocus = 'hour'"
    @blur="holdsFocus = ''"
    @keydown="event => validateKey(event)"
    :value="formatInputValue('hour', dtInput.hour)"
    @input="event => onNumberInput(event, 'hour')"
    >
  <span class="dtinput-separator">:</span>
  <input
    type="number"
    class="dtinput-field dtinput-no-number-arrows"
    placeholder="MM"
    size=2 min="0" max="60"
    pattern="[0-9]{2}"
    @focus="holdsFocus = 'minute'"
    @blur="holdsFocus = ''"
    @keydown="event => validateKey(event)"
    :value="formatInputValue('minute', dtInput.minute)"
    @input="event => onNumberInput(event, 'minute')"
    >
  <span class="dtinput-separator">|</span>
  <button type="button" class="dtinput-show-popup-button" @click="showingPopup=!showingPopup">
  📅
  </button>
  <template v-if="hasNow">
    <span class="dtinput-separator">|</span>
    <button
      class="dtinput-now-button"
      type="button" @click="dtInput.makeNow()"
    >Now</button>
  </template>
</div>

<div v-if="showingPopup" class="dtinput-popup flex-column">
  <div class="dtinput-calendar-view flex-column">
    <div>
      <div class="dtinput-time-input">
        <input
          type="number" step="1" min="0" max="23"
          class="dtinput-field"
          placeholder="HH"
          @focus="holdsFocus = 'hour'"
          @blur="holdsFocus = ''"
          @keydown="event => validateKey(event)"
          :value="formatInputValue('hour', dtInput.hour)"
          @input="event => onNumberInput(event, 'hour')"
          >
        <span class="dtinput-separator">:</span>
        <input
          type="number" step="1" min="0" max="59"
          placeholder="MM"
          class="dtinput-field"
          @focus="holdsFocus = 'minute'"
          @blur="holdsFocus = ''"
          @keydown="event => validateKey(event)"
          :value="formatInputValue('minute', dtInput.minute)"
          @input="event => onNumberInput(event, 'minute')"
          >
        <span class="dtinput-separator">:</span>
        <input
          type="number" step="1" min="0" max="59"
          placeholder="SS"
          class="dtinput-field"
          @focus="holdsFocus = 'second'"
          @blur="holdsFocus = ''"
          @keydown="event => validateKey(event)"
          :value="formatInputValue('second', dtInput.second)"
          @input="event => onNumberInput(event, 'second')"
          >
      </div>
      <div class="dtinput-date-input">
        <input
          type="number" step="1" min="1" :max="daysInMonth(dtInput.toDate())"
          class="dtinput-field"
          placeholder="dd"
          @focus="holdsFocus = 'day'"
          @blur="holdsFocus = ''"
          @keydown="event => validateKey(event)"
          :value="formatInputValue('day', dtInput.day)"
          @input="event => onNumberInput(event, 'day')"
          >
        <span class="dtinput-separator">,&nbsp;</span>
        <select
          class="dtinput-month-input"
          @change="adjustDateTime()"
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
        <span class="dtinput-separator">&nbsp;</span>
        <input
          type="number"
          class="dtinput-field"
          step="1" min="2000" max="2100"
          placeholder="yyyy" @change="adjustDateTime()"
          @focus="holdsFocus = 'year'"
          @blur="holdsFocus = ''"
          @keydown="event => validateKey(event)"
          :value="formatInputValue('year', dtInput.year)"
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
    <button type="button" @click="clearInput()">Clear</button>
    <span class="dtinput-separator">&nbsp;&nbsp;</span>
    <button type="button" @click="showingPopup=false">Close</button>
  </div>
</div>
`
};
//export default { DTInput, DateTime };
