ARG RUST_VERSION="1.75.0"
ARG DEBIAN_VERSION="bullseye"

FROM rust:${RUST_VERSION}-slim-${DEBIAN_VERSION} AS base

RUN \
    apt-get update \
    && apt-get install --yes --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* /usr/share/man /usr/share/doc /usr/share/doc-base

WORKDIR /build

FROM base AS rust_base

RUN cargo install lazy_static || true

FROM base AS system

RUN \
    apt-get update --yes \
    && apt-get install --yes --no-install-recommends unzip musl musl-dev musl-tools build-essential pkgconf linux-libc-dev xutils-dev make wget git jq clang \
    && apt-get autoremove -yqq --purge \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* /usr/share/man /usr/share/doc /usr/share/doc-base

FROM base AS sccache

ARG VERSION="0.3.0"

ADD "https://github.com/mozilla/sccache/releases/download/v${VERSION}/sccache-v${VERSION}-x86_64-unknown-linux-musl.tar.gz" "sccache.tar.gz"
RUN \
    tar -xf sccache.tar.gz \
    && mkdir -p bin \
    && mv sccache*/sccache bin/ \
    && rm -rf sccache*

FROM system AS final

COPY --from=sccache /build/bin/* /usr/local/bin/

RUN \
    chmod 0755 /usr/local/bin/* \
    && rustup component add clippy \
    && rustup target add x86_64-unknown-linux-musl

WORKDIR /workspace
ENTRYPOINT [""]
