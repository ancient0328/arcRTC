# Fixture/Scenario Register

Task: T0.3

Correlation field: required in every report using fixture or scenario data.
Evidence class: test or runtime-in-test only.
Close-not-claimed: fixture use never proves live production behavior.
Rerun condition: rerun when fixture provenance, scenario identity, mutation policy, or golden admission changes.

## fixture provenance

Fixtures are synthetic v0.2 test data. v0.1 material is historical input only unless explicitly re-admitted by Canonical.

## scenario identity

Every scenario must declare scenario ID, target Roadmap task, covered Canonical, and expected result class.

## mutation policy

Mutation is allowed only through explicit test setup. Golden fixture mutation requires a new scenario ID.

## golden-data admission

Golden data is admitted only when canonical serialization format/version and digest relation are declared.

## report linkage fields

- scenario ID
- fixture provenance
- mutation class
- golden-data admission state
- evidence report path
