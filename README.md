[![License: CC BY 4.0](https://img.shields.io/badge/License-CC%20BY%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by/4.0/) [![CI/CD](https://github.com/jonasthewolf/gsn2x/actions/workflows/rust.yml/badge.svg)](https://github.com/jonasthewolf/gsn2x/actions/workflows/rust.yml) [![codecov](https://codecov.io/gh/jonasthewolf/gsn2x/branch/master/graph/badge.svg?token=YQKUQQOYS3)](https://codecov.io/gh/jonasthewolf/gsn2x)

# gsn2x

This little program converts [Goal Structuring Notation](https://scsc.uk/gsn) in a YAML format to the DOT format of [Graphviz](https://graphviz.org). From there it can be rendered to different graphic formats.

![Example](examples/example.gsn.svg "Example")

Graphviz is required to create an image from the output of this tool.

Feel free to use it and please let me know. Same applies if you have feature requests, bug reports or contributions.


## Usage

On Windows you can just run:

    gsn2x.exe <yourgsnfile.yaml> | dot -Tsvg > <yourgsnfile.svg>

On other systems you can create an SVG like this:

    gsn2x <yourgsnfile.yaml> | dot -Tsvg > <yourgsnfile.svg>

If a second optional argument is provided, the output is not written to stdout, but to the file named by the second argument.
If called with option `-c` or `--check` the input file is only checked for validity, but the resulting graph is not written.
    
**You can find pre-built binaries for Windows, Linux and MacOS on the [releases page](https://github.com/jonasthewolf/gsn2x/releases).**

## Syntax in YAML

The following Goal Structuring Notation (GSN) elements are supported:
 - Goal (G), 
 - Assumption (A), 
 - Justification (J), 
 - Solution (Sn),
 - Context (C), and
 - Strategy (S)

Every element is defined by a prefix (as shown in the list above) and a number.
Actually, the number can be an arbitrary identifier then.

The (optional) `supportedBy` gives a list of the supporting arguments. Thus, Goal, Strategy and Solution can be listed here.

The (optional) `inContextOf` links Justifications, Contexts or Assumptions. 

Every element may have an optional `url` attribute that will be used by Graphviz accordingly for a node in the graph.
This should support finding information more easily. Please note the supported output formats by Graphviz.

Goals and Strategies can be undeveloped i.e., without supporting Goals, Strategies or Solutions.
These elements should marked with `undeveloped: true`, otherwise validation will emit warnings.

### Example

    G1:
      text: This is a Goal
      supportedBy: [S1]
      inContextOf: [C1]
    
    S1:
      text: This is a Strategy
    
    C1: 
      text: This is a Context


Please see [example.gsn.yaml] for an example of the used syntax.

## Validation checks

The tool automatically performs the following validation checks on the input YAML:

 - There is only one top-level element (G,S,C,J,A,Sn) unreferenced. 
 - The top-level element is a Goal.
 - All referenced elements (`supportedBy` and `inContextOf`) exist and only reference valid elements 
   (e.g. a Justification cannot be listed under `supportedBy`).
 - All IDs start with a known prefix, i.e. there are only known elements.
 - All Goals and Strategies are either marked with `undeveloped: true` or have supporting Goals, Strategies or Solutions.
 - Goals and Strategies marked as undeveloped, must have no supporting arguments.

Uniqueness of keys is automatically enforced by the YAML format.

Error messages and warnings are printed to stderr.

## Additional layers

Additional attributes of a node are ignored by default.
With the command line option `-l` or `--layers` you can enable the output of those additional attributes.
Using this feature, different views on the GSN can be generated.

### Example

    G1:
      text: This is a Goal
      supportedBy: [S1]
      inContextOf: [C1]
      layer1: This is additional information for G1.
    
    S1:
      text: This is a Strategy
      layer1: This is additional information for S1.
    
    C1: 
      text: This is a Context
      layer1: This is additional information for C1.

In this example, a call to `gsn2x -l layer1` will show the additional information to each element prefixed with _`LAYER1: `_.
Of course, using `text`, `inContextOf`, `supportedBy`, `url`, `undeveloped`, `level` or `classes` are not sensible parameters to pass for the `-l` option.

It is intentional that information is only added for a view, but not hidden to ensure consistency of the GSN in all variants.

## Stylesheets for SVG rendering

You can provide a custom stylesheet for SVG via the `-s` or `--stylesheet` options.

Please see [Graphviz stylesheet](https://graphviz.org/docs/attrs/stylesheet/) and [Graphviz class](https://graphviz.org/docs/attrs/class/) for more details.

Every element will also be addressable by `id`. The `id` is the same as the YAML id.

Elements are assigned `gsnelem` class, edges are assigned `gsnedge` class. 

The complete diagram is assigned `gsndiagram` class.

You can assign additional classes by adding the `classes:` attribute. It must be a list of classes you want to assign. Additional layers will be added as CSS classes, too. A `layer1` will e.g. be added as `gsnlay_layer1`.

### Example

    G1:
      text: This is a Goal
      classes: [additionalclass1, additionalclass2]

## Logical levels for elements

To influence the rendered image, you can add an identifier to a GSN element with the `level` attribute. All elements with the same identifier for `level` will now have the same rank for Graphviz. 

This is especially useful, if e.g., two goals or strategies are on the same logical level, but have a different "depth" in the argumentation (i.e. a different number of goals or strategies in their path to the root goal).

See the [example](example.gsn.yaml) for usage. The strategies S1 and S2 are on the same level.

It is recommended to use `level` only for goals, since related contexts, justifications and assumptions are automatically put on the same level, i.e. the same rank in Graphviz.

## List of evidences

With the `-e` option you can create an additional file that lists all the evidences in the input file.

The format can be used in Markdown and reStructuredText files.

## Standard support

This tool is based on the [Goal Structuring Notation Community Standard Version 3](https://scsc.uk/r141C:1).

| Standard                    | Support                 |
|-----------------------------|-------------------------|
|Core GSN                     | :heavy_check_mark: full |
|Argument Pattern Extension   | :x: not planned         |
|Modular Extension            | :construction: see #6   |
|Confidence Argument Extension| :x: not planned         |
|Dialectic Extension          | :x: not planned         |
