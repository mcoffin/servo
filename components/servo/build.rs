/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{
    env,
    collections::HashMap,
};
use std::fs;
use std::path::Path;
use std::process::Command;
use cargo::Vars as CargoVars;

fn main() {
    println!("cargo:rerun-if-changed=../../python/servo/gstreamer.py");

    let output = Command::new(find_python())
        .arg("../../python/servo/gstreamer.py")
        .arg(std::env::var_os("TARGET").unwrap())
        .output()
        .unwrap();
    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1)
    }
    let cvars = CargoVars::from_env();
    {
        let mut features = cvars.features()
            .peekable();
        if features.peek().is_none() {
            println!("cargo:warning={}", "no features found");
        } else {
            for (k, v) in features {
                println!("cargo:warning={}", format_args!("{}={}", k, v));
            }
        }
    }
    cvars.iter()
        .map(|(k, ..)| k)
        .for_each(|k| {
            println!("cargo:warning={}", k);
        });
    let path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("gstreamer_plugins.rs");
    fs::write(path, output.stdout).unwrap();
}

fn find_python() -> String {
    env::var("PYTHON3").ok().unwrap_or_else(|| {
        let candidates = if cfg!(windows) {
            ["python3.8.exe", "python38.exe", "python.exe"]
        } else {
            ["python3.8", "python3", "python"]
        };
        for &name in &candidates {
            if Command::new(name)
                .arg("--version")
                .output()
                .ok()
                .map_or(false, |out| out.status.success())
            {
                return name.to_owned();
            }
        }
        panic!(
            "Can't find python (tried {})! Try fixing PATH or setting the PYTHON3 env var",
            candidates.join(", ")
        )
    })
}

mod cargo {
    use std::{
        env,
        collections::HashMap,
    };

    #[repr(transparent)]
    pub struct Vars(HashMap<String, String>);

    #[inline(always)]
    fn feature_key(name: &str) -> String {
        format!("CARGO_FEATURE_{}", name.replace('-', "_").to_uppercase())
    }

    impl Vars {
        pub fn from_env() -> Self {
            let vars: HashMap<String, String> = env::vars()
                .filter(|&(ref k, ref _v)| k.starts_with("CARGO_"))
                .collect();
            Vars(vars)
        }

        #[inline]
        pub fn iter<'a>(&'a self) -> impl Iterator<Item=(&'a str, &'a str)> + 'a {
            self.0.iter()
                .map(|(k, v)| (k.as_ref(), v.as_ref()))
        }

        pub fn features<'a>(&'a self) -> impl Iterator<Item=(&'a str, &'a str)> + 'a {
            self.0.iter()
                .filter(|&(k, ..)| k.starts_with("CARGO_FEATURE_"))
                .map(|(k, v)| (k.as_ref(), v.as_ref()))
        }

        pub fn has_feature(&self, name: &str) -> bool {
            self.0.get(&feature_key(name))
                .is_some()
        }
    }
}
