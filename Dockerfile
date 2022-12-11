FROM rust:1.65.0 AS builder

# Copy local code to the container image.
WORKDIR /usr/src/app
COPY . .

# Install production dependencies and build a release artifact.
RUN cargo install --path .
ENV PORT 8080
EXPOSE 8080
CMD ["fun-together-server", "0.0.0.0:8080"]

# Second stage, to have a small docker image.
FROM alpine:latest AS runner
COPY --from=builder /usr/src/app/target/release/fun-together-server /bin
ENV PORT 8080
EXPOSE 8080
CMD ["fun-together-server", "0.0.0.0:8080"]
