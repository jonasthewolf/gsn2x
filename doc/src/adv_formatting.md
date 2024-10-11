
# Formatting

gsn2x allows for formatting of hyperlinks 


## Hyperlinks

Markdown without title, only text and href
URL without whitespace


hyperlinks are automatically detected http://, https://, file://

Please note that the link created by `url:` cannot have additional text, since it is invisible and applicable to the complete node.

### Example

## Text emphasis

### Example

# Text layout within elements

You can control line breaks by YAML means, e.g. following this example:

```yaml
G1:
  text: |
    This
    is
    shown
    on
    separate
    lines
```

Alternatively, you can use the `-w` option and provide a global number of characters after which lines are wrapped.

You can also use the optional `charWrap` attribute for an element to individually define the number of characters after which line is wrapped. The same attribute can be applied for a complete module at the `module` [section](./mod_info.md).

**Please note that wrapping is done if a whitespace is detected after the given number of characters.**




