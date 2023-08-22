fn main() {
    let git_hash = match option_env!("GIT_HASH") {
        Some(git_hash) => git_hash.to_string(),
        None => {
            let repo = gix::discover(std::env::current_dir().unwrap())
                .expect("current directory should be a git repository");
            let rev = repo
                .rev_parse_single("HEAD")
                .expect("HEAD in the repository should have a revision id");
            rev.to_string()
        }
    };

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}
