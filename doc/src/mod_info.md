
# Optional module information

It is possible to add additional `module` information in the source YAML.
This allows describing the module`s name and an optional brief description.
Even arbitrary information can be added. 

`name` and `brief` are mandatory if a `module` is added.

```yaml

module: 
   name: MainModule
   brief: This is a short description of the module
   additionalInformation: 
    v1: Changed line 2
    v2: Added line 4

```

The module information is printed as part of a legend for the argument view.

To influence the position in the architecture view, you can use the `horizontalIndex` and `rankIncrement` as you would for elements in the Argument view (see [Layout of elements](adv_layout.md#placement-of-elements) ).

You can use the `-G` option to suppress the legend completely, 
or the `-g` option to limit it to `name`, `brief` and the time and date of generation of the SVG.

## Including other modules

Multiple modules, i.e. files, can be provided at the command line:

    gsn2x index.gsn.yaml required_module1.gsn.yaml required_module2.gsn.yaml required_module3.gsn.yaml

However, this might requires long and complex calls at the command line for reasonably large arguments.

Another way to use multiple modules is to "use" them from others. You can do so by adding a `uses` attribute to a module:

```yaml
module:
   name: MainModule
   brief: This is the index.gsn.yaml file
   uses: [required_module1.yaml, required_module2.gsn.yaml, required_module3.gsn.yaml]
```

The command line is now simplified to:

    gsn2x index.gsn.yaml

Of course, "used" modules can also include other modules on their own.
There is a check implemented to prevent circular includes.
