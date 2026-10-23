//! `ktav::edit::edit_value` / `edit_value_segments`: точечная правка
//! исходного текста по пути. Оракулы — литеральные строки результата:
//! тест доказывает, что байты вне цели не тронуты.

use ktav::edit::{edit_value, edit_value_segments, EditError};
use ktav::parse;
use ktav::value::Value;

fn s(v: &str) -> Value {
    Value::String(v.into())
}

// --- скаляры, все типы ------------------------------------------------------

#[test]
fn string_replacement_plain() {
    let t = "port: 8080\nhost: db\n";
    assert_eq!(
        edit_value(t, "port", &s("db.internal")).unwrap(),
        "port: db.internal\nhost: db\n"
    );
}

#[test]
fn int_replacement_switches_raw_marker_off() {
    // `:: 8080` был String; замена на Integer обязана сменить маркер.
    let t = "port:: 8080\n";
    assert_eq!(
        edit_value(t, "port", &Value::Integer("9090".into())).unwrap(),
        "port: 9090\n"
    );
}

#[test]
fn numeric_string_replacement_switches_raw_marker_on() {
    let t = "port: 8080\n";
    assert_eq!(edit_value(t, "port", &s("42")).unwrap(), "port:: 42\n");
}

#[test]
fn keyword_like_string_replacement_needs_raw() {
    let t = "mode: auto\n";
    assert_eq!(edit_value(t, "mode", &s("true")).unwrap(), "mode:: true\n");
}

#[test]
fn raw_marker_preserved_for_string() {
    let t = "k:: v1\n";
    assert_eq!(edit_value(t, "k", &s("v2")).unwrap(), "k:: v2\n");
}

#[test]
fn empty_string_replacement_keeps_raw() {
    // Контракт: существующий raw сохраняется; пустое raw-тело —
    // String("").
    let t = "k:: x\n";
    assert_eq!(edit_value(t, "k", &s("")).unwrap(), "k:: \n");
}

#[test]
fn null_and_bool_replacements() {
    let t = "a: x\nb: y\nc: z\n";
    assert_eq!(
        edit_value(t, "a", &Value::Null).unwrap(),
        "a: null\nb: y\nc: z\n"
    );
    assert_eq!(
        edit_value(t, "b", &Value::Bool(false)).unwrap(),
        "a: x\nb: false\nc: z\n"
    );
    assert_eq!(
        edit_value(t, "c", &Value::Bool(true)).unwrap(),
        "a: x\nb: y\nc: true\n"
    );
}

#[test]
fn float_replacement() {
    let t = "ratio: 1\n";
    assert_eq!(
        edit_value(t, "ratio", &Value::Float("0.5".into())).unwrap(),
        "ratio: 0.5\n"
    );
}

// --- вложенность, массивы, ключи -------------------------------------------

#[test]
fn nested_dotted_path() {
    let t = "srv: {\n  host: a\n  port: 1\n}\n";
    assert_eq!(
        edit_value(t, "srv.host", &s("b.example")).unwrap(),
        "srv: {\n  host: b.example\n  port: 1\n}\n"
    );
}

#[test]
fn dotted_source_key_leaf() {
    let t = "a.b: 1\nc: 2\n";
    assert_eq!(
        edit_value(t, "a.b", &Value::Integer("3".into())).unwrap(),
        "a.b: 3\nc: 2\n"
    );
}

#[test]
fn array_index_replacement() {
    let t = "list: [\n  one\n  two\n]\n";
    assert_eq!(
        edit_value(t, "list.1", &s("TWO")).unwrap(),
        "list: [\n  one\n  TWO\n]\n"
    );
}

#[test]
fn numeric_segment_is_string_key_in_object() {
    // В Object сегмент "0" — строковый ключ, а не индекс.
    let t = "0: first\nlist: [\n  x\n]\n";
    assert_eq!(
        edit_value(t, "0", &s("zero")).unwrap(),
        "0: zero\nlist: [\n  x\n]\n"
    );
    assert_eq!(
        edit_value(t, "list.0", &s("y")).unwrap(),
        "0: first\nlist: [\n  y\n]\n"
    );
}

#[test]
fn escaped_dot_in_path_vs_real_key() {
    let t = "\"a.b\": 1\na: {\n  b: 2\n}\n";
    assert_eq!(
        edit_value(t, "a\\.b", &Value::Integer("9".into())).unwrap(),
        "\"a.b\": 9\na: {\n  b: 2\n}\n"
    );
    assert_eq!(
        edit_value(t, "a.b", &Value::Integer("9".into())).unwrap(),
        "\"a.b\": 1\na: {\n  b: 9\n}\n"
    );
}

