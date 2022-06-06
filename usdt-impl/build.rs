// Copyright 2022 Oxide Computer Company
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use std::env;

use version_check;

#[derive(Copy, Clone)]
enum Backend {
    // Standard (read: illumos) probe registration
    Standard,
    // MacOS linker-aware probe registration
    Linker,
    // Provide probe macros, but probes are no-ops (dtrace-less OSes)
    NoOp,
}

// Extract any -C ARGS ... passed to RUSTFLAGS
fn extract_encoded_rustc_flags() -> Vec<String> {
    match env::var_os("CARGO_ENCODED_RUSTFLAGS").as_deref() {
      Some(rustflags) => {
          let mut atoms = rustflags.to_str().unwrap_or("").split(' ');
          let mut flags = vec![];
          while let Some(atom) = atoms.next() {
              if atom.starts_with("-C") {
                  flags.push(atom[2..].to_string());
              }
          }
          flags
      }
      _ => vec![]
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // `asm` feature was stabilized in 1.59
    let have_stable_asm = version_check::is_min_version("1.59").unwrap_or(false);
    // XXX: `asm_sym` feature is not yet stable
    let have_stable_asm_sym = false;
    let have_stable_used_with_arg = false;

    // Are we being built with a compiler which allows feature flags (nightly)
    let is_nightly = version_check::is_feature_flaggable().unwrap_or(false);

    let feat_asm = env::var_os("CARGO_FEATURE_ASM").is_some();
    let feat_strict_asm = env::var_os("CARGO_FEATURE_STRICT_ASM").is_some();

    // Check if upstream have enabled link-dead-code, which in those cases we can
    // enable standard backend for FreeBSD. We check this by finding the last
    // -C link-dead-code* flag, and check if it is a negation of link-dead-code
    let have_link_dead_code = extract_encoded_rustc_flags().iter()
        .filter(|flag| flag.contains("link-dead-code"))
        .map(|flag| !flag.contains("link-dead-code=n"))
        .last().unwrap_or(false);

    let backend = match env::var("CARGO_CFG_TARGET_OS").ok().as_deref() {
        Some("macos") if feat_asm => {
            if have_stable_asm && have_stable_asm_sym {
                Backend::Linker
            } else if feat_strict_asm || is_nightly {
                if !have_stable_asm {
                    println!("cargo:rustc-cfg=usdt_need_feat_asm");
                }
                if !have_stable_asm_sym {
                    println!("cargo:rustc-cfg=usdt_need_feat_asm_sym");
                }
                Backend::Linker
            } else {
                Backend::NoOp
            }
        }
        Some("illumos") | Some("solaris") if feat_asm => {
            if have_stable_asm {
                Backend::Standard
            } else if feat_strict_asm || is_nightly {
                println!("cargo:rustc-cfg=usdt_need_feat_asm");
                Backend::Standard
            } else {
                Backend::NoOp
            }
        }
        Some("freebsd") if feat_asm => {
            // FreeBSD require used(linker) to preserve __(start|stop)_set_dtrace_probes
            // without explicit "link-dead-code" by consumer
            if have_link_dead_code || have_stable_used_with_arg || is_nightly {
               if !have_stable_used_with_arg && is_nightly {
                   println!("cargo:rustc-cfg=usdt_need_used_with_arg");
               }
               if have_stable_asm {
                   Backend::Standard
               } else if feat_strict_asm || is_nightly {
                   Backend::Standard
               } else {
                   Backend::NoOp
               }
            } else {
                Backend::NoOp
            }
        }
        _ => Backend::NoOp,
    };

    // Since visibility of the `asm!()` macro differs between the nightly feature and the
    // stabilized version, the consumer requires information about its availability
    if have_stable_asm {
        println!("cargo:rustc-cfg=usdt_stable_asm");
    }

    match backend {
        Backend::NoOp => {
            println!("cargo:rustc-cfg=usdt_backend_noop");
        }
        Backend::Linker => {
            println!("cargo:rustc-cfg=usdt_backend_linker");
        }
        Backend::Standard => {
            println!("cargo:rustc-cfg=usdt_backend_standard");
        }
    }
}
