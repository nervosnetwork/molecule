
### Introduction

See https://rust-fuzz.github.io/book/introduction.html

### Run
```
cargo fuzz run data
```

### Coverage Report

```
cargo fuzz coverage data
cargo cov -- show target/x86_64-unknown-linux-gnu/coverage/x86_64-unknown-linux-gnu/release/data \
    --format=html \
    -instr-profile=fuzz/coverage/data/coverage.profdata \
    > data.html
```
