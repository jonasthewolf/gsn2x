#!/bin/sh

# argument_view
cargo run -- examples/example.gsn.yaml -G -s examples/example.css -t
cp examples/example.gsn.svg examples/example.gsn_dark.svg
cargo run -- examples/example.gsn.yaml -G 

# arch_view
cargo run -- -N -E -F -G examples/modular/main.gsn.yaml examples/modular/sub1.gsn.yaml examples/modular/sub3.gsn.yaml

# multiple_view
cargo run -- -A -E -F -G -s modular.css examples/modular/main.gsn.yaml examples/modular/sub1.gsn.yaml examples/modular/sub3.gsn.yaml

# complete_view
cargo run -- -N -E -A -G examples/modular/main.gsn.yaml examples/modular/sub1.gsn.yaml examples/modular/sub3.gsn.yaml
