ARG RUST_TARGET=x86_64-unknown-linux-musl

# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

# FROM openfaas/of-watchdog:0.9.3 as watchdog
# ARG RUST_TARGET

FROM ekidd/rust-musl-builder:latest as builder
ARG RUST_TARGET

# create a new empty shell project
RUN cargo new --lib function

COPY --chown=rust:rust function/Cargo.lock  function/Cargo.toml ./function/
COPY --chown=rust:rust main ./main

# dirty hack to have the `handle::main` function in the library for compilation of the cache
COPY --chown=rust:rust main/lib.rs.template ./function/src/lib.rs

# this build step will cache the dependencies
# RUN cd function && cargo build --release
RUN cd main && cargo build --release

RUN rm -rf function
RUN rm main/target/$RUST_TARGET/release/deps/handler*  main/target/$RUST_TARGET/release/deps/main*

# copy all the sources
COPY --chown=rust:rust function ./function

# build for release
# RUN cd main && cargo build --target x86_64-unknown-linux-musl --release
RUN cd main && cargo build --release

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:latest as runner
ARG RUST_TARGET

RUN apk --no-cache add ca-certificates

RUN addgroup -g 1000 bobby

RUN adduser -D -s /bin/sh -u 1000 -G bobby bobby

# Copy of-watchdog binary
# COPY --from=build /usr/bin/fwatchdog /usr/bin/fwatchdog
ADD https://github.com/openfaas/of-watchdog/releases/download/0.9.3/fwatchdog-amd64 /usr/bin/fwatchdog
# ADD https://github.com/openfaas/of-watchdog/releases/download/0.7.7/of-watchdog /usr/bin/fwatchdog

RUN chmod +x /usr/bin/fwatchdog

COPY --from=builder \
    /home/rust/src/main/target/$RUST_TARGET/release/main \
    /usr/local/bin/main
RUN chown bobby:bobby /usr/local/bin/main

# Set up watchdog for HTTP mode
# OpenFaaS vars
ENV fprocess="main"
ENV mode="http"

# https://github.com/openfaas/of-watchdog/pull/41
ENV buffer_http="true"

ENV upstream_url="http://127.0.0.1:3000"
ENV cgi_headers="true"
ENV exec_timeout="10s"
ENV write_timeout="15s"
ENV read_timeout="15s"

HEALTHCHECK --interval=3s CMD [ -e /tmp/.lock ] || exit 1

USER bobby

CMD ["fwatchdog"]
