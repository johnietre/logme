server: server/*.go
	-@mkdir -p bin
	go build -o bin/server $^
