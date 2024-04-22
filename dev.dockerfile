ARG RUST_VERSION="1.69.0"
ARG ALPINE_VERSION="3.16"

FROM rust:${RUST_VERSION}-alpine${ALPINE_VERSION} as build

RUN \
  apk update && apk upgrade \
  && apk add pkgconf openssl-dev musl-dev cmake make gcc g++ curl

RUN \
  rustup update \
  && rustup default stable

ENV OPENSSL_DIR=/usr

WORKDIR /app

ENTRYPOINT [ "cargo", "run", "--" ]
