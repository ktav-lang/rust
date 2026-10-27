//! `ktav::edit::{insert_value, remove_value, rename_key}` и их
//! segments-варианты. Оракулы двух независимых видов: (1) литеральная
//! строка результата — доказывает нетронутые байты вне правки;
//! (2) независимо построенное ожидаемое `Value` (и, где важен порядок,
//! отдельно снятая последовательность ключей), сравниваемое с разбором
//! результата. Негативные тесты проверяют typed Err; ключевые сценарии
//! дополнительно проверяют независимо построенное дерево. Мутация регрессии (например,
//! `shift_remove` -> установка Null или вставка без EOL перед closer)
//! ломает литеральные строки.

use ktav::edit::{
    insert_value, insert_value_segments, remove_value, remove_value_segments, rename_key,
    rename_key_segments, EditError,
};
use ktav::parse;
use ktav::value::{ObjectMap, Value};

fn s(v: &str) -> Value {
    Value::String(v.into())
}

fn obj(pairs: &[(&str, Value)]) -> Value {
    let mut o = ObjectMap::default();
    for (k, v) in pairs {
        o.insert((*k).into(), v.clone());
    }
    Value::Object(o)
}

fn keys_of(v: &Value) -> Vec<&str> {
    match v {
        Value::Object(o) => o.keys().map(|k| k.as_str()).collect(),
        _ => panic!("не объект"),
    }
}

// --- insert: неявный корень и финальный EOL ----------------------------------

#[test]
fn insert_appends_to_implicit_root_keeps_final_eol() {
    let t = "a: 1\n";
    assert_eq!(
        insert_value(t, "b", &Value::Integer("2".into())).unwrap(),
        "a: 1\nb: 2\n"
    );
}

#[test]
fn insert_to_root_without_final_eol_adds_none() {
    let t = "a: 1";
    assert_eq!(insert_value(t, "b", &s("v")).unwrap(), "a: 1\nb: v");
}

#[test]
fn insert_to_root_appends_after_trailing_comment() {
    // Комментарий остаётся на своих строках, байты не сдвигаются.
    let t = "a: 1\n## tail\n";
    assert_eq!(
        insert_value(t, "b", &s("v")).unwrap(),
        "a: 1\n## tail\nb: v\n"
    );
}

#[test]
fn insert_to_root_crlf_and_lone_cr() {
    assert_eq!(
        insert_value("a: 1\r\n", "b", &s("v")).unwrap(),
        "a: 1\r\nb: v\r\n"
    );
    assert_eq!(
        insert_value("a: 1\rb: 2", "c", &s("v")).unwrap(),
        "a: 1\rb: 2\rc: v"
    );
}

#[test]
fn insert_to_root_with_bom() {
    let t = "\u{FEFF}a: 1\n";
    assert_eq!(
        insert_value(t, "b", &s("v")).unwrap(),
        "\u{FEFF}a: 1\nb: v\n"
    );
}

#[test]
fn insert_to_comment_only_and_bom_only_documents() {
    assert_eq!(
        insert_value("## header\n", "k", &s("v")).unwrap(),
        "## header\nk: v\n"
    );
    assert_eq!(
        insert_value("\u{FEFF}", "k", &s("v")).unwrap(),
        "\u{FEFF}\nk: v"
    );
}

#[test]
fn insert_into_empty_inline_root_stays_inline() {
    // Короткая читаемая вставка сохраняет inline-форму пустого корня.
    assert_eq!(
        insert_value("{}\n", "k", &Value::Integer("1".into())).unwrap(),
        "{k: 1}\n"
    );
    assert_eq!(insert_value("{}", "k", &s("v")).unwrap(), "{k: v}");
}

#[test]
fn insert_into_empty_explicit_block_root() {
    // Отдельный блок-кейс: явный корень, открытый строкой `{`.
    assert_eq!(
        insert_value("{\n}\n", "k", &s("v")).unwrap(),
        "{\n    k: v\n}\n"
    );
}

#[test]
fn insert_to_dotted_reopen_root() {
    // Корень из dotted/reopen-определений: строка в конец документа.
    let t = "a.b: 1\na.c: 2\n";
    assert_eq!(
        insert_value(t, "d", &s("v")).unwrap(),
        "a.b: 1\na.c: 2\nd: v\n"
    );
}

// --- insert: блок и inline ----------------------------------------------------

#[test]
fn insert_appends_own_line_to_block_object() {
    // Отступ по source (как у соседей); перед closer'ом — EOL.
    let t = "a: {\n  x: 1\n}\nb: 2\n";
    assert_eq!(
        insert_value(t, "a.k", &s("v")).unwrap(),
        "a: {\n  x: 1\n  k: v\n}\nb: 2\n"
    );
}

#[test]
fn insert_into_empty_block_object() {
    let t = "a: {\n}\n";
    assert_eq!(
        insert_value(t, "a.k", &s("v")).unwrap(),
        "a: {\n    k: v\n}\n"
    );
}

