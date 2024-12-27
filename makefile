.PHONY: client clean databases

bin/logme: cmd/logme/*.go server/*.go
	-@mkdir -p bin
	go build -o $@ cmd/logme/*.go

logme: bin/logme

client:
	cd client && wasm-pack build --target=web --release

databases:
	-@mkdir -p databases/users

logs:
	-@mkdir -p logs/users

clean:
	rm -i databases/users.db
	rm -ir databases/users/*
	rm -ir logs/*