#[test]
fn escaped_backslash_in_path() {
    // Ключ, декодируемый в `a\b`; путь `a\\b` декодируется так же.
    let t = "\"a\\\\b\": 1\n";
    assert_eq!(
        edit_value(t, "a\\\\b", &Value::Integer("2".into())).unwrap(),
        "\"a\\\\b\": 2\n"
    );
}

#[test]
fn segments_api_equivalent() {
    let t = "a: {\n  b: 1\n}\n";
    let segs = vec!["a".to_string(), "b".to_string()];
    assert_eq!(
        edit_value_segments(t, &segs, &Value::Integer("2".into())).unwrap(),
        "a: {\n  b: 2\n}\n"
    );
}

// --- inline-контексты -------------------------------------------------------

#[test]
fn inline_pair_byte_exact() {
    let t = "a: {x: 1, y: 2}\n";
    assert_eq!(
        edit_value(t, "a.y", &Value::Integer("3".into())).unwrap(),
        "a: {x: 1, y: 3}\n"
    );
}

#[test]
fn inline_raw_pair() {
    let t = "a: {k:: v, j: w}\n";
    // Существующий raw-маркер сохраняется для String-замены.
    assert_eq!(edit_value(t, "a.k", &s("z")).unwrap(), "a: {k:: z, j: w}\n");
}

#[test]
fn inline_nonempty_compound_replacement_inline_exact() {
    // Непустой контейнер в inline-позиции рендерится inline-формой.
    let t = "a: {x:1}\n";
    let mut obj = ktav::value::ObjectMap::default();
    obj.insert("n".into(), Value::Integer("1".into()));
    assert_eq!(
        edit_value(t, "a.x", &Value::Object(obj)).unwrap(),
        "a: {x:{n: 1}}\n"
    );
}

#[test]
fn inline_nested_compound_replacement() {
    let t = "a: {x: 1}\n";
    let mut inner = ktav::value::ObjectMap::default();
    inner.insert("m".into(), Value::Integer("2".into()));
    let mut obj = ktav::value::ObjectMap::default();
    obj.insert("n".into(), Value::Array(vec![Value::Object(inner)]));
    assert_eq!(
        edit_value(t, "a.x", &Value::Object(obj)).unwrap(),
        "a: {x: {n: [{m: 2}]}}\n"
    );
}

// --- контейнеры -------------------------------------------------------------

#[test]
fn scalar_to_compound() {
    let t = "a: 1\n";
    let mut obj = ktav::value::ObjectMap::default();
    obj.insert("x".into(), Value::Integer("1".into()));
    assert_eq!(
        edit_value(t, "a", &Value::Object(obj)).unwrap(),
        "a: {\n    x: 1\n}\n"
    );
}

#[test]
fn empty_compound_to_filled() {
    let t = "a: {}\nb: 2\n";
    let mut obj = ktav::value::ObjectMap::default();
    obj.insert("x".into(), Value::Integer("1".into()));
    assert_eq!(
        edit_value(t, "a", &Value::Object(obj)).unwrap(),
        "a: {\n    x: 1\n}\nb: 2\n"
    );
}

#[test]
fn compound_to_scalar() {
    let t = "a: {\n  x: 1\n}\n";
    assert_eq!(
        edit_value(t, "a", &Value::Integer("5".into())).unwrap(),
        "a: 5\n"
    );
}

// --- reopen / noncontiguous -------------------------------------------------

#[test]
fn reopen_leaf_editable_whole_target_unsupported() {
    let t = "a: {}\na.y: 2\n";
    assert_eq!(
        edit_value(t, "a.y", &Value::Integer("3".into())).unwrap(),
        "a: {}\na.y: 3\n"
    );
    assert!(matches!(
        edit_value(t, "a", &Value::Integer("9".into())),
        Err(EditError::Unsupported { .. })
    ));
}

// --- multiline строки -------------------------------------------------------

#[test]
fn multiline_string_replacement() {
    let t = "s: (\n hi\n)\n";
    assert_eq!(edit_value(t, "s", &s("a\nb")).unwrap(), "s: ((\na\nb\n))\n");
}

#[test]
fn multiline_string_with_terminator_line_unsupported() {
    let t = "s: (\n hi\n)\n";
    assert!(matches!(
        edit_value(t, "s", &s("x\n))\ny")),
        Err(EditError::Unsupported { .. })
    ));
}

