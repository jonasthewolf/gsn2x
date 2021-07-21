[![License: CC0-1.0](https://img.shields.io/badge/License-CC0%201.0-lightgrey.svg)](http://creativecommons.org/publicdomain/zero/1.0/) [![CI/CD](https://github.com/jonasthewolf/gsn2x/actions/workflows/rust.yml/badge.svg)](https://github.com/jonasthewolf/gsn2x/actions/workflows/rust.yml)

# gsn2x

This little program converts Goal Structuring Notation in YAML to a graphical representation.

Feel free to use it and please let me know.

Graphviz dot is required.

## Usage

On Windows you can just run:

    gsn2x.cmd <yourgsnfile> [<output format, e.g. png>]

On other systems you can create a PNG like this:

    python yslt.py -s gsn2dot.yslt <yourgsnfile> | dot -Tpng > <yourgsnfile.png>

## Syntax in YAML

The following GSN elements are supported:
 - Goal (G), 
 - Assumption (A), 
 - Justification (J), 
 - Solution (Sn),
 - Context (C), and
 - Strategy (S)

Every element is defined by a letter and a number.
The first line of the element is its text. 
The `supportedBy` gives a list of the supporting arguments.
The `inContextOf` links justifications, context or assumptions. 

    G1: 
     - "Goal"
     - supportedBy: [S1]

    S1:
     - "Strategy"

Please see example.gsn.yaml for an example of the used syntax.