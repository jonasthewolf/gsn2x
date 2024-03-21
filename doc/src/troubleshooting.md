
# Troubleshooting

## YAML error messages

If you encounter an error message like this, 
please use a YAML editor or online validator to check for the well-formedness 
of the input file:

```
Error: Failed to parse YAML from file <filename>

Caused by:
    No valid GSN element can be found starting from line <n>.
    This typically means that the YAML is completely invalid, or 
    the `text:` attribute is missing for an element.
    Please see the documentation for details (https://jonasthewolf.github.io/gsn2x/troubleshooting.html).
    Original error message: data did not match any variant of untagged enum GsnDocument.
```

Please see the [YAML Syntax](./yaml_syntax.md) what is expected by the programs.

A good strategy to find out which element is causing the problem is to remove all but the first element from the input YAML file. Then running gsn2x again and incrementally adding the elements one by one again, until you hit the
error message again.

Unfortunately, it is currently not possible to improve on the location of the error messages in this case.

## Sanitizing files that you need to 

You can sanitize your files from your intellectual property using, e.g. https://mikefarah.gitbook.io/yq/ .

This might be necessary to provide input for getting support.

The statement replaces all text with `x`es while keeping the number of characters.
Please note that [additional layers](./adv_layers.md) are not sanitizied.

```
 yq "(.[] | select(. | has(\"text\"))) .text |=sub(\"[a-zA-Z0-9]\",\"x\")"  inputfile.yaml | yq "... comments=\"\"" > outputfile.yaml
```