// --- байт-в-байт нетронутость окружения -------------------------------------

#[test]
fn comments_blanks_and_trailing_bytes_preserved() {
    let t = "## top\n\na: 1\n\n## end\n";
    assert_eq!(
        edit_value(t, "a", &Value::Integer("2".into())).unwrap(),
        "## top\n\na: 2\n\n## end\n"
    );
}

#[test]
fn crlf_document_preserved() {
    let t = "a: 1\r\nb: two\r\n";
    assert_eq!(
        edit_value(t, "b", &s("three")).unwrap(),
        "a: 1\r\nb: three\r\n"
    );
}

#[test]
fn cr_only_document_preserved() {
    let t = "a: 1\rb: two\r";
    assert_eq!(edit_value(t, "b", &s("three")).unwrap(), "a: 1\rb: three\r");
}

#[test]
fn bom_and_no_final_newline_preserved() {
    let t = "\u{FEFF}a: 1";
    assert_eq!(
        edit_value(t, "a", &Value::Integer("2".into())).unwrap(),
        "\u{FEFF}a: 2"
    );
}

#[test]
fn utf8_values_and_keys() {
    let t = "ключ: значение\n";
    assert_eq!(edit_value(t, "ключ", &s("число")).unwrap(), "ключ: число\n");
}

// --- ошибки -----------------------------------------------------------------

#[test]
fn not_found_cases() {
    let t = "a: {\n  b: 1\n}\nlist: [\n  x\n]\n";
    assert_eq!(edit_value(t, "missing", &s("v")), Err(EditError::NotFound));
    assert_eq!(edit_value(t, "a.b.c", &s("v")), Err(EditError::NotFound));
    assert_eq!(edit_value(t, "list.5", &s("v")), Err(EditError::NotFound));
    assert_eq!(edit_value(t, "a.b.c.d", &s("v")), Err(EditError::NotFound));
    // Десцент через скаляр.
    let t2 = "a: scalar\n";
    assert_eq!(edit_value(t2, "a.b", &s("v")), Err(EditError::NotFound));
    // Пустой сегментный путь.
    let empty: Vec<String> = Vec::new();
    assert_eq!(
        edit_value_segments(t, &empty, &s("v")),
        Err(EditError::NotFound)
    );
}

#[test]
fn invalid_path_cases() {
    let t = "a: 1\n";
    assert!(matches!(
        edit_value(t, "a\\nb", &s("v")),
        Err(EditError::InvalidPath { offset: 1 })
    ));
    assert!(matches!(
        edit_value(t, "a\\", &s("v")),
        Err(EditError::InvalidPath { offset: 1 })
    ));
    assert!(matches!(
        edit_value(t, "", &s("v")),
        Err(EditError::InvalidPath { offset: 0 })
    ));
    assert!(matches!(
        edit_value(t, "a..b", &s("v")),
        Err(EditError::InvalidPath { offset: 2 })
    ));
    assert!(matches!(
        edit_value(t, ".a", &s("v")),
        Err(EditError::InvalidPath { offset: 0 })
    ));
}

#[test]
fn non_numeric_index_in_array_is_not_found() {
    let t = "list: [\n  x\n]\n";
    assert_eq!(
        edit_value_segments(t, &["list", "abc"], &s("v")),
        Err(EditError::NotFound)
    );
}

#[test]
fn malformed_source_is_unsupported_not_panic() {
    let t = "a: {\n  x: 1\n";
    assert!(matches!(
        edit_value(t, "a.x", &s("v")),
        Err(EditError::Unsupported { .. })
    ));
}

#[test]
fn patched_documents_reparse() {
    let cases = [
        ("port: 8080\n", "port", s("9090")),
        ("list: [\n  one\n]\n", "list.0", s("two")),
    ];
    for (t, path, repl) in cases {
        let out = edit_value(t, path, &repl).unwrap();
        assert!(parse(&out).is_ok(), "патч обязан парситься: {out:?}");
    }
}

// --- точное сохранение окружения: маркеры/пробелы/табы ----------------------

#[test]
fn pair_spacing_and_eol_preserved() {
    // Сплайс — только тело: colon, табы/пробелы, хвостовые пробелы и
    // CRLF не трогаются.
    let t = "  a:\t   old  \r\n";
    assert_eq!(edit_value(t, "a", &s("new")).unwrap(), "  a:\t   new  \r\n");
}

#[test]
fn int_after_raw_drops_only_extra_colon() {
    // `::  ` → `:  `: лишний colon снят, двойной пробел сохранён.
    let t = "k::  8080\n";
    assert_eq!(
        edit_value(t, "k", &Value::Integer("9090".into())).unwrap(),
        "k:  9090\n"
    );
}

