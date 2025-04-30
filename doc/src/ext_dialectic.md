
# Dialectic Extension

The Dialectic Extension introduces two new elements: the `Counter Goal` and the `Counter Solution`.

A Counter Goals or Counter Solutions `challenges` other elements.

If that leads to a defeat, those elements can be marked as `defeated: true`.
If this also affects a relation, a node can mark one of its relation as defeated using: `defeatedRelation: G1`.

## Example Source

The following YAML shows an example of their usage from the GSN standard: 

```yaml
{{#include examples/dialectic/first.gsn.yaml}}
```

## Rendered Example

![Rendered Example](examples/dialectic/first.gsn.svg)


```yaml
{{#include examples/dialectic/second.gsn.yaml}}
```

## Rendered Example

![Rendered Example](examples/dialectic/second.gsn.svg)