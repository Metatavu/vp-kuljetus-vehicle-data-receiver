## VP-Kuljetus Vehicle Data Receiver
### Generate Vehicle Management Service Client from OpenAPI
1. Install `libninja` with `cargo install --git https://github.com/kurtbuilds/libninja`
2. Generate client from project root with `sh generate-client.sh`


### Code Coverage
Code coverage is generated with `cargo llvm-cov --open --ignore-filename-regex build`. See [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) docs for more.