# Changelog

## Rosrust Unreleased
### Added
- Automatic caching of parameters

## Rosrust Msg 0.1.7 (2023-04-01)
### Added
- Find messages in `ROS_PACKAGE_PATH`

## Rosrust 0.9.11 (2023-04-01)
### Added
- Log with `once`, `throttle` and `throttle_identical`
- `SubscriptionHandler` interface to easily manage arbitrary subscriptions without fighting the borrow checker
- Find messages in `ROS_PACKAGE_PATH`
- `wall_time::now` for easier retrieval of actual time when running recordings
- Convenience conversions between ROS time structures and standard time structures
- `wait_for_subscribers` in publisher
### Fixed
- Clean up node on shutdown (by either using `spin()`, `shutdown()`, or `is_ok()` until it's false)
- Fix deeply nested relative field paths in dynamic messages

## Rosrust Msg 0.1.6 (2022-04-15)
### Changed
- Updated dependencies

## Rosrust 0.9.10 (2022-04-15)
### Changed
- `rosrust::client` no longer returns an error if targeted service doesn't exist yet
- `rosrust::client` attempts to find new service sources when existing ones shut down
- `rosrust::wait_for_service` checks for connection and headers when waiting