#[test]
fn numeric_string_adds_only_extra_colon() {
    let t = "n: 5\n";
    assert_eq!(edit_value(t, "n", &s("42")).unwrap(), "n:: 42\n");
}

#[test]
fn same_value_no_op_is_byte_identical() {
    let t = "a: x\n";
    assert_eq!(edit_value(t, "a", &s("x")).unwrap(), t);
}

#[test]
fn empty_body_pair_replacement() {
    let t = "k: \n";
    // Пустое тело: вставка даёт разделитель + тело; исходный хвостовой
    // пробел (вне диапазона тела) сохраняется.
    assert_eq!(edit_value(t, "k", &s("x")).unwrap(), "k: x \n");
    assert_eq!(edit_value(t, "k", &s("")).unwrap(), "k: \n");
}

// --- комментарии -------------------------------------------------------------

#[test]
fn compound_with_internal_comments_structural_replacement_rejected() {
    let t = "a: {\n  ## inner\n  x: 1\n}\n";
    let mut obj = ktav::value::ObjectMap::default();
    obj.insert("y".into(), Value::Integer("2".into()));
    assert!(matches!(
        edit_value(t, "a", &Value::Object(obj)),
        Err(EditError::Unsupported { .. })
    ));
}

#[test]
fn leaf_inside_commented_container_edits_and_keeps_comments() {
    let t = "a: {\n  ## inner\n  x: 1\n}\n";
    assert_eq!(
        edit_value(t, "a.x", &Value::Integer("2".into())).unwrap(),
        "a: {\n  ## inner\n  x: 2\n}\n"
    );
}

#[test]
fn hash_hash_inside_multiline_string_is_data_not_comment() {
    // `##` внутри multiline-строки — данные; структурная замена
    // контейнера обязана пройти (нет ложного reject).
    let t = "a: {\n  s: (\n    ## not comment\n  )\n}\n";
    assert_eq!(
        edit_value(t, "a", &Value::Integer("7".into())).unwrap(),
        "a: 7\n"
    );
}

// --- root-контексты ----------------------------------------------------------

#[test]
fn implicit_root_array_first_item_pair_shaped_gets_raw() {
    // Первый элемент неявного корня-массива: pair-подобное тело без
    // raw сделал бы корень Object (§ 5.0.1) → обязателен `:: `.
    let t = ":: old\nsecond\n";
    assert_eq!(edit_value(t, "0", &s("a: b")).unwrap(), ":: a: b\nsecond\n");
}

#[test]
fn implicit_root_array_second_item_bare_ok() {
    let t = ":: old\nsecond\n";
    assert_eq!(edit_value(t, "1", &s("a: b")).unwrap(), ":: old\na: b\n");
}

#[test]
fn explicit_root_array_first_item_raw_preserved() {
    // Существующий raw сохраняется для String-замены (root-dispatch
    // неприменим: корень явный).
    let t = "[\n  :: old\n  x\n]\n";
    assert_eq!(
        edit_value(t, "0", &s("k: v")).unwrap(),
        "[\n  :: k: v\n  x\n]\n"
    );
}

#[test]
fn root_inline_object_child() {
    let t = "{x: 1}\n";
    assert_eq!(
        edit_value(t, "x", &Value::Integer("2".into())).unwrap(),
        "{x: 2}\n"
    );
}

#[test]
fn root_inline_array_items() {
    let t = "[1, 2]\n";
    assert_eq!(
        edit_value(t, "0", &Value::Integer("7".into())).unwrap(),
        "[7, 2]\n"
    );
}

#[test]
fn inline_scalar_with_comma_escaped() {
    // `\,` — распознанный § 3.7-escape (escapes.rs: b',' => Some(','));
    // сплиттер пропускает `\X` целиком → запятая выражаема дословно.
    let t = "a: {x: 1}\n";
    assert_eq!(edit_value(t, "a.x", &s("y,z")).unwrap(), "a: {x: y\\,z}\n");
    assert_eq!(
        parse("a: {x: y\\,z}\n")
            .unwrap()
            .as_object()
            .unwrap()
            .get("a")
            .unwrap()
            .as_object()
            .unwrap()
            .get("x"),
        Some(&s("y,z"))
    );
}

