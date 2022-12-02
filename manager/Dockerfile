# syntax=docker/dockerfile:experimental
FROM rust@sha256:916fa3bc4599b3e394b03662452a853be8565fac7a8ed91bef62319dd549599d as build

RUN apt-get update &&\
    apt-get install musl-tools musl-dev -y &&\
    update-ca-certificates

WORKDIR /workplace

ADD . .

RUN mkdir bin

RUN --mount=type=cache,target=/usr/local/cargo/registry \
	--mount=type=cache,target=/workplace/target \
	--mount=type=cache,target=/usr/local/rustup \
	cargo build --release --target=x86_64-unknown-linux-gnu \
    && cp target/x86_64-unknown-linux-gnu/release/fog_node bin/ \
    && cp target/x86_64-unknown-linux-gnu/release/market bin/

FROM gcr.io/distroless/cc as market-default
COPY --from=build /workplace/bin/market .
EXPOSE 3003/tcp
EXPOSE 3004/tcp
CMD ["./market"]

FROM gcr.io/distroless/cc as fog_node-default
COPY --from=build /workplace/bin/fog_node .
EXPOSE 3003/tcp
EXPOSE 3004/tcp
CMD ["./fog_node"]
