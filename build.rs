use std::path::Path;
use std::process::Command;

fn main() {
    // Figure out where the root git directory of this project is
    let git_output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .ok();
    let git_dir = git_output.as_ref().and_then(|output| {
        std::str::from_utf8(&output.stdout)
            .ok()
            .and_then(|s| s.strip_suffix('\n').or_else(|| s.strip_suffix("\r\n")))
    });

    // Tell cargo to rebuild if any relevant git refs change.
    if let Some(git_dir) = git_dir {
        let git_path = Path::new(git_dir);
        let refs_path = git_path.join("refs");
        if git_path.join("HEAD").exists() {
            println!("cargo:rerun-if-changed={git_dir}/HEAD");
        }
        if git_path.join("packed-refs").exists() {
            println!("cargo:rerun-if-changed={git_dir}/packed-refs");
        }
        if refs_path.join("heads").exists() {
            println!("cargo:rerun-if-changed={git_dir}/refs/heads");
        }
        if refs_path.join("tags").exists() {
            println!("cargo:rerun-if-changed={git_dir}/refs/tags");
        }
    }

    // Figure out the current long-form git tag.
    let git_output = Command::new("git")
        .args(["describe", "--always", "--tags", "--long", "--dirty"])
        .output()
        .ok();
    let git_info = git_output
        .as_ref()
        .and_then(|output| std::str::from_utf8(&output.stdout).ok().map(str::trim));

    // Get the current package version set in Cargo.toml
    let cargo_pkg_version = env!("CARGO_PKG_VERSION");

    // Default git_describe to cargo package version
    let mut git_describe = String::from(cargo_pkg_version);

    if let Some(git_info) = git_info {
        // If the git_info contains CARGO_PKG_VERSION we just use git_info as-is.
        // Otherwise we prepend CARGO_PKG_VERSION.
        if git_info.contains(cargo_pkg_version) {
            // Remove the 'g' before the commit sha
            let git_info = &git_info.replace('g', "");
            git_describe = git_info.to_string();
        } else {
            git_describe = format!("v{cargo_pkg_version}-{git_info}");
        }
    }

    println!("cargo:rustc-env=_GIT_INFO={git_describe}");
}
