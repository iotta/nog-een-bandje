# ---- STAGE 1: Builder ----
# We use the official Rust image as our build environment.
# The 'slim' variant is smaller than the default, which speeds up downloads.
FROM rust:1.87-slim as builder

# Install the musl target for Rust. This allows us to compile a fully
# static binary that has no dependency on glibc.
RUN rustup target add x86_64-unknown-linux-musl

# Set the working directory inside the container.
WORKDIR /usr/src/nog-een-bandje

# Copy all our project files into the container's working directory.
# The .dockerignore file will prevent unnecessary files from being copied.
COPY . .

# Build the application for the musl target in release mode.
# This creates a highly optimized, single, static executable.
RUN cargo build --release --target x86_64-unknown-linux-musl

# ---- STAGE 2: Final Image ----
# We use the 'scratch' image, which is an empty base image.
# This is possible because our musl binary is statically linked and has no
# external runtime dependencies. This creates the smallest possible container.
FROM scratch

# Set the working directory for the final image.
WORKDIR /app

# Copy the compiled binary from the builder stage. Note the changed path due to the musl target.
COPY --from=builder /usr/src/nog-een-bandje/target/x86_64-unknown-linux-musl/release/nog-een-bandje .

# Copy the bands.json data file from the builder stage.
# The application will look for this file in its working directory.
COPY --from=builder /usr/src/nog-een-bandje/bands.json .

# Expose port 3000 to the outside world. This is the port our Axum
# server is listening on.
EXPOSE 3000

# Set the command to run when the container starts.
CMD ["/app/nog-een-bandje"]
