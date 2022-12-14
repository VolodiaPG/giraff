# syntax=docker/dockerfile:experimental
FROM rust@sha256:916fa3bc4599b3e394b03662452a853be8565fac7a8ed91bef62319dd549599d as build
RUN apt-get update \
	&& apt-get install -y libssl-dev pkg-config \
	&& update-ca-certificates

WORKDIR /workplace

ADD . .

RUN mkdir bin

RUN --mount=type=cache,target=/usr/local/cargo/registry \
	--mount=type=cache,target=/workplace/target \
	--mount=type=cache,target=/usr/local/rustup \
	cargo build --release --target=x86_64-unknown-linux-gnu \
	&& cp target/x86_64-unknown-linux-gnu/release/iot_emulation bin/

FROM gcr.io/distroless/cc as iot_emulation-default
COPY --from=build /workplace/bin/iot_emulation .

ENV SERVER_PORT 3003
EXPOSE 3003/tcp
CMD ["./iot_emulation"]
