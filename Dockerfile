# Builder stage
FROM rust:1.75.0-slim-bookworm as builder

# Create blank app for dependency layer
WORKDIR /usr/src
RUN USER=root cargo new app --bin --name pong
WORKDIR /usr/src/app

# Copy only the dependency files to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
RUN rm src/*.rs

# Copy the rest of the application source code
COPY . .

# Build the application
RUN rm ./target/release/deps/pong*
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

WORKDIR /usr/src/app

# Copy only the necessary files from the builder stage
COPY --from=builder /usr/src/app/target/release/pong /usr/src/app/
COPY --from=builder /usr/src/app/templates /usr/src/app/templates/

ENV RUST_LOG=debug

# Expose the port your application will run on
EXPOSE 3030

# Command to run the application
CMD ["./pong"]
