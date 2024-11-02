#![allow(missing_docs)]

fn main() {
    process_version_string();

    capnpc::CompilerCommand::new()
        .src_prefix("schema")
        .file("schema/session.capnp")
        .file("schema/control.capnp")
        .default_parent_module(vec!["protocol".into()])
        .run()
        .expect("capnpc invocation failed");
}

fn process_version_string() {
    let hash = git_short_hash().unwrap();
    println!("cargo:rustc-env=QCP_BUILD_GIT_HASH={hash}");
    let cargo_version = env!("CARGO_PKG_VERSION");

    let version_string = if let Some(tag) = github_tag() {
        // This is a tagged build running in CI
        println!("cargo:rustc-env=QCP_CI_TAG_VERSION={tag}");
        // Sanity check. We tag releases as "v1.2.3", so strip off the leading v before matching.
        let short_tag = tag.strip_prefix("v").unwrap_or(&tag);
        assert_eq!(
            cargo_version, short_tag,
            "mismatched cargo and CI version tags"
        );
        tag
    } else {
        format!("{cargo_version}+g{hash}")
    };
    println!("cargo:rustc-env=QCP_VERSION_STRING={version_string}");
}

fn github_tag() -> Option<String> {
    std::env::var("GITHUB_REF_TYPE")
        .is_ok_and(|v| v == "tag")
        .then(|| std::env::var("GITHUB_REF_NAME").unwrap())
}

fn git_short_hash() -> Option<String> {
    use std::process::Command;
    let args = &["rev-parse", "--short=8", "HEAD"];
    let output = Command::new("git").args(args).output().unwrap();
    let rev = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if rev.is_empty() {
        return None;
    }
    Some(rev)
}
