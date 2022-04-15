# Changelog

## Rosrust Unreleased
### Changed
- `rosrust::client` no longer returns an error if targeted service doesn't exist yet
- `rosrust::client` attempts to find new service sources when existing ones shut down
- `rosrust::wait_for_service` checks for connection and headers when waiting
