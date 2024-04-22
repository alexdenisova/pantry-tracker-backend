ARG VERSION="3.16"

FROM alpine:${VERSION} AS final

RUN \
    apk --no-cache add curl jq \
    && rm -rf /var/cache/apk/* /tmp/*

COPY dist/bin/* /bin/

WORKDIR /workspace
ENTRYPOINT [""]
