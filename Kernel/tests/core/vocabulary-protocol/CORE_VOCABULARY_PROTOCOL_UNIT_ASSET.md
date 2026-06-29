# Core Vocabulary And Protocol Unit Asset

Task: T2.1

Correlation field: required.
Evidence class: test.
Close-not-claimed: unit-test asset does not prove runtime integration.
Rerun condition: rerun when identity, command, reason, protocol, or canonical serialization changes.

This unit-test asset is owned by core and covers identity/reference validation, command/decision/result shape, idempotency/replay/correlation shape, reason catalog closure, and protocol encoding metadata.

forbidden driver ownership: external request, response, wire, and SDK payload types must remain outside core.
