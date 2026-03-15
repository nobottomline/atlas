//! `export_source!` — the primary ergonomic entry point for source authors.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use atlas_sdk::prelude::*;
//!
//! pub struct MySource;
//!
//! impl Default for MySource {
//!     fn default() -> Self { Self }
//! }
//!
//! impl Source for MySource {
//!     fn get_info(&self) -> Result<SourceInfo, SourceError> { ... }
//!     // ... implement remaining required methods
//! }
//!
//! export_source!(MySource);
//! ```
//!
//! The macro generates:
//! - `atlas_alloc` / `atlas_dealloc` exports for host memory management.
//! - One `extern "C"` export per source contract method.
//! - A singleton instance using `OnceLock` (safe on single-threaded WASM).
//!
//! The source type must implement `Default` for initialization.

#[macro_export]
macro_rules! export_source {
    ($source_type:ty) => {
        // ── Memory management ────────────────────────────────────────────────

        #[no_mangle]
        pub unsafe extern "C" fn atlas_alloc(size: u32) -> *mut u8 {
            ::atlas_sdk::abi::atlas_alloc_impl(size)
        }

        #[no_mangle]
        pub unsafe extern "C" fn atlas_dealloc(ptr: *mut u8, size: u32) {
            ::atlas_sdk::abi::atlas_dealloc_impl(ptr, size);
        }

        // ── Singleton source instance ────────────────────────────────────────

        fn _atlas_source_instance() -> &'static $source_type {
            static INSTANCE: ::std::sync::OnceLock<$source_type> =
                ::std::sync::OnceLock::new();
            INSTANCE.get_or_init(<$source_type>::default)
        }

        // ── Required exports ─────────────────────────────────────────────────

        #[no_mangle]
        pub unsafe extern "C" fn atlas_get_info() -> i64 {
            let result = _atlas_source_instance().get_info();
            ::atlas_sdk::abi::encode_result(result)
        }

        #[no_mangle]
        pub unsafe extern "C" fn atlas_search(ptr: *const u8, len: u32) -> i64 {
            let result = ::atlas_sdk::abi::decode_input::<
                ::atlas_spec::types::search::SearchQuery,
            >(ptr, len)
            .and_then(|q| _atlas_source_instance().search(q));
            ::atlas_sdk::abi::encode_result(result)
        }

        #[no_mangle]
        pub unsafe extern "C" fn atlas_get_manga_details(ptr: *const u8, len: u32) -> i64 {
            let result = ::atlas_sdk::abi::decode_input::<String>(ptr, len)
                .and_then(|id| _atlas_source_instance().get_manga_details(&id));
            ::atlas_sdk::abi::encode_result(result)
        }

        #[no_mangle]
        pub unsafe extern "C" fn atlas_get_chapters(ptr: *const u8, len: u32) -> i64 {
            let result = ::atlas_sdk::abi::decode_input::<String>(ptr, len)
                .and_then(|id| _atlas_source_instance().get_chapters(&id));
            ::atlas_sdk::abi::encode_result(result)
        }

        #[no_mangle]
        pub unsafe extern "C" fn atlas_get_pages(ptr: *const u8, len: u32) -> i64 {
            let result = ::atlas_sdk::abi::decode_input::<String>(ptr, len)
                .and_then(|id| _atlas_source_instance().get_pages(&id));
            ::atlas_sdk::abi::encode_result(result)
        }

        // ── Optional exports ─────────────────────────────────────────────────

        #[no_mangle]
        pub unsafe extern "C" fn atlas_get_filters() -> i64 {
            let result = _atlas_source_instance().get_filters();
            ::atlas_sdk::abi::encode_result(result)
        }

        #[no_mangle]
        pub unsafe extern "C" fn atlas_get_latest(ptr: *const u8, len: u32) -> i64 {
            let result = ::atlas_sdk::abi::decode_input::<u32>(ptr, len)
                .and_then(|page| _atlas_source_instance().get_latest(page));
            ::atlas_sdk::abi::encode_result(result)
        }

        #[no_mangle]
        pub unsafe extern "C" fn atlas_get_popular(ptr: *const u8, len: u32) -> i64 {
            let result = ::atlas_sdk::abi::decode_input::<u32>(ptr, len)
                .and_then(|page| _atlas_source_instance().get_popular(page));
            ::atlas_sdk::abi::encode_result(result)
        }

        #[no_mangle]
        pub unsafe extern "C" fn atlas_get_preferences_schema() -> i64 {
            let result = _atlas_source_instance().get_preferences_schema();
            ::atlas_sdk::abi::encode_result(result)
        }
    };
}
