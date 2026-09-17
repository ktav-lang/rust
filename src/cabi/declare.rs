/// Expand the full exported C ABI surface into the CALLING crate: six
/// document functions, `ktav_free`, `ktav_version` and
/// `ktav_abi_version`. Symbols defined here are exported by
/// construction — a dependency rlib's symbols would not survive into a
/// downstream cdylib.
///
/// Invoke it exactly once, at the top level of the binding's cdylib:
/// `ktav::declare_cabi!();`
///
/// Ownership contract: every returned buffer is freed by the caller
/// through `ktav_free(ptr, len)` exactly once, with the length it was
/// returned with. Return code is `0` on success and `1` on error, with
/// `out_err` holding the JSON error envelope on error.
///
/// The invoking crate must be edition ≤ 2021: edition 2024 renamed the
/// attribute to `#[unsafe(no_mangle)]`.
#[macro_export]
macro_rules! declare_cabi {
    () => {
        #[inline]
        unsafe fn ktav_cabi_emit(
            buf: ::std::vec::Vec<u8>,
            out_buf: *mut *mut u8,
            out_len: *mut usize,
        ) {
            let mut boxed = buf.into_boxed_slice();
            let len = boxed.len();
            let ptr = boxed.as_mut_ptr();
            // SAFETY: the boxed slice is intentionally leaked here; the
            // caller frees it via `ktav_free` with the returned length.
            unsafe {
                ::std::mem::forget(boxed);
            }
            *out_buf = ptr;
            *out_len = len;
        }

        unsafe fn ktav_cabi_emit_err(
            msg: ::std::string::String,
            out_err: *mut *mut ::core::ffi::c_char,
            out_err_len: *mut usize,
        ) {
            let bytes = msg.into_bytes();
            let mut boxed = bytes.into_boxed_slice();
            let len = boxed.len();
            let ptr = boxed.as_mut_ptr() as *mut ::core::ffi::c_char;
            // SAFETY: same leak-for-ownership contract as `ktav_cabi_emit`.
            unsafe {
                ::std::mem::forget(boxed);
            }
            *out_err = ptr;
            *out_err_len = len;
        }

        /// Parse Ktav source text to a JSON wire document.
        ///
        /// # Safety
        /// `src` must point to `src_len` valid bytes; the output pointers
        /// must be valid for writes; returned buffers are freed via
        /// `ktav_free`.
        #[no_mangle]
        pub unsafe extern "C" fn ktav_loads(
            src: *const u8,
            src_len: usize,
            out_buf: *mut *mut u8,
            out_len: *mut usize,
            out_err: *mut *mut ::core::ffi::c_char,
            out_err_len: *mut usize,
        ) -> ::core::ffi::c_int {
            *out_buf = ::core::ptr::null_mut();
            *out_len = 0;
            *out_err = ::core::ptr::null_mut();
            *out_err_len = 0;
            // SAFETY: `src`/`src_len` is this exported function's own
            // documented contract.
            let src = unsafe { ::core::slice::from_raw_parts(src, src_len) };
            match $crate::cabi::loads(src) {
                Ok(bytes) => {
                    ktav_cabi_emit(bytes, out_buf, out_len);
                    0
                }
                Err(msg) => {
                    ktav_cabi_emit_err(msg, out_err, out_err_len);
                    1
                }
            }
        }

        /// Parse Ktav source text (strict scalars) to a JSON wire document.
        ///
        /// # Safety
        /// `src` must point to `src_len` valid bytes; the output pointers
        /// must be valid for writes; returned buffers are freed via
        /// `ktav_free`.
        #[no_mangle]
        pub unsafe extern "C" fn ktav_loads_strict(
            src: *const u8,
            src_len: usize,
            out_buf: *mut *mut u8,
            out_len: *mut usize,
            out_err: *mut *mut ::core::ffi::c_char,
            out_err_len: *mut usize,
        ) -> ::core::ffi::c_int {
            *out_buf = ::core::ptr::null_mut();
            *out_len = 0;
            *out_err = ::core::ptr::null_mut();
            *out_err_len = 0;
            // SAFETY: `src`/`src_len` is this exported function's own
            // documented contract.
            let src = unsafe { ::core::slice::from_raw_parts(src, src_len) };
            match $crate::cabi::loads_strict(src) {
                Ok(bytes) => {
                    ktav_cabi_emit(bytes, out_buf, out_len);
                    0
                }
                Err(msg) => {
                    ktav_cabi_emit_err(msg, out_err, out_err_len);
                    1
                }
            }
        }

        /// Render a JSON wire document to Ktav source text.
        ///
        /// # Safety
        /// `src` must point to `src_len` valid bytes; the output pointers
        /// must be valid for writes; returned buffers are freed via
        /// `ktav_free`.
        #[no_mangle]
        pub unsafe extern "C" fn ktav_dumps(
            src: *const u8,
            src_len: usize,
            out_buf: *mut *mut u8,
            out_len: *mut usize,
            out_err: *mut *mut ::core::ffi::c_char,
            out_err_len: *mut usize,
        ) -> ::core::ffi::c_int {
            *out_buf = ::core::ptr::null_mut();
            *out_len = 0;
            *out_err = ::core::ptr::null_mut();
            *out_err_len = 0;
            // SAFETY: `src`/`src_len` is this exported function's own
            // documented contract.
            let src = unsafe { ::core::slice::from_raw_parts(src, src_len) };
            match $crate::cabi::dumps(src) {
                Ok(bytes) => {
                    ktav_cabi_emit(bytes, out_buf, out_len);
                    0
                }
                Err(msg) => {
                    ktav_cabi_emit_err(msg, out_err, out_err_len);
                    1
                }
            }
        }

        /// Render a JSON wire document to Ktav source, coercing every
        /// scalar to a literal string.
        ///
        /// # Safety
        /// `src` must point to `src_len` valid bytes; the output pointers
        /// must be valid for writes; returned buffers are freed via
        /// `ktav_free`.
        #[no_mangle]
        pub unsafe extern "C" fn ktav_dumps_force_strings(
            src: *const u8,
            src_len: usize,
            out_buf: *mut *mut u8,
            out_len: *mut usize,
            out_err: *mut *mut ::core::ffi::c_char,
            out_err_len: *mut usize,
        ) -> ::core::ffi::c_int {
            *out_buf = ::core::ptr::null_mut();
            *out_len = 0;
            *out_err = ::core::ptr::null_mut();
            *out_err_len = 0;
            // SAFETY: `src`/`src_len` is this exported function's own
            // documented contract.
            let src = unsafe { ::core::slice::from_raw_parts(src, src_len) };
            match $crate::cabi::dumps_force_strings(src) {
                Ok(bytes) => {
                    ktav_cabi_emit(bytes, out_buf, out_len);
                    0
                }
                Err(msg) => {
                    ktav_cabi_emit_err(msg, out_err, out_err_len);
                    1
                }
            }
        }

        /// Render a JSON wire document to canonical Ktav text.
        ///
        /// # Safety
        /// `src` must point to `src_len` valid bytes; the output pointers
        /// must be valid for writes; returned buffers are freed via
        /// `ktav_free`.
        #[no_mangle]
        pub unsafe extern "C" fn ktav_emit_canonical(
            src: *const u8,
            src_len: usize,
            out_buf: *mut *mut u8,
            out_len: *mut usize,
            out_err: *mut *mut ::core::ffi::c_char,
            out_err_len: *mut usize,
        ) -> ::core::ffi::c_int {
            *out_buf = ::core::ptr::null_mut();
            *out_len = 0;
            *out_err = ::core::ptr::null_mut();
            *out_err_len = 0;
            // SAFETY: `src`/`src_len` is this exported function's own
            // documented contract.
            let src = unsafe { ::core::slice::from_raw_parts(src, src_len) };
            match $crate::cabi::emit_canonical(src) {
                Ok(bytes) => {
                    ktav_cabi_emit(bytes, out_buf, out_len);
                    0
                }
                Err(msg) => {
                    ktav_cabi_emit_err(msg, out_err, out_err_len);
                    1
                }
            }
        }

        /// Format Ktav source text (input is Ktav source, NOT a JSON wire
        /// value); preserves comments and blank-line grouping.
        ///
        /// # Safety
        /// `src` must point to `src_len` valid bytes; the output pointers
        /// must be valid for writes; returned buffers are freed via
        /// `ktav_free`.
        #[no_mangle]
        pub unsafe extern "C" fn ktav_format(
            src: *const u8,
            src_len: usize,
            out_buf: *mut *mut u8,
            out_len: *mut usize,
            out_err: *mut *mut ::core::ffi::c_char,
            out_err_len: *mut usize,
        ) -> ::core::ffi::c_int {
            *out_buf = ::core::ptr::null_mut();
            *out_len = 0;
            *out_err = ::core::ptr::null_mut();
            *out_err_len = 0;
            // SAFETY: `src`/`src_len` is this exported function's own
            // documented contract.
            let src = unsafe { ::core::slice::from_raw_parts(src, src_len) };
            match $crate::cabi::format(src) {
                Ok(bytes) => {
                    ktav_cabi_emit(bytes, out_buf, out_len);
                    0
                }
                Err(msg) => {
                    ktav_cabi_emit_err(msg, out_err, out_err_len);
                    1
                }
            }
        }

        /// Canonical form from Ktav **source text** (input is Ktav source,
        /// NOT a JSON wire value). Unlike `ktav_emit_canonical`, nothing
        /// passes through a host value, so scalar spellings such as `1.0`
        /// and `1e9` survive byte-exactly in languages whose number type
        /// cannot carry the Integer/Float distinction.
        ///
        /// # Safety
        /// `src` must point to `src_len` valid bytes; the output pointers
        /// must be valid for writes; returned buffers are freed via
        /// `ktav_free`.
        #[no_mangle]
        pub unsafe extern "C" fn ktav_canonical_from_source(
            src: *const u8,
            src_len: usize,
            out_buf: *mut *mut u8,
            out_len: *mut usize,
            out_err: *mut *mut ::core::ffi::c_char,
            out_err_len: *mut usize,
        ) -> ::core::ffi::c_int {
            *out_buf = ::core::ptr::null_mut();
            *out_len = 0;
            *out_err = ::core::ptr::null_mut();
            *out_err_len = 0;
            // SAFETY: `src`/`src_len` is this exported function's own
            // documented contract.
            let src = unsafe { ::core::slice::from_raw_parts(src, src_len) };
            match $crate::cabi::canonical_from_source(src) {
                Ok(bytes) => {
                    ktav_cabi_emit(bytes, out_buf, out_len);
                    0
                }
                Err(msg) => {
                    ktav_cabi_emit_err(msg, out_err, out_err_len);
                    1
                }
            }
        }

        /// Free a buffer returned by any `ktav_*` output function.
        /// Null/zero is a no-op.
        ///
        /// # Safety
        /// Must be called exactly once per returned buffer, with the same
        /// length it was returned with.
        #[no_mangle]
        pub unsafe extern "C" fn ktav_free(ptr: *mut u8, len: usize) {
            if ptr.is_null() || len == 0 {
                return;
            }
            // SAFETY: the pointer was produced by `ktav_cabi_emit` /
            // `ktav_cabi_emit_err` as a leaked boxed slice of exactly
            // `len` bytes, and this runs once per buffer.
            let _ = unsafe {
                ::std::boxed::Box::from_raw(::core::ptr::slice_from_raw_parts_mut(ptr, len))
            };
        }

        /// NUL-terminated static version string of the `ktav` crate, for
        /// sanity checks that the loader picked up the right file.
        #[no_mangle]
        pub extern "C" fn ktav_version() -> *const ::core::ffi::c_char {
            $crate::cabi::VERSION_BYTES.as_ptr() as *const ::core::ffi::c_char
        }

        /// Version of the exported C ABI *shape*. Compare against the
        /// `ktav::cabi::ABI_VERSION` the host was compiled against and
        /// refuse to load on mismatch.
        #[no_mangle]
        pub extern "C" fn ktav_abi_version() -> u32 {
            $crate::cabi::ABI_VERSION
        }
    };
}
