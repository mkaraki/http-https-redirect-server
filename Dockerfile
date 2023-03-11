FROM rust:1-bullseye as build

ADD . /app
WORKDIR /app
RUN cargo build --release

FROM debian:bullseye-slim

COPY --from=build /app/target/release/http-https-redirect-server /usr/bin/
WORKDIR /app

EXPOSE 80
VOLUME /app

ENTRYPOINT [ "http-https-redirect-server" ]