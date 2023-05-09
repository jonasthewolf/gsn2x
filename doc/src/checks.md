
# Checks

## Validation checks

The tool automatically performs the following validation checks on the input YAML.

| ID  | Meaning                                                                                      |
|-----|----------------------------------------------------------------------------------------------|
| V01 | All IDs must start with a known prefix i.e., there are only known element types.             |
| V02 | All Goals and Strategies must be either marked with `undeveloped: true` or have supporting Goals, Strategies or Solutions. |
| V03 | Goals and Strategies marked as undeveloped, must have no supporting arguments.               |
| V04 | All elements listed under `supportedBy` and `inContextOf` must be known elements types and semantically sensible (e.g. a Justification cannot be listed under `supportedBy`). |
| V05 | All referenced elements in `supportedBy` and `inContextOf` must be unique i.e., no duplicates in the list.  |
| V06 | All referenced elements in `supportedBy` and `inContextOf` must not refer to the element itself.            |
| V07 | All elements listed as extending other elements must be known elements of the current module and semantically sensible (see V04). |



| ID  | Meaning                                                                                      |
|-----|----------------------------------------------------------------------------------------------|
| C01 | There should be only one but must be at least one top-level element (G,S,C,J,A,Sn) unreferenced. 
| C02 | The top-level element must be a Goal. A top-level element is an element that is not referenced by any other element.
| C03 | All referenced elements in `supportedBy` and `inContextOf` must exist.
| C04 | There must be no circular `supportedBy` references.
| C05 | There should be more than one usage of the same `level`.
| C06 | All module names must be unique.
| C07 | All IDs must be unique across all modules.
| C08 | All elements must be reachable from the root elements.      This message can e.g. happen if there are multiple independent graphs where one contains circular references only.
| C09 | All extended modules must exist.
| C10 | All extended elements must exist in the named module and must be undeveloped.

The checks (Cxx) always apply to the complete set of input files.

Uniqueness of keys (i.e. element IDs) is automatically enforced by the YAML format.

Error messages and warnings are printed to stderr.

If called with option `-c` or `--check` the input file is only checked for validity, but the resulting graph is not written.
The checks for references (Cxx) can be skipped for individual files by using the `-x` option.

TODO Format of errors