#[test]
fn inline_scalar_backslash_escaped() {
    // Одиночный слэш в plain inline удваивается (escape § 3.7) и
    // декодируется обратно в то же значение.
    let t = "a: {x: 1}\n";
    assert_eq!(
        edit_value(t, "a.x", &s("y\\z")).unwrap(),
        "a: {x: y\\\\z}\n"
    );
    assert_eq!(
        parse("a: {x: y\\\\z}\n")
            .unwrap()
            .as_object()
            .unwrap()
            .get("a")
            .unwrap()
            .as_object()
            .unwrap()
            .get("x"),
        Some(&s("y\\z"))
    );
}

// --- multiline строки: точные байты ------------------------------------------

#[test]
fn multiline_edge_whitespace_strings_lossless() {
    let t = "s: (\n hi\n)\n";
    assert_eq!(edit_value(t, "s", &s(" a ")).unwrap(), "s: ((\n a \n))\n");
    assert_eq!(edit_value(t, "s", &s("\na")).unwrap(), "s: ((\n\na\n))\n");
    assert_eq!(edit_value(t, "s", &s("a\n")).unwrap(), "s: ((\na\n\n))\n");
    assert_eq!(
        edit_value(t, "s", &s("a\n\n")).unwrap(),
        "s: ((\na\n\n\n))\n"
    );
}

#[test]
fn cr_replacement_rejected() {
    let t = "s: (\n hi\n)\n";
    assert!(matches!(
        edit_value(t, "s", &s("a\r\nb")),
        Err(EditError::Unsupported { .. })
    ));
}

#[test]
fn compound_with_multiline_string_value_replacement_round_trips() {
    // Валидный рендер существует — отказ был бы ложным.
    let t = "a: 1\n";
    let mut obj = ktav::value::ObjectMap::default();
    obj.insert("s".into(), s("a\nb"));
    assert_eq!(
        edit_value(t, "a", &Value::Object(obj)).unwrap(),
        "a: {\n    s: (\n        a\n        b\n    )\n}\n"
    );
}

#[test]
fn edit_error_display_smoke() {
    let e = edit_value("a: 1\n", "", &s("v")).unwrap_err();
    assert!(e.to_string().contains("path"));
    let e2 = edit_value("a: 1\n", "zz", &s("v")).unwrap_err();
    assert!(e2.to_string().contains("existing"));
    let _: &dyn std::error::Error = &e2;
}

// --- физический inline-контекст на листе --------------------------------------

#[test]
fn dotted_key_inside_inline_escapes() {
    // Синтетический dotted-предок НЕ обязан наследовать контекст:
    // физический inline живёт на листе (b.c внутри `{}`).
    let t = "a: {b.c: old}\n";
    assert_eq!(
        edit_value(t, "a.b.c", &s("y,z")).unwrap(),
        "a: {b.c: y\\,z}\n"
    );
}

#[test]
fn reopened_former_inline_child_is_physical_line() {
    // Внешний `a.y` лежит на своей строке — запятая дословна, хотя
    // контейнер `a` когда-то был inline.
    let t = "a: {x:1}\na.y: y,z\n";
    assert_eq!(
        edit_value(t, "a.y", &s("q,w")).unwrap(),
        "a: {x:1}\na.y: q,w\n"
    );
}

// --- inline-массив: `::` — данные, а не маркер --------------------------------

#[test]
fn inline_array_item_coloncolon_is_literal_data() {
    let t = "[:: old]\n";
    assert_eq!(edit_value(t, "0", &s("new")).unwrap(), "[new]\n");
}

#[test]
fn inline_array_item_string_with_marker_meaning() {
    // String ":: new" в inline-элементе — literal ':: new'.
    let t = "[x]\n";
    assert_eq!(edit_value(t, "0", &s(":: new")).unwrap(), "[:: new]\n");
    let obj = parse("[:: new]\n").unwrap();
    assert_eq!(obj.as_array().unwrap().first(), Some(&s(":: new")));
}

// --- multiline-БЛОК: роль (pair/item) независима от формы ----------------------

#[test]
fn block_item_empty_string_becomes_unit_form() {
    // Пустая String в элементе: `()` парсится как Scalar("").
    let t = "a: [\n(\nx\n)\n]\n";
    assert_eq!(edit_value(t, "a.0", &s("")).unwrap(), "a: [\n()\n]\n");
}

#[test]
fn block_item_comment_like_string_gets_item_raw() {
    let t = "a: [\n(\nx\n)\n]\n";
    assert_eq!(
        edit_value(t, "a.0", &s("## text")).unwrap(),
        "a: [\n:: ## text\n]\n"
    );
    let obj = parse("a: [\n:: ## text\n]\n").unwrap();
    assert_eq!(
        obj.as_object()
            .unwrap()
            .get("a")
            .unwrap()
            .as_array()
            .unwrap()
            .first(),
        Some(&s("## text"))
    );
}

