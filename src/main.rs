use anyhow::{anyhow, Context, Result};
use clap::{Arg, ErrorKind};
use std::fs::File;
use std::io::BufReader;

mod diagnostics;
mod gsn;
mod render;
// mod tera;
mod util;
mod yaml_fix;

use diagnostics::Diagnostics;
use gsn::GsnNode;
use yaml_fix::MyMap;

///
/// Main entry point.
///
///
fn main() -> Result<()> {
    let mut app = clap::command!()
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
                .conflicts_with("VALONLY")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("SUPPRESSARGUMENT")
                .help("Suppress output of argument view for provided input files.")
                .short('n')
                .long("noarg")
                .conflicts_with_all(&["OUTPUT", "VALONLY"])
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("VALONLY")
                .help("Only check the input file(s), but do not output graphs.")
                .short('c')
                .long("check")
                .help_heading("VALIDATION"),
        )
        .arg(
            Arg::new("EXCLUDED_MODULE")
                .help("Exclude this module from validation.")
                .short('x')
                .long("exclude")
                .multiple_occurrences(true)
                .takes_value(true)
                .help_heading("VALIDATION"),
        )
        .arg(
            Arg::new("COMPLETE_VIEW")
                .help("Additionally output the complete view to this file.")
                .short('f')
                .long("full")
                .takes_value(true)
                .conflicts_with("VALONLY")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("ARCHITECTURE_VIEW")
                .help("Additionally output the architecture view to this file.")
                .short('a')
                .long("arch")
                .takes_value(true)
                .conflicts_with("VALONLY")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("MASK_MODULE")
                .help("Hide this module from the complete view.")
                .short('m')
                .long("mask")
                .multiple_occurrences(true)
                .takes_value(true)
                .requires("COMPLETE_VIEW")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("LAYERS")
                .help("Output additional layers.")
                .short('l')
                .long("layers")
                .takes_value(true)
                .multiple_occurrences(true)
                .use_value_delimiter(true)
                .conflicts_with("VALONLY")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("STYLESHEET")
                .help("Sets a stylesheet that is used by Graphviz in SVG output.")
                .short('s')
                .long("stylesheet")
                .takes_value(true)
                .multiple_occurrences(false)
                .conflicts_with("VALONLY")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("EVIDENCES")
                .help("Additionally output list of all evidences in given file.")
                .short('e')
                .long("evidences")
                .takes_value(true)
                .multiple_occurrences(false)
                .conflicts_with("VALONLY")
                .help_heading("OUTPUT"),
        );
    let matches = app.get_matches_mut();
    if matches.is_present("OUTPUT") && matches.occurrences_of("INPUT") > 1 {
        app.error(
            ErrorKind::ArgumentConflict,
            // When supported by clap, the '-o' should be colored
            "The argument '-o' cannot be used with multiple input files.",
        )
        .exit();
    }
    let mut diags = Diagnostics::default();
    let inputs: Vec<&str> = matches.values_of("INPUT").unwrap().collect();
    let mut nodes = MyMap::<String, GsnNode>::new();
    let layers = matches
        .values_of("LAYERS")
        .map(|x| x.collect::<Vec<&str>>());
    let stylesheet = matches.value_of("STYLESHEET");
    let excluded_modules = matches
        .values_of("EXCLUDED_MODULE")
        .map(|x| x.collect::<Vec<&str>>());
    let modules = inputs
        .iter()
        .map(util::escape_text)
        .collect::<Vec<String>>();
    let static_render_context = render::StaticRenderContext {
        modules: &modules,
        input_files: &inputs,
        layers: &layers,
        stylesheet,
    };

    // Read input
    read_inputs(&inputs, &mut nodes, &mut diags)?;
    // Validate
    validate(&inputs, excluded_modules, &mut diags, &nodes, &layers);

    // TODO Check that only one global top-level element remains
    if diags.errors == 0 {
        // Output argument view
        print_outputs(&matches, &inputs, nodes, static_render_context)?;
    }
    // Output diagnostic messages
    output_messages(&diags)
}

