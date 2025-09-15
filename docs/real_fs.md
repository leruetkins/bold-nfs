# Real File System Support in Bold

## Overview

Bold now includes support for serving real file systems, in addition to the existing in-memory file system support. This allows you to expose any directory on your system via NFS.

## Implementation Details

The real file system support is implemented through several new modules:

1. `real_fs.rs` - Implements the `RealFS` struct that provides NFS operations on a real file system
2. `fs_util.rs` - Contains utility functions for working with file system metadata
3. `vfs.rs` - Defines the virtual file system interface that both real and in-memory file systems implement
4. `nfs.rs` - Contains NFS protocol definitions

## Usage

To use the real file system support, you can run the `bold-real-fs` binary:

```bash
cargo run -p bold-mem --bin bold-real-fs -- /path/to/directory/to/serve
```

This will start an NFS server that serves the specified directory.

## Architecture

The real file system implementation follows the same architecture as the in-memory file system:

1. The `RealFS` struct implements the `NFSFileSystem` trait
2. File operations are performed directly on the underlying file system
3. Metadata is converted between system format and NFS format using utility functions
4. The server handles NFS protocol communication with clients

## Features

- Full read/write support for files and directories
- Support for file attributes (permissions, timestamps, etc.)
- Directory listing and navigation
- File creation and deletion
- Directory creation and removal
- Symlink support (on Unix-like systems)
- Proper error handling for file system operations

## Limitations

- Some advanced NFS features may not be fully implemented yet
- Performance optimizations for large directories/files may be needed
- Platform-specific behavior may vary (especially for symlinks on Windows)

## Future Improvements

- Add caching layer for improved performance
- Implement more advanced NFS features
- Add support for extended attributes
- Improve error handling and reporting
- Add more comprehensive testing