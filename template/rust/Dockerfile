# syntax=docker/dockerfile:1.3-labs
FROM openfaas/classic-watchdog:0.18.0 as watchdog

FROM rust:1.58-alpine as build

RUN apk update && apk upgrade \
    && apk add --no-cache ca-certificates pkgconfig libressl-dev musl-dev

WORKDIR /usr/src/openfaas

# create a new empty shell project
RUN cargo new --bin main && \
    cargo new --lib function

# copy over the manifests
COPY function/Cargo.lock ./function/Cargo.lock
COPY function/Cargo.toml ./function/Cargo.toml
COPY main ./main

# Dirty hack to have the `handle` function in the library for compilation of the cache
RUN echo -e 'use std::error::Error;\n\
pub async fn handle(req : String) -> Result<String, Box<dyn Error>> {\n\
    Ok(req)\n\
}\n'\
> ./function/src/lib.rs

# this build step will cache the dependencies
RUN cd function && cargo build --release
RUN cd main && cargo build --release
RUN rm function/src/*.rs

# copy the source tree
COPY function/src ./function/src

# build for release
# RUN rm ./target/release/deps/handler*
# RUN cargo install --path ./main
RUN cd main && cargo build --release

# Take a slimmer image as the runtime base
FROM alpine:3.15 as runtime

# prepare the watchdog for open-faas
COPY --from=watchdog /fwatchdog /usr/bin/fwatchdog
RUN chmod +x /usr/bin/fwatchdog

# install the soft dependencies
# RUN apk update --no-cache && apk upgrade --no-cache \
    # && apk add --no-cache ca-certificates

# Get the binary
# COPY --from=build /usr/local/cargo/bin/main /usr/local/bin/main
COPY --from=build /usr/src/openfaas/main/target/release/main /usr/local/bin/main

HEALTHCHECK --interval=5s CMD [ -e /tmp/.lock ] || exit 1

ENV fprocess="main"

CMD ["fwatchdog"]

