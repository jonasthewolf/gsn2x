use anyhow::{anyhow, Context, Result};
use clap::{Arg, ArgAction};
use render::RenderOptions;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

mod diagnostics;
mod dirgraphsvg;
mod gsn;
mod render;
mod yaml_fix;

use diagnostics::Diagnostics;
use dirgraphsvg::escape_text;
use gsn::{GsnDocumentNode, GsnNode, Module, ModuleInformation};

const MODULE_INFORMATION_NODE: &str = "module";

///
/// Main entry point.
///
///
fn main() -> Result<()> {
    let app = clap::command!()
        .arg(
            Arg::new("INPUT")
                .help("Sets the input file(s) to use.")
                .action(ArgAction::Append)
                .required(true),
        )
        .arg(
            Arg::new("CHECKONLY")
                .help("Only check the input file(s), but do not output graphs.")
                .short('c')
                .long("check")
                .action(ArgAction::SetTrue)
                .help_heading("CHECKS"),
        )
        .arg(
            Arg::new("EXCLUDED_MODULE")
                .help("Exclude this module from reference checks.")
                .short('x')
                .long("exclude")
                .action(ArgAction::Append)
                .help_heading("CHECKS"),
        )
        .arg(
            Arg::new("NO_ARGUMENT_VIEW")
                .help("Do not output of argument view for provided input files.")
                .short('N')
                .long("no-arg")
                .action(ArgAction::SetTrue)
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("COMPLETE_VIEW")
                .help("Output the complete view to <COMPLETE_VIEW>.")
                .short('f')
                .long("full")
                .action(ArgAction::Set)
                .conflicts_with_all(["CHECKONLY", "NO_COMPLETE_VIEW"])
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("NO_COMPLETE_VIEW")
                .help("Do not output the complete view.")
                .short('F')
                .long("no-full")
                .action(ArgAction::SetTrue)
                .conflicts_with("COMPLETE_VIEW")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("ARCHITECTURE_VIEW")
                .help("Output the architecture view to <ARCHITECTURE_VIEW>.")
                .short('a')
                .long("arch")
                .action(ArgAction::Set)
                .conflicts_with_all(["CHECKONLY", "NO_ARCHITECTURE_VIEW"])
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("NO_ARCHITECTURE_VIEW")
                .help("Do not output the architecture view.")
                .short('A')
                .long("no-arch")
                .action(ArgAction::SetTrue)
                .conflicts_with("ARCHITECTURE_VIEW")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("EVIDENCES")
                .help("Output list of all evidences to <EVIDENCES>.")
                .short('e')
                .long("evidences")
                .action(ArgAction::Append)
                .conflicts_with_all(["CHECKONLY", "NO_EVIDENCES"])
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("NO_EVIDENCES")
                .help("Do not output list of all evidences.")
                .short('E')
                .long("no-evidences")
                .action(ArgAction::SetTrue)
                .conflicts_with("EVIDENCES")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("LAYERS")
                .help("Output additional layer. Can be used multiple times.")
                .short('l')
                .long("layer")
                .action(ArgAction::Append)
                .use_value_delimiter(true)
                .conflicts_with("CHECKONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        .arg(
            Arg::new("STYLESHEETS")
                .help("Links a stylesheet in SVG output. Can be used multiple times.")
                .short('s')
                .long("stylesheet")
                .action(ArgAction::Append)
                .conflicts_with("CHECKONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        .arg(
            Arg::new("EMBED_CSS")
                .help("Embed stylehseets instead of linking them.")
                .short('t')
                .long("embed-css")
                .action(ArgAction::SetTrue)
                .conflicts_with("CHECKONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        // .arg(
        //     Arg::new("MASK_MODULE")
        //         .help("Do not unroll this module in the complete view.")
        //         .short('m')
        //         .long("mask")
        //         .multiple_occurrences(true)
        //         .takes_value(true)
        //         .requires("COMPLETE_VIEW")
        //         .help_heading("OUTPUT MODIFICATION"),
        // )
        .arg(
            Arg::new("NO_LEGEND")
                .help("Do not output a legend based on module information.")
                .short('G')
                .long("no-legend")
                .action(ArgAction::SetTrue)
                .conflicts_with("CHECKONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        .arg(
            Arg::new("FULL_LEGEND")
                .help("Output a legend based on all module information.")
                .short('g')
                .long("full-legend")
                .action(ArgAction::SetTrue)
                .conflicts_with("CHECKONLY")
                .help_heading("OUTPUT MODIFICATION"),
        );
    let matches = app.get_matches();
    let inputs: Vec<&str> = matches
        .get_many::<String>("INPUT")
        .into_iter()
        .flatten()
        .map(AsRef::as_ref)
        .collect();
    let layers = matches
        .get_many::<String>("LAYERS")
        .into_iter()
        .flatten()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>();
    let excluded_modules = matches
        .get_many::<String>("EXCLUDED_MODULE")
        .into_iter()
        .flatten()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>();

    let mut diags = Diagnostics::default();
    let mut nodes = BTreeMap::<String, GsnNode>::new();
    // Module name to module mapping
    let mut modules: HashMap<String, Module> = HashMap::new();

    // Read input
    read_inputs(&inputs, &mut nodes, &mut modules, &mut diags)?;

    // Validate
    validate_and_check(&mut nodes, &modules, &mut diags, &excluded_modules, &layers);

    if diags.errors == 0 && !matches.get_flag("CHECKONLY") {
        let render_options = RenderOptions::from(&matches);
        // Output views
        print_outputs(nodes, &modules, &render_options)?;
    }
    // Output diagnostic messages
    output_messages(&diags)
}

///
/// Read inputs
///
///
fn read_inputs(
    inputs: &[&str],
    nodes: &mut BTreeMap<String, GsnNode>,
    modules: &mut HashMap<String, Module>,
    diags: &mut Diagnostics,
) -> Result<()> {
    for input in inputs {
        let reader =
            BufReader::new(File::open(input).context(format!("Failed to open file {input}"))?);

        let mut n: BTreeMap<String, GsnDocumentNode> = serde_yaml::from_reader(reader)
            .map(|n: yaml_fix::YamlFixMap<String, GsnDocumentNode>| n.into_inner())
            .map_err(|e| {
                anyhow!(format!(
                    "No valid GSN element can be found starting from line {}.\n\
                     This typically means that the YAML is completely invalid, or \n\
                     the `text:` attribute is missing for an element.\n\
                     Original error message: {}.",
                    e.location()
                        .map(|e| e.line().to_string())
                        .unwrap_or_else(|| "unknown".to_owned()),
                    e
                ))
            })
            .context(format!("Failed to parse YAML from file {input}"))?;
        let mut meta: Option<ModuleInformation> = match n.remove_entry(MODULE_INFORMATION_NODE) {
            Some((_, GsnDocumentNode::ModuleInformation(x))) => Some(x),
            _ => None,
        };
        // Add filename and module name to module list
        let module = if let Some(m) = &meta {
            m.name.to_owned()
        } else {
            let module_name = escape_text(input);
            meta = Some(ModuleInformation {
                name: module_name.to_owned(),
                brief: None,
                extends: None,
                additional: BTreeMap::new(),
            });
            module_name
        };

        if let std::collections::hash_map::Entry::Vacant(e) = modules.entry(module.to_owned()) {
            e.insert(Module {
                filename: input.to_owned().to_owned(),
                meta,
            });
        } else {
            diags.add_error(
                Some(&module),
                format!(
                    "C06: Module name {} in {} was already present in {}.",
                    module,
                    input,
                    modules.get(&module).unwrap().filename // unwrap is ok, otherwise Entry would not have been Vacant
                ),
            );
        }

        // Check for duplicates, since they might be in separate files.
        let node_names: Vec<String> = n.keys().cloned().collect();
        for node_name in node_names {
            if let Some((k, v)) = n.remove_entry(&node_name) {
                if let std::collections::btree_map::Entry::Vacant(e) = nodes.entry(k.to_owned()) {
                    match v {
                        GsnDocumentNode::GsnNode(mut x) => {
                            // Remember module for node
                            x.module = module.to_owned();
                            e.insert(x);
                        }
                        _ => unreachable!(), // There can be only one MetaNode
                    }
                } else {
                    diags.add_error(
                        Some(&module),
                        format!(
                            "C07: Element {} in {} was already present in {}.",
                            k,
                            input,
                            nodes.get(&k).unwrap().module // unwrap is ok, otherwise Entry would not have been Vacant
                        ),
                    );
                    break;
                }
            }
        }
    }
    if nodes.is_empty() {
        Err(anyhow!("No input elements found."))
    } else {
        Ok(())
    }
}

///
/// Validate and check modules
///
///
///
///
fn validate_and_check(
    nodes: &mut BTreeMap<String, GsnNode>,
    modules: &HashMap<String, Module>,
    diags: &mut Diagnostics,
    excluded_modules: &[&str],
    layers: &Vec<&str>,
) {
    for (module_name, module_info) in modules {
        // Validation for well-formedness is done unconditionally.
        gsn::validation::validate_module(diags, module_name, module_info, nodes);
        if diags.errors > 0 {
            break;
        }
    }
    if diags.errors == 0 {
        gsn::extend_modules(diags, nodes, modules);
        gsn::check::check_nodes(diags, nodes, excluded_modules);
        if !layers.is_empty() {
            gsn::check::check_layers(diags, nodes, layers);
        }
    }
}

///
/// Print outputs
///
///
///
///
fn print_outputs(
    nodes: BTreeMap<String, GsnNode>,
    modules: &HashMap<String, Module>,
    render_options: &RenderOptions,
) -> Result<()> {
    if !render_options.skip_argument {
        for (module_name, module) in modules {
            let pbuf = std::path::PathBuf::from(&module.filename).with_extension("svg");
            let mut output_file = Box::new(
                File::create(&pbuf)
                    .context(format!("Failed to open output file {}", &pbuf.display()))?,
            ) as Box<dyn std::io::Write>;

            render::render_argument(
                &mut output_file,
                module_name,
                modules,
                &nodes,
                render_options,
            )?;
        }
    }
    if modules.len() > 1 {
        if !render_options.skip_architecture {
            // unwrap is ok, since we just checked that modules has at least two elements.
            let pbuf = std::path::PathBuf::from(&modules.iter().next().unwrap().1.filename)
                .with_file_name("architecture.svg");
            let output_filename = render_options
                .architecture_filename
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| pbuf);
            let mut output_file = File::create(&output_filename).context(format!(
                "Failed to open output file {}",
                output_filename.display()
            ))?;
            let deps = crate::gsn::calculate_module_dependencies(&nodes);
            render::render_architecture(&mut output_file, modules, deps, render_options)?;
        }
        if !render_options.skip_complete {
            // unwrap is ok, since we just checked that modules has at least two elements.
            let pbuf = std::path::PathBuf::from(&modules.iter().next().unwrap().1.filename)
                .with_file_name("complete.svg");
            let output_filename = render_options
                .complete_filename
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| pbuf);
            let mut output_file = File::create(&output_filename).context(format!(
                "Failed to open output file {}",
                output_filename.display()
            ))?;
            render::render_complete(&mut output_file, &nodes, render_options)?;
        }
    }
    if !render_options.skip_evidences {
        // Unwrap is ok, since `modules` contains at least one module
        let pbuf = std::path::PathBuf::from(&modules.iter().next().unwrap().1.filename)
            .with_file_name("evidences.md");
        let output_filename = render_options
            .evidences_filename
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| pbuf);
        let mut output_file = File::create(&output_filename).context(format!(
            "Failed to open output file {}",
            output_filename.display()
        ))?;
        render::render_evidences(&mut output_file, &nodes, render_options)?;
    }
    Ok(())
}

///
/// Render to dot-file if not only validation is active.
/// Output summary of warnings and errors.
///
fn output_messages(diags: &Diagnostics) -> Result<()> {
    for msg in &diags.messages {
        eprintln!("{msg}");
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
