FROM openfaas/of-watchdog:0.8.2 as watchdog

FROM rust:1.58-alpine as builder

RUN apk update && apk upgrade \
    && apk add --no-cache ca-certificates pkgconfig libressl-dev musl-dev

WORKDIR /usr/src/openfaas

# create a new empty shell project
RUN cargo new --lib function

# copy over the manifests
COPY function/Cargo.lock ./function/Cargo.lock
COPY function/Cargo.toml ./function/Cargo.toml
COPY main ./main

# dirty hack to have the `handle` function in the library for compilation of the cache
RUN echo -e 'use std::error::Error;\n\
use hyper::{Body, Request, Response};\n\
const PHRASE: &str = "Hello, World!";\n\
pub async fn handle(_req: Request<Body>) -> Result<Response<Body>, Box<dyn Error>> {\n\
    Ok(Response::new(Body::from(PHRASE)))\n\
}\n'\
> ./function/src/lib.rs

# this build step will cache the dependencies
# RUN cd function && cargo build --release
RUN cd main && cargo build --release
RUN rm function/src/*.rs

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
ENV fprocess="./main"
ENV mode="http"
ENV upstream_url="http://127.0.0.1:3000"

HEALTHCHECK --interval=3s CMD [ -e /tmp/.lock ] || exit 1

CMD ["fwatchdog"]
