FROM rust:latest

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY ./src ./src

EXPOSE 8080

CMD ["cargo", "run"]
