
# Syntax in YAML

## Elements

The following Goal Structuring Notation (GSN) core elements are supported:

| Element Type   | Prefix |
|----------------|--------|
| Goal           |   G    | 
| Assumption     |   A    |
| Justification  |   J    | 
| Solution       |   Sn   |   
| Context        |   C    |
| Strategy       |   S    |

Every element is defined by a prefix (as shown in the table above) and an arbitrary identifier then.

### Examples
 
```yaml
G1:

G-TopLevelGoal:

C_A_certain_context:
```

## Attributes

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

## Summary


| Attribute        | Optional |
|------------------|----------|
| text             |    no    | 
| supportedBy      |    yes   |
| inContextOf      |    yes   | 
| undeveloped[^nu] |    yes   |
| url              |    yes   |   
| classes[^nc]     |    yes   |
| level[^nl]       |    yes   |
| nodeType[^nt]    |    yes   |

[^nu]: Mutually exclusive to `supportedBy`.

[^nc]: See [Stylesheets](./adv_stylesheets.md).

[^nl]: See [Layout](./adv_layout.md).

[^nt]: When providing a `nodeType` you do not need to follow the standard prefix scheme above.
       Just give `Goal`, `Assumption`, `Justification`, `Solution`, `Context` and `Strategy` to give the type of the element.
