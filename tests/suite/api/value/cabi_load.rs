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
type EditFn = unsafe extern "C" fn(
    *const u8,
    usize,
    *const u8,
    usize,
    *const u8,
    usize,
    *mut *mut u8,
    *mut usize,
    *mut *mut c_char,
    *mut usize,
) -> c_int;
/// Signature of the two-input mutation functions (`ktav_remove_*`).
type Mutate2Fn = unsafe extern "C" fn(
    *const u8,
    usize,
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

fn call_edit(
    lib: &libloading::Library,
    name: &[u8],
    source: &[u8],
    path: &[u8],
    replacement: &[u8],
) -> (c_int, Vec<u8>, String) {
    // SAFETY: имя нового export и точная ABI2 сигнатура зафиксированы общим контрактом.
    let f: libloading::Symbol<EditFn> = unsafe { lib.get(name).unwrap() };
    let (mut out, mut out_len, mut err, mut err_len) =
        (std::ptr::null_mut(), 0usize, std::ptr::null_mut(), 0usize);
    // SAFETY: inputs читаемы; output slots раздельны и живут до конца вызова.
    let rc = unsafe {
        f(
            source.as_ptr(),
            source.len(),
            path.as_ptr(),
            path.len(),
            replacement.as_ptr(),
            replacement.len(),
            &mut out,
            &mut out_len,
            &mut err,
            &mut err_len,
        )
    };
    if rc == 0 {
        assert!(err.is_null());
        assert_eq!(err_len, 0);
    } else {
        assert!(out.is_null());
        assert_eq!(out_len, 0);
    }
    // SAFETY: каждый полученный buffer читается и освобождается ровно один раз той же library.
    let bytes = unsafe { take_out(lib, out, out_len) };
    // SAFETY: error buffer принадлежит той же library, length получена этим вызовом.
    let error = unsafe { take_err(lib, err, err_len) };
    (rc, bytes, error)
}

/// Call a named two-input mutation function; asserts the mutual
/// exclusivity of success/error outputs and frees through the library.
fn call_remove(
    lib: &libloading::Library,
    name: &[u8],
    source: &[u8],
    path: &[u8],
) -> (c_int, Vec<u8>, String) {
    // SAFETY: имя export и точная ABI2 сигнатура зафиксированы контрактом.
    let f: libloading::Symbol<Mutate2Fn> = unsafe { lib.get(name).unwrap() };
    let (mut out, mut out_len, mut err, mut err_len) =
        (std::ptr::null_mut(), 0usize, std::ptr::null_mut(), 0usize);
    // SAFETY: inputs читаемы; output slots раздельны и живут до конца вызова.
    let rc = unsafe {
        f(
            source.as_ptr(),
            source.len(),
            path.as_ptr(),
            path.len(),
            &mut out,
            &mut out_len,
            &mut err,
            &mut err_len,
        )
    };
    if rc == 0 {
        assert!(err.is_null());
        assert_eq!(err_len, 0);
    } else {
        assert!(out.is_null());
        assert_eq!(out_len, 0);
    }
    // SAFETY: каждый buffer читается и освобождается ровно один раз той же library.
    let bytes = unsafe { take_out(lib, out, out_len) };
    // SAFETY: error buffer принадлежит той же library.
    let error = unsafe { take_err(lib, err, err_len) };
    (rc, bytes, error)
}

/// Out-slot sentinels: ненулевые адреса без выделенной памяти. Callee
/// обязан перезаписать все четыре слота до возврата; попытка deref/free
/// sentinel — ошибка теста, а не UB.
fn sentinel_slots() -> (*mut u8, usize, *mut c_char, usize) {
    (
        usize::MAX as *mut u8,
        usize::MAX,
        usize::MAX as *mut c_char,
        usize::MAX,
    )
}

/// Сырой результат вызова: неинициализированные слоты — sentinel.
struct RawResult {
    rc: c_int,
    out: *mut u8,
    out_len: usize,
    err: *mut c_char,
    err_len: usize,
}

impl RawResult {
    /// Проверяет инициализацию слотов и эксклюзивность success/error,
    /// затем читает и освобождает выданные buffers той же library.
    ///
    /// # Safety
    /// Слоты, перезаписанные callee, указывают на валидные buffers
    /// library; sentinel-слоты отсекаются assert до любого deref.
    unsafe fn settle(self, lib: &libloading::Library, ctx: &str) -> (bool, Vec<u8>, String) {
        let max = usize::MAX;
        assert!(
            self.out as usize != max && self.out_len != max,
            "out slots not initialized: {ctx}"
        );
        assert!(
            self.err as usize != max && self.err_len != max,
            "err slots not initialized: {ctx}"
        );
        let ok = self.rc == 0;
        if ok {
            assert!(
                self.err.is_null() && self.err_len == 0,
                "success must null error slots: {ctx}"
            );
        } else {
            assert!(
                !self.err.is_null() && self.err_len > 0,
                "error must return envelope: {ctx}"
            );
            assert!(
                self.out.is_null() && self.out_len == 0,
                "error must null out slots: {ctx}"
            );
        }
        // SAFETY: слоты прошли assert-проверку; buffers выданы этой library.
        let bytes = unsafe { take_out(lib, self.out, self.out_len) };
        // SAFETY: error buffer принадлежит той же library.
        let text = unsafe { take_err(lib, self.err, self.err_len) };
        (ok, bytes, text)
    }
}

/// Raw 3-input вызов (insert/rename): явные ptr/len для каждого входа,
/// out slots — sentinel.
fn call_edit_raw3(
    lib: &libloading::Library,
    name: &[u8],
    src: &[u8],
    path: (*const u8, usize),
    extra: (*const u8, usize),
) -> RawResult {
    // SAFETY: source живёт до возврата; прочие spans заданы тестовыми данными.
    unsafe { call_edit_raw_parts(lib, name, [(src.as_ptr(), src.len()), path, extra]) }
}

/// # Safety
/// Ненулевые spans читаемы до возврата; NULL проверяется экспортом до чтения.
unsafe fn call_edit_raw_parts(
    lib: &libloading::Library,
    name: &[u8],
    inputs: [(*const u8, usize); 3],
) -> RawResult {
    // SAFETY: имя export и точная ABI2 сигнатура зафиксированы контрактом.
    let f: libloading::Symbol<EditFn> = unsafe { lib.get(name).unwrap() };
    let (mut out, mut out_len, mut err, mut err_len) = sentinel_slots();
    // SAFETY: входы соответствуют контракту helper; output slots раздельны и writable.
    let rc = unsafe {
        f(
            inputs[0].0,
            inputs[0].1,
            inputs[1].0,
            inputs[1].1,
            inputs[2].0,
            inputs[2].1,
            &mut out,
            &mut out_len,
            &mut err,
            &mut err_len,
        )
    };
    RawResult {
        rc,
        out,
        out_len,
        err,
        err_len,
    }
}

/// Raw 2-input вызов (remove): явные ptr/len, out slots — sentinel.
fn call_remove_raw(
    lib: &libloading::Library,
    name: &[u8],
    src: &[u8],
    path: (*const u8, usize),
) -> RawResult {
    // SAFETY: source живёт до возврата; path задан тестовыми данными.
    unsafe { call_remove_raw_parts(lib, name, [(src.as_ptr(), src.len()), path]) }
}

/// # Safety
/// Ненулевые spans читаемы до возврата; NULL проверяется экспортом до чтения.
unsafe fn call_remove_raw_parts(
    lib: &libloading::Library,
    name: &[u8],
    inputs: [(*const u8, usize); 2],
) -> RawResult {
    // SAFETY: имя export и точная ABI2 сигнатура зафиксированы контрактом.
    let f: libloading::Symbol<Mutate2Fn> = unsafe { lib.get(name).unwrap() };
    let (mut out, mut out_len, mut err, mut err_len) = sentinel_slots();
    // SAFETY: входы соответствуют контракту helper; output slots раздельны и writable.
    let rc = unsafe {
        f(
            inputs[0].0,
            inputs[0].1,
            inputs[1].0,
            inputs[1].1,
            &mut out,
            &mut out_len,
            &mut err,
            &mut err_len,
        )
    };
    RawResult {
        rc,
        out,
        out_len,
        err,
        err_len,
    }
}

/// Один boundary/precision сценарий: name, path, wire/new_key, ожидаемые байты.
type RawCase<'a> = (&'a [u8], &'a [u8], &'a [u8], &'a [u8]);

/// FFI boundary matrix: для каждого из шести экспортов NULL-positive
/// на КАЖДОМ входе даёт rc=1/Message без source-координат; NULL-zero
/// входы — валидный пустой transport с честным результатом. Out slots
/// стартуют sentinel и обязаны быть перезаписаны.
fn mutation_ffi_boundary_matrix(lib: &libloading::Library) {
    let names3 = [
        b"ktav_insert_value".as_slice(),
        b"ktav_insert_value_segments".as_slice(),
        b"ktav_rename_key".as_slice(),
        b"ktav_rename_key_segments".as_slice(),
    ];
    let names2 = [
        b"ktav_remove_value".as_slice(),
        b"ktav_remove_value_segments".as_slice(),
    ];
    let src = b"a: 1\n";
    let path = (b"a".as_ptr(), 1usize);
    let wire = (b"null".as_ptr(), 4usize);
    let key = (b"x".as_ptr(), 1usize);
    let null_pair = (std::ptr::null(), 0usize);

    // NULL-positive на каждом входе по очереди -> rc=1, Message,
    // без source-координат (line/span null).
    for name in names3 {
        for slot in 0..3usize {
            let mut inputs = [(src.as_ptr(), src.len()), path, wire];
            inputs[slot] = (std::ptr::null(), 5);
            // SAFETY: единственный NULL отвергается до чтения; остальные spans живы.
            let raw = unsafe { call_edit_raw_parts(lib, name, inputs) };
            let (ok, bytes, text) = unsafe { raw.settle(lib, "NULL-positive 3-input") };
            assert!(!ok, "{name:?} slot {slot}: NULL-positive must fail");
            assert!(bytes.is_empty());
            let envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(envelope["error"], "Message", "{name:?} slot {slot}");
            assert!(envelope["line"].is_null(), "{name:?} slot {slot}");
            assert!(envelope["span"].is_null(), "{name:?} slot {slot}");
        }
    }
    for name in names2 {
        for slot in 0..2usize {
            let mut inputs = [(src.as_ptr(), src.len()), path];
            inputs[slot] = (std::ptr::null(), 5);
            // SAFETY: единственный NULL отвергается до чтения; остальные spans живы.
            let raw = unsafe { call_remove_raw_parts(lib, name, inputs) };
            let (ok, bytes, text) = unsafe { raw.settle(lib, "NULL-positive 2-input") };
            assert!(!ok, "{name:?} slot {slot}: NULL-positive must fail");
            assert!(bytes.is_empty());
            let envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(envelope["error"], "Message", "{name:?} slot {slot}");
            assert!(envelope["line"].is_null());
            assert!(envelope["span"].is_null());
        }
    }

    // NULL-zero inputs: честные результаты по семействам.
    // Textual path пуст -> EditInvalidPath, offset 0.
    for name in [
        b"ktav_insert_value".as_slice(),
        b"ktav_rename_key".as_slice(),
    ] {
        let raw = call_edit_raw3(lib, name, src, null_pair, wire);
        let (ok, bytes, text) = unsafe { raw.settle(lib, "NULL-zero text path") };
        assert!(!ok);
        assert!(bytes.is_empty());
        let envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(envelope["error"], "EditInvalidPath", "{name:?}");
        assert_eq!(envelope["body"], "0");
    }
    let raw = call_remove_raw(lib, b"ktav_remove_value", src, null_pair);
    let (ok, bytes, text) = unsafe { raw.settle(lib, "NULL-zero remove path") };
    assert!(!ok);
    assert!(bytes.is_empty());
    let envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(envelope["error"], "EditInvalidPath");
    assert_eq!(envelope["body"], "0");

    // Segments bytes пустые -> transport Message; JSON [] -> EditNotFound.
    let empty = (std::ptr::null(), 0usize);
    for name in [
        b"ktav_insert_value_segments".as_slice(),
        b"ktav_remove_value_segments".as_slice(),
        b"ktav_rename_key_segments".as_slice(),
    ] {
        let raw = if name.starts_with(b"ktav_remove") {
            call_remove_raw(lib, name, src, empty)
        } else {
            call_edit_raw3(lib, name, src, empty, wire)
        };
        let (ok, bytes, text) = unsafe { raw.settle(lib, "empty segments bytes") };
        assert!(!ok);
        assert!(bytes.is_empty());
        let envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(envelope["error"], "Message", "{name:?}");
        assert!(envelope["line"].is_null());
        let arr = (br#"[]"#.as_ptr(), 2usize);
        let raw = if name.starts_with(b"ktav_remove") {
            call_remove_raw(lib, name, src, arr)
        } else {
            call_edit_raw3(lib, name, src, arr, wire)
        };
        let (ok, bytes, text) = unsafe { raw.settle(lib, "JSON [] segments") };
        assert!(!ok);
        assert!(bytes.is_empty());
        let envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(envelope["error"], "EditNotFound", "{name:?}");
    }

    // Пустой replacement (insert) -> Message; пустой new_key (rename)
    // -> EditUnsupported; оба через NULL-zero указателя.
    let raw = call_edit_raw3(lib, b"ktav_insert_value", src, path, null_pair);
    let (ok, bytes, text) = unsafe { raw.settle(lib, "empty replacement") };
    assert!(!ok);
    assert!(bytes.is_empty());
    let envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(envelope["error"], "Message");
    let raw = call_edit_raw3(lib, b"ktav_rename_key", src, path, null_pair);
    let (ok, bytes, text) = unsafe { raw.settle(lib, "empty new key") };
    assert!(!ok);
    assert!(bytes.is_empty());
    let envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(envelope["error"], "EditUnsupported");

    // Пустой source разбирается как пустой корень: честный контракт.
    // SAFETY: NULL/0 обозначает пустой source; остальные spans живы.
    let raw = unsafe { call_edit_raw_parts(lib, b"ktav_insert_value", [null_pair, path, wire]) };
    let (ok, bytes, text) = unsafe { raw.settle(lib, "insert into empty source") };
    assert!(ok, "insert into empty source: {text}");
    assert_eq!(bytes, b"a: null");
    // SAFETY: NULL/0 обозначает пустой source; path читаем.
    let raw = unsafe { call_remove_raw_parts(lib, b"ktav_remove_value", [null_pair, path]) };
    let (ok, _bytes, text) = unsafe { raw.settle(lib, "remove from empty source") };
    assert!(!ok);
    let envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(envelope["error"], "EditNotFound");
    // SAFETY: NULL/0 обозначает пустой source; path и key читаемы.
    let raw = unsafe { call_edit_raw_parts(lib, b"ktav_rename_key", [null_pair, path, key]) };
    let (ok, _bytes, text) = unsafe { raw.settle(lib, "rename in empty source") };
    assert!(!ok);
    let envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(envelope["error"], "EditNotFound");
}

/// C-уровень: точность скаляров, precedence исходника над транспортом,
/// no-op witness на BOM+CRLF+неканоничном Integer, разрешённый deep
/// mixed array traversal для всех трёх операций.
fn mutation_precision_and_traversal(lib: &libloading::Library) {
    // Malformed tagged wire -> Message; typed >i64 -> EditUnsupported
    // (ни transport, ни parser не усекают числа).
    for (name, wire, expected) in [
        (
            b"ktav_insert_value".as_slice(),
            br#"{"$i":"abc"}"#.as_slice(),
            "Message",
        ),
        (
            b"ktav_insert_value".as_slice(),
            br#"{"$f":"1.5x"}"#.as_slice(),
            "Message",
        ),
        (
            b"ktav_insert_value".as_slice(),
            br#"{"$i":"9223372036854775808"}"#.as_slice(),
            "EditUnsupported",
        ),
    ] {
        let raw = call_edit_raw3(
            lib,
            name,
            b"a: 1\n",
            (b"b".as_ptr(), 1usize),
            (wire.as_ptr(), wire.len()),
        );
        let (ok, bytes, text) = unsafe { raw.settle(lib, "scalar wire") };
        assert!(!ok);
        assert!(bytes.is_empty());
        let envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(envelope["error"], expected, "{name:?} {wire:?}");
    }

    // Позитивная точность: 2^53+1 и signed-zero float через C.
    let precision_cases: [RawCase; 2] = [
        (
            b"ktav_insert_value",
            b"b",
            br#"{"$i":"9007199254740993"}"#,
            b"a: 1\nb: 9007199254740993\n",
        ),
        (
            b"ktav_insert_value_segments",
            br#"["b"]"#,
            br#"{"$f":"-0.0"}"#,
            b"a: 1\nb: -0.0\n",
        ),
    ];
    for (name, path, wire, expected) in precision_cases {
        let raw = call_edit_raw3(
            lib,
            name,
            b"a: 1\n",
            (path.as_ptr(), path.len()),
            (wire.as_ptr(), wire.len()),
        );
        let (ok, bytes, text) = unsafe { raw.settle(lib, "scalar precision") };
        assert!(ok, "precision case failed: {text}");
        assert_eq!(bytes, expected);
    }

    // Precedence: битый source (UTF-8 / EOF) при ломаном, но
    // валидном по ptr/len транспорте через фактический C export.
    let bad_src = [b'b', b'a', b'd', b':', b' ', 0xff];
    let bad_path_bytes = [0xff_u8];
    let bad_path = (bad_path_bytes.as_ptr(), bad_path_bytes.len());
    let bad_wire = (b"not json".as_ptr(), 8usize);
    let raw = call_edit_raw3(lib, b"ktav_rename_key", &bad_src, bad_path, bad_wire);
    let (ok, bytes, text) = unsafe { raw.settle(lib, "utf8 precedence") };
    assert!(!ok);
    assert!(bytes.is_empty());
    let envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(envelope["error"], "InvalidUtf8");
    let eof_src = b"a: {\n b: [";
    let raw = call_edit_raw3(
        lib,
        b"ktav_rename_key_segments",
        eof_src,
        (b"bad".as_ptr(), 3usize),
        bad_wire,
    );
    let (ok, bytes, text) = unsafe { raw.settle(lib, "eof precedence") };
    assert!(!ok);
    assert!(bytes.is_empty());
    let envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(envelope["error"], "UnclosedCompound");
    assert_eq!(envelope["line_text"], " b: [");

    // No-op witness: BOM + CRLF + неканоничный Integer +1 —
    // byte-identical результат.
    let bom_crlf_src = [
        0xEFu8, 0xBB, 0xBF, b'k', b':', b' ', b'+', b'1', b'\r', b'\n',
    ];
    let raw = call_edit_raw3(
        lib,
        b"ktav_rename_key",
        &bom_crlf_src,
        (b"k".as_ptr(), 1usize),
        (b"k".as_ptr(), 1usize),
    );
    let (ok, bytes, text) = unsafe { raw.settle(lib, "noop witness") };
    assert!(ok, "noop rename failed: {text}");
    assert_eq!(bytes, bom_crlf_src);

    // Deep mixed array traversal: операция разрешена (Object-элемент).
    let deep = b"top: [\n  {q: 1}\n]\n";
    let deep_cases: [RawCase; 2] = [
        (
            b"ktav_insert_value",
            b"top.0.w",
            br#"{"$i":"2"}"#,
            b"top: [\n  {q: 1, w: 2}\n]\n",
        ),
        (
            b"ktav_rename_key",
            b"top.0.q",
            b"z",
            b"top: [\n  {z: 1}\n]\n",
        ),
    ];
    for (name, path, wire, expected) in deep_cases {
        let raw = call_edit_raw3(
            lib,
            name,
            deep,
            (path.as_ptr(), path.len()),
            (wire.as_ptr(), wire.len()),
        );
        let (ok, bytes, text) = unsafe { raw.settle(lib, "deep traversal") };
        assert!(ok, "deep {name:?} failed: {text}");
        assert_eq!(bytes, expected, "deep {name:?}");
    }
    let raw = call_remove_raw(
        lib,
        b"ktav_remove_value",
        deep,
        (b"top.0.q".as_ptr(), 7usize),
    );
    let (ok, bytes, text) = unsafe { raw.settle(lib, "deep remove") };
    assert!(ok, "deep remove failed: {text}");
    assert_eq!(bytes, b"top: [\n  {}\n]\n");
}

#[test]
fn loads_fixture_and_exercises_the_abi() {
    let path = build_fixture();
    let lib = unsafe { libloading::Library::new(&path).expect("failed to dlopen the fixture") };

    // Все восемнадцать ABI2 symbols должны разрешаться в реальном cdylib:
    // eager-FFI loaders отклоняют library при отсутствии любого symbol.
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
    // Шесть новых mutation operations (task114): 3-input и 2-input формы.
    for name in [
        b"ktav_insert_value".as_slice(),
        b"ktav_insert_value_segments".as_slice(),
        b"ktav_rename_key".as_slice(),
        b"ktav_rename_key_segments".as_slice(),
    ] {
        let _f: libloading::Symbol<EditFn> = unsafe { lib.get(name).unwrap() };
    }
    for name in [
        b"ktav_remove_value".as_slice(),
        b"ktav_remove_value_segments".as_slice(),
    ] {
        let _f: libloading::Symbol<Mutate2Fn> = unsafe { lib.get(name).unwrap() };
    }
    for (name, edit_path) in [
        (b"ktav_edit_value".as_slice(), b"a.0".as_slice()),
        (
            b"ktav_edit_value_segments".as_slice(),
            br#"["a","0"]"#.as_slice(),
        ),
    ] {
        let (rc, out, err) =
            call_edit(&lib, name, b"a: [old, keep]\n", edit_path, br#""one\ntwo""#);
        assert_eq!(rc, 0, "source-edit failed: {err}");
        assert_eq!(out, b"a: [\n    ((\none\ntwo\n    ))\n    keep\n]\n");
        let (rc, out, err) = call_edit(&lib, name, b"a: 1\n", edit_path, b"null");
        assert_eq!(rc, 1);
        assert!(out.is_empty());
        let envelope: serde_json::Value = serde_json::from_str(&err).unwrap();
        assert_eq!(envelope["error"], "EditNotFound");
        assert!(envelope["span"].is_null());
    }

    // --- task114: шесть новых mutation exports -----------------------------
    // Insert: textual и segments пути; фактические отредактированные байты.
    for (name, path) in [
        (b"ktav_insert_value".as_slice(), b"b".as_slice()),
        (
            b"ktav_insert_value_segments".as_slice(),
            br#"["b"]"#.as_slice(),
        ),
    ] {
        let (rc, out, err) = call_edit(&lib, name, b"a: 1\n", path, br#"{"$i":"2"}"#);
        assert_eq!(rc, 0, "insert failed: {err}");
        assert_eq!(out, b"a: 1\nb: 2\n");
        // AlreadyExists — явная коллизия, никакой перезаписи.
        let (rc, out, err) = call_edit(&lib, name, b"a: 1\nb: 2\n", path, b"null");
        assert_eq!(rc, 1);
        assert!(out.is_empty());
        let envelope: serde_json::Value = serde_json::from_str(&err).unwrap();
        assert_eq!(envelope["error"], "EditAlreadyExists");
        assert!(envelope["body"].is_null());
    }
    // Remove: два входных буфера, textual и segments.
    for (name, path) in [
        (b"ktav_remove_value".as_slice(), b"a".as_slice()),
        (
            b"ktav_remove_value_segments".as_slice(),
            br#"["a"]"#.as_slice(),
        ),
    ] {
        let (rc, out, err) = call_remove(&lib, name, b"a: 1\nb: keep\n", path);
        assert_eq!(rc, 0, "remove failed: {err}");
        assert_eq!(out, b"b: keep\n");
        let (rc, out, err) = call_remove(&lib, name, b"b: 1\n", path);
        assert!(out.is_empty());
        assert_eq!(rc, 1);
        let envelope: serde_json::Value = serde_json::from_str(&err).unwrap();
        assert_eq!(envelope["error"], "EditNotFound");
    }
    // Rename: textual; no-op same key — byte-identical, включая spelling.
    let (rc, out, err) = call_edit(&lib, b"ktav_rename_key", b"a: 1\nb: 2\n", b"b", b"x");
    assert_eq!(rc, 0, "rename failed: {err}");
    assert_eq!(out, b"a: 1\nx: 2\n");
    let (rc, out, _err) = call_edit(&lib, b"ktav_rename_key", b"k:: 042\n", b"k", b"k");
    assert_eq!(rc, 0);
    assert_eq!(out, b"k:: 042\n");
    // Rename segments + invalid new key UTF-8 -> Message.
    let (rc, out, _err) = call_edit(
        &lib,
        b"ktav_rename_key_segments",
        b"a: 1\n",
        br#"["a"]"#,
        b"x",
    );
    assert_eq!(rc, 0);
    assert_eq!(out, b"x: 1\n");
    let (rc, _out, err) = call_edit(&lib, b"ktav_rename_key", b"a: 1\n", b"a", &[0xff]);
    assert_eq!(rc, 1);
    let envelope: serde_json::Value = serde_json::from_str(&err).unwrap();
    assert_eq!(envelope["error"], "Message");

    // FFI boundary matrix для всех шести экспортов (sentinel out slots).
    mutation_ffi_boundary_matrix(&lib);
    // C-уровень: точность скаляров, precedence исходника, no-op witness,
    // разрешённый deep mixed array traversal.
    mutation_precision_and_traversal(&lib);
    eprintln!(
        "resolved all eighteen ktav_cabi symbols from {}",
        path.display()
    );

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
