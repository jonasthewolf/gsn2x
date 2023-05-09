
# Advanced use-cases

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


## List of evidences

An additional file that lists all the evidences in the input file is output by default in `evidences.md`.

See [examples/evidences.md](examples/evidences.md) for an example.

The format can be used in Markdown and reStructuredText files.

If the list of evidences should not be output, use the `-E` option.
