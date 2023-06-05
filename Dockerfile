ARG ALPINE_VERSION=3.18.0

FROM alpine:${ALPINE_VERSION}

ARG S6_OVERLAY_VERSION=3.1.5.0

RUN apk add --no-cache \
        bash \
        iproute2 \
        nftables \
        tcpdump \
        wireguard-tools

ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-noarch.tar.xz /tmp
RUN tar -C / -Jxpf /tmp/s6-overlay-noarch.tar.xz
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-x86_64.tar.xz /tmp
RUN tar -C / -Jxpf /tmp/s6-overlay-x86_64.tar.xz

COPY /image /

EXPOSE 55555/udp

ENTRYPOINT [ "/init" ]