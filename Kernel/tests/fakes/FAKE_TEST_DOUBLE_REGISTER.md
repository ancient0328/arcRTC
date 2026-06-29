# Fake/Test-Double Register

Task: T0.2

Correlation field: required in every report using a fake.
Evidence class: test or runtime-in-test only.
Close-not-claimed: fake use never proves driver runtime, live network, production readiness, or benchmark capacity.
Rerun condition: rerun when fake behavior, port contract, or covered Canonical changes.

## fake/test-double ownership

Test doubles are owned by `tests/fakes`. They may implement core-owned ports for deterministic tests.

## allowed substitution surfaces

- deterministic clock/randomness port behavior
- persistence port in-memory observation
- network driver observation without socket I/O
- observability sink capture
- security verifier deterministic outcome

## forbidden semantic substitution surfaces

- must not own core semantics
- must not replace Signaling/SFU/TURN decision rules
- must not emulate production network success as live proof
- must not hide driver conversion failure

## report disclosure fields

- fake class
- substituted port or driver boundary
- deterministic input
- forbidden substitution review
- non-adoption state when fake output is diagnostic only
