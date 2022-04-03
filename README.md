[![License: CC BY 4.0](https://img.shields.io/badge/License-CC%20BY%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by/4.0/) [![CI/CD](https://github.com/jonasthewolf/gsn2x/actions/workflows/rust.yml/badge.svg)](https://github.com/jonasthewolf/gsn2x/actions/workflows/rust.yml) [![codecov](https://codecov.io/gh/jonasthewolf/gsn2x/branch/master/graph/badge.svg?token=YQKUQQOYS3)](https://codecov.io/gh/jonasthewolf/gsn2x)

# gsn2x

This little program renders [Goal Structuring Notation](https://scsc.uk/gsn) in a YAML format to a scalable vector graphics (SVG) image.

![Example](examples/example.gsn.svg "Example")

Feel free to use it and please let me know. Same applies if you have feature requests, bug reports or contributions.

## Usage

You can create an SVG like this:

    gsn2x <yourgsnfile.yaml> 

The output is automatically written to `<yourgsnfile.svg>`. 

    
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

Every element may have an optional `url` attribute that creates a navigation link in the resulting SVG.
This should support finding information more easily.

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


Please see [examples/example.gsn.yaml] for an example of the used syntax.

## Validation checks

The tool automatically performs the following validation checks on the input YAML:

 - V01: All IDs start with a known prefix i.e., there are only known element types.
 - V02: All Goals and Strategies are either marked with `undeveloped: true` or have supporting Goals, Strategies or Solutions.
 - V03: Goals and Strategies marked as undeveloped, must have no supporting arguments.
 - V04: All elements listed under `supportedBy` and `inContextOf` are known elements types and semantically sensible
        (e.g. a Justification cannot be listed under `supportedBy`).
 - V05: All referenced elelemts in `supportedBy` and `inContextOf` are unique i.e., no duplicates in the list.
 - V06: All referenced elelemts in `supportedBy` and `inContextOf` do not refer to the node itself.
 - C01: There is only one top-level element (G,S,C,J,A,Sn) unreferenced. 
 - C02: The top-level element is a Goal. A top-level element is an element that is not referenced by any other element.
 - C03: All referenced elements in `supportedBy` and `inContextOf` exist.
 - C04: There are no circular `supportedBy` references.
 - C05: There is more than one usage of the same `level`.

The checks (Cxx) always apply to the complete set of input files.

Uniqueness of keys is automatically enforced by the YAML format.

Error messages and warnings are printed to stderr.

If called with option `-c` or `--check` the input file is only checked for validity, but the resulting graph is not written.
The checks for references (Cxx) can be skipped for individual files by using the `-x` option.

## Additional layers

Additional attributes of a node are ignored by default.
With the command line option `-l` or `--layers` you can enable the output of those additional attributes.
By using this feature different views on the GSN can be generated.

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

Please note that using `module` and passing it as a layer option will also not work. 

It is intentional that information is only added for a view, but not hidden to ensure consistency of the GSN in all variants.

## Stylesheets for SVG rendering

You can provide custom stylesheets for SVG via the `-s` or `--stylesheet` options.

Every element will also be addressable by `id`. The `id` is the same as the YAML id.

Elements are assigned `gsnelem` class, edges are assigned `gsnedge` class. 

The complete diagram is assigned `gsndiagram` class.

You can assign additional classes by adding the `classes:` attribute. It must be a list of classes you want to assign. Additional layers will be added as CSS classes, too. A `layer1` will e.g. be added as `gsnlay_layer1`.

### Example

    G1:
      text: This is a Goal
      classes: [additionalclass1, additionalclass2]

## Logical levels for elements

To influence the rendered image, you can add an identifier to a GSN element with the `level` attribute. All elements with the same identifier for `level` will now show up on the same horizontal level. 

This is especially useful, if e.g., two goals or strategies are on the same logical level, but have a different "depth" in the argumentation (i.e. a different number of goals or strategies in their path to the root goal).

See the [example](examples/example.gsn.yaml) for usage. The strategies S1 and S2 are on the same level.

It is recommended to use `level` only for goals, since related contexts, justifications and assumptions are automatically put on the same level.

## Modular Extension

gsn2x partially supports the Modular Extension of the GSN standard (see [Standard support](#standard-support)).
Module Interfaces (Section 1:4.6) and Inter-Module Contracts (Section 1:4.7) are not supported.

Each module is a separate file. The name of the module is the file name (incl. the path provided to the gsn2x command line).

If modules are used, all related module files must be provided to the command line of gsn2x.
Element IDs must be unique accross all modules. Checks will by default be performed accross all modules.
Check messages for individual modules can be omitted using the `-x` option.

The argument view of individual modules will show "away" elements if elements from other modules are referenced.

In addition to the default argument view for each module, there are two output files generated (if more than one input file is provided):
1) Complete View (complete.svg)
2) Architecture View (architecture.svg)

If the argument view should not be updated, use the `-N` option.
If the complete view should not be output, use the `-F` option.
If the architecture view should not be output, use the `-A` option.

### Complete View

The complete view is a similar to an argument view for a single module, but showing all modules within the same diagram. The modules are "unrolled". Modules can be masked i.e., unrolling is prevented, by additionally 
adding those modules with the `-m` option.

### Architecture View

The architecture view only shows the selected modules and their dependencies.

### Example:
    
    gsn2x -f full.dot -a arch.dot -m sub1.yml main.yml sub1.yml sub3.yml sub5.yml  

This will generate the argument view for each module, the complete view (`-f full.dot`) of all modules and the architecture view (`-a arch.dot`). In the complete view, the elements of the `sub1` module will be represented by a module.

## List of evidences

An additional file that lists all the evidences in the input file is output by default in `evidences.md`.

See [examples/evidences.md] for an example.

The format can be used in Markdown and reStructuredText files.

If the list of evidences should not be output, use the `-E` option.

## Standard support

This tool is based on the [Goal Structuring Notation Community Standard Version 3](https://scsc.uk/r141C:1).

This table shows the support of `gsn2x` for the different parts of the standard.

| Standard                    | Support                                                                        |
|-----------------------------|--------------------------------------------------------------------------------|
|Core GSN                     | :heavy_check_mark: full                                                        |
|Argument Pattern Extension   | :x: not planned                                                                |
|Modular Extension            | :part_alternation_mark: partially, see [Modular Extension](#modular-extension) |
|Confidence Argument Extension| :x: not planned                                                                |
|Dialectic Extension          | :x: not planned                                                                |
