# Server Composition Asset

Task: T4.1

Correlation field: required.
Evidence class: test.
Close-not-claimed: entrypoint composition test asset does not prove live server readiness.
Rerun condition: rerun when entrypoint composition root, Signaling, SFU, or TURN contract changes.

Signaling, SFU, and TURN server entrypoints compose core and selected drivers.

must not own domain semantics: entrypoints may wire dependencies but must not own Signaling/SFU/TURN decision rules.
