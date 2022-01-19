use anyhow::{anyhow, Context, Result};
use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use tera::Tera;

mod gsn;
mod wordwrap;
mod yaml_fix;

use gsn::GsnNode;
use wordwrap::WordWrap;
use yaml_fix::MyMap;

use crate::gsn::get_levels;

///
/// Main entry point.
///
///
fn main() -> Result<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::new("INPUT")
                .help("Sets the input file(s) to use.")
                .multiple_occurrences(true)
                .required(true),
        )
        .arg(
            Arg::new("OUTPUT")
                .help("Sets the optional output file to use.")
                .short('o')
                .long("output")
                .required(false),
        )
        .arg(
            Arg::new("VALONLY")
                .help("Only check the input file, but do not output the result.")
                .short('c')
                .long("check")
                .required(false),
        )
        .arg(
            Arg::new("LAYERS")
                .help("Output additional layers.")
                .short('l')
                .long("layers")
                .takes_value(true)
                .multiple_occurrences(true)
                .use_delimiter(true)
                .required(false),
        )
        .arg(
            Arg::new("STYLESHEET")
                .help("Sets a stylesheet that is used by Graphviz in SVG output.")
                .short('s')
                .long("stylesheet")
                .takes_value(true)
                .multiple_occurrences(false)
                .required(false),
        )
        .arg(
            Arg::new("EVIDENCES")
                .help("Additionally output list of all evidences in given file.")
                .short('e')
                .long("evicdenes")
                .takes_value(true)
                .multiple_occurrences(false)
                .required(false),
        )
        .get_matches();

    // Read input
    let input = matches.value_of("INPUT").unwrap();
    let mut reader = BufReader::new(
        File::open(&input).with_context(|| format!("Failed to open file {}", input))?,
    );
    let nodes = read_input(&mut reader)
        .with_context(|| format!("Failed to parse YAML from file {}", input))?;

    // Validate
    let mut d = gsn::validate(&mut std::io::stderr(), &nodes)?;
    let layers = matches
        .values_of("LAYERS")
        .map(|x| x.collect::<Vec<&str>>());
    if let Some(lays) = &layers {
        d += gsn::check_layers(&mut std::io::stderr(), &nodes, lays)?;
    }

    // Output
    let input_filename = std::path::Path::new(input)
        .file_name()
        .with_context(|| format!("{} is not a file.", input))?
        .to_str()
        .unwrap();
    output(
        input_filename,
        &nodes,
        &layers,
        matches.value_of("STYLESHEET"),
        matches.is_present("VALONLY"),
        d,
        &mut match matches.value_of("OUTPUT") {
            Some(output) => Box::new(
                File::create(output)
                    .with_context(|| format!("Failed to open output file {}", output))?,
            ) as Box<dyn std::io::Write>,
            None => Box::new(std::io::stdout()) as Box<dyn std::io::Write>,
        },
    )?;

    if let Some(output) = matches.value_of("EVIDENCES") {
        let mut output_file = File::create(output)
            .with_context(|| format!("Failed to open output file {}", output))?;
        output_evidences(input_filename, &nodes, &layers, &mut output_file)?;
    }
    Ok(())
}

///
/// Create separate function for testability reasons.
///
fn read_input(input: &mut impl Read) -> Result<MyMap<String, GsnNode>, anyhow::Error> {
    let nodes: MyMap<String, GsnNode> = serde_yaml::from_reader(input)?;
    Ok(nodes)
}

///
/// Render to dot-file if not only validation is active.
/// Output summary of warnings and errors.
///
fn output(
    input: &str,
    nodes: &MyMap<String, GsnNode>,
    layers: &Option<Vec<&str>>,
    stylesheet: Option<&str>,
    validonly: bool,
    d: gsn::Diagnostics,
    output: &mut impl Write,
) -> Result<()> {
    if !validonly {
        render_result(input, nodes, layers, stylesheet, output)?;
    }
    if d.errors == 0 {
        if d.warnings > 0 {
            eprintln!("Warning: {} warnings detected.", d.warnings);
        }
        Ok(())
    } else {
        Err(anyhow!(
            "{} errors and {} warnings detected.",
            d.errors,
            d.warnings
        ))
    }
}

///
/// Use Tera to create dot-file.
/// Templates are inlined in executable.
///
fn render_result(
    input: &str,
    nodes: &MyMap<String, GsnNode>,
    layers: &Option<Vec<&str>>,
    stylesheet: Option<&str>,
    output: &mut impl Write,
) -> Result<(), anyhow::Error> {
    let mut context = tera::Context::new();
    context.insert("filename", input);
    context.insert("nodes", &nodes);
    context.insert("layers", &layers);
    context.insert("levels", &get_levels(nodes));
    context.insert("stylesheet", &stylesheet);
    let mut tera = Tera::default();
    tera.register_filter("wordwrap", WordWrap);
    tera.add_raw_templates(vec![
        ("macros.dot", include_str!("../templates/macros.dot")),
        ("gsn2dot.dot", include_str!("../templates/gsn2dot.dot")),
    ])?;
    tera.render_to("gsn2dot.dot", &context, output)
        .with_context(|| "Failed to write to output.")?;
    Ok(())
}

