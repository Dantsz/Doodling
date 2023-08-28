FROM rust:1.70

EXPOSE 3000

WORKDIR /usr/src/DoodlingServer
COPY . .

RUN cargo install --path .

ENTRYPOINT ["DoodlingServer"]