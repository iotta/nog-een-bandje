# ---- STAGE 1: Builder ----
# We use the official Rust image as our build environment.
# The 'slim' variant is smaller than the default, which speeds up downloads.
FROM rust:1.87-slim as builder

# Set the working directory inside the container.
WORKDIR /usr/src/nog-een-bandje

# Copy all our project files into the container's working directory.
# The .dockerignore file will prevent unnecessary files from being copied.
COPY . .

# Build the application in release mode. This creates a highly optimized,
# single binary executable.
RUN cargo build --release

# ---- STAGE 2: Final Image ----
# We use a minimal Debian image for our final container. This keeps the
# image size small and reduces the attack surface.
FROM debian:bullseye-slim

# Set the working directory for the final image.
WORKDIR /app

# Copy the compiled binary from the builder stage.
COPY --from=builder /usr/src/nog-een-bandje/target/release/nog-een-bandje .

# Copy the bands.json data file from the builder stage.
# The application will look for this file in its working directory.
COPY --from=builder /usr/src/nog-een-bandje/bands.json .

# Expose port 3000 to the outside world. This is the port our Axum
# server is listening on.
EXPOSE 3000

# Set the command to run when the container starts.
CMD ["./nog-een-bandje"]
