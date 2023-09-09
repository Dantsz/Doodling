FROM rust:1.70 as build_canvas_app
WORKDIR /usr/src/DoodlingCanvas
COPY ./DoodlingCanvas .
RUN cargo install wasm-pack
RUN wasm-pack build --release --target web

FROM rust:1.70 as build_service

WORKDIR /usr/src/DoodlingServer
COPY ./DoodlingServer .

RUN cargo build --release

FROM ubuntu:latest
EXPOSE 3000

WORKDIR /usr/DoodlingServer
COPY ./DoodlingServer/DoodlingHtmx ./DoodlingHtmx
COPY --from=build_service /usr/src/DoodlingServer/target/release/doodling_server ./doodling_server
COPY --from=build_canvas_app /usr/src/DoodlingCanvas/pkg ./DoodlingHtmx/resources/pkg
ENTRYPOINT ["./doodling_server"]