ARG RUST_TARGET=x86_64-unknown-linux-musl
ARG HEALTHCHECK_PATH=/tmp/.lock

# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------
FROM rustlang/rust@sha256:5d413e11424513d41549952517fed570d07b374fa3e2caec61340026de9bbbc7 as builder
ARG RUST_TARGET
ARG HEALTHCHECK_PATH=/tmp/.lock

RUN apt-get update &&\
    apt-get install musl-tools musl-dev -y &&\
    update-ca-certificates

RUN rustup target add $RUST_TARGET

WORKDIR /builder

ADD https://github.com/openfaas/of-watchdog/releases/download/0.9.6/fwatchdog-amd64 fwatchdog
RUN chmod +x fwatchdog

RUN cargo new --bin healthcheck \
    && echo "use std::path::Path;fn main(){if Path::new(\"$HEALTHCHECK_PATH\").exists(){std::process::exit(0);}std::process::exit(1);}" > healthcheck/src/main.rs \
    && cd healthcheck \
    && cargo build --release --target $RUST_TARGET

# create a new empty shell project
RUN mkdir main
COPY main main/

RUN mkdir function/
COPY function/Cargo.lock function/Cargo.toml function/
# dirty hack to have the `handle::main` function in the library for compilation of the cache
COPY main/lib.rs.template function/src/lib.rs

# this build step will cache the dependencies
# RUN cd function && cargo build --release
RUN cd main \
    && cargo build --release --target $RUST_TARGET

RUN rm main/target/$RUST_TARGET/release/deps/handler* main/target/$RUST_TARGET/release/deps/libhandler*  main/target/$RUST_TARGET/release/deps/main*
RUN rm -rf function/src/*.rs

RUN touch main/src/main.rs
RUN touch function/src/lib.rs

# copy all the sources
COPY function/src function/src

# build for release
# RUN cd main && cargo build --target x86_64-unknown-linux-musl --release
RUN cd main \
    && cargo build --release --target $RUST_TARGET

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------
FROM scratch as runner
ARG RUST_TARGET

COPY --from=builder \
    /builder/main/target/$RUST_TARGET/release/main \
    /usr/local/bin/main

COPY --from=builder \
    /builder/healthcheck/target/$RUST_TARGET/release/healthcheck \
    /usr/local/bin/healthcheck

COPY --from=builder \
    /builder/fwatchdog \
    ./fwatchdog

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

HEALTHCHECK --interval=3s CMD [ /usr/local/bin/healthcheck ]

#CMD ["/usr/local/bin/main"]
CMD ["./fwatchdog"]
