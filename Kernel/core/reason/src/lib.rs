// 公開 API を crate root に保ったまま、reason model と closed catalog を物理分割します。
// REASON_DEFINITIONS の型と意味論は維持し、catalog 本体だけを専用 shard に分離します。
include!("lib_parts/reason_model.rs");
include!("lib_parts/reason_definitions.rs");
include!("lib_parts/reason_adversarial_registry.rs");
