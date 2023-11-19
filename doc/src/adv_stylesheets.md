# Stylesheets for SVG rendering

You can provide (multiple) custom CSS stylesheets for SVG via the `-s` or `--stylesheet` options. 

The path may be relative to the current working directory, absolute, or an URL (i.e. starting with `http://`, `https://` or `file://`).

When adding `-t` or `--embed-css` on the command line, the CSS stylesheets will be embedded in the SVG. 

If an output path is provided (see [Basic usage](./basic_usage.md)), the stylesheet(s) will be copied there.
If a relative path is used, the relative path to the current working directory is preserved. 
If an absolute path is used, the stylesheet will be copied to the root of the output path.

If a URL (see above for definition) is provided for a stylesheet, it is neither embedded nor copied to an output directory.

## Classes and styles

Every element will also be addressable by `id`. The `id` is the same as the YAML id.

This table shows the CSS classes assigned to a certain element:

| Class               | Assigned to                                | SVG Element  |
|---------------------|--------------------------------------------|--------------|
| gsndiagram          | The complete diagram                       | svg          |
| gsnelem             | All elements                               | g            |
| gsngoal             | Goal                                       | g            |
| gsn_undeveloped     | Undeveloped                                | g            |
| gsnsltn             | Solution                                   | g            |
| gsnawaysltn         | Away Solution                              | g            |
| gsnstgy             | Strategy                                   | g            |
| gsnasmp             | Assumption                                 | g            | 
| gsnawayasmp         | Away Assumption                            | g            |
| gsnjust             | Justification                              | g            |
| gsnawayjust         | Away Justification                         | g            |
| gsnctxt             | Context                                    | g            |
| gsnawayctxt         | Away Context                               | g            |
| gsnmodule           | Module                                     | g            |
| gsn_module_`module` | Module name                                | g            |
| gsnedge             | All edges                                  | path         |
| gsnlay_`<layer>`    | Layer `<layer>`                            | path         |
| gsninctxt           | In Context Of                              | path         |
| gsnspby             | Supported By                               | path         | 
| gsncomposite        | Composite (In Context Of AND Supported By) | path         |

You can assign additional classes by adding the `classes:` attribute. It must be a list of classes you want to assign. 
Additional layers will be added as CSS classes, too. A `layer1` will e.g. be added as `gsnlay_layer1`.

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

# Highlighting elements when navigating

The CSS `:target` pseudo class can be used to highlight the element you clicked on in the previous image.

An example could look like this:

```css
g:target path {
    fill: lightsteelblue;
}
```



