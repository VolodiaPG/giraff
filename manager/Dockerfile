FROM ghcr.io/volodiapg/rust:nightly as cargo-build

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
    && rm -f target/x86_64-unknown-linux-musl/release/deps/manager* \
    && rm -f target/x86_64-unknown-linux-musl/release/deps/market*
RUN rm src/*.rs

COPY ./src ./src

RUN touch src/bin/fog_node/main.rs  \
    && touch src/bin/market/main.rs

RUN /root/.cargo/bin/cargo build --release --target=x86_64-unknown-linux-musl --package manager --bin fog_node

RUN /root/.cargo/bin/cargo build --release --target=x86_64-unknown-linux-musl --package manager --bin market

FROM ghcr.io/volodiapg/alpine:latest as market
COPY --from=cargo-build /manager/target/x86_64-unknown-linux-musl/release/market /usr/bin
CMD ["market"]

FROM ghcr.io/volodiapg/alpine:latest as fog_node
COPY --from=cargo-build /manager/target/x86_64-unknown-linux-musl/release/fog_node /usr/bin
CMD ["fog_node"]
