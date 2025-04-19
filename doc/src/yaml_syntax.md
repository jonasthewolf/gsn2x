
# Syntax in YAML

## Elements

The following Goal Structuring Notation (GSN) core elements are supported:

| Element Type     | Prefix |
|------------------|--------|
| Goal             |   G    | 
| Assumption       |   A    |
| Justification    |   J    | 
| Solution         |   Sn   |   
| Context          |   C    |
| Strategy         |   S    |
| Counter Goal     |   CG   |
| Counter Solution |   CSn  |

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


| Attribute       | Optional | Notes                                                  |
|-----------------|----------|--------------------------------------------------------|
| text            |    no    |                                                        | 
| supportedBy     |    yes   |                                                        |
| inContextOf     |    yes   |                                                        | 
| undeveloped     |    yes   | Mutually exclusive to `supportedBy`.                   |
| url             |    yes   |                                                        |   
| classes         |    yes   | See [Stylesheets](./adv_stylesheets.md).               |
| nodeType        |    yes   | See footnote[^nt]                                      |
| rankIncrement   |    yes   | See [Layout](./adv_layout.md).                         |
| horizontalIndex |    yes   | See [Layout](./adv_layout.md).                         |
| charWrap        |    yes   | See [Line Breaks](./adv_formatting.md).                |
| acp             |    yes   | See [Confidence Argument Extension](./ext_confidence.md).  |
| challenges      |    yes   | See [Dialectic Extension](./ext_dialectic.md).             |
| defeated        |    yes   | See [Dialectic Extension](./ext_dialectic.md).             |

[^nt]: When providing a `nodeType` you do not need to follow the standard prefix scheme above.
       Just set `nodeType` to `Goal`, `Assumption`, `Justification`, `Solution`, `Context`, `Strategy`, `CounterGoal` and  `CounterSolution` to give the type of the element.
