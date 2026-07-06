// 公開 API と test root を維持したまま、責務ごとの source shard に物理分割します。
// 各 shard は同じ Rust module scope に include され、意味論上の owner はこの crate/file root に残します。
include!("lib_parts/part_001.rs");
include!("lib_parts/part_002.rs");
include!("lib_parts/part_003.rs");
include!("lib_parts/part_004.rs");
