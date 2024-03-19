ARG RUST_VERSION="1.76.0"
ARG ALPINE_VERSION="3.18"

FROM rust:${RUST_VERSION}-alpine${ALPINE_VERSION} as build

RUN \
  apk update && apk upgrade \
  && apk add pkgconf openssl-dev musl-dev cmake make gcc g++ curl

ENV OPENSSL_DIR=/usr

WORKDIR /app

ENTRYPOINT [ "cargo", "run", "--" ]
