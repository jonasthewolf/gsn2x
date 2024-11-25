
# Layout of elements

## How does the default layout of the graph work?

At first, the root elements are identified. Root elements are elements 
that are not referenced as `supportedBy` or `inContextOf`. 

Each level in the rendered graph is called rank. 
The elements on each rank are sorted lexicographically before being placed on that rank. 
Then, starting with the first element on the current rank, the elements of the next rank are identified.
An element is only ranked if all elements referencing it are already placed.
Finally, the `inContextOf` elements are placed on that rank.

## Placement of elements 

### Vertical placement

To influence the vertical placement i.e., the rank, of an element in the rendered graph, 
you can use `rankIncrement` for a node. 
A rank increment is a positive number to push an element this amount of ranks downwards.
It is not possible to decrease the rank that would be assigned by the above algorithm.

Incrementing the rank is especially useful, if e.g., two goals or strategies are on the same logical level, 
but have a different "depth" in the argumentation (i.e. a different number of goals or strategies in their path to the root goal).

See the [example](examples/example.gsn.yaml) for usage. The strategy S2 has an incremented rank.

### Horizontal placement

The order of the GSN elements on the same rank is in the first place defined by their ID.
The elements are sorted lexicographically. Thus, a goal `G1` if placed on the same rank is placed left to `G2`.

You can use `horizontalIndex` to reorder elements after lexicographical sorting. 
The index can be modified by giving a relative or absolute index.

In the following example, G1 and G2 are placed on the same rank. 
However, G2 is placed left of G1 because of the relative horizontal index.

```yaml
G1:
    text: Goal 1
    undeveloped: true

G2: 
    text: Goal 2
    undeveloped: true
    horizontalIndex:
         relative: -1
```

Please see Sn2 and Sn4 in [example](examples/example.gsn.yaml) for how to use the absolute index. 

Typical use-cases for defining an absolute index are putting elements at the beginning or the end within a rank.
Please see the following example for how to use an absolute index. 

```yaml
G1:
    text: Goal 1
    undeveloped: true
    horizontalIndex:
        absolute: last

G2: 
    text: Goal 2
    undeveloped: true
    horizontalIndex:
        absolute: 0
```

Please note that giving the horizontal index for G1 and G2 is redundant. 

The absolute index is zero-based. You can use `last` to move elements 
to the very right of the graph.

The horizontal index can also be applied to `inContextOf` elements. 
You would typically use an absolute index with either `0` or `last` to place them
either left or right of the element they are referenced from.

`horizontalIndex` and `rankIncrement` can also be used for `module` elements. 
They will be used for the Architecture View then (see [Modular extension](modular_extension.md#architecture-view)).

### Troubleshooting

There can be situations (e.g. a n:m relation between goals and solutions) 
that lead to weird looking graphs.
You may even encounter the following message 
`Diagram took too many iterations ({run}). See documentation (https://jonasthewolf.github.io/gsn2x/) for hints how to solve this situation.`
In such cases, please use the mechanisms described above to support 
the algorithm in ranking the elements more sensibly.

Moreover, gsn2x also outputs a hint on the list of elements that might cause the problem.

If you have trouble doing so, please feel free to create an issue or 
start a discussion on the GitHub site. 
The issue template on GitHub shows you how to remove intellectual property
 from the files that I would ask for then.

#### Example

The following small example will yield the above mentioned message:

```yaml
{{#include examples/entangled.gsn.yaml}}
```

The created SVG is hardly readable: 

![entangled example](examples/entangled.gsn.svg)

The reason for this is, that `G1` is lexicographically ordered before `G2`, but `Sn2` after `Sn1`.
`Sn1` thus "pushes" `Sn2` to the right in each iteration of the layout algorithm.

The problem is easily solvable in different ways:

1. Rename `G1` and/or `G2` to change their lexicographical order.
2. Rename `Sn1` and/or `Sn2` to change their lexicographical order.
3. Apply `horizontalIndex: absolute: last` to `G1` **or** `Sn1`.
4. Apply `horizontalIndex: absolute: 0` to `G2` **or** `Sn2`.
5. Apply `horizontalIndex: relative: +1` to `G1` **or** `Sn1`.
6. Apply `horizontalIndex: relative: -1` to `G2` **or** `Sn2`.

The order above can be considered - as a general rule of thumb - as a suggested order when trying to resolve layout issues.