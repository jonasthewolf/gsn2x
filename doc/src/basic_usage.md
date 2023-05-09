
# Basic usage

This little program renders [Goal Structuring Notation](https://scsc.uk/gsn) in a YAML format to a scalable vector graphics (SVG) image.

![example](../../examples/example.gsn.svg)

Feel free to use it and please let me know. Same applies if you have feature requests, bug reports or contributions.

## Usage

You can create an SVG like this:

    gsn2x <yourgsnfile.yaml> 

The output is an argument view in SVG format and automatically written to `<yourgsnfile.svg>`. If more than one input file is provided, they are treated as [modules](#modular-extension).
