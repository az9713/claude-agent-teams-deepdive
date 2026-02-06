FROM rust:1.83-alpine AS builder
RUN apk add --no-cache musl-dev git
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:3.19
RUN apk add --no-cache git
COPY --from=builder /app/target/release/todos /usr/local/bin/todos
ENTRYPOINT ["todos"]
