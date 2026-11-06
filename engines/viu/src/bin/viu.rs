// SPDX-License-Identifier: MIT OR Apache-2.0
fn main() -> eframe::Result {
    viu::app_host::run(std::env::args().skip(1).collect())
}