///
/// Output a list of evidences.
/// Tera does not have support for a counter, thus we write it here directly.
///
/// URL and additional layers are also added for the solution.
///
fn output_evidences(
    input: &str,
    nodes: &MyMap<String, GsnNode>,
    layers: &Option<Vec<&str>>,
    output: &mut impl Write,
) -> Result<()> {
    let num_solutions = nodes.iter().filter(|(id, _)| id.starts_with("Sn")).count();
    if num_solutions > 0 {
        let width = (num_solutions as f32).log10().ceil() as usize;

        writeln!(output)?;
        writeln!(output, "List of evidences in {}", input)?;
        writeln!(output)?;

        let mut i = 1;
        for (id, node) in nodes.iter() {
            if id.starts_with("Sn") {
                writeln!(
                    output,
                    "{:width$}. {}: {}",
                    i,
                    id,
                    node.get_text(),
                    width = width
                )?;
                if let Some(url) = node.get_url() {
                    writeln!(output, "{:width$}{}", " ", url, width = width + 2)?;
                }
                if let Some(layers) = layers {
                    for l in layers {
                        if let Some(layer_text) = node.get_layer(l.to_owned()) {
                            writeln!(
                                output,
                                "{:width$}{}: {}",
                                " ",
                                l,
                                layer_text,
                                width = width + 2
                            )?;
                        }
                    }
                }
                i += 1;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::gsn::Diagnostics;
    use crate::*;
    use std::fs::OpenOptions;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::io::{Seek, SeekFrom};

    #[test]
    fn example_back_to_back() -> Result<(), Box<dyn std::error::Error>> {
        let mut output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .read(true)
            .open("examples/example.gsn.test.dot")?;
        let mut reader = BufReader::new(File::open("examples/example.gsn.yaml")?);
        let nodes = crate::read_input(&mut reader)?;
        let d = gsn::validate(&mut std::io::stderr(), &nodes)?;
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 0);
        crate::output(
            "examples/example.gsn.yaml",
            &nodes,
            &None,
            None,
            false,
            d,
            &mut output,
        )?;
        output.flush()?;
        output.seek(SeekFrom::Start(0))?;
        let orig = BufReader::new(std::fs::File::open("examples/example.gsn.dot")?).lines();
        let test = BufReader::new(&output).lines();
        assert!(orig.map(|x| x.unwrap()).eq(test.map(|x| x.unwrap())));
        Ok(())
    }

    #[test]
    fn example_back_to_back_evidences() -> Result<(), Box<dyn std::error::Error>> {
        let mut output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .read(true)
            .open("examples/example.gsn.test.md")?;
        let mut reader = BufReader::new(File::open("examples/example.gsn.yaml")?);
        let nodes = crate::read_input(&mut reader)?;
        crate::output_evidences(
            "examples/example.gsn.yaml",
            &nodes,
            &Some(vec!["layer1"]),
            &mut output,
        )?;
        output.flush()?;
        output.seek(SeekFrom::Start(0))?;
        let orig = BufReader::new(std::fs::File::open("examples/example.gsn.md")?).lines();
        let test = BufReader::new(&output).lines();
        assert!(orig.map(|x| x.unwrap()).eq(test.map(|x| x.unwrap())));
        Ok(())
    }

    #[test]
    fn validcheck() {
        let nodes = MyMap::<String, GsnNode>::new();
        let d = Diagnostics {
            warnings: 2,
            errors: 3,
        };
        let mut output = Vec::<u8>::new();
        let res = crate::output("", &nodes, &None, None, true, d, &mut output);
        assert!(res.is_err());
        assert_eq!(
            format!("{:?}", res),
            "Err(3 errors and 2 warnings detected.)"
        );
        assert_eq!(output.len(), 0);
    }

    #[test]
    fn checkyamlworkaround() {
        let input = "A:\n text: absd\n\nA:\n text: cdawer\n\nB:\n text: asfas";
        let res = read_input(&mut input.as_bytes());
        assert!(res.is_err());
        assert_eq!(
            format!("{:?}", res),
            "Err(Element A is already existing at line 1 column 2)"
        );
    }

    #[test]
    fn checkyamlworkaround_unknownformat() {
        let input = "- A\n\n- B\n\n- C\n";
        let res = read_input(&mut input.as_bytes());
        assert!(res.is_err());
        assert_eq!(
            format!("{:?}", res),
            "Err(invalid type: sequence, expected a map with unique keys at line 1 column 1)"
        );
    }

    #[test]
    fn no_evidences() {
        let mut output = Vec::<u8>::new();
        let mut map = MyMap::default();
        map.insert("G1".to_owned(), GsnNode::default());
        assert!(crate::output_evidences("abc.yml", &map, &None, &mut output).is_ok());
        assert!(output.is_empty());
    }

    #[test]
    fn one_evidence() {
        let mut output = Vec::<u8>::new();
        let input = "Sn1:\n  text: absd\n  url: link\n";
        let map = read_input(&mut input.as_bytes()).unwrap();

        assert!(crate::output_evidences("abc.yml", &map, &None, &mut output).is_ok());
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "\nList of evidences in abc.yml\n\n1. Sn1: absd\n  link\n"
        );
    }
}
