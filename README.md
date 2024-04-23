[![License: MIT](https://img.shields.io/github/license/jonasthewolf/gsn2x?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![tests](https://img.shields.io/github/actions/workflow/status/jonasthewolf/gsn2x/rust.yml?branch=main&label=tests&style=for-the-badge)](https://github.com/jonasthewolf/gsn2x/actions/workflows/rust.yml)
[![codecov](https://img.shields.io/codecov/c/github/jonasthewolf/gsn2x/main?token=YQKUQQOYS3&style=for-the-badge)](https://codecov.io/gh/jonasthewolf/gsn2x)
![size](https://img.shields.io/github/languages/code-size/jonasthewolf/gsn2x?style=for-the-badge)
[![downloads](https://img.shields.io/github/downloads/jonasthewolf/gsn2x/total?style=for-the-badge&color=blue)](https://github.com/jonasthewolf/gsn2x/releases)
[![Static Badge](https://img.shields.io/badge/documentation-blue?style=for-the-badge)](https://jonasthewolf.github.io/gsn2x)


# gsn2x

This little program renders [Goal Structuring Notation](https://scsc.uk/gsn) from a YAML format to a scalable vector graphics (SVG) image.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="examples/example.gsn_dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="examples/example.gsn.svg">
  <img alt="Example" src="examples/example.gsn.svg">
</picture>

Feel free to use it and please let me know. Same applies if you have feature requests, bug reports or contributions.
    
**You can find pre-built binaries for Windows, Linux and MacOS on the [releases page](https://github.com/jonasthewolf/gsn2x/releases).**


## Usage

You can create an SVG like this:

    gsn2x <yourgsnfile.yaml> 

The output is an argument view in SVG format and automatically written to `<yourgsnfile.svg>`. If more than one input file is provided, they are treated as modules.

## Documentation

For further information see the [documentation](https://jonasthewolf.github.io/gsn2x).