#[test]
fn block_item_closer_string_gets_item_raw() {
    let t = "a: [\n(\nx\n)\n]\n";
    assert_eq!(edit_value(t, "a.0", &s("]")).unwrap(), "a: [\n:: ]\n]\n");
    let obj = parse("a: [\n:: ]\n]\n").unwrap();
    assert_eq!(
        obj.as_object()
            .unwrap()
            .get("a")
            .unwrap()
            .as_array()
            .unwrap()
            .first(),
        Some(&s("]"))
    );
}

#[test]
fn block_pair_empty_string_stays_valid() {
    // Pair-роль блока: пустая String — пустое тело после `: `.
    let t = "s: (\nx\n)\n";
    assert_eq!(edit_value(t, "s", &s("")).unwrap(), "s: \n");
    let obj = parse("s: \n").unwrap();
    assert_eq!(obj.as_object().unwrap().get("s"), Some(&s("")));
}

// --- doc-пример (README, все 3 языка) -----------------------------------------

#[test]
fn doc_example_source_edit() -> Result<(), ktav::edit::EditError> {
    use ktav::edit::{edit_value, edit_value_segments};
    use ktav::Value;
    let source = "## top\n\na: 1\n\n## end\n";
    let replacement = Value::Integer("2".into());
    let expected = "## top\n\na: 2\n\n## end\n";
    assert_eq!(edit_value(source, "a", &replacement)?, expected);
    assert_eq!(edit_value_segments(source, &["a"], &replacement)?, expected);
    let path = vec![String::from("a")];
    assert_eq!(edit_value_segments(source, &path, &replacement)?, expected);
    Ok(())
}

// --- типосохраняющий inline-writer: точные байты -------------------------------

#[test]
fn inline_item_keyword_string_gets_unicode_provenance() {
    let t = "[old]\n";
    assert_eq!(edit_value(t, "0", &s("true")).unwrap(), "[\\u0074rue]\n");
    let obj = parse("[\\u0074rue]\n").unwrap();
    assert_eq!(obj.as_array().unwrap().first(), Some(&s("true")));
}

#[test]
fn inline_item_numeric_string_gets_unicode_provenance() {
    let t = "[old]\n";
    assert_eq!(edit_value(t, "0", &s("42")).unwrap(), "[\\u00342]\n");
    let obj = parse("[\\u00342]\n").unwrap();
    assert_eq!(obj.as_array().unwrap().first(), Some(&s("42")));
}

#[test]
fn inline_item_empty_string_becomes_unit() {
    let t = "[old]\n";
    assert_eq!(edit_value(t, "0", &s("")).unwrap(), "[()]\n");
    let obj = parse("[()]\n").unwrap();
    assert_eq!(obj.as_array().unwrap().first(), Some(&s("")));
}

#[test]
fn inline_raw_pair_to_int_drops_only_second_colon() {
    // 'a: {x::  old}' → Integer 9: второй ':' снят, двойной пробел
    // сохранён.
    let t = "a: {x::  old}\n";
    assert_eq!(
        edit_value(t, "a.x", &Value::Integer("9".into())).unwrap(),
        "a: {x:  9}\n"
    );
}

#[test]
fn inline_raw_pair_keeps_marker_for_string() {
    let t = "a: {x:: old}\n";
    assert_eq!(edit_value(t, "a.x", &s("y}z")).unwrap(), "a: {x:: y\\}z}\n");
    let obj = parse("a: {x:: y\\}z}\n").unwrap();
    let inner = obj
        .as_object()
        .unwrap()
        .get("a")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(inner.get("x"), Some(&s("y}z")));
}

#[test]
fn inline_raw_pair_to_compound_switches_to_plain() {
    // raw-тело literal — контейнер под raw невозможен: снят только
    // второй ':', escape-провенанс сохраняет String-детей.
    let t = "a: {x:: old}\n";
    let mut obj = ktav::value::ObjectMap::default();
    obj.insert("v".into(), s("true"));
    assert_eq!(
        edit_value(t, "a.x", &Value::Object(obj)).unwrap(),
        "a: {x: {v: \\u0074rue}}\n"
    );
    let parsed = parse("a: {x: {v: \\u0074rue}}\n").unwrap();
    let inner = parsed
        .as_object()
        .unwrap()
        .get("a")
        .unwrap()
        .as_object()
        .unwrap()
        .get("x")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(inner.get("v"), Some(&s("true")));
}

