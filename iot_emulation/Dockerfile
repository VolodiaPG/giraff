ARG RUST_TARGET=x86_64-unknown-linux-musl
ARG HEALTHCHECK_PATH=/tmp/.lock

# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------
FROM rustlang/rust@sha256:cbf8122e51e0b8beea34c0901d6cc347888ad8e274e8b0b7a358c7b9cdb34015 as builder
ARG RUST_TARGET

RUN apt-get update &&\
    apt-get install musl-tools musl-dev -y &&\
    update-ca-certificates

RUN rustup target add $RUST_TARGET

WORKDIR /builder

# create a new empty shell project
RUN mkdir -p iot_emulation/src \
    && echo "fn main(){println!(\"Still initial config is here.\");std::process::exit(1);}" > iot_emulation/src/main.rs
COPY Cargo.lock Cargo.toml iot_emulation/

# this build step will cache the dependencies
# RUN cd function && cargo build --release
RUN cd iot_emulation \
    && cargo build --release --target $RUST_TARGET

RUN rm iot_emulation/target/$RUST_TARGET/release/deps/iot_emulation*
RUN rm -rf iot_emulation/src/*.rs

RUN touch iot_emulation/src/main.rs

# copy all the sources
COPY src iot_emulation/src

# build for release
# RUN cd main && cargo build --target x86_64-unknown-linux-musl --release
RUN cd iot_emulation \
    && cargo build --release --target $RUST_TARGET

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------
FROM scratch as runner
ARG RUST_TARGET

COPY --from=builder \
    /builder/iot_emulation/target/$RUST_TARGET/release/iot_emulation \
    /usr/local/bin/iot_emulation

ENV ROCKET_PORT="3030"
ENV ROCKET_ADDRESS="0.0.0.0"
EXPOSE 3030

CMD ["iot_emulation"]
