#!/bin/sh

cargo build --release

# argument_view
./target/release/gsn2x -G -E examples/example.gsn.yaml -s=examples/example.css -t
cp examples/example.gsn.svg examples/example.gsn_dark.svg
./target/release/gsn2x -G -E examples/example.gsn.yaml

# arch_view
./target/release/gsn2x -G -E -F -N -o=examples/modular examples/modular/index.gsn.yaml 

# multiple_view
./target/release/gsn2x -G -E -A -F -s=https://github.com/jonasthewolf/gsn2x/blob/3439402d093ba54af4771b295e78f2488bd1b978/examples/modular/modular.css examples/modular/index.gsn.yaml 

# complete_view
./target/release/gsn2x -G -E -A -N -o=examples/modular examples/modular/index.gsn.yaml 

# multi context
./target/release/gsn2x -G -E -A -F tests/multi_context.gsn.yaml

# minimal css example
./target/release/gsn2x -G -E -t -s=examples/minimalcss/min.css examples/minimalcss/min.gsn.yaml

# entangled
./target/release/gsn2x -G -E examples/entangled.gsn.yaml

# additionals
./target/release/gsn2x -G -E -l=add1 -l=additional -l=unsupported tests/additionals.yaml

# confidence example
./target/release/gsn2x -G -E examples/confidence.gsn.yaml

# dialectic example
./target/release/gsn2x -G -E examples/dialectic/first.gsn.yaml
./target/release/gsn2x -G -E examples/dialectic/second.gsn.yaml

# bullet lists
./target/release/gsn2x -G -E examples/bullet_lists.gsn.yaml

# font metrics
./target/release/gsn2x -G -E tests/font_metrics.gsn.yaml


# issue regressions
./target/release/gsn2x -G -E tests/issue84_1.yaml
./target/release/gsn2x -G -E tests/issue84_2.yaml
./target/release/gsn2x -G -E tests/issue84_3.yaml
./target/release/gsn2x -G -E tests/issue84_4.yaml
./target/release/gsn2x -G -E tests/issue249.yaml
./target/release/gsn2x -G -E tests/issue250.yaml
./target/release/gsn2x -G -E tests/issue313.yaml
./target/release/gsn2x -G -E tests/issue339.yaml
./target/release/gsn2x -G -E tests/issue358.yaml -l=layer1 -l=layer2
./target/release/gsn2x -G -E -w=35 tests/issue365.yaml
./target/release/gsn2x -G -E tests/issue371.yaml
./target/release/gsn2x -G -E -w=35 tests/issue372.yaml
./target/release/gsn2x -G -E -w=35 tests/issue377.yaml
./target/release/gsn2x -G -E -A -F tests/issue391_1.yaml tests/issue391_2.yaml
./target/release/gsn2x -G -E -w=20 tests/issue393_1.yaml
./target/release/gsn2x -G -E -w=20 tests/issue393_2.yaml
./target/release/gsn2x -G -E -w=20 tests/issue389.yaml
./target/release/gsn2x -G -E -w=20 -l=layer1 tests/issue396.yaml
./target/release/gsn2x -G -E -w=20 -F -A tests/issue433_1.yaml
./target/release/gsn2x -G -E tests/multi_parents.gsn.yaml
./target/release/gsn2x -G -E tests/issue407.yaml -l=layer2
./target/release/gsn2x -G -E tests/issue467.yaml
./target/release/gsn2x -G -E tests/issue453.yaml -l=layer2
./target/release/gsn2x -G -E tests/multi_children.yaml
./target/release/gsn2x -G -E tests/multi_children_min.yaml
./target/release/gsn2x -G -E tests/issue561.yaml
