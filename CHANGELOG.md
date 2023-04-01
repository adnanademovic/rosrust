# Changelog

## Rosrust Msg Unreleased
### Added
- Find messages in `ROS_PACKAGE_PATH`

## Rosrust Unreleased
### Added
- Log with `once`, `throttle` and `throttle_identical`
- `SubscriptionHandler` interface to easily manage arbitrary subscriptions without fighting the borrow checker
- Find messages in `ROS_PACKAGE_PATH`
- `wall_time::now` for easier retrieval of actual time when running recordings
- Convenience conversions between ROS time structures and standard time structures

### Fixed
- Fix deeply nested relative field paths in dynamic messages

## Rosrust Msg 0.1.6 (2022-04-15)
### Changed
- Updated dependencies

## Rosrust 0.9.10 (2022-04-15)
### Changed
- `rosrust::client` no longer returns an error if targeted service doesn't exist yet
- `rosrust::client` attempts to find new service sources when existing ones shut down
- `rosrust::wait_for_service` checks for connection and headers when waiting
