# Builder
FROM rust:alpine AS builder
WORKDIR /app
COPY . .
RUN apk add --no-cache --update alpine-sdk openssl-dev openssl-libs-static && \
    cargo build --release

# Production
FROM alpine AS prod

COPY --from=builder /app/target/release/swiftstream .

CMD [ "./swiftstream" ]
