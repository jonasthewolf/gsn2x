
# More influence on the layout of elements

## Vertical placement

To influence the rendered image, you can add an identifier to a GSN element with the `level` attribute. 
All elements with the same identifier for `level` will now show up on the same vertical level. 

This is especially useful, if e.g., two goals or strategies are on the same logical level, 
but have a different "depth" in the argumentation (i.e. a different number of goals or strategies in their path to the root goal).

See the [example](examples/example.gsn.yaml) for usage. The strategies S1 and S2 are on the same level.

It is recommended to use `level` only for goals, since related contexts, 
justifications and assumptions are automatically put on the same level.

## Horizontal placement

The order of the GSN elements on the same horizontal rank can be influenced by their ID.
The elements are sorted lexicographically. Thus, a goal `G1` if placed on the same vertical level is placed before `G2`, 
if they have the same depth of their supporting arguments.

There can be situations (e.g. a n:m relation between goals and solutions) that lead to weird looking graphs.
You may even encounter the following message `Rendering a diagram took too many iterations. See README.md for hints how to solve this situation.`. In such cases, please file an issue on GitHub, so I can see how the algorithm can be improved.

