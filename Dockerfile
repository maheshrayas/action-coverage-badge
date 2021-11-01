FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

# Create appuser
ENV USER=cover
ENV UID=10001

ARG GH_VERSION=2.2.0

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


RUN wget -O gh.tar.gz --progress=dot:mega "https://github.com/cli/cli/releases/download/v${GH_VERSION}/gh_${GH_VERSION}_linux_amd64.tar.gz" && \
    tar -xf gh.tar.gz -C /tmp && cp "/tmp/gh_${GH_VERSION}_linux_amd64/bin/gh" /tmp && chmod +x /tmp/gh

COPY src src
COPY Cargo.toml Cargo.toml
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM ubuntu:bionic

RUN set -eux \
    && DEBIAN_FRONTEND=noninteractive \
    && apt-get update \
    && apt-get upgrade -y \
    && apt-get install --yes --no-install-recommends \
        git \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY --from=builder /tmp/gh /usr/local/bin/

WORKDIR /cover

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

COPY --from=builder /target/x86_64-unknown-linux-musl/release/cover ./

# USER cover:cover

ENTRYPOINT [ "/cover/cover" ]