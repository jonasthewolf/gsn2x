
# Modular extension

gsn2x partially supports the Modular Extension of the GSN standard (see [Standard support](./standard_support.md)).

Each module is a separate file. The name of the module is the file name (incl. the path provided to the gsn2x command line).

If modules are used, all related module files must be provided to the command line of gsn2x.
Element IDs must be unique across all modules. Checks will by default be performed across all modules.
Check messages for individual modules can be omitted using the `-x` option.

The argument view of individual modules will show "away" elements if elements from other modules are referenced.
All elements are public, meaning they can be referenced from other modules.

Note: There is no "away strategy" in the standard.

In addition to the default argument view for each module, there are two output files generated (if more than one input file is provided):
1) Complete View (default to: `complete.svg`)
2) Architecture View (default to: `architecture.svg`)

You can only change the file names of these additional views. 
They are put in the directory that all input files have in common.
The `-o` option can be used for these views, too.

If the argument view should not be updated, use the `-N` option.
If the complete view should not be output, use the `-F` option.
If the architecture view should not be output, use the `-A` option.

## Complete view

The complete view is a similar to an argument view for a single module, 
but showing all modules within the same diagram. The modules are "unrolled". 
<!-- Modules can be masked i.e., unrolling is prevented, 
by additionally adding those modules with the `-m` option. -->

![example complete](examples/modular/complete.svg)

See [example](examples/modular/complete.svg) here.

## Architecture view

The architecture view only shows the selected modules and their dependencies.
THe architecture view is navigable to the module argument view.

The architecture view only contains the links to the individual module files, if they actually exist when generating the architecture view.

![example architecture](examples/modular/architecture.svg)

See [example](examples/modular/architecture.svg) here.

## Example:
    
    gsn2x -f full.svg -a arch.svg main.yml sub1.yml sub3.yml

This will generate the argument view for each module, the complete view (`-f full.svg`) of all modules and the architecture view (`-a arch.svg`). <!-- In the complete view, the elements of the `sub1` module will be represented by a module. -->
