#!/bin/sh

# argument_view
cargo run -- examples/example.gsn.yaml -G -s examples/example.css -t
cp examples/example.gsn.svg examples/example.gsn_dark.svg
cargo run -- examples/example.gsn.yaml -G 

# arch_view
cargo run -- -N -E -F -G -o examples/modular examples/modular/main.gsn.yaml examples/modular/sub1.gsn.yaml examples/modular/sub3.gsn.yaml

# multiple_view
cargo run -- -A -E -F -G -s https://github.com/jonasthewolf/gsn2x/blob/3439402d093ba54af4771b295e78f2488bd1b978/examples/modular/modular.css examples/modular/main.gsn.yaml examples/modular/sub1.gsn.yaml examples/modular/sub3.gsn.yaml

# complete_view
cargo run -- -N -E -A -G -o examples/modular examples/modular/main.gsn.yaml examples/modular/sub1.gsn.yaml examples/modular/sub3.gsn.yaml

# multi context
cargo run -- -A -E -F -G tests/multi_context.gsn.yaml

# minimal css example
cargo run -- -G -t -s examples/minimalcss/min.css examples/minimalcss/min.gsn.yaml
