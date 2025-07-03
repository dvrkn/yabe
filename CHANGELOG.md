# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Enhanced exclude pattern matching to support substring matching, path component matching, and improved glob pattern support
- Exclude patterns now work consistently in both sort-only and regular diff modes
- Fixed issue where exclude patterns like `--exclude 'configgen'` had no effect

## [0.1.8] - 2024-12-XX

### Added
- Exclude patterns support for sort-only mode
- Ability to skip specific files during sorting using `--exclude` flag
- Configuration file support for exclude patterns

### Changed
- Improved CLI documentation with exclude pattern examples

## [0.1.7] - 2024-12-XX

### Added
- Sort-only mode feature (`--sort-only` flag)
- Ability to sort YAML files without diff computation
- Support for recursive glob patterns (`**/*.yaml`)
- In-place sorting capability with `-i` flag

### Changed
- Enhanced CLI with new sorting options
- Updated README with sort-only examples

## [0.1.6] - 2024-12-XX

### Added
- Regex path parser feature
- Enhanced path pattern matching

### Changed
- Improved dependency versions
- Updated package lock file

## [0.1.5] - 2024-12-XX

### Added
- Initial crates.io publishing support
- CI/CD workflow improvements

### Changed
- Dependency upgrades
- Code cleanup and optimization

## [0.1.4] - 2024-12-XX

### Fixed
- Package lock file updates
- Various bug fixes and improvements

---

## Release Notes

This project follows semantic versioning. Each release includes:
- **Added** for new features
- **Changed** for changes in existing functionality
- **Deprecated** for soon-to-be removed features
- **Removed** for now removed features
- **Fixed** for any bug fixes
- **Security** for security vulnerabilities