#[test]
fn insert_into_block_object_crlf() {
    let t = "a: {\r\n}\r\n";
    assert_eq!(
        insert_value(t, "a.k", &s("v")).unwrap(),
        "a: {\r\n    k: v\r\n}\r\n"
    );
}

#[test]
fn insert_inline_trailing_structural_comma_root_and_nested() {
    let cases = [
        ("{x: 1,}\n", "{x: 1, y: 2,}\n", false),
        ("{x: 1,   }\n", "{x: 1, y: 2,   }\n", false),
        ("a: {x: 1,}\n", "a: {x: 1, y: 2,}\n", true),
        ("a: {x: 1,   }\n", "a: {x: 1, y: 2,   }\n", true),
    ];
    for (source, expected_text, nested) in cases {
        let (path, segments): (&str, &[&str]) = if nested {
            ("a.y", &["a", "y"])
        } else {
            ("y", &["y"])
        };
        let by_text = insert_value(source, path, &Value::Integer("2".into())).unwrap();
        let by_segments =
            insert_value_segments(source, segments, &Value::Integer("2".into())).unwrap();
        assert_eq!(by_text, expected_text);
        assert_eq!(by_segments, expected_text);

        let expected = if nested {
            obj(&[(
                "a",
                obj(&[
                    ("x", Value::Integer("1".into())),
                    ("y", Value::Integer("2".into())),
                ]),
            )])
        } else {
            obj(&[
                ("x", Value::Integer("1".into())),
                ("y", Value::Integer("2".into())),
            ])
        };
        assert_eq!(parse(&by_text).unwrap(), expected);
        assert_eq!(parse(&by_segments).unwrap(), expected);
    }
}

#[test]
fn insert_inline_trailing_comma_with_escaped_scalar_bytes() {
    let source = r#"{x: p\,q\\r,   }"#;
    let expected = r#"{x: p\,q\\r, y: 2,   }"#;
    let out = insert_value(source, "y", &Value::Integer("2".into())).unwrap();
    assert_eq!(out, expected);
    let semantic = obj(&[("x", s("p,q\\r")), ("y", Value::Integer("2".into()))]);
    assert_eq!(parse(&out).unwrap(), semantic);
}

