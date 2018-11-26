mod features {
    use std::{
        env, fs, io,
        path::Path,
        process::{Command, Stdio},
    };

    fn test_example(path: &Path) {
        let mut success = true;

        let out_file = path.join("stdout");
        let old_rlib = path.join("libold.rlib").to_str().unwrap().to_owned();
        let new_rlib = path.join("libnew.rlib").to_str().unwrap().to_owned();

        {
            let stdout = fs::File::create(&out_file).expect("could not create `stdout` file");
            let stderr = stdout
                .try_clone()
                .expect("could not create `stderr` file by cloning `stdout`");

            success &= Command::new("rustc")
                .args(&["--crate-type=lib", "-o", &old_rlib])
                .arg(path.join("old.rs"))
                .env("RUST_BACKTRACE", "full")
                .stdin(Stdio::null())
                .status()
                .expect("could not run rustc")
                .success();

            assert!(success, "couldn't compile old");

            success &= Command::new("rustc")
                .args(&["--crate-type=lib", "-o", &new_rlib])
                .arg(path.join("new.rs"))
                .env("RUST_BACKTRACE", "full")
                .stdin(Stdio::null())
                .status()
                .expect("could not run rustc")
                .success();

            assert!(success, "couldn't compile new");

            success &= Command::new(
                Path::new(".")
                    .join("target")
                    .join("debug")
                    .join("rust-semverver")
                    .to_str()
                    .unwrap(),
            )
            .args(&[
                "--crate-type=lib",
                "-Zverbose",
                "--extern",
                &format!("old={}", old_rlib),
                "--extern",
                &format!("new={}", new_rlib),
                Path::new("tests")
                    .join("helper")
                    .join("test.rs")
                    .to_str()
                    .unwrap(),
            ])
            .env("RUST_BACKTRACE", "full")
            .env("RUST_SEMVER_CRATE_VERSION", "1.0.0")
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .status()
            .expect("could not run rust-semverver")
            .success();

            assert!(success, "rust-semverver");

            {
                // replace root path with with $REPO_PATH
                use self::io::{Read, Write};
                let current_dir = env::current_dir().expect("could not determine current dir");
                let mut contents = {
                    let mut f = fs::File::open(&out_file).expect("file not found");
                    let mut contents = String::new();
                    f.read_to_string(&mut contents)
                        .expect("something went wrong reading the file");
                    contents
                };

                contents = contents.replace(current_dir.to_str().unwrap(), "$REPO_PATH");

                if cfg!(target_os = "windows") {
                    let mut lines = Vec::new();

                    for line in contents.lines() {
                        if line.contains("$REPO_PATH") {
                            lines.push(line.replace('\\', "/"));
                        } else {
                            lines.push(line.to_owned());
                        }
                    }
                    lines.push(String::new());
                    contents = lines.join("\r\n");
                }

                let mut file = fs::File::create(&out_file).expect("cannot create file");
                file.write_all(contents.as_bytes())
                    .expect("cannot write to file");
            }
        }

        success &= Command::new("git")
            .args(&[
                "diff",
                "--ignore-space-at-eol",
                "--exit-code",
                out_file.to_str().unwrap(),
            ])
            .env("PAGER", "")
            .status()
            .expect("could not run git diff")
            .success();

        assert!(success, "git");

        Command::new("rm")
            .args(&[&old_rlib, &new_rlib])
            .status()
            .expect("could not run rm");
    }

    macro_rules! test {
        ($name:ident) => {
            #[test]
            fn $name() {
                let path = Path::new("tests").join("cases").join(stringify!($name));
                test_example(&path);
            }
        };
        ($($name:ident),*) => {
            $(test!($name);)*
        }
    }

    test! {
        addition,
        addition_path,
        addition_use,
        bounds,
        circular,
        consts,
        enums,
        func,
        func_local_items,
        infer,
        infer_regress,
        inherent_impls,
        issue_34,
        issue_50,
        kind_change,
        macros,
        max_priv,
        mix,
        pathologic_paths,
        pub_use,
        regions,
        removal,
        removal_path,
        removal_use,
        sealed_traits,
        structs,
        swap,
        traits,
        trait_impls,
        trait_objects,
        ty_alias
    }
}
