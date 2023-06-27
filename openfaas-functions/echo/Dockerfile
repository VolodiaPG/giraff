# syntax=docker/dockerfile:experimental
ARG FLAGS

FROM busybox:1.36.0-uclibc as busybox
ADD https://github.com/openfaas/of-watchdog/releases/download/0.9.11/fwatchdog-amd64 /bin/fwatchdog
RUN chmod +x /bin/fwatchdog

# ------------------------------------------------------------------------------
# Main Function
# ------------------------------------------------------------------------------

FROM rust@sha256:56418f03475cf7b107f87d7fabe99ce9a4a9f9904daafa99be7c50d9e7b8f84d as builder
ARG FLAGS
RUN apt-get update \
    && apt-get install -y libssl-dev pkg-config \
    && update-ca-certificates

WORKDIR /workplace-echo

COPY . .
COPY ./model/* ./model/
COPY ./helper/* ./helper/

RUN mkdir -p /output

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/workplace-echo/target \
    --mount=type=cache,target=/usr/local/rustup \
    touch src/main.rs && \
    cargo build --release --target=x86_64-unknown-linux-gnu $FLAGS  \
    && mv target/x86_64-unknown-linux-gnu/release/echo /output/

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------
FROM gcr.io/distroless/cc as echo

COPY --from=builder \
    /output/echo \
    /bin/echo

COPY --from=busybox \
    /bin/fwatchdog \
    /bin/fwatchdog

COPY --from=busybox \
    /bin/sh \
    /bin/sh

ENV RUST_LOG warn

# Set up watchdog for HTTP mode
# OpenFaaS vars
ENV fprocess="echo"
ENV mode="http"
ENV http_upstream_url="http://127.0.0.1:3000"

EXPOSE 8080

HEALTHCHECK --interval=3s CMD [ -e /tmp/.lock ] || exit 1

CMD ["fwatchdog"]
