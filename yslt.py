""" Stylesheet transformer for YAML input files and stylesheets in jinja2 format. """


import argparse
from pathlib import Path
import yaml
from jinja2 import Environment, FileSystemLoader

def main():
    """ Main entry point for YAML Stylesheet Transformer """
    parser = argparse.ArgumentParser(
        description='Apply stylesheet to input file.')
    parser.add_argument('-s', '--stylesheet', dest='stylesheet',
                        help='stylesheet to apply', required=True)
    parser.add_argument('input', help='input file')

    args = parser.parse_args()
    stylesheet_path = Path(args.stylesheet)
    input_path = Path(args.input)
    if not input_path.exists() or not input_path.is_file():
        print('Input file ' + args.input + ' does not exist.')
    elif not stylesheet_path.exists() or not stylesheet_path.is_file():
        print('Stylesheet ' + args.stylesheet + ' does not exist.')
    else:
        env = Environment(loader=FileSystemLoader(str(stylesheet_path.resolve().parent)))
        template = env.get_template(stylesheet_path.name)
        context = yaml.load(input_path.read_text())
        print('## ' + str(context))
        output = template.render(context=context, filename=input_path.name)
        print(output)


if __name__ == "__main__":
    main()
