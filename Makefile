format:
	cargo fmt

start:
	cargo run

build:
	cargo build --release

up:
	docker-compose up --build

docker:
	docker build -t cachier-core .
	docker run -p 8080:8080 cachier-core
