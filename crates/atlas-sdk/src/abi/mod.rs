pub mod alloc;
pub mod codec;

pub use alloc::{atlas_alloc_impl, atlas_dealloc_impl};
pub use codec::{decode_host_result, decode_input, encode_result, pack_ptr_len, AtlasResult};
