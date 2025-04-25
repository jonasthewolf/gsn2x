
# Checks

## Validation checks

The tool automatically performs the following validation checks on the input YAML.

Validations can be performed on individual input files.

| ID  | Meaning                                                                                      |
|-----|----------------------------------------------------------------------------------------------|
| V01 | All IDs must either start with a known prefix or a `node_type` must explicitly set.          |
| V02 | All Goals and Strategies must be either marked with `undeveloped: true` or have supporting Goals, Strategies or Solutions. |
| V03 | Goals and Strategies marked as undeveloped, must have no supporting arguments.               |
| V04 | All elements listed under `supportedBy` and `inContextOf` must be known elements types and semantically sensible (e.g. a Justification cannot be listed under `supportedBy`). |
| V05 | All referenced elements in `supportedBy` and `inContextOf` must be unique i.e., no duplicates in the list.  |
| V06 | All referenced elements in `supportedBy` and `inContextOf` must not refer to the element itself.            |
| V07 | All elements listed as extending other elements must be known elements of the current module and semantically sensible (see V04). |
| V08 | The ID's start contradicts the type of the element set with `node_type`. **Note: only reported with `--extended-check` option.** |
| V09 | Element has an assurance claim point that references another element, that this is neither its own ID nor any of the connected elements.|
| V10 | An element is marked as defeated, but has no other elements challenging it. |
| V11 | A CounterGoal or CounterSolution is used in input files. **Note: only reported with `--warn-dialectic` option.** |

The following checks apply to the complete set of input files.

| ID  | Meaning                                                                                                |
|-----|--------------------------------------------------------------------------------------------------------|
| C01 | There should be only one but must be at least one top-level element (G,S,C,J,A,Sn) unreferenced.       |
| C02 | The top-level element must be a Goal. A top-level element is an element that is not referenced by any other element.|
| C03 | All referenced elements in `supportedBy`, `inContextOf`, `challenges` must exist.                      |
| C04 | There must be no circular `supportedBy` references.                                                    |
| C06 | All module names must be unique.                                                                       |
| C07 | All IDs must be unique across all modules.                                                             |
| C08 | All elements must be reachable from the root elements. This message can e.g. happen if there are multiple independent graphs where one contains circular references only.|
| C09 | All extended modules must exist.                                                                       |
| C10 | All extended elements must exist in the named module and must be undeveloped.                          |
| C11 | The reference that is not found (see C03), could actually be a list, but a YAML string was used. Use [] around your comma separated references. |

Uniqueness of keys (i.e. element IDs) is automatically enforced by the YAML format.

If called with option `-c` or `--check` the input file is only checked for validity, but the resulting graph is not written.
The checks for references (Cxx) can be skipped for individual files by using the `-x` option.

## Format of messages

Error messages and warnings are printed to stderr.

The following format is used:

    (Warning|Error): \((?<module>.+)\) \((?<num>[CV][0-9][0-9])\): (?<msg>.+) 
