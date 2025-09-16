# GEMINI.md

## Project Overview

This project, named **Bold**, is a Network File System (NFS) server that is compatible with the 4-series version. It is developed in asynchronous Rust, utilizing the Tokio runtime. The project is structured as a Rust workspace, comprising three main components:

*   `exec`: This component contains the executable binaries.
    *   `bold-mem`: Serves an in-memory file system defined by a YAML file.
    *   `bold-nfs`: Shares a directory from the local file system.
*   `lib`: This is the core library of the NFS server.
*   `proto`: This component is responsible for handling the NFS protocol.

Additionally, the project includes a suite of tests written in Python, which leverage the `pytest` and `pytest-benchmark` frameworks.

## Building and Running

### Building the Project

To build the project, you can use the standard Cargo command:

```sh
cargo build
```

### Running the Binaries

#### `bold-mem` (In-memory file system)

This binary serves an in-memory file system based on a YAML configuration file.

1.  **Run the server:**

    ```sh
    cargo run -p bold-exec --bin bold-mem -- --debug exec/memoryfs.yaml
    ```

2.  **Mount the file system (on a separate terminal):**

    ```sh
    mkdir /tmp/demo
    sudo mount.nfs4 -n -v -o fg,soft,sec=none,vers=4.0,port=11112 127.0.0.1:/ /tmp/demo
    ```

3.  **Unmount the file system:**

    ```sh
    sudo umount /tmp/demo
    ```

#### `bold-nfs` (Real file system)

This binary shares a directory from your local file system.

1.  **Run the server:**

    ```sh
    cargo run -p bold-exec --bin bold-nfs -- /path/to/your/directory
    ```

2.  **Mount the file system (on a separate terminal):**

    ```sh
    mkdir /tmp/demo
    sudo mount.nfs4 -n -v -o fg,soft,sec=none,vers=4.0,port=11112 127.0.0.1:/ /tmp/demo
    ```

3.  **Unmount the file system:**

    ```sh
    sudo umount /tmp/demo
    ```

### Running Tests

The project's tests are written in Python using `pytest`. To run the tests, you will need to have Python and `poetry` installed.

1.  **Install dependencies:**

    ```sh
    poetry install
    ```

2.  **Run the tests:**

    ```sh
    poetry run pytest
    ```

## Development Conventions

*   The project follows standard Rust and Python coding conventions.
*   The codebase is organized into a Rust workspace, promoting modularity and separation of concerns.
*   Testing is a key part of the development process, with a dedicated test suite in Python.
