# Operation Runtime Quality Unit Asset

Task: T2.6

Correlation field: required.
Evidence class: test.
Close-not-claimed: operation/runtime/quality unit evidence does not prove live operation.
Rerun condition: rerun when cross-plane, runtime, state, recovery, quality, quota, or backpressure Canonical changes.

This unit-test asset is owned by core and covers cross-plane binding, operational lifecycle, runtime ports, time normalization, state/recovery, quality metrics, admission, and backpressure semantics.

forbidden driver ownership: runtime resources and external observations are converted through ports before core decision.
