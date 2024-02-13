#!/bin/sh

# argument_view
cargo run -- examples/example.gsn.yaml -E -G -s examples/example.css -t
cp examples/example.gsn.svg examples/example.gsn_dark.svg
cargo run -- examples/example.gsn.yaml -E -G

# arch_view
cargo run -- -N -E -F -G -o examples/modular examples/modular/main.gsn.yaml examples/modular/sub1.gsn.yaml examples/modular/sub3.gsn.yaml

# multiple_view
cargo run -- -A -E -F -G -s https://github.com/jonasthewolf/gsn2x/blob/3439402d093ba54af4771b295e78f2488bd1b978/examples/modular/modular.css examples/modular/main.gsn.yaml examples/modular/sub1.gsn.yaml examples/modular/sub3.gsn.yaml

# complete_view
cargo run -- -N -E -A -G -o examples/modular examples/modular/main.gsn.yaml examples/modular/sub1.gsn.yaml examples/modular/sub3.gsn.yaml

# multi context
cargo run -- -A -E -F -G tests/multi_context.gsn.yaml

# minimal css example
cargo run -- -G -E -t -s examples/minimalcss/min.css examples/minimalcss/min.gsn.yaml

# issue regressions
cargo run -- -G -E tests/issue84_1.yaml
cargo run -- -G -E tests/issue84_2.yaml
cargo run -- -G -E tests/issue84_3.yaml
cargo run -- -G -E tests/issue84_4.yaml
cargo run -- -G -E tests/issue249.yaml
cargo run -- -G -E tests/issue250.yaml
cargo run -- -G -E tests/issue313.yaml
cargo run -- -G -E tests/issue339.yaml
cargo run -- -G -E tests/issue358.yaml -l layer1 -l layer2
cargo run -- -G -E -w 35 tests/issue365.yaml
cargo run -- -G -E tests/issue371.yaml
cargo run -- -G -E -w 35 tests/issue372.yaml
cargo run -- -G -E -w 35 tests/issue377.yaml
cargo run -- -G -E tests/issue391_1.yaml tests/issue391_2.yaml
cargo run -- -G -E -w 20 tests/issue393_1.yaml
cargo run -- -G -E -w 20 tests/issue393_2.yaml
cargo run -- -G -E tests/multi_parents.gsn.yaml