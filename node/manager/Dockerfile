# ARG RUST_TARGET=x86_64-unknown-linux-musl
# # ARG RUST_TARGET=x86_64-unknown-linux-gnu

# # ------------------------------------------------------------------------------
# # Cargo Build Stage
# # ------------------------------------------------------------------------------

# # FROM openfaas/of-watchdog:0.9.3 as watchdog
# # ARG RUST_TARGET

# # FROM ekidd/rust-musl-builder:1.55.0 as builder
# FROM rust:1.60 as builder
# # FROM rust:alpine3.15 as builder
# ARG RUST_TARGET
# WORKDIR /workdir

# RUN rustup target add x86_64-unknown-linux-musl

# COPY . ./manager

# WORKDIR /workdir/manager

# RUN cargo build --release --package manager --bin fog_node --target x86_64-unknown-linux-musl

# # ------------------------------------------------------------------------------
# # Final Stage
# # ------------------------------------------------------------------------------

# FROM alpine:3.15.4 as runner
# ARG RUST_TARGET
# WORKDIR /

# RUN apk --no-cache add ca-certificates

# RUN addgroup -g 1000 bobby &&\
#      adduser -D -s /bin/sh -u 1000 -G bobby bobby

# COPY --from=builder \
#     /workdir/rust/src/main/target/$RUST_TARGET/release/main \
#     /usr/local/bin/main
# RUN chown bobby:bobby /usr/local/bin/main

# USER bobby

# CMD ["fwatchdog"]


FROM rustdocker/rust:nightly as cargo-build 

RUN apt-get update &&\
    apt-get install musl-tools -y

RUN /root/.cargo/bin/rustup target add x86_64-unknown-linux-musl

WORKDIR /manager
RUN mkdir -p src/bin/fog_node \
    && mkdir -p src/bin/market \
    && touch src/lib.rs \
    && echo 'fn main() {println!("Hello, world!");}' > src/bin/fog_node/main.rs  \
    && echo 'fn main() {println!("Hello, world!");}' > src/bin/market/main.rs
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock 

RUN RUSTFLAGS=-Clinker=musl-gcc /root/.cargo/bin/cargo build --release --target=x86_64-unknown-linux-musl

RUN rm -f target/x86_64-unknown-linux-musl/release/deps/fog_node*  \
    && rm -f target/x86_64-unknown-linux-musl/release/deps/manager*
RUN rm src/*.rs 

COPY ./src ./src

RUN touch src/bin/fog_node/main.rs  \
    && /root/.cargo/bin/cargo build --release --target=x86_64-unknown-linux-musl --package manager --bin fog_node

FROM alpine:latest 
COPY --from=cargo-build /manager/target/x86_64-unknown-linux-musl/release/fog_node .
CMD ["./fog_node"]