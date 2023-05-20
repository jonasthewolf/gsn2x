# Stylesheets for SVG rendering

You can provide (multiple) custom CSS stylesheets for SVG via the `-s` or `--stylesheet` options. 
The path is not interpreted by gsn2x and, thus, is relative to the SVG if relative.

Every element will also be addressable by `id`. The `id` is the same as the YAML id.

Elements are assigned `gsnelem` class, edges are assigned `gsnedge` class. 

The complete diagram is assigned `gsndiagram` class.

You can assign additional classes by adding the `classes:` attribute. It must be a list of classes you want to assign. 
Additional layers will be added as CSS classes, too. A `layer1` will e.g. be added as `gsnlay_layer1`.

When using `-t` or `--embed-css` instead of `-s` the CSS stylesheets will be embedded in the SVG. The path is interpreted as relative to the current working directory then.

For more information on how to use CSS with SVGs, see [here](https://developer.mozilla.org/en-US/docs/Web/SVG/Tutorial/SVG_and_CSS).

## Example

The GSN YAML: 

```yaml
{{#include examples/minimalcss/min.gsn.yaml}}
```

The corresponding CSS:

```css
{{#include examples/minimalcss/min.css}}
```

The result looks like this:

![Styled Example](examples/minimalcss/min.gsn.svg)



