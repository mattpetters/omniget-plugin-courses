# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Added
- Exposed authenticated Udemy curriculum data for localhost/API automation.

### Changed
- Added course IDs to Udemy completion events so asynchronous API clients can correlate terminal success and failure.

<!-- git-changelog-on-commit: 714974aa916fc21e1c1feb8454985735fad45c9f -->
### Changed
- Merged upstream 1.0.5 (ABI v3, fresh Udemy media URLs, and supplementary resources) while preserving paginated enrollment discovery.
- Integrated full-course Udemy Widevine downloads with the shared omniget DRM toolchain, fresh UA-bound license tokens, cancellation, aggregate progress, and non-empty output validation.
- DRM lecture failures now fail the course download explicitly instead of being skipped and reported as a successful completion.
