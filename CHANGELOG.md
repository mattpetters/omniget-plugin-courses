# Changelog

All notable changes to this project will be documented in this file.

## 1.1.0-mpfork-0.1

- Merge upstream 1.1.0, including Rocketseat, Meta-Analysis Academy, Hotmart OIDC, section selection, and detailed lecture outcomes.
- Preserve managed Udemy session recovery, persisted download settings, paginated enrollment, Widevine downloads, and course-ID completion events.
- Return both raw curriculum chapters for existing automation and section summaries for the updated app.
- Fail incomplete courses explicitly while retaining successfully downloaded lectures for retries.
- Build against the matching customized OmniGet core and Rust 1.97.0.

## Earlier fork changes

<!-- git-changelog-on-commit: 9795691f1eb2f1497ac9b2ee12959e5f00c1dac0 -->
### Changed
- Updated Cargo.lock,Cargo.toml plugin.json,src/commands/udemy_auth.rs src/platforms/udemy/auth.rs,src/settings_reader.rs based on the staged diff so the commit records the current implementation changes.

### Added
- Exposed authenticated Udemy curriculum data for localhost/API automation.

### Changed
- Added course IDs to Udemy completion events so asynchronous API clients can correlate terminal success and failure.
- Courses now honor the app's persisted download settings instead of silently falling back to plugin defaults.
- Udemy sessions now restore automatically from OmniGet's managed cookie account, keeping the Courses UI and extension cookie workflow in sync after restarts.

<!-- git-changelog-on-commit: 714974aa916fc21e1c1feb8454985735fad45c9f -->
### Changed
- Merged upstream 1.0.5 (ABI v3, fresh Udemy media URLs, and supplementary resources) while preserving paginated enrollment discovery.
- Integrated full-course Udemy Widevine downloads with the shared omniget DRM toolchain, fresh UA-bound license tokens, cancellation, aggregate progress, and non-empty output validation.
- DRM lecture failures now fail the course download explicitly instead of being skipped and reported as a successful completion.
