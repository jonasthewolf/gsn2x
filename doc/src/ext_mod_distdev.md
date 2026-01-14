
# Developing undeveloped elements from other modules

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
