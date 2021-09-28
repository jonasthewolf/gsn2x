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

fn main() -> Result<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("Sets the optional output file to use")
                .required(false)
                .index(2),
        )
        .arg(
            Arg::with_name("VALONLY")
                .help("Only check the input file, but do not output the result.")
                .short("c")
                .long("check")
                .required(false),
        )
        .arg(
            Arg::with_name("LAYERS")
                .help("Output additional layers.")
                .short("l")
                .long("layers")
                .takes_value(true)
                .multiple(true)
                .use_delimiter(true)
                .required(false),
        )
        .arg(
            Arg::with_name("STYLESHEET")
                .help("Sets a stylesheet that is used by Graphviz in SVG output.")
                .short("s")
                .long("stylesheet")
                .takes_value(true)
                .multiple(false)
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
    let d = gsn::validate(&mut std::io::stderr(), &nodes)?;

    // Output
    output(
        input,
        nodes,
        matches.values_of("LAYERS").map(|x| x.collect()),
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
    )
}

fn read_input(input: &mut impl Read) -> Result<MyMap<String, GsnNode>, anyhow::Error> {
    let nodes: MyMap<String, GsnNode> = serde_yaml::from_reader(input)?;
    Ok(nodes)
}

fn output(
    input: &str,
    nodes: MyMap<String, GsnNode>,
    layers: Option<Vec<&str>>,
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

fn render_result(
    input: &str,
    nodes: MyMap<String, GsnNode>,
    layers: Option<Vec<&str>>,
    stylesheet: Option<&str>,
    output: &mut impl Write,
) -> Result<(), anyhow::Error> {
    let mut context = tera::Context::new();
    context.insert("filename", input);
    context.insert("nodes", &nodes);
    context.insert("layers", &layers);
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
            .open("example.gsn.test.dot")?;
        let mut reader = BufReader::new(File::open("example.gsn.yaml")?);
        let nodes = crate::read_input(&mut reader)?;
        let d = gsn::validate(&mut std::io::stderr(), &nodes)?;
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 0);
        crate::output("example.gsn.yaml", nodes, None, false, d, &mut output)?;
        output.flush()?;
        output.seek(SeekFrom::Start(0))?;
        let orig = BufReader::new(std::fs::File::open("example.gsn.dot")?).lines();
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
        let res = crate::output("", nodes, None, true, d, &mut output);
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
}
