//! Spec 0.7 § 5.9.0 — the representability predicate and its pre-pass.
//!
//! `check_representable` runs before any bytes are emitted so a
//! rejection leaves no partial output, and so the root Object-or-Array
//! constraint is evaluated BEFORE node-representability is checked
//! recursively (the one fixed precedence § 5.9.0 mandates).
//!
//! All rejections emerge as [`Error::UnrepresentableAt`] carrying the
//! decoded key path from the document root to the offending pair's key
//! (empty path = root-level offense), per the § 5.9.0 structured-error
//! contract (issue rust#12).

use crate::error::{Error, ReasonCode, Result};
use crate::value::Value;

use super::helpers::{choose_multiline_form, string_needs_multiline};

/// Reject a non-representable Value (§ 5.9.0) with the matching
/// [`ReasonCode`], reported as [`Error::UnrepresentableAt`] carrying
/// the key path to the offending pair. Runs the root-kind check first,
/// then recurses.
pub(crate) fn check_representable(value: &Value) -> Result<()> {
    match value {
        Value::Object(_) | Value::Array(_) => check_node(value, &mut Vec::new()),
        _ => Err(Error::UnrepresentableAt {
            code: ReasonCode::ScalarRoot,
            path: Vec::new(),
        }),
    }
}

fn check_node<'k>(value: &'k Value, path: &mut Vec<&'k str>) -> Result<()> {
    match value {
        Value::Object(pairs) => {
            for (k, v) in pairs {
                path.push(k);
                if k.is_empty() {
                    return Err(at_path(
                        Error::Unrepresentable(ReasonCode::EmptyKeyName),
                        path,
                    ));
                }
                check_node(v, path)?;
                path.pop();
            }
            Ok(())
        }
        Value::Array(items) => {
            for item in items {
                check_node(item, path)?;
            }
            Ok(())
        }
        Value::Float(s) => match s.parse::<f64>() {
            Ok(v) if v.is_nan() || v.is_infinite() => Err(at_path(
                Error::Unrepresentable(ReasonCode::NonFiniteFloat),
                path,
            )),
            // A stored payload that is not an f64 lexical form at all
            // is outside the Float domain but not one of the seven
            // § 5.9.0 reason codes; emission keeps its historical
            // pass-through behaviour for it.
            _ => Ok(()),
        },
        Value::String(s) => {
            if s.contains('\r') {
                return Err(at_path(Error::Unrepresentable(ReasonCode::CRByte), path));
            }
            if string_needs_multiline(s) {
                // The chooser's error paths ARE the three § 5.9.7
                // collision codes; calling it with `prefer_stripped =
                // false` detects exactly the same rejection set as the
                // pretty writers' `true` (the error condition
                // `!verbatim_ok && !stripped_lossless` is independent
                // of the preference).
                choose_multiline_form(s, false)
                    .map(|_| ())
                    .map_err(|e| at_path(e, path))
            } else {
                Ok(())
            }
        }
        Value::Null | Value::Bool(_) | Value::Integer(_) => Ok(()),
    }
}

