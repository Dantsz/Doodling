FROM rust:1.67

EXPOSE 3000

WORKDIR /usr/src/DoodlingServer
COPY . .

RUN cargo install --path .

CMD ["DoodlingServer"]