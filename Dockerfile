FROM rust:1.71 as build
WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04
WORKDIR /app
COPY --from=build /app/target/release/udp-websocket .

ENTRYPOINT ["./udp-websocket"]