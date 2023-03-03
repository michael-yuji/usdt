//! Test verifying that renaming the provider/probes in various ways works when using a build
//! script.

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

#![cfg_attr(usdt_need_feat_asm, feature(asm))]
#![cfg_attr(usdt_need_feat_asm_sym, feature(asm_sym))]
include!(concat!(env!("OUT_DIR"), "/test.rs"));

fn main() {
    usdt::register_probes().unwrap();

    // Renamed the module that the probes are generated to `still_test`. So naming them as
    // `test::start_work` will fail.
    still_test::start_work!(|| 0);
}
