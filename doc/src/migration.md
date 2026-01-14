# Migration hints

## Migrating from Version 2.x to Version 3.x

Version 2 of gsn2x tried to fully automate the layout of diagrams.
Version 3 intentionally changed this approach.

To get similar or even better renderings, please follow the guidelines below.

**Replace `level` with `rankIncrement`**

`level` was deprecated. Instead, increment the rank of the elements that should be pushed further down.

Please see [Layout](./adv_layout.md) for more information on how to use `rankIncrement`.

**Use strict lexicographical sorting and horizontal reordering to optimize the graph.**

If you find situations that are not rendered correctly by default,
please check if your IDs are sensibly defined (i.e. lexicographically increasing).
Use `horizontalIndex` as described in the [layout section](./adv_layout.md) to fix e.g., crossing edges.

## Migration from Version 3.x to Version 4.x

Remove all `horizontalIndex` again and try to render the diagrams. The ranking algorithm was improved.
Thus, problems should be minimized. Introduce the `horizontalIndex` again, where necessary to achieve the desired diagram.

I change the options are parsed. Since the input files are handed over using positional arguments, all options now require a `=` to assign the values.

When drafting a new assurance case you can now have "empty" elements with just an identifier.
To do so, use the empty dictionary syntax of YAML:

```yaml
G1: {}
```
