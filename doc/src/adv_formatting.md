
# Formatting

gsn2x allows for different ways to formatting text.
Formatting loosely follows Markdown syntax, even though not the full specification is supported.

## Text emphasis

You can use `*` and `_` to emphasize text.

Text enclosed in `*`'s will be assigned a CSS class "bold".

Text enclosed in `_`'s will be assigned a CSS class "italic".

A default inline style is also assigned.

### Example

This text:

```
This is a *bold* text. This is an _italic_ text.
```

will be rendered to:

`This is a`**`bold`**`text. This is an`*`italic`*`text.`


## Hyperlinks

You can add hyperlinks for `text:` attributes as well as all additional layers.
Hyperlinks are automatically detected when they start with "http://", "https://", "file://".

If you like to hide the actual URL, you can assign a text to the link that is rendered instead.
The syntax for this follows Markdown syntax. However, a title is not supported, only `text` and `href` are.
URLs may not contain whitespace characters. If you URL has one, just replace with `%20`.

Please note that the link created by `url:` cannot have additional text, since it is anyway invisible and applicable to the complete node.

### Example

This link:

```
[Link Text](https://github.com/jonasthewolf/gsn2x)
```

will be rendered to:

[Link Text](https://github.com/jonasthewolf/gsn2x)

This link:

```
https://github.com/jonasthewolf/gsn2x
```

will be rendered to:

[https://github.com/jonasthewolf/gsn2x](https://github.com/jonasthewolf/gsn2x)

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




