FROM rust@sha256:916fa3bc4599b3e394b03662452a853be8565fac7a8ed91bef62319dd549599d as build
RUN apt-get update \
    && apt-get install -y libssl-dev pkg-config \
    && update-ca-certificates

WORKDIR /workplace-manager

COPY . .

RUN mkdir -p bin/release

# --features async_routes 
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/workplace-manager/target \
    --mount=type=cache,target=/usr/local/rustup \
    touch fog_node/src/main.rs market/src/main.rs && \
    cargo build --release --target=x86_64-unknown-linux-gnu \
    && mv target/x86_64-unknown-linux-gnu/release/fog_node /output/ \
    && mv target/x86_64-unknown-linux-gnu/release/market /output/

FROM gcr.io/distroless/cc as market-default
USER 1003
COPY --from=build /output/market .
EXPOSE 3003/tcp
ENV SERVER_PORT 3003
CMD ["./market"]

FROM gcr.io/distroless/cc as fog_node-default
USER 1003
COPY --from=build /output/fog_node .
EXPOSE 3003/tcp
EXPOSE 3004/tcp
CMD ["./fog_node"]