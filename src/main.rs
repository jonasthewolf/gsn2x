use anyhow::{anyhow, Context, Result};
use clap::{app_from_crate, Arg, ErrorKind};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use tera::Tera;

mod diagnostics;
mod gsn;
mod wordwrap;
mod yaml_fix;

use diagnostics::Diagnostics;
use gsn::GsnNode;
use wordwrap::WordWrap;
use yaml_fix::MyMap;

use crate::gsn::get_levels;

///
/// Main entry point.
///
///
fn main() -> Result<()> {
    let mut app = app_from_crate!()
        .arg(
            Arg::new("INPUT")
                .help("Sets the input file(s) to use.")
                .multiple_occurrences(true)
                .required(true),
        )
        .arg(
            Arg::new("OUTPUT")
                .help("Writes output to standard output (only possible with single input).")
                .short('o')
                .long("output")
                .required(false)
                .conflicts_with("VALONLY")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("VALONLY")
                .help("Only check the input file, but do not output the result.")
                .short('c')
                .long("check")
                .required(false),
        )
        .arg(
            Arg::new("COMPLETE_VIEW")
                .help("Additionally output the complete view to this file.")
                .short('f')
                .long("full")
                .takes_value(true)
                .required(false)
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("ARCHITECTURE_VIEW")
                .help("Additionally output the architecture view to this file.")
                .short('a')
                .long("arch")
                .takes_value(true)
                .required(false)
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("MODULE")
                .help("Hide this module from the complete view.")
                .short('m')
                .long("mod")
                .multiple_occurrences(true)
                .takes_value(true)
                .required(false)
                .help_heading("MODIFICATIONS"),
        )
        .arg(
            Arg::new("LAYERS")
                .help("Output additional layers.")
                .short('l')
                .long("layers")
                .takes_value(true)
                .multiple_occurrences(true)
                .use_delimiter(true)
                .required(false)
                .help_heading("MODIFICATIONS"),
        )
        .arg(
            Arg::new("STYLESHEET")
                .help("Sets a stylesheet that is used by Graphviz in SVG output.")
                .short('s')
                .long("stylesheet")
                .takes_value(true)
                .multiple_occurrences(false)
                .required(false)
                .help_heading("MODIFICATIONS"),
        )
        .arg(
            Arg::new("EVIDENCES")
                .help("Additionally output list of all evidences in given file.")
                .short('e')
                .long("evicdenes")
                .takes_value(true)
                .multiple_occurrences(false)
                .required(false)
                .help_heading("OUTPUT"),
        );
    let matches = app.get_matches_mut();
    if matches.is_present("OUTPUT") && matches.occurrences_of("INPUT") > 1 {
        app.error(
            ErrorKind::ArgumentConflict,
            "The argument '-o' cannot be used with multiple input files.",
        )
        .exit();
    }

    let mut diags = Diagnostics::default();
    // Read input
    let inputs: Vec<&str> = matches.values_of("INPUT").unwrap().collect();
    let mut nodes = MyMap::<String, GsnNode>::new();
    let mut modules_keys = BTreeMap::<String, Vec<String>>::new();
    for input in &inputs {
        let mut reader = BufReader::new(
            File::open(&input).with_context(|| format!("Failed to open file {}", input))?,
        );
        let mut n = read_input(&mut reader)
            .with_context(|| format!("Failed to parse YAML from file {}", input))?;
        // Check for duplicates, since they might be in separate files.
        for (mod_n, mod_keys) in &modules_keys {
            for k in n.keys() {
                if mod_keys.contains(k) {
                    diags.add_error(
                        input,
                        format!(
                            "Element {} in {} was already present in {}.",
                            k, input, mod_n
                        ),
                    );
                }
            }
        }
        // Remember from which file IDs are coming.
        modules_keys.insert(input.to_string(), n.keys().cloned().collect());
        // Merge nodes for further processing.
        nodes.append(&mut n);
    }
    for input in inputs {
        // Validate
        // When validating a module, all references are resolved.
        gsn::validate_module(&mut diags, input, &nodes);
        let layers = matches
            .values_of("LAYERS")
            .map(|x| x.collect::<Vec<&str>>());
        if let Some(lays) = &layers {
            gsn::check_layers(&mut diags, input, &nodes, lays);
        }

        // Output
        let input_filename = std::path::Path::new(input)
            .file_name()
            .with_context(|| format!("{} is not a file.", input))?
            .to_str()
            .unwrap();
        let mut pbuf = std::path::PathBuf::from(input);
        pbuf.set_extension("dot");
        let output_filename = pbuf.as_path();
        // It is already checked that if OUTPUT is set, only one input file is provided.
        let mut output_handle = if matches.is_present("OUTPUT") {
            Box::new(std::io::stdout()) as Box<dyn std::io::Write>
        } else {
            Box::new(File::create(output_filename).with_context(|| {
                format!("Failed to open output file {}", output_filename.display())
            })?) as Box<dyn std::io::Write>
        };
        output(
            &diags,
            input_filename,
            &nodes,
            &layers,
            matches.value_of("STYLESHEET"),
            matches.is_present("VALONLY"),
            &mut output_handle,
        )?;

        if let Some(output) = matches.value_of("EVIDENCES") {
            let mut output_file = File::create(output)
                .with_context(|| format!("Failed to open output file {}", output))?;
            output_evidences(input_filename, &nodes, &layers, &mut output_file)?;
        }
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
    diags: &Diagnostics,
    input: &str,
    nodes: &MyMap<String, GsnNode>,
    layers: &Option<Vec<&str>>,
    stylesheet: Option<&str>,
    validonly: bool,
    output: &mut impl Write,
) -> Result<()> {
    if !validonly {
        render_result(input, nodes, layers, stylesheet, output)?;
    }
    for msg in &diags.messages {
        eprintln!("{}", msg);
    }
    if diags.errors == 0 {
        if diags.warnings > 0 {
            eprintln!("Warning: {} warnings detected.", diags.warnings);
        }
        Ok(())
    } else {
        Err(anyhow!(
            "{} errors and {} warnings detected.",
            diags.errors,
            diags.warnings
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
    use crate::diagnostics::Diagnostics;
    use crate::*;
    use std::fs::OpenOptions;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::io::{Seek, SeekFrom};

    #[test]
    fn example_back_to_back() -> Result<(), Box<dyn std::error::Error>> {
        let mut d = Diagnostics::default();
        let mut output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .read(true)
            .open("examples/example.gsn.test.dot")?;
        let mut reader = BufReader::new(File::open("examples/example.gsn.yaml")?);
        let nodes = crate::read_input(&mut reader)?;
        gsn::validate_module(&mut d, "examples/example.gsn.yaml", &nodes);
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 0);
        crate::output(
            &d,
            "examples/example.gsn.yaml",
            &nodes,
            &None,
            None,
            false,
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
            ..Default::default()
        };
        let mut output = Vec::<u8>::new();
        let res = crate::output(&d, "", &nodes, &None, None, true, &mut output);
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
