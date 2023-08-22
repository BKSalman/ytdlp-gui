fn main() {
    let repo = gix::discover(".").expect("current directory should be a git repo");
    let rev = repo
        .rev_parse_single("HEAD")
        .expect("HEAD revision should exist");
    let rev_id = rev.to_string();

    println!("cargo:rustc-env=GIT_HASH={}", rev_id);
}
