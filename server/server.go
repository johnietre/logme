// TODO: checks for if user is deleted
// TODO: sessions (for tokens)?
// TODO: max logs
// TODO: rate limiting?
// TODO: max DB connections open
// TODO: frequents
// TODO: cookie params by config (name, domain, path, etc.)
package server

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"io"
	logpkg "log"
	"net/http"
	"os"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"
	"time"

	"github.com/golang-jwt/jwt/v5"
	jmux "github.com/johnietre/go-jmux"
	utils "github.com/johnietre/utils/go"
	_ "github.com/mattn/go-sqlite3"
	"github.com/spf13/cobra"
	"golang.org/x/crypto/bcrypt"
)

const (
	maxLogMsgLen = 256
	maxLogs      = 1000

	tokenDur        = time.Hour * 24 * 365
	cookieDur       = tokenDur
	tokenCookieName = "logme-token"
)

var (
	usersDb                 *sql.DB
	databasesDir            string
	jwtKey                  []byte
	indexPath, staticPath   string
	manifestPath, iconsPath string
)

func init() {
	_, thisFile, _, _ := runtime.Caller(0)
	thisDir := filepath.Dir(thisFile)
	parentDir := filepath.Dir(thisDir)

	databasesDir = filepath.Join(parentDir, "databases")
	indexPath = filepath.Join(parentDir, "static", "html", "index.html")
	staticPath = filepath.Join(parentDir, "static")
	manifestPath = filepath.Join(parentDir, "assets", "manifest.json")
	iconsPath = filepath.Join(parentDir, "assets", "icons/ios")
}

func MakeCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:                   "logme",
		Run:                   run,
		DisableFlagsInUseLine: true,
	}
	flags := cmd.Flags()
	flags.String("addr", "127.0.0.1:8000", "Address to run on")
	flags.String("log-file", "", "Path to log file (empty means stderr)")
	return cmd
}

func Run() {
	if err := MakeCmd().Execute(); err != nil {
		logpkg.Fatal(err)
	}
}

func run(cmd *cobra.Command, _ []string) {
	logpkg.SetFlags(0)

	flags := cmd.Flags()

	addr, _ := flags.GetString("addr")
	logPath, _ := flags.GetString("log-file")

	if logPath != "" {
		f, err := utils.OpenAppend(logPath)
		if err != nil {
			logpkg.Fatalf("error opening log file: %v", err)
		}
		logpkg.SetOutput(f)
	}

	jwtKeyStr, err := utils.EnvFileOrVar("LOGME_JWT_KEY")
	if err != nil {
		logpkg.Print(
			"error reading file LOGME_JWT_KEY env variable, using regular value: ",
			err,
		)
	}
	jwtKey = []byte(jwtKeyStr)
	if len(jwtKey) == 0 {
		logpkg.Fatal("jwt key not set, set using LOGME_JWT_KEY or a file with its path stored in LOGME_JWT_KEY_FILE")
	}

	usersDb, err = openUsersDb()
	if err != nil {
		logpkg.Fatal("error opening users database: ", err)
	}

	srvr := &http.Server{
		Addr: addr,
	}

	r := jmux.NewRouter()
	r.Get("/", jmux.WrapF(homeHandler))
	staticFS := http.FileServer(http.Dir(staticPath))
	r.Get("/static/", jmux.WrapH(http.StripPrefix("/static", staticFS))).
		MatchAny(jmux.MethodsGet())
	r.GetFunc("/manifest.json", func(c *jmux.Context) {
		http.ServeFile(c.Writer, c.Request, manifestPath)
	})
	r.Get(
		"/icons/",
		jmux.WrapH(http.StripPrefix(
			"/icons",
			http.FileServer(http.Dir(iconsPath)),
		)),
	)
	r.PostFunc("/user", newUserHandler)
	r.PostFunc("/token", newTokenHandler)

	r.Delete("/token", authMiddleware(jmux.HandlerFunc(deleteTokenHandler)))
	r.Get("/user", authMiddleware(jmux.HandlerFunc(getUserHandler)))
	r.Get("/logs", authMiddleware(jmux.HandlerFunc(getLogsHandler)))
	r.Post("/logs", authMiddleware(jmux.HandlerFunc(newLogHandler)))
	r.Put("/logs", authMiddleware(jmux.HandlerFunc(editLogHandler)))
	r.Delete("/logs", authMiddleware(jmux.HandlerFunc(deleteLogHandler)))

	srvr.Handler = r
	logpkg.Printf("running on %s", srvr.Addr)
	logpkg.Fatalf("error running server: %v", srvr.ListenAndServe())
}