///
/// Print outputs
///
///
///
///
fn print_outputs(
    matches: &clap::ArgMatches,
    inputs: &[&str],
    nodes: MyMap<String, GsnNode>,
    static_render_context: render::StaticRenderContext,
) -> Result<(), anyhow::Error> {
    if !(matches.is_present("VALONLY") || matches.is_present("SUPPRESSARGUMENT")) {
        for input in inputs {
            // It is already checked that if OUTPUT is set, only one input file is provided.
            let mut output_file = if matches.is_present("OUTPUT") {
                Box::new(std::io::stdout()) as Box<dyn std::io::Write>
            } else {
                let mut pbuf = std::path::PathBuf::from(input);
                pbuf.set_extension("dot");
                let output_filename = pbuf.as_path();
                Box::new(File::create(output_filename).context(format!(
                    "Failed to open output file {}",
                    output_filename.display()
                ))?) as Box<dyn std::io::Write>
            };
            render::render_view(
                &util::escape_text(input),
                &nodes,
                None,
                &mut output_file,
                render::View::Argument,
                &static_render_context,
            )?;
        }
    }
    if let Some(arch_view) = matches.value_of("ARCHITECTURE_VIEW") {
        let mut output_file =
            File::create(arch_view).context(format!("Failed to open output file {}", arch_view))?;
        let deps = crate::gsn::calculate_module_dependencies(&nodes);
        render::render_view(
            &util::escape_text(&arch_view),
            &nodes,
            Some(&deps),
            &mut output_file,
            render::View::Architecture,
            &static_render_context,
        )?;
    }
    if let Some(compl_view) = matches.value_of("COMPLETE_VIEW") {
        let mut output_file = File::create(compl_view)
            .context(format!("Failed to open output file {}", compl_view))?;
        render::render_view(
            &util::escape_text(&compl_view),
            &nodes,
            None,
            &mut output_file,
            render::View::Complete,
            &static_render_context,
        )?;
    }
    if let Some(output) = matches.value_of("EVIDENCES") {
        let mut output_file =
            File::create(output).context(format!("Failed to open output file {}", output))?;
        render::render_evidences(&nodes, &mut output_file, &static_render_context)?;
    }
    Ok(())
}

///
/// Validate modules
///
///
///
///
fn validate(
    inputs: &[&str],
    excluded_modules: Option<Vec<&str>>,
    diags: &mut Diagnostics,
    nodes: &MyMap<String, GsnNode>,
    layers: &Option<Vec<&str>>,
) {
    for input in inputs {
        let module = util::escape_text(input);
        // When validating a module, all references are resolved.
        if let Some(excluded) = &excluded_modules {
            // Only allow excluding files from validation if there is more than one.
            if excluded.contains(input) && inputs.len() > 1 {
                continue;
            }
        }
        gsn::validate_module(diags, &module, nodes);
        if let Some(lays) = &layers {
            gsn::check_layers(diags, &module, nodes, lays);
        }
    }
}

///
/// Read inputs
///
///
fn read_inputs(
    inputs: &[&str],
    nodes: &mut MyMap<String, GsnNode>,
    diags: &mut Diagnostics,
) -> Result<(), anyhow::Error> {
    for input in inputs {
        let module = util::escape_text(input);
        let reader =
            BufReader::new(File::open(&input).context(format!("Failed to open file {}", input))?);
        let mut n: MyMap<String, GsnNode> = serde_yaml::from_reader(reader)
            .context(format!("Failed to parse YAML from file {}", input))?;
        // Remember module for node
        n.iter_mut()
            .for_each(|(_, mut x)| x.module = module.to_string());
        // Check for duplicates, since they might be in separate files.
        for k in n.keys() {
            if nodes.contains_key(k) {
                diags.add_error(
                    input,
                    format!(
                        "Element {} in {} was already present in {}.",
                        k,
                        input,
                        nodes.get(k).unwrap().module
                    ),
                );
                break;
            }
        }
        // Merge nodes for further processing.
        nodes.append(&mut n);
    }
    Ok(())
}

///
/// Render to dot-file if not only validation is active.
/// Output summary of warnings and errors.
///
fn output_messages(diags: &Diagnostics) -> Result<()> {
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

#[cfg(test)]
mod test {
    use crate::diagnostics::Diagnostics;

    #[test]
    fn check_output_messages_errors() {
        let d = Diagnostics {
            warnings: 2,
            errors: 3,
            ..Default::default()
        };
        let res = crate::output_messages(&d);
        assert!(res.is_err());
        assert_eq!(
            format!("{:?}", res),
            "Err(3 errors and 2 warnings detected.)"
        );
    }

    #[test]
    fn check_output_messages_warnings() {
        let d = Diagnostics {
            warnings: 5,
            errors: 0,
            ..Default::default()
        };
        let res = crate::output_messages(&d);
        assert!(res.is_ok());
        assert_eq!(format!("{:?}", res), "Ok(())");
    }

    #[test]
    fn check_output_messages_no() {
        let d = Diagnostics {
            warnings: 0,
            errors: 0,
            ..Default::default()
        };
        let res = crate::output_messages(&d);
        assert!(res.is_ok());
        assert_eq!(format!("{:?}", res), "Ok(())");
    }
}
