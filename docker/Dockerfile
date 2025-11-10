FROM rust:1.91.0-alpine3.22 AS build
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry \
    apk add --no-cache musl-dev \
    && cargo build --locked --release \
    && cp target/release/aladdin-2fa-cli /

FROM scratch
COPY --from=build /aladdin-2fa-cli /
ENTRYPOINT ["/aladdin-2fa-cli"]
