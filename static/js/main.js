import { DTInput, DateTimeInput, daysInMonth } from "/static/js/DTInput.js";
//import DTInput from "/static/js/DTInput.js";

(function() {
  // TODO: start/stop

function dtlStrToTs(str) {
  return Math.floor((new Date(str)).getTime() / 1000);
}

function defaultSortFilters() {
  return {
    sort: "timestamp",
    desc: true,
    start: "",
    end: "",
    limit: 50,
    offset: 0
  };
}

function newUser() {
  return {
    "id": 0,
    "email": ""
  };
}

const App = {
  data() {
    return {
      testModel: new DateTimeInput(),

      user: newUser(),
      password: "",
      passwordConf: "",
      loggedIn: false,
      registering: false,
      tokStr: "",

      dateTimeInput: new DateTimeInput(),
      showingDateTimePopup: false,

      newLog: {},
      logs: [],
      checkedLogs: [],

      editingLog: -1,

      showingSortFilters: false,
      newSortFilters: defaultSortFilters(),
      sortFilters: defaultSortFilters(),

      _blank: null 
    };
  },
  async mounted() {
    await this.getUser();
    if (this.loggedIn) {
      await this.getLogs();
    }
  },
  methods: {
    async login() {
      const authStr = btoa(`${this.user.email}:${this.password}`);
      const resp = await fetch(
        "/token",
        this.newFetchParams("POST", {"Authorization": `Basic ${authStr}`}),
      );
      if (!resp.ok) {
        alert(`Error logging in: ${await resp.text()}`);
        return;
      }
      if (!navigator.cookieEnabled) {
        this.tokStr = await resp.text();
      }
      this.loggedIn = true, this.registering = false;
      this.password = "";
      await this.getLogs();
    },
    async logout() {
      await fetch("/token", this.newFetchParams("DELETE"));
      this.loggedIn = false;
      const email = this.email;
      this.user = newUser();
      // TODO: delete?
      this.user.email = email;
    },
    async register() {
      if (this.password != this.passwordConf) {
        alert("Passwords must match");
        return;
      }
      const authStr = btoa(`${this.user.email}:${this.password}`);
      const body = {"email": this.user.email};
      const resp = await fetch(
        "/user",
        this.newFetchParams("POST", {"Authorization": `Basic ${authStr}`}, body),
      );
      if (!resp.ok) {
        alert(`Error registering: ${await this.textOrDefault(resp)}`);
        return;
      }
      if (!navigator.cookieEnabled) {
        this.tokStr = await resp.text();
      }
      this.registering = false;
      this.passwordConf = "";
      await this.login();
    },
    async getUser() {
      const resp = await fetch("/user", this.newFetchParams());
      if (!resp.ok) {
        if (resp.status == 401) {
          if (this.loggedIn) {
            alert(`Error getting user data: ${await resp.text()}`);
            alert(`Logged out`);
          }
        } else {
          await this.checkResp(resp, "Error getting user: ");
        }
        this.loggedIn = false;
        return;
      }
      this.user = await resp.json();
      this.loggedIn = true;
    },
    async getLogs() {
      this.showingSortFilters = false;
      this.sortFilters = Object.assign({}, this.newSortFilters);

      const parts = [
        `sort=${this.sortFilters.sort}`,
        `desc=${this.sortFilters.desc}`,
        `limit=${this.sortFilters.limit}`,
        `offset=${this.sortFilters.offset}`
      ];
      if (this.sortFilters.start != "") {
        parts.push(`start=${dtlStrToTs(this.sortFilters.start)}`);
      }
      if (this.sortFilters.end != "") {
        parts.push(`end=${dtlStrToTs(this.sortFilters.end)}`);
      }

      const query = parts.join("&");
      const resp = await fetch(`logs?${query}`, this.newFetchParams());
      if (!(await this.checkResp(resp, "Error getting logs: "))) {
        return;
      }

      const json = await resp.json();
      this.logs = json.logs ?? [];
      this.checkedLogs = Array(this.logs.length).fill(false);
      if (json.error !== undefined) {
        alert(`Partial error: ${json.error}`);
      } else {
        //alert("Success");
      }
    },
    async submitLog(ev) {
      ev.preventDefault();
      let timestamp = undefined;
      if (this.newLog.timestamp != "") {
        timestamp = Math.floor((new Date(this.newLog.timestamp)).getTime() / 1000);
      }
      const log = {"timestamp": timestamp, "msg": this.newLog.msg};
      let url, method;
      if (this.editingLog < 0) {
        url = "/logs";
        method = "POST";
      } else {
        url = `/logs?id=${this.newLog.id}`;
        method = "PUT";
      }
      this.newLog = {};
      this.editingLog = -1;
      const resp = await fetch(url, {
        "method": method,
        "headers": {"Authorization": `Basic ${this.authStr}`},
        "body": JSON.stringify(log)
      });
      if (!resp.ok) {
        alert(await resp.text());
        return;
      }
      //alert("Success");
      await this.getLogs();
    },
    async deleteChecked() {
      const ids = [];
      for (var i = 0; i < this.logs.length; i++) {
        if (this.checkedLogs[i]) {
          ids.push(this.logs[i].id);
        }
      }
      if (ids.length == 0) {
        return;
      } else if (!confirm("Are you sure?")) {
        return;
      }
      const resp = await fetch(`/logs?ids=${ids.join(",")}`, {
        "method": "DELETE",
        "headers": {"Authorization": `Basic ${this.authStr}`}
      });
      if (!resp.ok) {
        alert(await resp.text());
      } else {
        //alert("Success");
        await this.getLogs();
      }
    },
    newFetchParams(method, headers, body) {
      if (method === undefined) {
        method = "GET";
      }
      if (headers === undefined) {
        headers = {};
      }
      if (this.tokStr !== "") {
        headers = {"Authorization": `Bearer ${this.tokStr}`, ...headers};
      }
      if (typeof body === "object") {
        body = JSON.stringify(body);
      }
      return {
        "method": method,
        "headers": headers,
        "body": body,
      };
    },
    async checkResp(resp, msg) {
      if (resp.ok) {
        return true;
      }
      const text = await this.textOrDefault(resp);
      if (msg) {
        alert(msg + text);
      } else {
        alert(`Error: ${text}`);
      }
      if (resp.status == 401) {
        if (this.loggedIn) {
          alert(`Logged out`);
        }
        this.loggedIn = false;
      }
      return false;
    },
    async textOrDefault(resp) {
      const text = await resp.text();
      if (text !== "") {
        return text;
      }
      return resp.statusText;
    },
    setTimestampNow() {
      const now = new Date();
      now.setMinutes(now.getMinutes() - now.getTimezoneOffset());
      this.newLog.timestamp = now.toISOString().slice(0, 16);
    },
    resetSortFilters() {
      //this.showingSortFilters = false;
      this.newSortFilters = defaultSortFilters();
    },
    cancelSortFilters() {
      this.showingSortFilters = false;
      this.newSortFilters = Object.assign({}, this.sortFilters);
    },
    editLog(i) {
      if (this.editingLog == i) {
        this.cancelEditing();
        return;
      }
      this.editingLog = i;
      this.newLog = Object.assign({}, this.logs[i]);
      const dt = new Date(this.newLog.timestamp * 1000);
      dt.setMinutes(dt.getMinutes() - dt.getTimezoneOffset());
      this.newLog.timestamp = dt.toISOString().slice(0, 16);
    },
    cancelEditing() {
      this.editingLog = -1;
      this.newLog = {};
    },
    uncheckAll() {
      this.checkedLogs = Array(this.logs.length).fill(false);
    },
    checkAll() {
      this.checkedLogs = Array(this.logs.length).fill(true);
    },
    showHideDateTimePopup() {
      this.showingDateTimePopup = !this.showingDateTimePopup;
    },

    /* UTILITIES */

    daysInMonth(date) {
      return daysInMonth(date);
    },

    tsToDateTimeStr(timestamp) {
      const date = new Date(timestamp * 1000);
      return `${date.toLocaleDateString()} ${date.toLocaleTimeString()}`;
    }
  }
};
const app = Vue.createApp(App)
app.component("dtinput", DTInput);
app.mount("#app");

})()
