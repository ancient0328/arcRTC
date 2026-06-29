# Signaling Unit Asset

Task: T2.2

Correlation field: required.
Evidence class: test.
Close-not-claimed: Signaling unit evidence does not prove WebSocket, HTTP, SDK, or live server behavior.
Rerun condition: rerun when Signaling contract or state machine Canonical changes.

This unit-test asset is owned by core and covers Signaling command admission, state transition classification, and forbidden media-plane behavior.

forbidden driver ownership: driver may parse external frames, but core owns Signaling accept/reject semantics.
