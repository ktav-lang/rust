//! dlopen test for the `cabi` design: this is the test that proves the
//! central claim — the `#[no_mangle]` symbols from `ktav::declare_cabi!`
//! expand into the fixture cdylib (`tests/cabi-fixture/`) and resolve
//! from a real dlopen via `libloading`. CI runs `--all-features`, so it
//! executes on every push; a plain `cargo test` skips it silently.
#![cfg(feature = "cabi")]

use std::ffi::{c_char, c_int, CStr};
use std::path::PathBuf;
use std::process::Command;

/// Signature shared by the six document functions.
type DocFn = unsafe extern "C" fn(
    *const u8,
    usize,
    *mut *mut u8,
    *mut usize,
    *mut *mut c_char,
    *mut usize,
) -> c_int;
type FreeFn = unsafe extern "C" fn(*mut u8, usize);
type VersionFn = unsafe extern "C" fn() -> *const c_char;
type AbiVersionFn = unsafe extern "C" fn() -> u32;

/// Build the fixture cdylib into the gitignored `target/` tree and
/// return the path of the produced shared library.
fn build_fixture() -> PathBuf {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("cabi-fixture")
        .join("Cargo.toml");
    let target = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("cabi-fixture");
    let output = Command::new(&cargo)
        .args(["build", "--manifest-path"])
        .arg(&manifest)
        .arg("--target-dir")
        .arg(&target)
        .output()
        .expect("failed to spawn cargo build for the cabi fixture");
    assert!(
        output.status.success(),
        "cabi fixture build failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let path = target.join("debug").join(format!(
        "{}ktav_cabi_fixture{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    ));
    assert!(
        path.exists(),
        "fixture artifact not found at {}",
        path.display()
    );
    path
}

/// Copy an owned output buffer out and free it through THE LOADED
/// LIBRARY's `ktav_free` — never any other allocator.
unsafe fn take_out(lib: &libloading::Library, ptr: *mut u8, len: usize) -> Vec<u8> {
    let free: libloading::Symbol<FreeFn> = unsafe { lib.get(b"ktav_free").unwrap() };
    if ptr.is_null() || len == 0 {
        free(ptr, len);
        return Vec::new();
    }
    let out = std::slice::from_raw_parts(ptr, len).to_vec();
    free(ptr, len);
    out
}

/// Copy an owned length-delimited error string out and free it through
/// THE LOADED LIBRARY's `ktav_free`.
unsafe fn take_err(lib: &libloading::Library, ptr: *mut c_char, len: usize) -> String {
    let free: libloading::Symbol<FreeFn> = unsafe { lib.get(b"ktav_free").unwrap() };
    if ptr.is_null() || len == 0 {
        free(ptr as *mut u8, len);
        return String::new();
    }
    // The error out-param is a `len`-byte slice (no interior NUL), so
    // copy by length; `ktav_free` still releases it.
    let out = std::slice::from_raw_parts(ptr as *const u8, len).to_vec();
    free(ptr as *mut u8, len);
    String::from_utf8(out).expect("error envelope is not UTF-8")
}

/// Call a named document function and take ownership of its outputs.
fn call(lib: &libloading::Library, name: &[u8], src: &[u8]) -> (c_int, Vec<u8>, String) {
    let f: libloading::Symbol<DocFn> = unsafe { lib.get(name).unwrap() };
    let mut out_ptr: *mut u8 = std::ptr::null_mut();
    let mut out_len: usize = 0;
    let mut err_ptr: *mut c_char = std::ptr::null_mut();
    let mut err_len: usize = 0;
    let rc = unsafe {
        f(
            src.as_ptr(),
            src.len(),
            &mut out_ptr,
            &mut out_len,
            &mut err_ptr,
            &mut err_len,
        )
    };
    let out = unsafe { take_out(lib, out_ptr, out_len) };
    let err = unsafe { take_err(lib, err_ptr, err_len) };
    (rc, out, err)
}

#[test]
fn loads_fixture_and_exercises_the_abi() {
    let path = build_fixture();
    let lib = unsafe { libloading::Library::new(&path).expect("failed to dlopen the fixture") };

    // Resolution itself is the assertion: all ten symbols must come
    // out of the dlopened fixture cdylib. Ten rather than nine since
    // ktav_canonical_from_source joined them — a dry run against the
    // real bindings found that Deno resolves an FFI symbol table
    // eagerly, so one missing symbol poisons the whole library handle
    // rather than failing only the call that needs it.
    let _loads: libloading::Symbol<DocFn> = unsafe { lib.get(b"ktav_loads").unwrap() };
    let _loads_strict: libloading::Symbol<DocFn> =
        unsafe { lib.get(b"ktav_loads_strict").unwrap() };
    let _dumps: libloading::Symbol<DocFn> = unsafe { lib.get(b"ktav_dumps").unwrap() };
    let _dumps_force: libloading::Symbol<DocFn> =
        unsafe { lib.get(b"ktav_dumps_force_strings").unwrap() };
    let _emit_canonical: libloading::Symbol<DocFn> =
        unsafe { lib.get(b"ktav_emit_canonical").unwrap() };
    let _format: libloading::Symbol<DocFn> = unsafe { lib.get(b"ktav_format").unwrap() };
    let canonical_from_source: libloading::Symbol<DocFn> =
        unsafe { lib.get(b"ktav_canonical_from_source").unwrap() };
    let free: libloading::Symbol<FreeFn> = unsafe { lib.get(b"ktav_free").unwrap() };
    let version: libloading::Symbol<VersionFn> = unsafe { lib.get(b"ktav_version").unwrap() };
    let abi_version: libloading::Symbol<AbiVersionFn> =
        unsafe { lib.get(b"ktav_abi_version").unwrap() };
    eprintln!("resolved all ten ktav_cabi symbols from {}", path.display());

    // Silence the unused binding; resolution above is the assertion.
    let _ = &canonical_from_source;

    // Text in, canonical text out, with no host value in between: the
    // float spelling survives, which is the reason this symbol exists.
    let (rc, out, err) = call(
        &lib,
        b"ktav_canonical_from_source",
        b"ratio: 1.0\nbig: 1e9\n",
    );
    assert_eq!(rc, 0, "canonical_from_source failed: {err}");
    assert_eq!(String::from_utf8(out).unwrap(), "ratio: 1.0\nbig: 1e9\n");

    // ABI and crate version oracles.
    assert_eq!(unsafe { abi_version() }, ktav::cabi::ABI_VERSION);
    let ver = unsafe { CStr::from_ptr(version()).to_str().unwrap() };
    assert_eq!(ver, env!("CARGO_PKG_VERSION"));

    // ktav_loads success path: integer stays typed on the wire.
    let (rc, out, err) = call(&lib, b"ktav_loads", b"port: 8080\n");
    assert_eq!(rc, 0, "loads failed: {err}");
    assert!(!out.is_empty());
    let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(parsed["port"], serde_json::json!({"$i": "8080"}));

    // Error path: the nine-key envelope, exact category and line.
    let (rc, out, err) = call(&lib, b"ktav_loads", b"anchor: ok\nport:8080\n");
    assert_eq!(rc, 1);
    assert!(out.is_empty());
    let envelope: serde_json::Value = serde_json::from_str(&err).unwrap();
    let keys: Vec<&str> = envelope
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys.len(), 10, "expected ten envelope keys, got {keys:?}");
    for key in [
        "error",
        "reason",
        "line",
        "line_text",
        "span",
        "path",
        "body",
        "canonical",
        "spec_section",
        "message",
    ] {
        assert!(envelope.get(key).is_some(), "missing envelope key {key}");
    }
    assert_eq!(envelope["error"], "MissingSeparatorSpace");
    assert_eq!(envelope["line"], 2);

    // dumps of invalid JSON reports the Message error.
    let (rc, _out, err) = call(&lib, b"ktav_dumps", b"{");
    assert_eq!(rc, 1);
    let envelope: serde_json::Value = serde_json::from_str(&err).unwrap();
    assert_eq!(
        envelope.as_object().unwrap().len(),
        10,
        "expected ten envelope keys"
    );
    assert_eq!(envelope["error"], "Message");
    let body = envelope["body"].as_str().unwrap();
    assert!(
        body.starts_with("input JSON"),
        "body should start with \"input JSON\", got {body:?}"
    );
    // `message` crosses the ABI too — it is what every binding shows
    // the user, so a host never has to render its own.
    assert_eq!(
        envelope["message"].as_str().unwrap(),
        body,
        "Error::Message renders as its own body"
    );

    // format is idempotent through the FFI: formatting the output of a
    // format call reproduces the same bytes (fixed point).
    let (rc, first, err) = call(&lib, b"ktav_format", b"a: {x: 1}\n");
    assert_eq!(rc, 0, "format failed: {err}");
    let (rc, second, err) = call(&lib, b"ktav_format", &first);
    assert_eq!(rc, 0, "re-format failed: {err}");
    assert_eq!(second, first, "format is not a fixed point through the FFI");

    // free(null, 0) is a documented no-op.
    unsafe { free(std::ptr::null_mut(), 0) };
}