#[test]
fn inline_compound_empty_string_array_element() {
    let t = "a: {x:: old}\n";
    let mut obj = ktav::value::ObjectMap::default();
    obj.insert("v".into(), Value::Array(vec![s("")]));
    assert_eq!(
        edit_value(t, "a.x", &Value::Object(obj)).unwrap(),
        "a: {x: {v: [()]}}\n"
    );
    let parsed = parse("a: {x: {v: [()]}}\n").unwrap();
    let inner = parsed
        .as_object()
        .unwrap()
        .get("a")
        .unwrap()
        .as_object()
        .unwrap()
        .get("x")
        .unwrap()
        .as_object()
        .unwrap()
        .get("v")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(inner.first(), Some(&s("")));
}

#[test]
fn whole_inline_container_target_is_line_bare() {
    // Цель — весь inline-контейнер: он привязан к строке пары
    // (inline=false) → запятая дословна, без escape.
    let t = "a: {x:1}\n";
    assert_eq!(edit_value(t, "a", &s("y,z")).unwrap(), "a: y,z\n");
}

// --- Block: pair-роль, маркерные переходы с сохранением WS ----------------------

#[test]
fn block_pair_plain_string_replacement() {
    let t = "s: (\nx\n)\n";
    assert_eq!(edit_value(t, "s", &s("new")).unwrap(), "s: new\n");
}

#[test]
fn block_pair_numeric_string_switches_to_raw_keeps_ws() {
    // 's:\t (\nx\n)\n' + String "42" → 's::\t 42\n': второй ':' добавлен,
    // исходный WS после ':' сохранён.
    let t = "s:\t (\nx\n)\n";
    assert_eq!(edit_value(t, "s", &s("42")).unwrap(), "s::\t 42\n");
    let obj = parse("s::\t 42\n").unwrap();
    assert_eq!(obj.as_object().unwrap().get("s"), Some(&s("42")));
}

// --- WS-пресервация при неизменном/сменённом маркере: точные байты -----------

#[test]
fn plain_two_spaces_preserved() {
    let t = "a:  old\n";
    assert_eq!(edit_value(t, "a", &s("new")).unwrap(), "a:  new\n");
}

#[test]
fn raw_tab_preserved_for_string() {
    let t = "k::\tv\n";
    assert_eq!(edit_value(t, "k", &s("z")).unwrap(), "k::\tz\n");
}

#[test]
fn item_raw_to_plain_keeps_after_marker_ws() {
    // '::' снят, но WS между маркером и телом сохранён (продолжает
    // строку после отступа): '  ::\t 9' → '  \t 10'.
    let t = "  ::\t 9\nsecond\n";
    assert_eq!(
        edit_value(t, "0", &Value::Integer("10".into())).unwrap(),
        "  \t 10\nsecond\n"
    );
}

#[test]
fn raw_pair_to_int_keeps_exact_ws() {
    // 'n:: 8' → 'n: 9': снят только второй ':', исходный WS сохранён.
    let t = "n:: 8\n";
    assert_eq!(
        edit_value(t, "n", &Value::Integer("9".into())).unwrap(),
        "n: 9\n"
    );
}

#[test]
fn non_inline_backslash_is_literal() {
    // Вне inline escape-декодирования нет: тело вставляется дословно и
    // парсится в тот же String с обратным слэшем.
    let t = "a: x\n";
    assert_eq!(edit_value(t, "a", &s("a\\b")).unwrap(), "a: a\\b\n");
    assert_eq!(
        parse("a: a\\b\n").unwrap().as_object().unwrap().get("a"),
        Some(&s("a\\b"))
    );
}

#[test]
fn leading_zero_index_is_digits_context() {
    // Только ASCII-цифры; ведущие нули допустимы (01 == 1), знак/плюс —
    // не индекс (задокументировано в ktav::edit).
    let t = "list: [\n  one\n  two\n]\n";
    let segs = vec!["list".to_string(), "01".to_string()];
    assert_eq!(
        edit_value_segments(t, &segs, &s("TWO")).unwrap(),
        "list: [\n  one\n  TWO\n]\n"
    );
    let segs_plus = vec!["list".to_string(), "+1".to_string()];
    assert_eq!(
        edit_value_segments(t, &segs_plus, &s("x")),
        Err(EditError::NotFound)
    );
}

// --- маркерные переходы: точные байты (trace parent-кейсов) ------------------

