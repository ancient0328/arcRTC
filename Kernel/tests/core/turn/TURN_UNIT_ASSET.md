# TURN Unit Asset

Task: T2.4

Correlation field: required.
Evidence class: test.
Close-not-claimed: TURN unit evidence does not prove UDP/TCP socket behavior or public traversal.
Rerun condition: rerun when TURN contract or lifecycle Canonical changes.

This unit-test asset is owned by core and covers TURN command admission, allocation lifecycle, permission lifecycle, channel binding, refresh, and teardown semantics.

forbidden driver ownership: wire decode and HMAC backend are drivers; TURN lifecycle decision is core.
