use anyhow::{Context, Result};
use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
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
        .get_matches();
    gsn_convert(
        matches.value_of("INPUT").unwrap(),
        matches.value_of("OUTPUT"),
    )?;

    Ok(())
}

fn gsn_convert(input: &str, output: Option<&str>) -> Result<(), anyhow::Error> {
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

    // Output
    let mut output_file = match output {
        Some(output) => Box::new(
            File::create(output)
                .with_context(|| format!("Failed to open output file {}", input))?,
        ) as Box<dyn std::io::Write>,
        None => Box::new(std::io::stdout()),
    };

    writeln!(output_file, "## {:?}\n\n", &nodes)?;
    let mut tera = Tera::default();
    tera.register_filter("wordwrap", WordWrap);
    tera.add_raw_templates(vec![
        ("macros.dot", include_str!("../templates/macros.dot")),
        ("gsn2dot.dot", include_str!("../templates/gsn2dot.dot")),
    ])?;
    tera.render_to("gsn2dot.dot", &context, output_file)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::gsn_convert;
    use std::io::BufRead;
    use std::io::BufReader;
    #[test]
    fn example_back_to_back() -> Result<(), Box<dyn std::error::Error>> {
        gsn_convert("example.gsn.yaml", Some("example.gsn.test.dot"))?;
        let orig = BufReader::new(std::fs::File::open("example.gsn.dot")?).lines();
        let test = BufReader::new(std::fs::File::open("example.gsn.test.dot")?).lines();
        for (t, o) in test.zip(orig) {
            assert_eq!(t?, o?);
        }
        Ok(())
    }
}
