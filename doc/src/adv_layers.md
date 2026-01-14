
# Additional layers

Additional attributes of an element are not output into the rendered diagram.

With the command line option `-l` or `--layers` you can enable the output of those additional attributes.
By using this feature different views on the GSN can be generated.

## Example

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

In this example, a call to `gsn2x -l layer1` will show the additional information to each element prefixed with _`LAYER1:`_.

Of course, using `text`, `inContextOf`, `supportedBy`, `url`, `undeveloped`,
`horizontalRank`, `rankIncrement`, `acp`, `defeated`, `challenges` or `classes` are not sensible parameters to pass for the `-l` option.

Please note that using `module` and passing it as a layer option will also not work.

It is intentional that information is only added for a view, but not hidden to ensure consistency of the GSN in all variants.

Only additional associative arrays with a string key can be used as additional layers.
