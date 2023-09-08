FROM rust:1.70 as build

WORKDIR /usr/src/DoodlingServer
COPY . .

RUN cargo build --release

FROM ubuntu:latest
EXPOSE 3000

WORKDIR /usr
COPY ./DoodlingHtmx ./DoodlingServer/DoodlingHtmx
COPY --from=build /usr/src/DoodlingServer/target/release/doodling_server ./DoodlingServer/doodling_server

ENTRYPOINT ["./DoodlingServer/doodling_server"]