ARG FLAGS
FROM rust@sha256:916fa3bc4599b3e394b03662452a853be8565fac7a8ed91bef62319dd549599d as build
ARG FLAGS
RUN apt-get update \
	&& apt-get install -y libssl-dev pkg-config \
	&& update-ca-certificates

WORKDIR /workplace-iot

ADD . .

RUN mkdir /output

RUN --mount=type=cache,target=/usr/local/cargo/registry \
	--mount=type=cache,target=/workplace-iot/target \
	--mount=type=cache,target=/usr/local/rustup \
	cargo build --release --target=x86_64-unknown-linux-gnu $FLAGS \
	&& mv target/x86_64-unknown-linux-gnu/release/iot_emulation /output/

FROM gcr.io/distroless/cc as iot_emulation-default
COPY --from=build /output/iot_emulation .

ENV SERVER_PORT 3003
EXPOSE 3003/tcp
CMD ["./iot_emulation"]
