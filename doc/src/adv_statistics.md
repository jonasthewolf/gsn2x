
# Statistics

To print out statistics of the assurance case, use the `--statistics` command line option.

The `--statistics` option can be combined with the `-c` option to only check the input files, 
and not generate diagrams, but just the statistics.

If you run, e.g. `gsn2x -c --statistics examples/example.gsn.yaml`, you get the following output on standard output:

```
Statistics
==========
Number of modules:   1
Number of nodes:     20
  Goals:             7
  Strategies:        2
  Solutions:         5
  Assumptions:       2
  Justifications:    2
  Contexts:          2
  Counter Goals:     0
  Counter Solutions: 0
  Defeated Elements: 0
```

The output is valid Markdown.
