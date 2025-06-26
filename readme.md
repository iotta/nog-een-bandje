# Festival Artist Explorer

A simple and efficient web application built with Rust and Axum that allows users to explore artist performance data from the Pinkpop and Lowlands festivals (2008-2019).

The application provides a web interface for random discovery and live searching, as well as a JSON API for programmatic access to the data. It's designed to be lightweight, fast, and easy to deploy using Docker.

![Screenshot of the Festival Artist Explorer UI](https://placehold.co/600x400/1f2937/9ca3af?text=App+Screenshot+Here)

## Features

- **Interactive Web UI**: A clean, responsive interface built with Tailwind CSS.
- **Random Artist Discovery**: Get a random selection of 1-5 artist performances.
- **Live Artist Search**: Instantly search through all historical performances as you type.
- **Data Download**: Download the complete, aggregated dataset as a single JSON file.
- **JSON API**: Simple endpoints for fetching random or complete data.
- **Containerized**: Includes a multi-stage `Dockerfile` that builds a minimal, fully static container using `musl` to avoid `glibc` versioning issues.

## Tech Stack

- **Backend**: Rust
- **Web Framework**: Axum
- **Asynchronous Runtime**: Tokio
- **Styling**: Tailwind CSS (via CDN)
- **Containerization**: Docker

## Prerequisites

- **Rust**: Ensure you have the Rust toolchain installed. You can get it from [rustup.rs](https://rustup.rs/).
- **Docker** (Optional): Required if you want to build and run the application as a Docker container.

## Getting Started

Follow these steps to get the application running on your local machine.

### 1. Clone the Repository

```bash
git clone <your-repository-url>
cd festival-artist-explorer
```

### 2. Prepare the Data

Ensure the performance data file, bands.json, is present in the root of the project directory. The application will read this file on startup.

### 3. Running Locally with Cargo
To run the application directly using Cargo:

    # This will compile and run the application in debug mode
    cargo run

The server will start, and you can access it at `http://localhost:3000`.

### 4. Building and Running with Docker

This project is configured to build a minimal, statically-linked binary that runs in a `scratch` (empty) container for maximum portability and security. To build and run the application inside a Docker container:

    # 1. Build the Docker image
    # The tag 'band-explorer' is an example; you can name it anything.
    docker build -t band-explorer .

    # 2. Run the container
    # This maps your local port 3000 to the container's port 3000.
    docker run -p 3000:3000 --rm --name band-app band-explorer


The containerized application will be available at `http://localhost:3000`.

### API Endpoints

The application exposes the following API endpoints:

- `GET /`
  - **Description**: Serves the main HTML user interface.
  - **Response**: `text/html`

- `GET /api/random-bands`
  - **Description**: Returns a random selection of artist performances.
  - **Query Parameters**: count (optional, number): The number of artists to return. Defaults to 1. Clamped between 1 and 5.
  - **Example**: `http://localhost:3000/api/random-bands?count=3`
  - **Response**: `application/json`
    ```json
    [
    {
        "name": "The Killers",
        "festival": "Pinkpop",
        "year": 2009
    },
    {
        "name": "Major Lazer",
        "festival": "Lowlands",
        "year": 2015
    }
    ]
    ```

- `GET /api/all-bands`
  - **Description**: Returns the complete list of all performances. The Content-Disposition header is set to prompt a file download.
  - **Response**: `application/json`
