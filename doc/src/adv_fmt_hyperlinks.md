
# Hyperlinks

You can add hyperlinks for `text:` attributes as well as all additional layers.
Hyperlinks are automatically detected when they start with "http://", "https://", "file://".

If you like to hide the actual URL, you can assign a text to the link that is rendered instead.
The syntax for this follows Markdown syntax. However, a title is not supported, only `text` and `href` are.
URLs may not contain whitespace characters. If you URL has one, just replace with `%20`.

Please note that the link created by `url:` cannot have additional text, since it is anyway invisible and applicable to the complete element.

## Example

This link:

```markdown
[Link Text](https://github.com/jonasthewolf/gsn2x)
```

will be rendered to:

[Link Text](https://github.com/jonasthewolf/gsn2x)

This link:

```markdown
https://github.com/jonasthewolf/gsn2x
```

will be rendered to:

[https://github.com/jonasthewolf/gsn2x](https://github.com/jonasthewolf/gsn2x)