func authMiddleware(next jmux.Handler) jmux.Handler {
	return jmux.HandlerFunc(func(c *jmux.Context) {
		tokStr, ok := getTokenStr(c)
		if !ok {
			c.WriteError(http.StatusUnauthorized, "missing or invalid auth")
			return
		}
		tok, err := parseToken(tokStr)
		if err != nil {
			// TODO: error to return
			c.WriteHeader(http.StatusUnauthorized)
			return
		} else if !tok.Valid {
			logpkg.Print("invalid token")
			// TODO?
			c.WriteHeader(http.StatusUnauthorized)
			return
		}
		user, err := userFromToken(tok)
		if err != nil {
			c.WriteHeader(http.StatusInternalServerError)
			logpkg.Print(err)
			return
		}
		c.Request = c.Request.WithContext(
			context.WithValue(c.Request.Context(), userCtxKey, user),
		)
		next.ServeC(c)
	})
}

func homeHandler(w http.ResponseWriter, r *http.Request) {
	http.ServeFile(w, r, indexPath)
}

func newTokenHandler(c *jmux.Context) {
	email, password, ok := c.Request.BasicAuth()
	if !ok {
		c.WriteError(http.StatusBadRequest, "missing authorization")
		return
	}
	user, err := getUserByEmail(email)
	if err != nil {
		if errors.Is(err, errUserNotExist) {
			c.WriteError(http.StatusUnauthorized, "bad credentials")
		} else {
			logpkg.Printf("error getting user for %s: %v", email, err)
			c.WriteHeader(http.StatusInternalServerError)
		}
		return
	}
	err = bcrypt.CompareHashAndPassword(
		[]byte(user.passwordHash),
		[]byte(password),
	)
	// TODO: is this correct (no server errors possible)?
	if err != nil {
		c.WriteError(http.StatusUnauthorized, "bad credentials")
		return
	}
	tok, err := generateToken(user.Id)
	if err != nil {
		logpkg.Printf("error generating token for user %d: %v", user.Id, err)
		c.WriteHeader(http.StatusInternalServerError)
		return
	}
	http.SetCookie(c.Writer, newCookie(tok))
	c.WriteString(tok)
}

func deleteTokenHandler(c *jmux.Context) {
	cookie := newCookie("")
	cookie.Expires, cookie.MaxAge = time.Time{}, -1
	http.SetCookie(c.Writer, cookie)
}

func newCookie(value string) *http.Cookie {
	return &http.Cookie{
		Name:  tokenCookieName,
		Value: value,
		// TODO: set
		Path:   "",
		Domain: "",
		// TODO: one or the other?
		Expires:  time.Now().Add(cookieDur),
		MaxAge:   int(cookieDur.Seconds()),
		HttpOnly: true,
		SameSite: http.SameSiteStrictMode,
	}
}

func newUserHandler(c *jmux.Context) {
	_, password, ok := c.Request.BasicAuth()
	if !ok {
		c.WriteError(http.StatusBadRequest, "missing password")
		return
	}
	var user User
	if err := c.ReadBodyJSON(&user); err != nil {
		clientErr := utils.IsUnmarshalError(err) ||
			errors.Is(err, io.EOF) ||
			errors.Is(err, io.ErrUnexpectedEOF)
		if clientErr {
			c.WriteError(http.StatusBadRequest, fmt.Sprint("bad json: ", err))
		} else {
			logpkg.Print("error reading user json: ", err)
			c.WriteHeader(http.StatusInternalServerError)
		}
		return
	}
	user.passwordHash = password
	if err := user.hashPassword(); err != nil {
		c.WriteError(http.StatusBadRequest, "invalid password")
	}
	if err := user.create(); err != nil {
		if errors.Is(err, errUserExists) {
			c.WriteError(http.StatusBadRequest, "user with email already exists")
		} else {
			logpkg.Printf("error creating user: %v", err)
			c.WriteHeader(http.StatusInternalServerError)
		}
		return
	}
}

