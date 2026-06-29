# SFU Unit Asset

Task: T2.3

Correlation field: required.
Evidence class: test.
Close-not-claimed: SFU unit evidence does not prove live media forwarding or public network behavior.
Rerun condition: rerun when SFU contract, state machine, packet semantic view, transform, negotiation, congestion, pacing, or retransmission Canonical changes.

This unit-test asset is owned by core and covers SFU routing decisions, borrowed packet semantic view, transform intent, media negotiation, congestion, pacing, and retransmission semantics.

forbidden driver ownership: drivers retain buffer storage; core owns packet semantic view and routing decision.
