FROM rustlang/rust@sha256:cbf8122e51e0b8beea34c0901d6cc347888ad8e274e8b0b7a358c7b9cdb34015 as cargo-build

RUN apt-get update &&\
    apt-get install musl-tools musl-dev -y &&\
    update-ca-certificates

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /manager
RUN mkdir -p src/bin/fog_node \
    && mkdir -p src/bin/market \
    && touch src/lib.rs \
    && echo 'fn main() {println!("Hello, world!");}' > src/bin/fog_node/main.rs  \
    && echo 'fn main() {println!("Hello, world!");}' > src/bin/market/main.rs
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN cargo build --release --target=x86_64-unknown-linux-musl

RUN rm -f target/x86_64-unknown-linux-musl/release/deps/fog_node*  \
    && rm -f target/x86_64-unknown-linux-musl/release/deps/manager* \
    && rm -f target/x86_64-unknown-linux-musl/release/deps/libmanager* \
    && rm -f target/x86_64-unknown-linux-musl/release/deps/market* \
    && rm -f target/x86_64-unknown-linux-musl/release/deps/libmanager* \
    && rm -f target/x86_64-unknown-linux-musl/release/fog_node* \
    && rm -f target/x86_64-unknown-linux-musl/release/market*
RUN rm src/*.rs

COPY ./src ./src

RUN touch src/bin/fog_node/main.rs  \
    && touch src/bin/market/main.rs

RUN cargo build --release --target=x86_64-unknown-linux-musl --package manager --bin fog_node

RUN cargo build --release --target=x86_64-unknown-linux-musl --package manager --bin market

FROM scratch as market
COPY --from=cargo-build /manager/target/x86_64-unknown-linux-musl/release/market .
CMD ["./market"]

FROM scratch as fog_node
COPY --from=cargo-build /manager/target/x86_64-unknown-linux-musl/release/fog_node .
CMD ["./fog_node"]
