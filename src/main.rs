use anyhow::{Context, Result};
use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use tera::Tera;

mod gsn;

use gsn::GsnNode;

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
        .get_matches();
    let input = matches.value_of("INPUT").unwrap(); // Unwrap is ok, since argument is required
    let mut reader = BufReader::new(
        File::open(&input).with_context(|| format!("Failed to open file {}", input))?,
    );
    let nodes: BTreeMap<String, GsnNode> = serde_yaml::from_reader(&mut reader)
        .with_context(|| format!("Failed to parse YAML from file {}", input))?;
    let mut context = tera::Context::new();
    context.insert("filename", input);
    context.insert("nodes", &nodes);

    // Validate
    gsn::validate(&mut std::io::stderr(), &nodes);

    // Output either to stdout or to file
    let mut output = if matches.is_present("OUTPUT") {
        // Unwrap is ok here, since the presence of the argument is checked
        Box::new(
            File::create(matches.value_of("OUTPUT").unwrap())
                .with_context(|| format!("Failed to open output file {}", input))?,
        ) as Box<dyn std::io::Write>
    } else {
        Box::new(std::io::stdout())
    };
    // The following errors are considered not to occur during normal usage.
    // Thus, normal users cannot resolve them even with better error messages.
    writeln!(output, "## {:?}\n\n", &nodes)?;
    let tera = Tera::new("templates/*.dot")?;
    tera.render_to("gsn2dot.dot", &context, output)?;
    
    Ok(())
}
