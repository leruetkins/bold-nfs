# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Added support for serving real file systems via NFS
- Created `bold-real-fs` binary for serving real file system directories
- Added `RealFS` implementation for real file system operations
- Added `fs_util` module with file system utility functions
- Added `vfs` module with virtual file system interface
- Added `nfs` module with NFS protocol definitions
- Added documentation for real file system support
- Added example usage of real file system support
- Added tests for real file system functionality

### Changed
- Updated README.md with information about real file system support
- Updated Cargo.toml files to include new dependencies
- Refactored file manager implementation to support both in-memory and real file systems
- Improved error handling for file system operations

### Fixed
- Fixed issues with file handle management
- Fixed issues with attribute handling for real files
- Fixed issues with directory operations

## [0.1.0] - 2025-09-14

### Added
- Initial release of Bold NFS server
- Support for in-memory file system via YAML configuration
- Basic NFS v4.0 operations (GETATTR, LOOKUP, ACCESS, READ, WRITE, CREATE, REMOVE, MKDIR, etc.)
- Client management and file handle management
- Basic testing framework