cargo build
# λ=1 and μ=0.0001 keeps the compression ratio to maximum while keeping SED error under 0.0001
cargo run -- 20081023025304-0.plt  1 0.0001 20081023025304-0.csv
# λ=5 and μ=0 has compression ratio to λ while minimizing SED error.
cargo run -- 20081023025304-0.plt  5 0 20081023025304-0.txt