func getUserHandler(c *jmux.Context) {
	user, ok := userFromContext(c.Request.Context())
	if !ok {
		logpkg.Print("no user in context")
		c.WriteHeader(http.StatusInternalServerError)
		return
	}
	id := user.Id
	user, err := getUserById(user.Id)
	if err != nil {
		logpkg.Printf("error getting user for ID %d: %v", id, err)
		c.WriteHeader(http.StatusInternalServerError)
		return
	}
	if user.Deleted {
		// TODO: not found? no message?
		c.WriteError(http.StatusUnauthorized, "user deleted")
		return
	}
	if err := c.WriteMarshaledJSON(user); err != nil {
		logpkg.Print("error mashaling user JSON: ", err)
		c.WriteHeader(http.StatusInternalServerError)
	}
}

type GetLogsResp struct {
	Logs  []Log  `json:"logs"`
	Error string `json:"error,omitempty"`
}

func getLogsHandler(c *jmux.Context) {
	user, ok := userFromContext(c.Request.Context())
	if !ok {
		logpkg.Print("no user in context")
		c.WriteHeader(http.StatusInternalServerError)
		return
	}
	w, r := c.Writer, c.Request

	var (
		start, end, limit, offset int64 = -1, -1, -1, -1
		err                       error
		sortBy                    = "id"
		sortDesc                  bool
	)
	query := r.URL.Query()
	if val := query.Get("start"); val != "" {
		if start, err = strconv.ParseInt(val, 10, 64); err != nil {
			http.Error(w, fmt.Sprintf("Bad start value: %s", val), http.StatusBadRequest)
			return
		}
	}
	if val := query.Get("end"); val != "" {
		if end, err = strconv.ParseInt(val, 10, 64); err != nil {
			http.Error(w, fmt.Sprintf("Bad end value: %s", val), http.StatusBadRequest)
			return
		}
	}
	if val := query.Get("limit"); val != "" {
		if limit, err = strconv.ParseInt(val, 10, 64); err != nil || limit <= 0 {
			http.Error(w, fmt.Sprintf("Bad limit value: %s", val), http.StatusBadRequest)
			return
		}
	}
	if val := query.Get("offset"); val != "" {
		if offset, err = strconv.ParseInt(val, 10, 64); err != nil || offset < 0 {
			http.Error(w, fmt.Sprintf("Bad offset value: %s", val), http.StatusBadRequest)
			return
		}
	}
	if val := query.Get("sort"); val != "" {
		switch val {
		case "id":
		case "timestamp":
		default:
			http.Error(w, fmt.Sprintf("Bad sort category: %s", sortBy), http.StatusBadRequest)
			return
		}
		sortBy = val
	}
	if val := query.Get("desc"); val != "" {
		sortDesc, err = strconv.ParseBool(val)
		if err != nil {
			http.Error(w, fmt.Sprintf("Bad desc value: %s", val), http.StatusBadRequest)
			return
		}
	}

	withUserDb(user.Id, func(db *sql.DB, err error) {
		if err != nil {
			logpkg.Printf("error getting db for user %d: %v", user.Id, err)
			return
		}
		logs, err := getLogsFromDb(
			db,
			GetLogsParams{
				Start:    start,
				End:      end,
				SortBy:   sortBy,
				SortDesc: sortDesc,
				Limit:    limit,
				Offset:   offset,
			},
		)
		resp := GetLogsResp{Logs: logs}
		if err != nil {
			logpkg.Printf("error getting logs for user %d: %v", user.Id, err)
			//resp.Error = err.Error()
			resp.Error = "internal server error"
		}
		c.WriteJSON(resp)
	})
}

