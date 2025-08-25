# IC Call Utils

A collection of utilities for making calls on the Internet Computer.

## Components

### ic-call-chaos
A library for testing and simulating call failures on the Internet Computer.

### ic-call-retry
A library for retrying calls on the Internet Computer with various retry strategies.

### ic-safe-upgrades
A library for safely upgrading canisters on the Internet Computer.

## Usage

Add the desired components to your `Cargo.toml`:

```toml
[dependencies]
ic-call-chaos = { path = "path/to/ic_call_utils/call_chaos" }
ic-call-retry = { path = "path/to/ic_call_utils/retry" }
ic-safe-upgrades = { path = "path/to/ic_call_utils/safe_upgrade" }
```