/// Convert a path-less writer rejection into the path-carrying
/// `Error::UnrepresentableAt`, materializing the borrowed segments only
/// on the error path.
fn at_path(err: Error, path: &[&str]) -> Error {
    match err {
        Error::Unrepresentable(code) => Error::UnrepresentableAt {
            code,
            path: path.iter().map(|s| (*s).to_string()).collect(),
        },
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ReasonCode;
    use crate::value::{ObjectMap, Scalar};

    fn obj(pairs: &[(&str, Value)]) -> Value {
        let mut map = ObjectMap::default();
        for (k, v) in pairs {
            map.insert(Scalar::from(*k), v.clone());
        }
        Value::Object(map)
    }

    fn check_code(value: &Value) -> ReasonCode {
        check_representable(value)
            .unwrap_err()
            .reason_code()
            .unwrap()
    }

    #[test]
    fn choose_multiline_form_direct() {
        use super::super::helpers::choose_multiline_form;
        use crate::render::helpers::MultilineForm;
        for prefer in [false, true] {
            // With `prefer_stripped = true` the unconditionally-safe
            // stripped form is picked; only the ERROR codes are
            // required to be identical across both preferences.
            assert!(matches!(
                choose_multiline_form("a\nb", prefer),
                Ok(MultilineForm::Verbatim | MultilineForm::Stripped)
            ));
            assert!(matches!(
                choose_multiline_form("a\n))", prefer),
                Ok(MultilineForm::Stripped)
            ));
            // R8-F5: differing leading code points share NO common
            // prefix (§ 5.6 compares code-point-for-code-point), so the
            // stripped fallback stays lossless even though every line
            // is indented.
            assert!(matches!(
                choose_multiline_form("\talpha\n ))", prefer),
                Ok(MultilineForm::Stripped)
            ));
            // U+2000 vs U+2001 share their first two UTF-8 bytes but
            // are different code points — still no common prefix.
            assert!(matches!(
                choose_multiline_form("\u{2000}alpha\n\u{2001}))", prefer),
                Ok(MultilineForm::Stripped)
            ));
            let code = choose_multiline_form("))\n)", prefer)
                .unwrap_err()
                .reason_code()
                .unwrap();
            assert_eq!(code, ReasonCode::BothFormsRequired);
            let code = choose_multiline_form("))\nx ", prefer)
                .unwrap_err()
                .reason_code()
                .unwrap();
            assert_eq!(code, ReasonCode::TrailingWhitespaceCollision);
            let code = choose_multiline_form(" ))\n x", prefer)
                .unwrap_err()
                .reason_code()
                .unwrap();
            assert_eq!(code, ReasonCode::LeadingWhitespaceCollision);
            let code = choose_multiline_form("))\n  \nx", prefer)
                .unwrap_err()
                .reason_code()
                .unwrap();
            assert_eq!(code, ReasonCode::TrailingWhitespaceCollision);
        }
    }

    #[test]
    fn scalar_root() {
        assert_eq!(check_code(&Value::Null), ReasonCode::ScalarRoot);
    }

    #[test]
    fn empty_key_name() {
        let v = obj(&[("", Value::Null)]);
        assert_eq!(check_code(&v), ReasonCode::EmptyKeyName);
    }

    #[test]
    fn non_finite_floats() {
        let in_obj = obj(&[("k", Value::Float("NaN".into()))]);
        assert_eq!(check_code(&in_obj), ReasonCode::NonFiniteFloat);
        let in_arr = Value::Array(vec![Value::Float("inf".into())]);
        assert_eq!(check_code(&in_arr), ReasonCode::NonFiniteFloat);
        let neg = obj(&[("k", Value::Float("-Infinity".into()))]);
        assert_eq!(check_code(&neg), ReasonCode::NonFiniteFloat);
    }

    #[test]
    fn node_check_runs_after_root_passes() {
        let v = obj(&[("k", Value::Float("NaN".into()))]);
        assert_eq!(check_code(&v), ReasonCode::NonFiniteFloat);
    }

    #[test]
    fn finite_floats_ok() {
        assert!(check_representable(&obj(&[("a", Value::Float("1.5".into()))])).is_ok());
        assert!(check_representable(&obj(&[("a", Value::Float("-0.0".into()))])).is_ok());
    }

    #[test]
    fn cr_byte() {
        let in_obj = obj(&[("k", Value::String("a\rb".into()))]);
        assert_eq!(check_code(&in_obj), ReasonCode::CRByte);
        let in_arr = Value::Array(vec![Value::String("a\rb".into())]);
        assert_eq!(check_code(&in_arr), ReasonCode::CRByte);
    }

    #[test]
    fn empty_roots_ok() {
        assert!(check_representable(&Value::Object(ObjectMap::default())).is_ok());
        assert!(check_representable(&Value::Array(Vec::new())).is_ok());
    }
}