func newLogHandler(c *jmux.Context) {
	user, ok := userFromContext(c.Request.Context())
	if !ok {
		logpkg.Print("no user in context")
		c.WriteHeader(http.StatusInternalServerError)
		return
	}

	log := Log{}
	if err := c.ReadBodyJSON(&log); err != nil {
		clientErr := utils.IsUnmarshalError(err) ||
			errors.Is(err, io.EOF) ||
			errors.Is(err, io.ErrUnexpectedEOF)
		if clientErr {
			c.WriteError(http.StatusBadRequest, fmt.Sprint("bad json: ", err))
		} else {
			logpkg.Print("error reading log json: ", err)
			c.WriteHeader(http.StatusInternalServerError)
		}
		return
	}
	if len(log.Msg) > maxLogMsgLen {
		c.WriteError(http.StatusBadRequest, "exceeds max log Amessage length")
		return
	}
	if err := log.populateTags(); err != nil {
		c.WriteError(http.StatusBadRequest, err.Error())
	}
	if log.Timestamp <= 0 {
		log.Timestamp = time.Now().Unix()
	}
	withUserDb(user.Id, func(db *sql.DB, err error) {
		if err != nil {
			logpkg.Printf("error getting db for user %d: %v", user.Id, err)
			return
		}
		if err := log.insertIntoDb(db); err != nil {
			logpkg.Print("error inserting log: ", err)
			c.WriteHeader(http.StatusInternalServerError)
			return
		}
		if err := c.WriteMarshaledJSON(log); err != nil {
			logpkg.Printf("error writing new json result: %v", err)
			c.WriteHeader(http.StatusInternalServerError)
		}
	})
}

func editLogHandler(c *jmux.Context) {
	user, ok := userFromContext(c.Request.Context())
	if !ok {
		logpkg.Print("no user in context")
		c.WriteHeader(http.StatusInternalServerError)
		return
	}
	r := c.Request
	var id int64

	idStr := r.URL.Query().Get("id")
	if idStr != "" {
		var err error
		if id, err = strconv.ParseInt(idStr, 10, 64); err != nil {
			c.WriteError(http.StatusBadRequest, "bad id: "+idStr)
			return
		}
	}

	ld := LogDiff{}
	if err := c.ReadBodyJSON(&ld); err != nil {
		clientErr := utils.IsUnmarshalError(err) ||
			errors.Is(err, io.EOF) ||
			errors.Is(err, io.ErrUnexpectedEOF)
		if clientErr {
			c.WriteError(http.StatusBadRequest, fmt.Sprint("bad json: ", err))
		} else {
			logpkg.Print("error reading log diff json: ", err)
			c.WriteHeader(http.StatusInternalServerError)
		}
		return
	}
	if id != 0 {
		ld.Id = id
	}
	if err := ld.populateTags(); err != nil {
		c.WriteError(http.StatusBadRequest, err.Error())
	}

	withUserDb(user.Id, func(db *sql.DB, err error) {
		if err := ld.updateInDb(db); err != nil {
			logpkg.Printf("error updating log for user %d: %v", user.Id, err)
			c.WriteHeader(http.StatusInternalServerError)
		}
	})
}

func deleteLogHandler(c *jmux.Context) {
	user, ok := userFromContext(c.Request.Context())
	if !ok {
		logpkg.Print("no user in context")
		c.WriteHeader(http.StatusInternalServerError)
		return
	}
	r := c.Request

	var ids []int64
	for _, idsStr := range r.URL.Query()["ids"] {
		for _, idStr := range strings.Split(idsStr, ",") {
			id, err := strconv.ParseInt(idStr, 10, 64)
			if err != nil {
				c.WriteError(http.StatusBadRequest, fmt.Sprint("invalid id: ", idStr))
				return
			}
			ids = append(ids, id)
		}
	}
	for _, idStr := range r.URL.Query()["id"] {
		id, err := strconv.ParseInt(idStr, 10, 64)
		if err != nil {
			c.WriteError(http.StatusBadRequest, fmt.Sprint("invalid id: ", idStr))
			return
		}
		ids = append(ids, id)
	}
	withUserDb(user.Id, func(db *sql.DB, err error) {
		if err != nil {
			logpkg.Printf("error getting db for user %d: %v", user.Id, err)
			return
		}
		if err := deleteLogsByIds(db, ids...); err != nil {
			logpkg.Printf("error deleting logs for user %d: %v", user.Id, err)
			c.WriteHeader(http.StatusInternalServerError)
		}
	})
}

type Tags = uint64

