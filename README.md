# gsn2x

This little script converts Goal Structuring Notation in YAML to a graphical representation.

Python 3 is required with the following modules `pathlib`, `jinja2`, `yaml`.
Moreover, Graphviz dot is required.

## Usage

On Windows you can just run:

    gsn2x.cmd <yourgsnfile> [<output format, e.g. png>]

On other systems you can create a PNG like this:

    python yslt.py -s gsn2dot.yslt <yourgsnfile> | dot -Tpng > <yourgsnfile.png>

## Syntax in YAML

The following GSN elements are supported:
Goals, Argument, Justification, Context, Solution and Strategy.

    G1: 
     - "Goal"
     - supportedBy: [S1]

    S1:
     - "Strategy"
