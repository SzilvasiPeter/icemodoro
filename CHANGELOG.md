# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/SzilvasiPeter/icemodoro/compare/v0.1.0...v0.1.1) - 2025-10-14

### Other

- Move the release-plz action to the CD pipeline
- Push only on version tag
- release v0.1.0

## [0.1.0](https://github.com/SzilvasiPeter/icemodoro/releases/tag/v0.1.0) - 2025-10-14

### Other

- Add the cargo registry token environment variable
- Create release-plz.yml
- Add ctrl modifier when navigating tab forward, remove the unnecessary
- Remove x86_64-unknown-linux-musl target
- Add x86_64-unknown-linux-musl target
- Archive the binaries
- Fix the mv source name, and rename after strip
- Move and rename the binary suffix with target
- Append the target to the binary name using env var
- Add token env var to avoid 402 payment required error for macos build
- Add the target name to the file to avoid linux and macos name colliding
- The problem is the apple target build upload, not the token permission
- Disable build for macos
- Use the msvcrt target since ucrt only support debian 13, enable the
- Use ucrt mingw complier
- Enable the build job
- Add back tag push
- Stupid vercel (402 Payment Required) error...
- Trigger on push
- Disable build job to debug the release job
- Disable cargo-strip because apple target failed to quickinstall
- Use macos-latest os for apple target
- Use ubuntu for each target
- Add github token when creating release
- Add tag_name back
- Extend task updated match on finish to presist spent time
- Move the build and release to cd pipeline
- Replace tiny skia with wgpu due to buggy theme color switching, remove
- Use smaller mingw remove the --release option in cargo stip
- Install mingw for windows build
- Create release pipeline for testing
- Create rust.yml
- Add run cargo audit ci step
- Remove cargo audit step
- Add cargo audit action
- Use only a single target when setting up rust
- Add desktop targets to the setup rust action
- Fix the CD setup rust action
- Remove the targets option
- Update the actions version to the current latest
- Remove the long audit github actions, since no cache is available
- Add token for cargo audit
- The `setup-rust@v1` action caches by default
- Replace the outdated actions
- Remove the cargo bin cache
- Move back the "Install Rust" steps before the cargo bin cache
- Add CD pipeline for all desktop platforms
- Removeing the `strip` parameter since it did not reduce the binary size
- Add documentation for smaller binary release
- Reduce the release binary size
- Move "Install Rust" action after the cache actions
- Cache the build folder to speed up the cargo clippy
- Delete clippy as components since the default profile already contains
- Cache cargo registry git folder and install clippy
- Cache before installation step
- Cache the cargo audit
- Create ci pipeline
- Create makefile for linters
- Add error message on export or import failure
- Limit the number of active tasks to ten
- Remove the ALT modifier check
- Fix the arithmetic overflow warnings
- Remove unnecessary code comment
- Fix pedantic linter issues
- Remove the comment since the problem is related to the X11 window
- Add clippy fixes
- Add BUG tag to the comment
- Stop navigation the tab forward tab if the ALT key is pressed.
- Add advanced shaping strategy for the icons
- Change the edit symbol
- Update comment
- Add documentation for whole project
- Update README.md
- Initial pomofocus.io (pomodoro and todo) app reimplementation using iced
- Update README.md
- Initial commit
