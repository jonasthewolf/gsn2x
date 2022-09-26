[![License: CC BY 4.0](https://img.shields.io/github/license/jonasthewolf/gsn2x)](https://creativecommons.org/licenses/by/4.0/)
[![CI](https://img.shields.io/github/workflow/status/jonasthewolf/gsn2x/CI?label=CI)](https://github.com/jonasthewolf/gsn2x/actions/workflows/rust.yml)
[![codecov](https://img.shields.io/codecov/c/github/jonasthewolf/gsn2x/master?token=YQKUQQOYS3)](https://codecov.io/gh/jonasthewolf/gsn2x)
![LoC](https://img.shields.io/tokei/lines/github/jonasthewolf/gsn2x)
[![Downloads](https://img.shields.io/github/downloads/jonasthewolf/gsn2x/total)](https://github.com/jonasthewolf/gsn2x/releases)

# gsn2x

This little program renders [Goal Structuring Notation](https://scsc.uk/gsn) in a YAML format to a scalable vector graphics (SVG) image.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="examples/example.gsn_dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="examples/example.gsn.svg">
  <img alt="Example" src="examples/example.gsn.svg">
</picture>

Feel free to use it and please let me know. Same applies if you have feature requests, bug reports or contributions.

## Usage

You can create an SVG like this:

    gsn2x <yourgsnfile.yaml> 

The output is an argument view in SVG format and automatically written to `<yourgsnfile.svg>`. If more than one input file is provided, they are treated as [modules](#modular-extension).

    
**You can find pre-built binaries for Windows, Linux and MacOS on the [releases page](https://github.com/jonasthewolf/gsn2x/releases).**

## Syntax in YAML

The following Goal Structuring Notation (GSN) core elements are supported:
 - Goal (G), 
 - Assumption (A), 
 - Justification (J), 
 - Solution (Sn),
 - Context (C), and
 - Strategy (S)

Every element is defined by a prefix (as shown in the list above) and a number.
Actually, the number can be an arbitrary identifier then.

The only mandatory attribute is `text` that is the textual contents of the element.

An optional `supportedBy` gives a list of the supporting arguments. Thus, Goal, Strategy and Solution can be listed here.

An optional `inContextOf` links Justifications, Contexts or Assumptions. 

Every element may have an optional `url` attribute that creates a navigation link in the resulting SVG.
This should support finding information more easily.

Goals and Strategies can be undeveloped i.e., without supporting Goals, Strategies or Solutions.
These elements should marked with `undeveloped: true`, otherwise validation will emit warnings.

### Example

```yaml
G1:
  text: This is a Goal
  supportedBy: [S1]
  inContextOf: [C1]

S1:
  text: This is a Strategy

C1: 
  text: This is a Context
```

Please see [examples/example.gsn.yaml](examples/example.gsn.yaml) for an example of the used syntax.

## Validation checks

The tool automatically performs the following validation checks on the input YAML:

 - V01: All IDs must start with a known prefix i.e., there are only known element types.
 - V02: All Goals and Strategies must be either marked with `undeveloped: true` or have supporting Goals, Strategies or Solutions.
 - V03: Goals and Strategies marked as undeveloped, must have no supporting arguments.
 - V04: All elements listed under `supportedBy` and `inContextOf` must be known elements types and semantically sensible
        (e.g. a Justification cannot be listed under `supportedBy`).
 - V05: All referenced elements in `supportedBy` and `inContextOf` must be unique i.e., no duplicates in the list.
 - V06: All referenced elements in `supportedBy` and `inContextOf` must not refer to the element itself.
 - V07: All elements listed as extending other elements must be known elements of the current module and semantically sensible (see V04).
 - C01: There should be only one but must be at least one top-level element (G,S,C,J,A,Sn) unreferenced. 
 - C02: The top-level element must be a Goal. A top-level element is an element that is not referenced by any other element.
 - C03: All referenced elements in `supportedBy` and `inContextOf` must exist.
 - C04: There must be no circular `supportedBy` references.
 - C05: There should be more than one usage of the same `level`.
 - C06: All module names must be unique.
 - C07: All IDs must be unique across all modules.
 - C08: All elements must be reachable from the root elements.
        This message can e.g. happen if there are multiple independent graphs where one contains circular references only.
 - C09: All extended modules must exist.
 - C10: All extended elements must exist in the named module and must be undeveloped.

The checks (Cxx) always apply to the complete set of input files.

Uniqueness of keys (i.e. element IDs) is automatically enforced by the YAML format.

Error messages and warnings are printed to stderr.

If called with option `-c` or `--check` the input file is only checked for validity, but the resulting graph is not written.
The checks for references (Cxx) can be skipped for individual files by using the `-x` option.

## Additional layers

Additional attributes of an element are ignored by default.
With the command line option `-l` or `--layers` you can enable the output of those additional attributes.
By using this feature different views on the GSN can be generated.

### Example

```yaml
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
```

In this example, a call to `gsn2x -l layer1` will show the additional information to each element prefixed with _`LAYER1: `_.
Of course, using `text`, `inContextOf`, `supportedBy`, `url`, `undeveloped`, `level` or `classes` are not sensible parameters to pass for the `-l` option. 

Please note that using `module` and passing it as a layer option will also not work. 

It is intentional that information is only added for a view, but not hidden to ensure consistency of the GSN in all variants.

## Stylesheets for SVG rendering

You can provide (multiple) custom CSS stylesheets for SVG via the `-s` or `--stylesheet` options. 
The path is not interpreted by gsn2x and, thus, is relative to the SVG if relative.

Every element will also be addressable by `id`. The `id` is the same as the YAML id.

Elements are assigned `gsnelem` class, edges are assigned `gsnedge` class. 

The complete diagram is assigned `gsndiagram` class.

You can assign additional classes by adding the `classes:` attribute. It must be a list of classes you want to assign. 
Additional layers will be added as CSS classes, too. A `layer1` will e.g. be added as `gsnlay_layer1`.

When using `-t` or `--embed-css` instead of `-s` the CSS stylesheets will be embedded in the SVG. The path is interpreted as relative to the current working directory then.

### Example

```yaml
G1:
  text: This is a Goal
  classes: [additionalclass1, additionalclass2]
```

## Influencing layout of elements

### Vertical placement

To influence the rendered image, you can add an identifier to a GSN element with the `level` attribute. 
All elements with the same identifier for `level` will now show up on the same vertical level. 

This is especially useful, if e.g., two goals or strategies are on the same logical level, 
but have a different "depth" in the argumentation (i.e. a different number of goals or strategies in their path to the root goal).

See the [example](examples/example.gsn.yaml) for usage. The strategies S1 and S2 are on the same level.

It is recommended to use `level` only for goals, since related contexts, 
justifications and assumptions are automatically put on the same level.

### Horizontal placement

There can be situations (e.g. a n:m relation between goals and solutions) that lead to weird looking graphs.
You may even encounter the following message `Rendering a diagram took too many iterations. See README.md for hints how to solve this situation.`

The order of the GSN elements on the same horizontal rank can be influenced by their ID.
The elements are sorted lexicographically. Thus, a goal `G1` if placed on the same vertical level is placed before `G2`.

## Modular extension

gsn2x partially supports the Modular Extension of the GSN standard (see [Standard support](#standard-support)).
Module Interfaces (Section 1:4.6) and Inter-Module Contracts (Section 1:4.7) are not supported.

Each module is a separate file. The name of the module is the file name (incl. the path provided to the gsn2x command line).

If modules are used, all related module files must be provided to the command line of gsn2x.
Element IDs must be unique across all modules. Checks will by default be performed across all modules.
Check messages for individual modules can be omitted using the `-x` option.

The argument view of individual modules will show "away" elements if elements from other modules are referenced.

Note: There is no "away strategy" in the standard.

In addition to the default argument view for each module, there are two output files generated (if more than one input file is provided):
1) Complete View (default to: complete.svg)
2) Architecture View (default to: architecture.svg)

If the argument view should not be updated, use the `-N` option.
If the complete view should not be output, use the `-F` option.
If the architecture view should not be output, use the `-A` option.

### Complete view

The complete view is a similar to an argument view for a single module, 
but showing all modules within the same diagram. The modules are "unrolled". 
<!-- Modules can be masked i.e., unrolling is prevented, 
by additionally adding those modules with the `-m` option. -->

See [example](examples/modular/complete.svg) here.

### Architecture view

The architecture view only shows the selected modules and their dependencies.

See [example](examples/modular/architecture.svg) here.

### Example:
    
    gsn2x -f full.svg -a arch.svg -m sub1.yml main.yml sub1.yml sub3.yml sub5.yml  

This will generate the argument view for each module, the complete view (`-f full.svg`) of all modules and the architecture view (`-a arch.svg`). In the complete view, the elements of the `sub1` module will be represented by a module.

### Developing undeveloped elements from other modules

In a customer supplier relationship it may be helpful to develop otherwise undeveloped elements from other modules.
This allows creating distributed assurance cases.

Example for a module with undeveloped elements:

```yaml
module:
  name: template 
  brief: Template for an assurance case

G1:
  text: A claim somebody else should support
  undeveloped: true
```

Example for developing those elements in another module:

```yaml
module:
  name: instance
  brief: Extending instance
  extends: 
    - module: template
      develops:
        G1: [G2]

G2:
  text: This is the argument provided by somebody else.
  supportedBy: [Sn1]

Sn1:
  text: A solution
```


## List of evidences

An additional file that lists all the evidences in the input file is output by default in `evidences.md`.

See [examples/evidences.md](examples/evidences.md) for an example.

The format can be used in Markdown and reStructuredText files.

If the list of evidences should not be output, use the `-E` option.

## Optional module information

It is possible to add additional `module` information in the source YAML.
This allows describing the module`s name and an optional brief description.
Even arbitrary information can be added. 

`name` and `brief` are mandatory if a `module` is added.

```yaml

module: 
   name: MainModule
   brief: This is a short description of the module
   additionalInformation: 
    v1: Changed line 2
    v2: Added line 4

```

The module information is printed as part of a legend for the argument view.

You can use the `-G` option to suppress the legend completely, 
or the `-g` option to limit it to `name`, `brief` and the time and date of generation of the SVG.

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

## Design goals

I noticed that it might make sense to add some information about the goals I have set for myself for this project:

- Everything-as-code

  The tool should work in a continuos integration environment.
  The input format should be diff-able to support common git workflows with pull/merge requests.

- Simplicity

  I would like to keep things simple. Simple for me and others.

  That means the input format should be simple to learn and edit manually in any editor. 
  I also did not want invent a new DSL (domain-specific language) for that purpose.
  YAML (input file format) might not be the best format, but it serves as a good tradeoff for my purposes.
  Moreover, it can be parsed by other programs easily, too.

- Standard conformance

  I would like the program output to be very close to the GSN standard.

  I don't want to redefine the semantics or add additional ones. 
  The standard was created that as many people as possible have some common grounds.
  If I added new fancy stuff, everyone might have a different interpretation of that again.

- As few dependencies as possible

  Since I understand that this tool might be used in some corporate environment where usage of 
  free and open-source software might be limited, I try to keep the dependencies of this program 
  as few as possible.

## History

I also noticed that (also for myself) it is good to note down some history of the project:

- It all started out in 2017 with the need for graphically representing some argumentation at work.
  I wrote a tiny Python script that used a jinja template to transform the YAML syntax
  into something Graphviz could understand.

  From there Graphviz could generate different output formats. That's where the `x` in `gsn2x` is from.

- It got obvious that some validation, especially on the uniqueness and reference resolution is needed
  to handle larger argumentation.
  
  I did not want to write those validations in Python, but in my favorite programming language Rust.
  I released the first Rust version in July 2021.
  
- I desperately tried adding the modular extension by convincing Graphviz to draw what I want, but I failed.
  I finally made decided to no longer output DOT, but directly generate SVGs from the program.
  This required writing a specialized version for rendering the tree on my own which ended up in version 2 
  finally released in April 2022.

Any feedback, especially the use-case in your company is very much appreciated.

