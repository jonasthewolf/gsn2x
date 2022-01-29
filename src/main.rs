use ::tera::Tera;
use anyhow::{anyhow, Context, Result};
use clap::{app_from_crate, Arg, ErrorKind};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};

mod diagnostics;
mod gsn;
mod tera;
mod yaml_fix;

use crate::tera::Pad;
use crate::tera::Ralign;
use crate::tera::WordWrap;
use diagnostics::Diagnostics;
use gsn::{GsnNode, ModuleDependency};
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
                .use_delimiter(true)
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
    let excluded_modules = matches
        .values_of("EXCLUDED_MODULE")
        .map(|x| x.collect::<Vec<&str>>());

    // Read input
    for input in &inputs {
        let module = escape_module_name(input);
        let mut reader =
            BufReader::new(File::open(&input).context(format!("Failed to open file {}", input))?);
        let mut n =
            read_input(&mut reader).context(format!("Failed to parse YAML from file {}", input))?;
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
                return output_messages(&diags);
            }
        }
        // Merge nodes for further processing.
        nodes.append(&mut n);
    }

    // Validate
    for input in &inputs {
        let module = escape_module_name(input);
        // When validating a module, all references are resolved.
        if let Some(excluded) = &excluded_modules {
            if excluded.contains(input) {
                continue;
            }
        }
        gsn::validate_module(&mut diags, &module, &nodes);
        if let Some(lays) = &layers {
            gsn::check_layers(&mut diags, &module, &nodes, lays);
        }
    }
    // TODO Check that only one global top-level element remains
    // TODO Return really necessary?
    if diags.errors > 0 {
        return output_messages(&diags);
    }

    // Output argument view
    if !(matches.is_present("VALONLY") || matches.is_present("SUPPRESSARGUMENT")) {
        for input in &inputs {
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
            render_view(
                &escape_module_name(input),
                &inputs,
                &nodes,
                &layers,
                matches.value_of("STYLESHEET"),
                None,
                &mut output_file,
                View::Argument,
            )?;
        }
    }

    //
    // Additional outputs
    //

    // Architecture view
    if let Some(arch_view) = matches.value_of("ARCHITECTURE_VIEW") {
        let mut output_file =
            File::create(arch_view).context(format!("Failed to open output file {}", arch_view))?;
        let deps = crate::gsn::calculate_module_dependencies(&nodes);
        render_view(
            &escape_module_name(&arch_view),
            &inputs,
            &nodes,
            &layers,
            matches.value_of("STYLESHEET"),
            Some(&deps),
            &mut output_file,
            View::Architecture,
        )?;
    }

    // Complete view
    if let Some(compl_view) = matches.value_of("COMPLETE_VIEW") {
        let mut output_file = File::create(compl_view)
            .context(format!("Failed to open output file {}", compl_view))?;
        render_view(
            &escape_module_name(&compl_view),
            &inputs,
            &nodes,
            &layers,
            matches.value_of("STYLESHEET"),
            None,
            &mut output_file,
            View::Complete,
        )?;
    }

    // List of evidences
    if let Some(output) = matches.value_of("EVIDENCES") {
        let mut output_file =
            File::create(output).context(format!("Failed to open output file {}", output))?;
        render_view(
            "Evidences",
            &inputs,
            &nodes,
            &layers,
            None, // No stylesheet for Markdown
            None,
            &mut output_file,
            View::Evidences,
        )?;
    }

    // Output diagnostic messages
    output_messages(&diags)
}

///
/// Escape module name
///
/// Remove espcially the "."'s, since the module name is used in the template as a key for a map.
/// However, Tera cannot cope with that. The dot is interpreted as a separator for attributes.
///
fn escape_module_name(input: &&str) -> String {
    input
        .replace(".", "_")
        .replace("-", "_")
        .replace(" ", "_")
        .replace("/", "_")
        .replace("\\", "_")
        .replace(":", "_")
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

enum View {
    Argument,
    Architecture,
    Complete,
    Evidences,
}

///
/// Use Tera to create dot-file.
/// Templates are inlined in executable.
///
/// TODO clippy warning, remove too many parameters e.g. by introducing a struct for context
///
fn render_view(
    module: &str,
    modules: &[&str],
    nodes: &MyMap<String, GsnNode>,
    layers: &Option<Vec<&str>>,
    stylesheet: Option<&str>,
    dependencies: Option<&BTreeMap<String, BTreeMap<String, ModuleDependency>>>,
    output: &mut impl Write,
    view: View,
) -> Result<(), anyhow::Error> {
    let mut context = ::tera::Context::new();
    // Note the max() at the end, so we don't get a NaN when calculating width
    let num_solutions = nodes
        .iter()
        .filter(|(id, _)| id.starts_with("Sn"))
        .count()
        .max(1);
    let width = (num_solutions as f32).log10().ceil() as usize;
    context.insert("module", module);
    context.insert("modules", modules);
    context.insert("dependencies", &dependencies);
    context.insert("nodes", &nodes);
    context.insert("layers", &layers);
    context.insert("levels", &get_levels(nodes));
    context.insert("stylesheet", &stylesheet);
    context.insert("evidences_width", &width);
    let mut tera = Tera::default();
    tera.register_filter("wordwrap", WordWrap);
    tera.register_filter("ralign", Ralign);
    tera.register_filter("pad", Pad);
    tera.add_raw_templates(vec![
        ("macros.dot", include_str!("../templates/macros.dot")),
        ("argument.dot", include_str!("../templates/argument.dot")),
        (
            "architecture.dot",
            include_str!("../templates/architecture.dot"),
        ),
        ("complete.dot", include_str!("../templates/complete.dot")),
        ("evidences.md", include_str!("../templates/evidences.md")),
    ])?;
    let template = match view {
        View::Argument => "argument.dot",
        View::Architecture => "architecture.dot",
        View::Complete => "complete.dot",
        View::Evidences => "evidences.md",
    };
    tera.render_to(template, &context, output)
        .context("Failed to write to output.")?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::diagnostics::Diagnostics;
    use crate::*;

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
