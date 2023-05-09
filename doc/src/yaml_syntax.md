
# Syntax in YAML

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

## Example

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