#[test]
fn plain_to_raw_keeps_wide_spacing() {
    let t = "n:  5\n";
    assert_eq!(edit_value(t, "n", &s("42")).unwrap(), "n::  42\n");
}

#[test]
fn raw_to_multiline_switches_to_plain_block() {
    // raw не открывает multiline-блок → снятие только второго `:`,
    // блок под plain-маркером, пробел сохранён.
    let t = "k:: x\n";
    assert_eq!(edit_value(t, "k", &s("a\nb")).unwrap(), "k: ((\na\nb\n))\n");
}

#[test]
fn item_raw_to_plain_drops_marker_keeps_indent() {
    let t = ":: 8080\nsecond\n";
    assert_eq!(
        edit_value(t, "0", &Value::Integer("9090".into())).unwrap(),
        " 9090\nsecond\n"
    );
}

#[test]
fn scalar_to_empty_array_and_back() {
    let t = "a: 1\n";
    assert_eq!(
        edit_value(t, "a", &Value::Array(Vec::new())).unwrap(),
        "a: []\n"
    );
    assert_eq!(
        edit_value("a: []\n", "a", &Value::Integer("1".into())).unwrap(),
        "a: 1\n"
    );
}

// --- no-op: байт-в-байт identical, до comment/noncontiguous отказов ----------

#[test]
fn no_op_lexical_number_spelling_preserved() {
    // '1_000' парсится как Integer(1000); замена тем же значением —
    // исходная ЛЕКСИКА сохраняется.
    let t = "n: 1_000\n";
    assert_eq!(
        edit_value(t, "n", &Value::Integer("1000".into())).unwrap(),
        t
    );
}

#[test]
fn no_op_commented_container_bypasses_comment_refusal() {
    let t = "a: {\n  ## c\n  x: 1\n}\n";
    let mut obj = ktav::value::ObjectMap::default();
    obj.insert("x".into(), Value::Integer("1".into()));
    assert_eq!(edit_value(t, "a", &Value::Object(obj)).unwrap(), t);
}

#[test]
fn no_op_noncontiguous_bypasses_unsupported() {
    let t = "a: {}\na.y: 2\n";
    let mut y = ktav::value::ObjectMap::default();
    y.insert("y".into(), Value::Integer("2".into()));
    assert_eq!(edit_value(t, "a", &Value::Object(y)).unwrap(), t);
}

#[test]
fn unicode_whitespace_comment_detected_and_preserved() {
    // U+00A0 — член § 3.3: строка '  \u{00A0}## c' — комментарий.
    let t = "a: {\n  \u{00A0}## c\n  x: 1\n}\n";
    let mut obj = ktav::value::ObjectMap::default();
    obj.insert("y".into(), Value::Integer("2".into()));
    assert!(matches!(
        edit_value(t, "a", &Value::Object(obj)),
        Err(EditError::Unsupported { .. })
    ));
    // Leaf внутри — комментарии сохранены байт-в-байт.
    assert_eq!(
        edit_value(t, "a.x", &Value::Integer("2".into())).unwrap(),
        t.replace("x: 1", "x: 2")
    );
}

#[test]
fn shrinking_large_value_does_not_retain_source_capacity() {
    let source = format!("target:: {}\n", "x".repeat(64 * 1024));
    let edited = edit_value(&source, "target", &Value::Integer("1".into())).unwrap();
    assert_eq!(edited, "target: 1\n");
    assert!(
        edited.capacity() <= 64,
        "retained {} bytes",
        edited.capacity()
    );
}

#[test]
fn missing_path_still_validates_source_before_lookup() {
    assert!(matches!(
        edit_value("a: {\n", "missing", &Value::Null),
        Err(EditError::Unsupported { .. })
    ));
}

#[test]
fn streaming_inline_writer_preserves_mixed_values() {
    let mut object = ktav::ObjectMap::default();
    object.insert("flag".into(), s("true"));
    object.insert(
        "items".into(),
        Value::Array(vec![
            s(""),
            Value::Bool(false),
            Value::Null,
            Value::Integer("2".into()),
        ]),
    );
    assert_eq!(
        edit_value("a: {x: old}\n", "a.x", &Value::Object(object)).unwrap(),
        "a: {x: {flag: \\u0074rue, items: [(), false, null, 2]}}\n"
    );
}

#[test]
fn streaming_inline_writer_preserves_utf8_edge_whitespace() {
    assert_eq!(
        edit_value("a: {x: old}\n", "a.x", &s(" café\u{00a0}")).unwrap(),
        "a: {x: \\u0020café\\u00A0}\n"
    );
}
