# Dependency Direction Static Asset

Task: T1.1

Correlation field: required.
Evidence class: source-shape.
Close-not-claimed: static dependency inspection does not prove runtime behavior.
Rerun condition: rerun when Cargo manifests, SDK manifests, workspace membership, or boundary Canonical changes.

## allowed package dependency directions

```text
core <- drivers
core <- entrypoints
drivers <- entrypoints
regulated -> core identity references only
sdk is independent Signaling-only public API surface
```

## forbidden package dependency directions

- core to drivers
- core to entrypoints
- core to regulated
- drivers to entrypoints
- drivers to regulated
- entrypoints to regulated
- sdk to drivers internals
- sdk to regulated internals