#[test]
fn insert_inline_escaped_comma_and_backslash_without_structural_trailing_comma() {
    for (source, expected, expected_value) in [
        (r#"{x: p\,q}"#, r#"{x: p\,q, y: 2}"#, s("p,q")),
        (r#"{x: p\,}"#, r#"{x: p\,, y: 2}"#, s("p,")),
        (r#"{x: p\\}"#, r#"{x: p\\, y: 2}"#, s("p\\")),
        (r#"{x: p\\,}"#, r#"{x: p\\, y: 2,}"#, s("p\\")),
        (r#"a: {x: p\,q   }"#, r#"a: {x: p\,q   , y: 2}"#, s("p,q")),
    ] {
        let out = insert_value(
            source,
            if source.starts_with("a:") { "a.y" } else { "y" },
            &Value::Integer("2".into()),
        )
        .unwrap();
        assert_eq!(out, expected);
        let expected_semantic = if source.starts_with("a:") {
            obj(&[(
                "a",
                obj(&[("x", expected_value), ("y", Value::Integer("2".into()))]),
            )])
        } else {
            obj(&[("x", expected_value), ("y", Value::Integer("2".into()))])
        };
        assert_eq!(parse(&out).unwrap(), expected_semantic);
    }
}

#[test]
fn insert_appends_segment_to_inline_object() {
    let t = "a: {x: 1}\n";
    assert_eq!(
        insert_value(t, "a.y", &Value::Integer("2".into())).unwrap(),
        "a: {x: 1, y: 2}\n"
    );
}

#[test]
fn insert_into_empty_inline_object() {
    assert_eq!(
        insert_value("a: {}\n", "a.k", &s("v")).unwrap(),
        "a: {k: v}\n"
    );
}

#[test]
fn insert_inline_string_with_comma_is_escaped() {
    let bs = char::from_u32(0x5C).unwrap();
    // Тело 'p,q': запятая экранируется inline-энкодером.
    assert_eq!(
        insert_value("a: {x: 1}\n", "a.k", &s("p,q")).unwrap(),
        format!("a: {{x: 1, k: p{bs},q}}\n")
    );
}

#[test]
fn insert_inline_numeric_looking_string_gets_provenance() {
    let bs = char::from_u32(0x5C).unwrap();
    assert_eq!(
        insert_value("a: {x: 1}\n", "a.k", &s("42")).unwrap(),
        format!("a: {{x: 1, k: {bs}u00342}}\n")
    );
}

#[test]
fn insert_multiline_string_expands_inline_parent_to_block() {
    let t = "a: {x: 1}\n";
    assert_eq!(
        insert_value(t, "a.k", &s("p\nq")).unwrap(),
        "a: {\n    x: 1\n    k: ((\np\nq\n    ))\n}\n"
    );
}

#[test]
fn insert_into_object_inside_inline_array_element() {
    let t = "top: [{a: 1}]\n";
    assert_eq!(
        insert_value(t, "top.0.c", &s("v")).unwrap(),
        "top: [{a: 1, c: v}]\n"
    );
}

#[test]
fn insert_into_object_inside_block_array_element() {
    let t = "top: [\n  {\n    a: 1\n  }\n]\n";
    assert_eq!(
        insert_value(t, "top.0.b", &s("v")).unwrap(),
        "top: [\n  {\n    a: 1\n    b: v\n  }\n]\n"
    );
}

#[test]
fn insert_container_replacement_renders_block() {
    let t = "a: 1\n";
    let repl = obj(&[("x", Value::Integer("1".into())), ("y", s("z"))]);
    assert_eq!(
        insert_value(t, "n", &repl).unwrap(),
        "a: 1\nn: {\n    x: 1\n    y: z\n}\n"
    );
}

// --- insert: ключи ------------------------------------------------------------

#[test]
fn insert_quoted_key_for_structural_segment() {
    // Общий key emitter выбирает quoted-форму для '.' — литеральный
    // оракул следует эмиттеру (bare `a\.b` эмиттер не выпускает).
    let t = "a: 1\n";
    assert_eq!(
        insert_value(t, r"a\.b", &s("v")).unwrap(),
        "a: 1\n\"a.b\": v\n"
    );
    assert_eq!(
        insert_value_segments(t, &["a.b"], &s("v")).unwrap(),
        "a: 1\n\"a.b\": v\n"
    );
    assert_eq!(
        insert_value(t, "x,y", &s("v")).unwrap(),
        "a: 1\n\"x,y\": v\n"
    );
}

#[test]
fn insert_backslash_key_is_bare_escaped() {
    let bs = char::from_u32(0x5C).unwrap();
    // Ключ x\y -> исходник x\y (bare-escape backslash).
    assert_eq!(
        insert_value_segments("k: 1\n", &["x\\y"], &s("v")).unwrap(),
        format!("k: 1\nx{0}{0}y: v\n", bs)
    );
}

#[test]
fn insert_numeric_object_key_is_a_key() {
    // Позитивный тест: `0` внутри Object — строковый ключ, не индекс.
    assert_eq!(
        insert_value("a: 1\n", "0", &s("v")).unwrap(),
        "a: 1\n0: v\n"
    );
}

// --- insert: ошибки -----------------------------------------------------------

#[test]
fn insert_existing_key_is_already_exists_never_overwrite() {
    let t = "a: 1\n";
    assert_eq!(insert_value(t, "a", &s("x")), Err(EditError::AlreadyExists));
    assert_eq!(
        insert_value(t, "a", &s("x")).unwrap_err().to_string(),
        "edit target already exists; insertion never overwrites"
    );
}

#[test]
fn insert_missing_intermediate_and_scalar_descent_not_found() {
    let t = "a: 1\n";
    assert_eq!(insert_value(t, "b.c", &s("v")), Err(EditError::NotFound));
    assert_eq!(insert_value(t, "a.c", &s("v")), Err(EditError::NotFound));
    assert_eq!(
        insert_value_segments(t, &[] as &[&str], &s("v")),
        Err(EditError::NotFound)
    );
    assert_eq!(
        insert_value(t, "", &s("v")),
        Err(EditError::InvalidPath { offset: 0 })
    );
}

#[test]
fn insert_into_array_is_explicit_unsupported() {
    let t = "list: [x]\n";
    let e = insert_value(t, "list.1", &s("v")).unwrap_err();
    assert!(matches!(e, EditError::Unsupported { .. }), "{e}");
    assert_eq!(
        insert_value(t, "list", &s("v")),
        Err(EditError::AlreadyExists)
    );
}

#[test]
fn insert_into_noncontiguous_parent_is_unsupported() {
    let t = "a: {}\na.y: 1\n";
    let e = insert_value(t, "a.k", &s("v")).unwrap_err();
    assert!(matches!(e, EditError::Unsupported { .. }), "{e}");
}

#[test]
fn insert_invalid_source_fails_closed_unsupported() {
    let e = insert_value("a: {\n", "k", &s("v")).unwrap_err();
    assert!(matches!(e, EditError::Unsupported { .. }));
}

// --- remove: строки и предки --------------------------------------------------

#[test]
fn remove_leaf_first_middle_last_and_eof() {
    assert_eq!(
        remove_value("a: 1\nb: 2\nc: 3\n", "a").unwrap(),
        "b: 2\nc: 3\n"
    );
    assert_eq!(
        remove_value("a: 1\nb: 2\nc: 3\n", "b").unwrap(),
        "a: 1\nc: 3\n"
    );
    assert_eq!(
        remove_value("a: 1\nb: 2\nc: 3\n", "c").unwrap(),
        "a: 1\nb: 2\n"
    );
    // Хвост без финального EOL: предыдущий терминатор уходит, свой
    // финальный EOL не придумывается.
    assert_eq!(remove_value("a: 1\nb: 2", "b").unwrap(), "a: 1");
    // Единственный член: пустой документ == пустой корень.
    assert_eq!(remove_value("a: 1\n", "a").unwrap(), "");
}

#[test]
fn remove_only_dotted_leaf_materializes_empty_parent() {
    let t = "a.b: 1\nc: 2\n";
    let out = remove_value(t, "a.b").unwrap();
    // `a` обязан остаться пустым Object (no-prune): строка материализуется.
    assert_eq!(out, "a: {}\nc: 2\n");
    assert_eq!(
        parse(&out).unwrap(),
        obj(&[
            ("a", Value::Object(ObjectMap::default())),
            ("c", Value::Integer("2".into()))
        ])
    );
}

#[test]
fn remove_deep_dotted_chain_materializes_full_path() {
    let t = "a.b.c: 1\nd: 2\n";
    let out = remove_value(t, "a.b.c").unwrap();
    assert_eq!(out, "a.b: {}\nd: 2\n");
    assert_eq!(
        parse(&out).unwrap(),
        obj(&[
            ("a", obj(&[("b", Value::Object(ObjectMap::default()))])),
            ("d", Value::Integer("2".into()))
        ])
    );
}

#[test]
fn remove_materializes_only_emptied_chain_with_sibling_kept() {
    let t = "a.b.c: 1\na.q: 2\n";
    let out = remove_value(t, "a.b.c").unwrap();
    assert_eq!(out, "a.b: {}\na.q: 2\n");
    assert_eq!(
        parse(&out).unwrap(),
        obj(&[(
            "a",
            obj(&[
                ("b", Value::Object(ObjectMap::default())),
                ("q", Value::Integer("2".into()))
            ])
        )])
    );
}

#[test]
fn remove_keeps_nonempty_dotted_parent() {
    let t = "a.b: 1\na.c: 2\n";
    let out = remove_value(t, "a.b").unwrap();
    assert_eq!(out, "a.c: 2\n");
    assert_eq!(
        parse(&out).unwrap(),
        obj(&[("a", obj(&[("c", Value::Integer("2".into()))]))])
    );
}

#[test]
fn remove_reopen_with_explicit_empty_definition_needs_no_materialization() {
    // `a: {}` уже определён; чистое вырезание строки reopen достаточно,
    // вторая `a: {}` не дублируется.
    let t = "a: {}\na.y: 1\n";
    let out = remove_value(t, "a.y").unwrap();
    assert_eq!(out, "a: {}\n");
    assert_eq!(
        parse(&out).unwrap(),
        obj(&[("a", Value::Object(ObjectMap::default()))])
    );
}

#[test]
fn remove_inside_block_keeps_block_form() {
    assert_eq!(
        remove_value("a: {\n  b: 1\n  c: 2\n}\nd: 3\n", "a.b").unwrap(),
        "a: {\n  c: 2\n}\nd: 3\n"
    );
    assert_eq!(
        remove_value("a: {\n  b: 1\n}\n", "a.b").unwrap(),
        "a: {\n}\n"
    );
}

// --- remove: inline и массивы -------------------------------------------------

#[test]
fn remove_inline_member_first_middle_last_only() {
    let t = "a: {x: 1, y: 2, z: 3}\n";
    assert_eq!(remove_value(t, "a.x").unwrap(), "a: {y: 2, z: 3}\n");
    assert_eq!(remove_value(t, "a.y").unwrap(), "a: {x: 1, z: 3}\n");
    assert_eq!(remove_value(t, "a.z").unwrap(), "a: {x: 1, y: 2}\n");
    assert_eq!(remove_value("a: {x: 1}\n", "a.x").unwrap(), "a: {}\n");
}

#[test]
fn remove_inline_array_element_first_middle_last_only() {
    assert_eq!(
        remove_value("a: [1, 2, 3]\n", "a.0").unwrap(),
        "a: [2, 3]\n"
    );
    assert_eq!(
        remove_value("a: [1, 2, 3]\n", "a.1").unwrap(),
        "a: [1, 3]\n"
    );
    assert_eq!(
        remove_value("a: [1, 2, 3]\n", "a.2").unwrap(),
        "a: [1, 2]\n"
    );
    assert_eq!(remove_value("a: [1]\n", "a.0").unwrap(), "a: []\n");
}

#[test]
fn remove_block_array_element_lines() {
    let t = "a: [\n1\n2\n3\n]\n";
    assert_eq!(remove_value(t, "a.0").unwrap(), "a: [\n2\n3\n]\n");
    assert_eq!(remove_value(t, "a.1").unwrap(), "a: [\n1\n3\n]\n");
    assert_eq!(remove_value(t, "a.2").unwrap(), "a: [\n1\n2\n]\n");
}

#[test]
fn remove_in_deep_mixed_form_arrays() {
    let t = "top: [\n  {\n    inner: old\n    keep: 2\n  }\n  {q: 1}\n]\n";
    // Лист внутри блочного элемента массива.
    let out = remove_value(t, "top.0.inner").unwrap();
    assert_eq!(out, "top: [\n  {\n    keep: 2\n  }\n  {q: 1}\n]\n");
    assert_eq!(
        parse(&out).unwrap(),
        obj(&[(
            "top",
            Value::Array(vec![
                obj(&[("keep", Value::Integer("2".into()))]),
                obj(&[("q", Value::Integer("1".into()))])
            ])
        )])
    );
    // Единственный член inline-элемента.
    let out2 = remove_value(t, "top.1.q").unwrap();
    assert_eq!(
        out2,
        "top: [\n  {\n    inner: old\n    keep: 2\n  }\n  {}\n]\n"
    );
}

#[test]
fn remove_whole_subtree_shifts_array() {
    let t = "a: {\n  x: 1\n}\nb: [p, q]\nc: 3\n";
    let out = remove_value(t, "a").unwrap();
    assert_eq!(out, "b: [p, q]\nc: 3\n");
    let mut expected = ObjectMap::default();
    expected.insert("b".into(), Value::Array(vec![s("p"), s("q")]));
    expected.insert("c".into(), Value::Integer("3".into()));
    assert_eq!(parse(&out).unwrap(), Value::Object(expected));
}

#[test]
fn remove_segments_success_path() {
    assert_eq!(
        remove_value_segments("a: 1\nb: 2\n", &["a"]).unwrap(),
        "b: 2\n"
    );
}

#[test]
fn remove_preserves_adjacent_comments_and_eols() {
    let t = "## top\na: 1\n## mid\nb: 2\n## end\n";
    assert_eq!(
        remove_value(t, "a").unwrap(),
        "## top\n## mid\nb: 2\n## end\n"
    );
    assert_eq!(remove_value("a: 1\r\nb: 2\r\n", "a").unwrap(), "b: 2\r\n");
    assert_eq!(remove_value("a: 1\rb: 2", "a").unwrap(), "b: 2");
}

#[test]
fn remove_keeps_bom_for_first_middle_only_member() {
    // BOM — метаданные исходника: не входит в диапазон удаления.
    assert_eq!(
        remove_value("\u{FEFF}a: 1\nb: 2\n", "a").unwrap(),
        "\u{FEFF}b: 2\n"
    );
    assert_eq!(
        remove_value("\u{FEFF}a: 1\nb: 2\n", "b").unwrap(),
        "\u{FEFF}a: 1\n"
    );
    assert_eq!(remove_value("\u{FEFF}a: 1", "a").unwrap(), "\u{FEFF}");
}

#[test]
fn remove_multiline_scalar_block_lines() {
    let t = "a: 1\ns: ((\nline1\nline2\n))\nb: 2\n";
    assert_eq!(remove_value(t, "s").unwrap(), "a: 1\nb: 2\n");
}

// --- remove: ошибки -----------------------------------------------------------

#[test]
fn remove_not_found_cases() {
    let t = "a: 1\n";
    assert_eq!(remove_value(t, "b"), Err(EditError::NotFound));
    assert_eq!(remove_value(t, "a.x"), Err(EditError::NotFound));
    assert_eq!(remove_value(t, "list.0"), Err(EditError::NotFound));
    assert_eq!(
        remove_value_segments(t, &[] as &[&str]),
        Err(EditError::NotFound)
    );
    assert_eq!(
        remove_value(t, ""),
        Err(EditError::InvalidPath { offset: 0 })
    );
    assert_eq!(
        remove_value(t, "a..b"),
        Err(EditError::InvalidPath { offset: 2 })
    );
}

#[test]
fn remove_noncontiguous_whole_target_is_unsupported_leaf_editable() {
    let t = "a: {}\na.y: 1\n";
    let e = remove_value(t, "a").unwrap_err();
    assert!(matches!(e, EditError::Unsupported { .. }), "{e}");
}

#[test]
fn remove_compound_with_internal_comments_is_unsupported() {
    let t = "a: {\n  ## inner\n  x: 1\n}\nb: 2\n";
    let e = remove_value(t, "a").unwrap_err();
    assert!(matches!(e, EditError::Unsupported { .. }), "{e}");
}

#[test]
fn remove_invalid_source_fails_closed_unsupported() {
    let e = remove_value("a: {\n", "a").unwrap_err();
    assert!(matches!(e, EditError::Unsupported { .. }));
}

// --- rename -------------------------------------------------------------------

#[test]
fn rename_preserves_position_order_and_value_exact_bytes() {
    let t = "a: 1\nb: 2\nc: 3\n";
    let out = rename_key(t, "b", "x").unwrap();
    assert_eq!(out, "a: 1\nx: 2\nc: 3\n");
    // Порядок ключей — отдельно от unordered-равенства Value.
    let doc = parse(&out).unwrap();
    assert_eq!(keys_of(&doc), vec!["a", "x", "c"]);
}

#[test]
fn rename_identical_key_is_byte_identical_no_op() {
    let t = "k:: 042\n";
    assert_eq!(rename_key(t, "k", "k").unwrap(), t);
}

#[test]
fn rename_replaces_only_last_dotted_segment() {
    // Path a.b адресует член b dotted-родителя a: меняется только b.
    let t = "a.b: 1\nz: 2\n";
    let out = rename_key(t, "a.b", "c").unwrap();
    assert_eq!(out, "a.c: 1\nz: 2\n");
    assert_eq!(
        parse(&out).unwrap(),
        obj(&[
            ("a", obj(&[("c", Value::Integer("1".into()))])),
            ("z", Value::Integer("2".into()))
        ])
    );
}

#[test]
fn rename_dotted_prefix_spelling_is_kept_verbatim() {
    // Префикс `x.` сохраняется байт-в-байт; последний сегмент
    // `y\.z` заменяется целиком.
    let t = "x.y\\.z: 1\n";
    let out = rename_key(t, r"x.y\.z", "m").unwrap();
    assert_eq!(out, "x.m: 1\n");
    assert_eq!(
        parse(&out).unwrap(),
        obj(&[("x", obj(&[("m", Value::Integer("1".into()))]))])
    );
}

#[test]
fn rename_literal_escaped_key_whole_segment() {
    // `a\.b` — ОДИН литеральный сегмент (экранированная точка), меняется
    // целиком; это другой адрес, чем dotted a.b.
    let t = "a\\.b: 1\nc: 2\n";
    let out = rename_key(t, r"a\.b", "m").unwrap();
    assert_eq!(out, "m: 1\nc: 2\n");
    assert_eq!(
        parse(&out).unwrap(),
        obj(&[
            ("m", Value::Integer("1".into())),
            ("c", Value::Integer("2".into()))
        ])
    );
}

#[test]
fn rename_quoted_segment_full_representation() {
    // Сегмент заменяется ЦЕЛИКОМ (кавычки входят в слайс): новый
    // bare-ключ даёт bare-написание, без смешивания представлений.
    let t = "\"q.k\": 1\nc: 2\n";
    let out = rename_key(t, r"q\.k", "m").unwrap();
    assert_eq!(out, "m: 1\nc: 2\n");
}

#[test]
fn rename_leaf_inside_reopened_object() {
    // Безопасный leaf rename в reopened-родителе работает.
    let t = "a.b: 1\na.q: 2\n";
    let out = rename_key(t, "a.b", "z").unwrap();
    assert_eq!(out, "a.z: 1\na.q: 2\n");
    assert_eq!(
        parse(&out).unwrap(),
        obj(&[(
            "a",
            obj(&[
                ("z", Value::Integer("1".into())),
                ("q", Value::Integer("2".into()))
            ])
        )])
    );
}

#[test]
fn rename_inside_block_preserves_comments_and_indent() {
    let t = "a: {\n  ## c\n  b: 1\n}\n";
    assert_eq!(
        rename_key(t, "a.b", "z").unwrap(),
        "a: {\n  ## c\n  z: 1\n}\n"
    );
}

#[test]
fn rename_inline_member_and_inline_array_element() {
    assert_eq!(
        rename_key("a: {x: 1, y: 2}\n", "a.x", "w").unwrap(),
        "a: {w: 1, y: 2}\n"
    );
    assert_eq!(
        rename_key("top: [{a: 1, b: 2}]\n", "top.0.a", "x").unwrap(),
        "top: [{x: 1, b: 2}]\n"
    );
}

#[test]
fn rename_emits_escaped_or_quoted_new_key() {
    // Точка в новом ключе: emitter выбирает quoted-форму.
    assert_eq!(rename_key("a: 1\n", "a", "x.y").unwrap(), "\"x.y\": 1\n");
    assert_eq!(rename_key("a: 1\n", "a", "x:y").unwrap(), "\"x:y\": 1\n");
    // Unicode — bare.
    assert_eq!(rename_key("a: 1\n", "a", "ключ").unwrap(), "ключ: 1\n");
    // Backslash — bare-escape (x\y в исходнике).
    let bs = char::from_u32(0x5C).unwrap();
    assert_eq!(
        rename_key("a: 1\n", "a", "x\\y").unwrap(),
        format!("x{0}{0}y: 1\n", bs)
    );
}

#[test]
fn rename_segments_literal_paths() {
    let t = "a.b: 1\nc: 2\n";
    // segments: без повторного unescape — b переименован в c.d.
    let out = rename_key_segments(t, &["a", "b"], "c.d").unwrap();
    assert_eq!(out, "a.\"c.d\": 1\nc: 2\n");
}

#[test]
fn rename_keeps_bom_and_separator_bytes() {
    assert_eq!(
        rename_key("\u{FEFF}a: 1\nb: 2\n", "a", "x").unwrap(),
        "\u{FEFF}x: 1\nb: 2\n"
    );
    // Пробелы вокруг разделителя не принадлежат ключу.
    assert_eq!(rename_key("k : v\n", "k", "m").unwrap(), "m : v\n");
    // Quoted-ключ соседа нетронут.
    assert_eq!(
        rename_key("\"a b\": 1\nc: 2\n", "c", "d").unwrap(),
        "\"a b\": 1\nd: 2\n"
    );
}

#[test]
fn rename_errors_collision_array_missing_empty() {
    let t = "a: 1\nb: 2\n";
    assert_eq!(rename_key(t, "a", "b"), Err(EditError::AlreadyExists));
    assert_eq!(rename_key(t, "x", "y"), Err(EditError::NotFound));
    assert_eq!(
        rename_key(t, "", "y"),
        Err(EditError::InvalidPath { offset: 0 })
    );
    assert_eq!(
        rename_key_segments(t, &[] as &[&str], "y"),
        Err(EditError::NotFound)
    );
    // Индекс массива — Unsupported.
    let e = rename_key("list: [p, q]\n", "list.0", "k").unwrap_err();
    assert!(matches!(e, EditError::Unsupported { .. }), "{e}");
    // Пустой новый ключ — невоспроизводим.
    let e = rename_key(t, "a", "").unwrap_err();
    assert!(matches!(e, EditError::Unsupported { .. }));
    // Целиком noncontiguous-член — Unsupported.
    let e = rename_key("a: {}\na.y: 1\n", "a", "z").unwrap_err();
    assert!(matches!(e, EditError::Unsupported { .. }), "{e}");
}

// --- inline fallback и минимальная цепочка ------------------------------------

#[test]
fn insert_cr_string_keeps_verified_inline_form() {
    let bs = char::from_u32(0x5C).unwrap();
    // CR в decoded Value представим inline-экранированием; standalone-блок
    // невыразим, но отказ запрещён — остаётся проверенная inline-форма.
    assert_eq!(
        insert_value("a: {x: 1}\n", "a.k", &s("a\rb")).unwrap(),
        format!("a: {{x: 1, k: a{bs}rb}}\n")
    );
}

#[test]
fn insert_multiline_with_close_marker_keeps_inline_form() {
    let bs = char::from_u32(0x5C).unwrap();
    // Строка )) делает verbatim-блок невыразимым — inline сохраняется.
    assert_eq!(
        insert_value("a: {x: 1}\n", "a.k", &s("a\n))\nb")).unwrap(),
        format!("a: {{x: 1, k: a{bs}n)){bs}nb}}\n")
    );
}

#[test]
fn insert_multiline_expands_minimal_inline_chain() {
    // Раскрывается вся цепочка внешних inline-обёрток до места правки.
    let t = "o: {i: {x: 1}}\n";
    assert_eq!(
        insert_value(t, "o.i.k", &s("p\nq")).unwrap(),
        "o: {\n    i: {\n        x: 1\n        k: ((\np\nq\n        ))\n    }\n}\n"
    );
}

#[test]
fn rename_single_and_backtick_quoted_keys_with_structural_bytes() {
    // , и { внутри single/backtick quoted-ключей — не границы пары.
    let t = "'a,b': 1\n`c{d`: 2\n";
    assert_eq!(
        rename_key_segments(t, &["a,b"], "x").unwrap(),
        "x: 1\n`c{d`: 2\n"
    );
    let t2 = "'a,b': 1\n`c{d`: 2\n";
    assert_eq!(
        rename_key_segments(t2, &["c{d"], "y").unwrap(),
        "'a,b': 1\ny: 2\n"
    );
}

#[test]
fn rename_double_quoted_key_to_dotted_new_key_no_double_quotes() {
    // Полный сегмент меняется целым представлением: двойных кавычек нет.
    let t = "\"q.k\": 1\n";
    assert_eq!(
        rename_key_segments(t, &["q.k"], "x.y").unwrap(),
        "\"x.y\": 1\n"
    );
}

#[test]
fn rename_keeps_dotted_whitespace_prefix() {
    // Пробелы вокруг точек префикса сохраняются байт-в-байт.
    let t = "a . b : 1\n";
    let out = rename_key(t, "a.b", "m").unwrap();
    assert_eq!(out, "a . m : 1\n");
    assert_eq!(
        parse(&out).unwrap(),
        obj(&[("a", obj(&[("m", Value::Integer("1".into()))]))])
    );
}

#[test]
fn remove_dotted_leaf_inside_explicit_object_materializes_relative_chain() {
    let t = "w: {\n  a.b: 1\n  q: 2\n}\n";
    let out = remove_value(t, "w.a.b").unwrap();
    assert_eq!(out, "w: {\n  a: {}\n  q: 2\n}\n");
    assert_eq!(
        parse(&out).unwrap(),
        obj(&[(
            "w",
            obj(&[
                ("a", Value::Object(ObjectMap::default())),
                ("q", Value::Integer("2".into()))
            ])
        )])
    );
}

#[test]
fn remove_dotted_leaf_inside_array_object_element() {
    let t = "top: [\n  {\n    a.b: 1\n    keep: 2\n  }\n]\n";
    let out = remove_value(t, "top.0.a.b").unwrap();
    assert_eq!(out, "top: [\n  {\n    a: {}\n    keep: 2\n  }\n]\n");
    assert_eq!(
        parse(&out).unwrap(),
        obj(&[(
            "top",
            Value::Array(vec![obj(&[
                ("a", Value::Object(ObjectMap::default())),
                ("keep", Value::Integer("2".into()))
            ])])
        )])
    );
}

#[test]
fn remove_materialization_without_final_eol_invents_none() {
    // Нет финального EOL — материализованная строка его не добавляет.
    let t = "a.b: 1\nc: 2";
    assert_eq!(remove_value(t, "a.b").unwrap(), "a: {}\nc: 2");
}

#[test]
fn remove_dotted_last_line_consumes_previous_terminator() {
    // Последняя строка без EOL: предыдущий терминатор становится
    // терминатором материализованной строки.
    let t = "c: 2\na.b: 1";
    let out = remove_value(t, "a.b").unwrap();
    assert_eq!(out, "c: 2\na: {}");
    assert_eq!(
        parse(&out).unwrap(),
        obj(&[
            ("c", Value::Integer("2".into())),
            ("a", Value::Object(ObjectMap::default()))
        ])
    );
}

// --- независимые семантические оракулы ----------------------------------------

#[test]
fn insert_result_matches_independently_built_tree() {
    let out = insert_value(
        "host: db\nport: 80\n",
        "retries",
        &Value::Integer("3".into()),
    )
    .unwrap();
    let mut expected = ObjectMap::default();
    expected.insert("host".into(), s("db"));
    expected.insert("port".into(), Value::Integer("80".into()));
    expected.insert("retries".into(), Value::Integer("3".into()));
    assert_eq!(parse(&out).unwrap(), Value::Object(expected));
    assert_eq!(
        keys_of(&parse(&out).unwrap()),
        vec!["host", "port", "retries"]
    );
}

#[test]
fn remove_result_matches_independently_built_tree() {
    let out = remove_value("a: {\n  x: 1\n  y: 2\n}\nb: 3\n", "a.x").unwrap();
    let mut inner = ObjectMap::default();
    inner.insert("y".into(), Value::Integer("2".into()));
    let mut expected = ObjectMap::default();
    expected.insert("a".into(), Value::Object(inner));
    expected.insert("b".into(), Value::Integer("3".into()));
    assert_eq!(parse(&out).unwrap(), Value::Object(expected));
}

#[test]
fn rename_result_matches_independently_built_tree() {
    let out = rename_key("one: 1\ntwo: 2\n", "one", "uno").unwrap();
    let mut expected = ObjectMap::default();
    expected.insert("uno".into(), Value::Integer("1".into()));
    expected.insert("two".into(), Value::Integer("2".into()));
    assert_eq!(parse(&out).unwrap(), Value::Object(expected));
    assert_eq!(keys_of(&parse(&out).unwrap()), vec!["uno", "two"]);
}

#[test]
fn remove_materialization_preserves_exact_leading_terminator() {
    for eol in ["\n", "\r\n", "\r"] {
        let source = format!("c: 2{eol}  a.b: 1");
        let out = remove_value(&source, "a.b").unwrap();
        assert_eq!(out, format!("c: 2{eol}  a: {{}}"));
        assert_eq!(
            parse(&out).unwrap(),
            obj(&[("c", Value::Integer("2".into())), ("a", obj(&[])),])
        );
    }
}

#[test]
fn insert_empty_block_ignores_braces_in_comments() {
    for eol in ["\n", "\r\n", "\r"] {
        let source = format!("## {{ header{eol}{{{eol}}}{eol}## tail{eol}");
        let out = insert_value(&source, "k", &s("v")).unwrap();
        assert_eq!(
            out,
            format!("## {{ header{eol}{{{eol}    k: v{eol}}}{eol}## tail{eol}")
        );
        assert_eq!(parse(&out).unwrap(), obj(&[("k", s("v"))]));
    }
}

#[test]
fn rename_inline_quoted_keys_keeps_structural_content_opaque() {
    for quote in ['\'', '"', '`'] {
        let source = format!("a: {{{quote}x,y{quote}: 1, keep: 2}}\n");
        let out = rename_key_segments(&source, &["a", "x,y"], "z.w").unwrap();
        assert_eq!(out, "a: {\"z.w\": 1, keep: 2}\n");
        assert_eq!(
            parse(&out).unwrap(),
            obj(&[(
                "a",
                obj(&[
                    ("z.w", Value::Integer("1".into())),
                    ("keep", Value::Integer("2".into())),
                ])
            )])
        );
    }
}

#[test]
fn insert_checks_inline_width_without_multiline_or_escapes() {
    let value = "x".repeat(101);
    let out = insert_value("a: {}\n", "a.k", &s(&value)).unwrap();
    assert_eq!(out, format!("a: {{\n    k: {value}\n}}\n"));
    assert_eq!(
        parse(&out).unwrap(),
        obj(&[("a", obj(&[("k", s(&value))]))])
    );
}
