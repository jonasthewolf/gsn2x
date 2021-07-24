use anyhow::{anyhow, Context, Result};
use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Write};
use tera::Tera;

mod gsn;
mod wordwrap;

use gsn::GsnNode;
use wordwrap::WordWrap;

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
        .get_matches();

    // Read input
    let input = matches.value_of("INPUT").unwrap();
    let nodes = read_input(input)?;

    // Validate
    let d = gsn::validate(&mut std::io::stderr(), &nodes);

    // Output
    output(
        input,
        nodes,
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

fn read_input(input: &str) -> Result<BTreeMap<String, GsnNode>, anyhow::Error> {
    let mut reader = BufReader::new(
        File::open(&input).with_context(|| format!("Failed to open file {}", input))?,
    );
    let nodes: BTreeMap<String, GsnNode> = serde_yaml::from_reader(&mut reader)
        .with_context(|| format!("Failed to parse YAML from file {}", input))?;
    Ok(nodes)
}

fn output(
    input: &str,
    nodes: BTreeMap<String, GsnNode>,
    validonly: bool,
    d: gsn::Diagnostics,
    output: &mut impl Write,
) -> Result<()> {
    if validonly {
        if d.errors == 0 {
            Ok(())
        } else {
            Err(anyhow!(
                "{} errors and {} warnings detected.",
                d.errors,
                d.warnings
            ))
        }
    } else {
        render_result(input, nodes, output)?;
        Ok(())
    }
}

fn render_result(
    input: &str,
    nodes: BTreeMap<String, GsnNode>,
    output: &mut impl Write,
) -> Result<(), anyhow::Error> {
    let mut context = tera::Context::new();
    context.insert("filename", input);
    context.insert("nodes", &nodes);
    writeln!(output, "## {:?}\n\n", &nodes).with_context(|| "Failed to write to output.")?;
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
    use crate::*;
    use std::fs::OpenOptions;
    use std::io::BufRead;
    use std::io::BufReader;
    #[test]
    fn example_back_to_back() -> Result<(), Box<dyn std::error::Error>> {
        let mut output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .read(true)
            .open("example.gsn.test.dot")?;

        let nodes = crate::read_input("example.gsn.yaml")?;
        let d = gsn::validate(&mut std::io::stderr(), &nodes);
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 0);
        crate::output("example.gsn.yaml", nodes, false, d, &mut output)?;

        let orig = BufReader::new(std::fs::File::open("example.gsn.dot")?).lines();
        let test = BufReader::new(&output).lines();
        for (t, o) in test.zip(orig) {
            assert_eq!(t?, o?);
        }
        Ok(())
    }
}
