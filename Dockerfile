FROM rust:1.86-slim AS builder
WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:stable-slim
WORKDIR /app
COPY --from=builder /build/target/release/onlyweb /app/onlyweb
EXPOSE 3999
CMD [ "./onlyweb" ]
