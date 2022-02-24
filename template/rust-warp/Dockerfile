FROM openfaas/of-watchdog:0.8.2 as watchdog

FROM rust:1.58-alpine as builder

RUN apk update && apk upgrade \
    && apk add --no-cache ca-certificates pkgconfig libressl-dev musl-dev

WORKDIR /usr/src/openfaas

# create a new empty shell project
RUN cargo new --lib function

COPY function/Cargo.lock  function/Cargo.toml ./function/
COPY main ./main

# dirty hack to have the `handle::main` function in the library for compilation of the cache
COPY main/lib.rs.template ./function/src/lib.rs

# this build step will cache the dependencies
# RUN cd function && cargo build --release
RUN cd main && cargo build --release

RUN rm function/src/*.rs
RUN rm main/target/release/deps/handler*  main/target/release/deps/main*

# copy all the sources
COPY function/src ./function/src

# build for release
# RUN cd main && cargo build --target x86_64-unknown-linux-musl --release
RUN cd main && cargo build --release

FROM alpine:3.15 as runner

# Add non-root user
RUN addgroup -S function && adduser -S -g function function

ENV USER=function

# Copy of-watchdog binary
COPY --from=watchdog /fwatchdog /usr/bin/fwatchdog
RUN chmod +x /usr/bin/fwatchdog

COPY --from=builder /usr/src/openfaas/main/target/release/main /usr/local/bin/main

# Set up watchdog for HTTP mode
# OpenFaaS vars
ENV fprocess="main"
ENV mode="http"
ENV upstream_url="http://127.0.0.1:3000"

HEALTHCHECK --interval=3s CMD [ -e /tmp/.lock ] || exit 1

CMD ["fwatchdog"]
