//! End-to-end tests backed by `.conf` files under `tests/fixtures/`.

#[path = "common/mod.rs"]
mod common;

use common::fixture;
use ktav::from_file;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Resocks5Config {
    port: u16,
    banned_patterns: Vec<String>,
}

#[test]
fn parses_resocks5_fixture() {
    let cfg: Resocks5Config = from_file(fixture("resocks5.conf")).unwrap();
    assert_eq!(cfg.port, 20082);
    assert_eq!(cfg.banned_patterns, vec![".*\\.onion:\\d+".to_string()]);
}

#[derive(Debug, Deserialize, PartialEq)]
struct ServerConfig {
    host: String,
    port: u16,
}
#[derive(Debug, Deserialize, PartialEq)]
struct AppInfo {
    debug: bool,
}
#[derive(Debug, Deserialize, PartialEq)]
struct HttpConfig {
    methods: Vec<String>,
}
#[derive(Debug, Deserialize, PartialEq)]
struct WebConfig {
    server: ServerConfig,
    app: AppInfo,
    http: HttpConfig,
}

#[test]
fn parses_web_fixture_with_dotted_keys_and_bool_keyword() {
    let cfg: WebConfig = from_file(fixture("web.conf")).unwrap();
    assert_eq!(cfg.server.host, "127.0.0.1");
    assert_eq!(cfg.server.port, 8080);
    assert!(cfg.app.debug);
    assert_eq!(cfg.http.methods, vec!["GET", "POST", "DELETE"]);
}

#[derive(Debug, Deserialize, PartialEq)]
struct Timeouts {
    read: u32,
    write: u32,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Upstream {
    host: String,
    port: u16,
    timeouts: Option<Timeouts>,
    weight: Option<u16>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct UpstreamConfig {
    port: u16,
    banned_patterns: Vec<String>,
    upstreams: Vec<Upstream>,
}

#[test]
fn parses_upstreams_fixture_with_nested_objects() {
    let cfg: UpstreamConfig = from_file(fixture("upstreams.conf")).unwrap();
    assert_eq!(cfg.port, 20082);
    assert_eq!(cfg.banned_patterns.len(), 1);
    assert_eq!(cfg.upstreams.len(), 2);

    let first = &cfg.upstreams[0];
    assert_eq!(first.host, "a.example");
    assert_eq!(first.port, 1080);
    assert_eq!(
        first.timeouts,
        Some(Timeouts {
            read: 30,
            write: 10
        })
    );
    assert!(first.weight.is_none());
    assert!(first.tags.is_empty());

    let second = &cfg.upstreams[1];
    assert_eq!(second.host, "b.example");
    assert_eq!(second.weight, Some(3));
    assert_eq!(second.tags, vec!["backup", "eu"]);
    assert!(second.timeouts.is_none());
}

#[derive(Debug, Deserialize, PartialEq)]
struct CommentedConfig {
    port: u16,
    name: String,
    items: Vec<String>,
}

#[test]
fn comments_are_ignored_everywhere() {
    let cfg: CommentedConfig = from_file(fixture("with_comments.conf")).unwrap();
    assert_eq!(cfg.port, 8080);
    assert_eq!(cfg.name, "demo");
    assert_eq!(cfg.items, vec!["a", "b", "c"]);
}

#[derive(Debug, Deserialize, PartialEq)]
struct RawValuesConfig {
    regex: String,
    template: String,
    ipv6_list: Vec<String>,
}

#[test]
fn raw_values_fixture() {
    let cfg: RawValuesConfig = from_file(fixture("raw_values.conf")).unwrap();
    assert_eq!(cfg.regex, "[a-z]+");
    assert_eq!(cfg.template, "{template.name}");
    assert_eq!(
        cfg.ipv6_list,
        vec!["[::1]", "127.0.0.1", "[2001:db8::1]:53"]
    );
}
