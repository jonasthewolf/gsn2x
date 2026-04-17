
# Troubleshooting

## Sanitizing files for e.g. support requests

You can sanitize your files from your intellectual property using, e.g. [https://mikefarah.gitbook.io/yq/](https://mikefarah.gitbook.io/yq/).

This statement replaces all text with `x`es while keeping the number of characters:

```console
 yq "(.[] | select(. | has(\"text\"))) .text |=sub(\"[a-zA-Z0-9]\",\"x\"), ... comments=\"\""  inputfile.yaml > outputfile.yaml
```

**Please note that element identifiers and [additional layers](./adv_layers.md) are not sanitizied.**
