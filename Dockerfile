FROM rust:1.70 as build

WORKDIR /usr/src/DoodlingServer
COPY ./DoodlingServer .

RUN cargo build --release

FROM ubuntu:latest
EXPOSE 3000

WORKDIR /usr/DoodlingServer
COPY ./DoodlingServer/DoodlingHtmx ./DoodlingHtmx
COPY --from=build /usr/src/DoodlingServer/target/release/doodling_server ./doodling_server

ENTRYPOINT ["./doodling_server"]