const (
	tagStart Tags = 1 << 0
	tagEnd   Tags = 1 << 1
)

func tagsToStrs(tags Tags) []string {
	var strs []string
	if tags&tagStart != 0 {
		strs = append(strs, "start")
	}
	if tags&tagEnd != 0 {
		strs = append(strs, "end")
	}
	return strs
}

func strsToTags(strs []string) (Tags, error) {
	var tags Tags
	for _, str := range strs {
		switch s := strings.ToLower(str); s {
		case "start":
			tags |= tagStart
		case "end":
			tags |= tagEnd
		default:
			return tags, fmt.Errorf("invalid tag string: %s", str)
		}
	}
	return tags, nil
}

type Log struct {
	Id            int64    `json:"id,omitempty"`
	Timestamp     int64    `json:"timestamp,omitempty"`
	Msg           string   `json:"msg,omitempty"`
	Tags          []string `json:"tags,omitempty"`
	tags          Tags
	CounterpartId *int64 `json:"counterpartId,omitempty"`
}

// populates log.Tags from log.tags
func (log *Log) populateTagStrs() {
	log.Tags = tagsToStrs(log.tags)
}

// populates log.tags from log.Tags
func (log *Log) populateTags() error {
	tags, err := strsToTags(log.Tags)
	if err == nil {
		log.tags = tags
	}
	return err
}

type LogDiff struct {
	Id        int64   `json:"id,omitempty"`
	Timestamp *int64  `json:"timestamp,omitempty"`
	Msg       *string `json:"msg,omitempty"`
	// To clear out (set as empty), an empty array must be passed in the JSON,
	// not null or nonexistent (i.e., "tags":[] NOT "tags":null).
	Tags          *[]string `json:"tags,omitempty"`
	tags          *Tags
	CounterpartId *int64 `json:"counterpartId,omitempty"`
}

// populates log.Tags from log.tags
func (ld *LogDiff) populateTagStrs() {
	if ld.tags == nil {
		ld.Tags = nil
	} else {
		*ld.Tags = tagsToStrs(*ld.tags)
	}
}

// populates log.tags from log.Tags
func (ld *LogDiff) populateTags() error {
	if ld.Tags == nil {
		ld.tags = nil
		return nil
	} else {
		tags, err := strsToTags(*ld.Tags)
		if err == nil {
			*ld.tags = tags
		}
		return err
	}
}

type GetLogsParams struct {
	Start, End    int64
	SortBy        string
	SortDesc      bool
	Offset, Limit int64
}

func getLogsFromDb(db *sql.DB, params GetLogsParams) ([]Log, error) {
	const maxLimit int64 = 1000

	var clauses []string
	if params.Start != -1 {
		clauses = append(clauses, fmt.Sprintf("timestamp >= %d", params.Start))
	}
	if params.End != -1 {
		clauses = append(clauses, fmt.Sprintf("timestamp < %d", params.End))
	}
	stmt := `SELECT * FROM logs`
	if len(clauses) != 0 {
		stmt += ` WHERE ` + strings.Join(clauses, " AND ")
	}
	stmt += ` ORDER BY ` + params.SortBy
	if params.SortDesc {
		stmt += ` DESC`
	}
	if params.Limit <= 0 {
		params.Limit = 50
	} else if params.Limit > maxLimit {
		params.Limit = maxLimit
	}
	stmt += fmt.Sprintf(` LIMIT %d`, params.Limit)
	if params.Offset != -1 {
		stmt += fmt.Sprintf(` OFFSET %d`, params.Offset)
	}

	rows, err := db.Query(stmt)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var logs []Log
	for rows.Next() {
		log := Log{}
		e := rows.Scan(
			&log.Id, &log.Timestamp, &log.Msg, &log.tags, &log.CounterpartId,
		)
		if e != nil {
			if err == nil {
				err = e
			}
		} else {
			log.populateTagStrs()
			logs = append(logs, log)
		}
	}
	return logs, err
}

