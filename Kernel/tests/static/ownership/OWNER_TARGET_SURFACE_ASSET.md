# Owner And Target Surface Static Asset

Task: T1.2

Correlation field: required.
Evidence class: source-shape.
Close-not-claimed: metadata inspection does not prove behavior.
Rerun condition: rerun when package manifests or Roadmap target surfaces change.

## owner_layer

Every Rust package manifest must declare `owner_layer`. SDK manifests must declare equivalent arcrtc owner metadata.

## package_role

Every package must declare the role of the production or test surface.

## production/test asset separation

Production packages must not depend on `tests` or `integration-tests`. Test packages may inspect production packages.

## source-set scan scope

Source-shape tests must inspect the effective package source set, including source shards and platform SDK source files, rather than only a root `lib.rs`.

## semantic-boundary

Core is a semantic modular monolith. Static ownership evidence must reject semantic owner transfer, dependency inversion, source shard scope drift, and hidden driver/entrypoint ownership.

## source-size review signal

Source file line count is a review signal only. It must not become a hard pass/fail gate above semantic ownership and dependency direction.

## core peer dependency closure

Core package dependencies must match the declared semantic peer/foundation dependency matrix.

## local lint suppression policy

Production source must not use local `#[allow(clippy::too_many_arguments)]`; high-arity semantic evidence records are governed by workspace Clippy policy.
