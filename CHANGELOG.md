# Changelog

## Rosrust Msg 0.1.6 (2022-04-15)
### Changed
- Updated dependencies

## Rosrust 0.9.10 (2022-04-15)
### Changed
- `rosrust::client` no longer returns an error if targeted service doesn't exist yet
- `rosrust::client` attempts to find new service sources when existing ones shut down
- `rosrust::wait_for_service` checks for connection and headers when waiting