// tags must be populated before calling. Will populate tag strs (log.Tags)
// on success
func (log *Log) insertIntoDb(db *sql.DB) error {
	res, err := db.Exec(
		`INSERT INTO logs(timestamp,msg,tags,counterpart_id) VALUES (?,?,?,?)`,
		log.Timestamp, log.Msg, log.tags, log.CounterpartId,
	)
	if err != nil {
		return err
	}
	id, err := res.LastInsertId()
	if err != nil {
		/*
		   errStr := fmt.Sprintf("error getting insert ID log: %v", err)
		*/
		// TODO
		return err
	}
	log.Id = id
	log.populateTagStrs()
	return nil
}

// tags must be populated before calling.
func (ld LogDiff) updateInDb(db *sql.DB) error {
	var stmt string
	if ld.Timestamp != nil {
		if *ld.Timestamp <= 0 {
			*ld.Timestamp = time.Now().Unix()
		}
		stmt += ",timestamp=" + strconv.FormatInt(*ld.Timestamp, 10)
	}
	if ld.Msg != nil {
		stmt += ",msg=" + *ld.Msg
	}
	if ld.tags != nil {
		stmt += ",tags=" + strconv.FormatUint(*ld.tags, 10)
	}
	if ld.CounterpartId != nil {
		if *ld.CounterpartId <= 0 {
			stmt += ",counterpart_id=null"
		} else {
			stmt += "counterpart_id=" + strconv.FormatInt(*ld.CounterpartId, 10)
		}
	}
	if stmt == "" {
		return nil
	}

	_, err := db.Exec(
		fmt.Sprintf(`UPDATE logs SET %s WHERE id=?`, stmt[1:]),
		ld.Id,
	)
	return err
}

func deleteLogsByIds(db *sql.DB, ids ...int64) error {
	idsStr := ""
	for _, id := range ids {
		idsStr += "," + strconv.FormatInt(id, 10)
	}
	idsStr = idsStr[1:]
	_, err := db.Exec(fmt.Sprintf(`DELETE FROM logs WHERE id IN (%s)`, idsStr))
	return err
}

type User struct {
	Id           int64  `json:"id,omitempty"`
	Email        string `json:"email,omitempty"`
	passwordHash string
	Deleted      bool `json:"deleted,omitempty"`
	MaxLogs      int  `json:"maxLogs,omitempty"`
}

var (
	errUserNotExist = fmt.Errorf("user not found")
	errUserExists   = fmt.Errorf("user exists")
)

func getUserById(userId int64) (user User, err error) {
	row := usersDb.QueryRow(
		`SELECT email,password_hash,deleted,max_logs FROM users WHERE id=?`,
		userId,
	)
	err = row.Scan(&user.Email, &user.passwordHash, &user.Deleted, &user.MaxLogs)
	if errors.Is(err, sql.ErrNoRows) {
		err = errUserNotExist
	}
	return
}

func getUserByEmail(email string) (user User, err error) {
	row := usersDb.QueryRow(
		`SELECT id,password_hash,deleted,max_logs FROM users WHERE email=?`,
		email,
	)
	err = row.Scan(&user.Id, &user.passwordHash, &user.Deleted, &user.MaxLogs)
	if errors.Is(err, sql.ErrNoRows) {
		err = errUserNotExist
	}
	return
}

func userFromContext(ctx context.Context) (user User, ok bool) {
	iUser := ctx.Value(userCtxKey)
	if iUser != nil {
		user, ok = iUser.(User)
	}
	return
}

func (user *User) hashPassword() error {
	hash, err := bcrypt.GenerateFromPassword(
		[]byte(user.passwordHash),
		bcrypt.DefaultCost,
	)
	if err == nil {
		user.passwordHash = string(hash)
	}
	return err
}

func (user *User) create() error {
	res, err := usersDb.Exec(
		`INSERT INTO users(email,password_hash,deleted,max_logs) VALUES (?,?,?,?)`,
		user.Email, user.passwordHash, false, maxLogs,
	)
	if err != nil {
		return err
	}
	user.Deleted = false
	id, err := res.LastInsertId()
	if err != nil {
		return err
	}
	user.Id = id
	_, err = createUserDb(id)
	return err
}

type userCtxKeyType string

const userCtxKey userCtxKeyType = "user"

type Claims struct {
	jwt.RegisteredClaims
}

