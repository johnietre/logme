package main

import (
	"database/sql"
	"encoding/json"
	"flag"
	"fmt"
	logpkg "log"
	"net/http"
	"os"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"
	"time"

	_ "github.com/mattn/go-sqlite3"
)

var (
	db                                 *sql.DB
	indexPath, manifestPath, iconsPath string
	username, password                 string
)

func init() {
	_, thisFile, _, _ := runtime.Caller(0)
	thisDir := filepath.Dir(thisFile)
	dbPath := filepath.Join(thisDir, "logs.db")
	var err error
	db, err = sql.Open("sqlite3", dbPath)
	if err != nil {
		logpkg.Fatalf("error opening database: %v", err)
	}
	if _, err := db.Exec(createTableStmt); err != nil {
		logpkg.Fatalf("error creating table: %v", err)
	}

	indexPath = filepath.Join(thisDir, "index.html")
	manifestPath = filepath.Join(thisDir, "manifest.json")
	iconsPath = filepath.Join(thisDir, "icons/ios")
}

func main() {
	logpkg.SetFlags(0)
	addr := flag.String("addr", "127.0.0.1:8000", "Address to run on")
	logPath := flag.String("log-file", "", "Path to log file (empty means stderr)")
	flag.Parse()

	username = os.Getenv("LOGME_USERNAME")
	password = os.Getenv("LOGME_PASSWORD")

	if *logPath != "" {
		f, err := os.OpenFile(*logPath, os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0644)
		if err != nil {
			logpkg.Fatalf("error opening log file: %v", err)
		}
		logpkg.SetOutput(f)
	}

	http.HandleFunc("/", homeHandler)
	http.HandleFunc("/manifest.json", func(w http.ResponseWriter, r *http.Request) {
		http.ServeFile(w, r, manifestPath)
	})
	http.Handle(
		"/icons/",
		http.StripPrefix("/icons", http.FileServer(http.Dir(iconsPath))),
	)
	http.Handle("/login", authMiddleware(http.HandlerFunc(loginHandler)))
	http.Handle("/logs", authMiddleware(http.HandlerFunc(logsHandler)))
	logpkg.Printf("Running on %s", *addr)
	logpkg.Fatalf("error running server: %v", http.ListenAndServe(*addr, nil))
}

func authMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		user, pass, ok := r.BasicAuth()
		if !ok {
			http.Error(w, "Missing or bad auth", http.StatusBadRequest)
			return
		}
		if user != username || pass != password {
			http.Error(w, "Invalid auth", http.StatusUnauthorized)
			return
		}
		next.ServeHTTP(w, r)
	})
}

func homeHandler(w http.ResponseWriter, r *http.Request) {
	http.ServeFile(w, r, indexPath)
}

func loginHandler(w http.ResponseWriter, r *http.Request) {
}

func logsHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodGet {
		handleGetLogs(w, r)
	} else if r.Method == http.MethodPost {
		handleNewLog(w, r)
	} else if r.Method == http.MethodDelete {
		handleDeleteLog(w, r)
	}
}

type GetLogsRes struct {
	Logs  []Log  `json:"logs"`
	Error string `json:"error,omitempty"`
}

