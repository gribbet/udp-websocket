FROM rust:1.78 as build
WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04
WORKDIR /app
COPY --from=build /app/target/release/udp-websocket .

ENV WEBSOCKET_ADDRESS=0.0.0.0:8080
ENV UDP_ADDRESS=0.0.0.0:14550

ENTRYPOINT ./udp-websocket $WEBSOCKET_ADDRESS $UDP_ADDRESS