func generateToken(userId int64) (string, error) {
	now := time.Now()
	claims := &Claims{
		jwt.RegisteredClaims{
			Subject:   fmt.Sprint(userId),
			ExpiresAt: jwt.NewNumericDate(now.Add(tokenDur)),
			IssuedAt:  jwt.NewNumericDate(now),
		},
	}
	tok := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	return tok.SignedString(jwtKey)
}

// Returns false if getting the token failed
func getTokenStr(c *jmux.Context) (string, bool) {
	cookie, err := c.Request.Cookie(tokenCookieName)
	if err == nil {
		if cookie.Valid() == nil || true {
			return cookie.Value, true
		}
	}

	const start = len("Bearer ")
	auth := c.Request.Header.Get("Authorization")
	if len(auth) < start {
		return "", false
	}
	return auth[start:], true
}

func parseToken(tokStr string) (tok *jwt.Token, err error) {
	claims := &Claims{}
	/*
	  defer func() {
	    if tok != nil {
	      tok.Claims = claims
	    }
	  }()
	*/
	return jwt.ParseWithClaims(tokStr, claims, func(tok *jwt.Token) (any, error) {
		if _, ok := tok.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", tok.Header["alg"])
		}
		return jwtKey, nil
	})
}

var errNoTokenUser = fmt.Errorf("no user data in token")

func userFromToken(tok *jwt.Token) (User, error) {
	claims, ok := tok.Claims.(*Claims)
	if !ok {
		return User{}, errNoTokenUser
	}
	userId, err := strconv.ParseInt(claims.Subject, 10, 64)
	if err != nil {
		return User{}, err
	}
	return User{
		Id: userId,
	}, nil
}

func openUsersDb() (db *sql.DB, err error) {
	const dbInitStmt = `
CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY,
  email TEXT UNIQUE NOT NULL,
  password_hash TEXT NOT NULL,
  deleted BOOLEAN NOT NULL,
  max_logs INTEGER NOT NULL
);
  `
	usersDbPath := filepath.Join(databasesDir, "users.db")

	if db, err = sql.Open("sqlite3", usersDbPath); err != nil {
		return
	}
	if _, err = db.Exec(dbInitStmt); err != nil {
		return
	}
	return
}

func getUserDbPath(userId int64) string {
	// Dir name includes all user IDs from -1000 up to (but not including) it
	// 0-999 in dir 1000, 1000-1999 in dir 2000, etc
	dir := fmt.Sprint(userId/1000 + 1000)
	return filepath.Join(databasesDir, "users", dir, fmt.Sprint(userId)+".db")
}

func createUserDb(userId int64) (db *sql.DB, err error) {
	const dbInitStmt = `
CREATE TABLE IF NOT EXISTS logs (
  id INTEGER PRIMARY KEY,
  -- Second-precision
  timestamp INTEGER NOT NULL,
  msg TEXT NOT NULL,
  -- E.g., could signify a start/end message
  tags INTEGER NOT NULL,
  -- The couterpart is the opening/closing of this log (e.g., the start or end)
  counterpart_id INTEGER
);

CREATE TABLE IF NOT EXISTS frequents (
  id INTEGER PRIMARY KEY,
  msg TEXT NOT NULL
);
  `
	dbPath := getUserDbPath(userId)
	// create new dir
	/*
	  if userId % 1000 == 0 {
	    err = os.Mkdir(filepath.Dir(dbPath), 0660)
	    if err != nil {
	      return
	    }
	  }
	*/
	dir := filepath.Dir(dbPath)
	if _, err = os.Stat(dir); err != nil {
		if os.IsNotExist(err) {
			err = os.Mkdir(filepath.Dir(dbPath), 0660)
		}
		if err != nil {
			return
		}
	}

	if db, err = sql.Open("sqlite3", dbPath); err != nil {
		return
	}
	if _, err = db.Exec(dbInitStmt); err != nil {
		return
	}
	return
}

func getUserDb(userId int64) (*sql.DB, error) {
	path := fmt.Sprintf("file:%s?mode=rw", getUserDbPath(userId))
	return sql.Open("sqlite3", path)
}

// Closes the database after `f` returns.
func withUserDb(userId int64, f func(db *sql.DB, err error)) {
	db, err := getUserDb(userId)
	f(db, err)
	if db != nil {
		db.Close()
	}
}
