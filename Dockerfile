FROM rust:1.71.0-slim-buster as builder
WORKDIR /app
COPY . .
RUN cargo build --release

RUN ls -la /app/target/release

FROM debian:buster-slim
COPY --from=builder /app/target/release/cachier-core .
EXPOSE 8080
CMD [ "./cachier-core" ]
