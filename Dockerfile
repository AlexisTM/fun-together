FROM rust:1.65.0 AS builder

# Copy local code to the container image.
WORKDIR /usr/src/app

# Make a layer with libs prebuilt, to reduce build times
COPY ./Cargo.toml .
COPY ./Cargo.lock .
RUN mkdir ./src && echo 'fn main() { println!("Dummy!"); }' > ./src/main.rs
RUN cargo build --release
RUN rm -rf ./src

# Build the actual software
COPY . .
RUN cargo install --path .
ENV PORT 8080
EXPOSE 8080
CMD ["fun-together-server", "0.0.0.0:8080"]

# Second stage, to have a small docker image.
FROM rust:1.65.0 AS runner
COPY --from=builder /usr/src/app/target/release/fun-together-server /usr/local/bin/
ENV PORT 8080
EXPOSE 8080
CMD ["/usr/local/bin/fun-together-server", "0.0.0.0:8080"]