func handleGetLogs(w http.ResponseWriter, r *http.Request) {
	var (
		start, end, limit, offset int64 = -1, -1, -1, -1
		err                       error
		sortBy                    = "id"
		sortDesc                  bool
	)
	if val := r.FormValue("start"); val != "" {
		if start, err = strconv.ParseInt(val, 10, 64); err != nil {
			http.Error(w, fmt.Sprintf("Bad start value: %s", val), http.StatusBadRequest)
			return
		}
	}
	if val := r.FormValue("end"); val != "" {
		if end, err = strconv.ParseInt(val, 10, 64); err != nil {
			http.Error(w, fmt.Sprintf("Bad end value: %s", val), http.StatusBadRequest)
			return
		}
	}
	if val := r.FormValue("limit"); val != "" {
		if limit, err = strconv.ParseInt(val, 10, 64); err != nil || limit <= 0 {
			http.Error(w, fmt.Sprintf("Bad limit value: %s", val), http.StatusBadRequest)
			return
		}
	}
	if val := r.FormValue("offset"); val != "" {
		if offset, err = strconv.ParseInt(val, 10, 64); err != nil || offset < 0 {
			http.Error(w, fmt.Sprintf("Bad offset value: %s", val), http.StatusBadRequest)
			return
		}
	}
	if val := r.FormValue("sort"); val != "" {
		switch val {
		case "id":
		case "timestamp":
		default:
			http.Error(w, fmt.Sprintf("Bad sort category: %s", sortBy), http.StatusBadRequest)
			return
		}
		sortBy = val
	}
	if val := r.FormValue("desc"); val != "" {
		sortDesc, err = strconv.ParseBool(val)
		if err != nil {
			http.Error(w, fmt.Sprintf("Bad desc value: %s", val), http.StatusBadRequest)
			return
		}
	}

	var clauses []string
	if start != -1 {
		clauses = append(clauses, fmt.Sprintf("timestamp >= %d", start))
	}
	if end != -1 {
		clauses = append(clauses, fmt.Sprintf("timestamp < %d", end))
	}
	stmt := `SELECT * FROM logs`
	if len(clauses) != 0 {
		stmt += ` WHERE ` + strings.Join(clauses, " AND ")
	}
	stmt += ` ORDER BY ` + sortBy
	if sortDesc {
		stmt += ` DESC`
	}
	stmt += fmt.Sprintf(` LIMIT %d`, limit)
	if offset != -1 {
		stmt += fmt.Sprintf(` OFFSET %d`, offset)
	}

	rows, err := db.Query(stmt)
	if err != nil {
		errStr := fmt.Sprintf("error quering database: %v", err)
		logpkg.Printf(errStr)
		http.Error(w, "Internal Server Error: "+errStr, http.StatusInternalServerError)
		return
	}
	defer rows.Close()
	var logs []Log
	errStr := ""
	for rows.Next() {
		log := Log{}
		if err := rows.Scan(&log.Id, &log.Timestamp, &log.Msg); err != nil {
			errStr += "\n" + err.Error()
		} else {
			logs = append(logs, log)
		}
	}
	if errStr != "" {
		errStr = errStr[1:]
	}
	err = json.NewEncoder(w).Encode(GetLogsRes{Logs: logs, Error: errStr})
	if err != nil {
		logpkg.Printf("error writing json result: %v", err)
	}
}

func handleNewLog(w http.ResponseWriter, r *http.Request) {
	log := Log{}
	if err := json.NewDecoder(r.Body).Decode(&log); err != nil {
		errStr := fmt.Sprintf("error decoding body: %v", err)
		logpkg.Printf(errStr)
		http.Error(w, "Internal Server Error: "+errStr, http.StatusInternalServerError)
		return
	}
	if log.Timestamp <= 0 {
		log.Timestamp = time.Now().Unix()
	}
	res, err := db.Exec(
		`INSERT INTO logs(timestamp,msg) VALUES (?,?)`,
		log.Timestamp, log.Msg,
	)
	if err != nil {
		errStr := fmt.Sprintf("error inserting log: %v", err)
		logpkg.Print(errStr)
		http.Error(w, "Internal Server Error: "+errStr, http.StatusInternalServerError)
		return
	}
	id, err := res.LastInsertId()
	if err != nil {
		errStr := fmt.Sprintf("error getting insert ID log: %v", err)
		logpkg.Print(errStr)
		http.Error(w, "Internal Server Error: "+errStr, http.StatusInternalServerError)
		return
	}
	log.Id = uint64(id)
	if err := json.NewEncoder(w).Encode(log); err != nil {
		logpkg.Printf("error writing new json result: %v", err)
	}
}

func handleDeleteLog(w http.ResponseWriter, r *http.Request) {
	idsStr := r.FormValue("ids")
	idStrs := strings.Split(idsStr, ",")
	for _, idStr := range idStrs {
		_, err := strconv.ParseUint(idStr, 10, 64)
		if err != nil {
			http.Error(w, fmt.Sprintf("invalid id: %s", idStr), http.StatusBadRequest)
			return
		}
	}
	_, err := db.Exec(fmt.Sprintf(`DELETE FROM logs WHERE id IN (%s)`, idsStr))
	if err != nil {
		errStr := fmt.Sprintf("error deleting logs: %v", err)
		logpkg.Printf(errStr)
		http.Error(w, "Internal Server Error: "+errStr, http.StatusInternalServerError)
	}
}

type Log struct {
	Id        uint64 `json:"id,omitempty"`
	Timestamp int64  `json:"timestamp"`
	Msg       string `json:"msg"`
}

const createTableStmt = `
CREATE TABLE IF NOT EXISTS logs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp INTEGER NOT NULL,
  msg TEXT NOT NULL
);
`
