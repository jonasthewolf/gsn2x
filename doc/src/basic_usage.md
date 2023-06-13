
# Basic usage

This little program renders [Goal Structuring Notation](https://scsc.uk/gsn) in a YAML format to a scalable vector graphics (SVG) image.

![example](examples/example.gsn.svg)


## Usage

You can create an SVG like this:

    gsn2x <yourgsnfile.yaml> 

The output is an argument view in SVG format and automatically written to `<yourgsnfile.svg>`. If more than one input file is provided, they are treated as [modules](./modular_extension.md).


## Options

    Usage: gsn2x [OPTIONS] <INPUT>...
    
    Arguments:
      <INPUT>...  Sets the input file(s) to use. Only relative paths are accepted.
    
    Options:
      -h, --help     Print help
      -V, --version  Print version
    
    CHECKS:
      -c, --check                      Only check the input file(s), but do not output graphs.
      -x, --exclude <EXCLUDED_MODULE>  Exclude this module from reference checks.
    
    OUTPUT:
      -N, --no-arg                         Do not output of argument view for provided input files.
      -f, --full <COMPLETE_VIEW>           Output the complete view to file with name <COMPLETE_VIEW>. [default: complete.svg]
      -F, --no-full                        Do not output the complete view.
      -a, --arch <ARCHITECTURE_VIEW>       Output the architecture view to file with name <ARCHITECTURE_VIEW>. [default:     architecture.svg]
      -A, --no-arch                        Do not output the architecture view.
      -e, --evidences <EVIDENCES>          Output list of all evidences to file with name <EVIDENCES>. [default: evidences.md]
      -E, --no-evidences                   Do not output list of all evidences.
      -o, --output-dir <OUTPUT_DIRECTORY>  Emit all output files to directory <OUTPUT_DIRECTORY>. [default: .]
    
    OUTPUT MODIFICATION:
      -l, --layer <LAYERS>            Output additional layer. Can be used multiple times.
      -s, --stylesheet <STYLESHEETS>  Links a stylesheet in SVG output. Can be used multiple times.
      -t, --embed-css                 Embed stylehseets instead of linking them.
      -G, --no-legend                 Do not output a legend based on module information.
      -g, --full-legend               Output a legend based on all module information.
    