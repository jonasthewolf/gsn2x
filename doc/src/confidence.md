
# Confidence Argument Extension

The Confidence Argument Extension is supported. It defines so called Assurance Claim Points (ACPs).

They can be defined either for an element or for the relation between elements.


The following YAML shows an example of their usage: 

```yaml
{{#include examples/confidence.gsn.yaml}}
```

Assurance Claim Points (ACP) are defined with the `acp` attribute.
The name of the ACP is followed by:
- either a reference to the element itself, or
- a reference or a list of references to elements that are directly related (`supportedBy` or `inContextOf`).

There can be multiple ACPs per element.

![Rendered Example](examples/confidence.gsn.svg)