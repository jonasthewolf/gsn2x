use anyhow::{anyhow, Context, Result};
use clap::{Arg, ArgAction};
use file_utils::{find_common_ancestors_in_paths, prepare_input_paths, translate_to_output_path};
use render::RenderOptions;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

mod diagnostics;
mod dirgraphsvg;
mod file_utils;
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
                .help("Output the complete view to file with name <COMPLETE_VIEW>.")
                .short('f')
                .long("full")
                .action(ArgAction::Set)
                .default_value("complete.svg")
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
                .help("Output the architecture view to file with name <ARCHITECTURE_VIEW>.")
                .short('a')
                .long("arch")
                .action(ArgAction::Set)
                .default_value("architecture.svg")
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
                .help("Output list of all evidences to file with name <EVIDENCES>.")
                .short('e')
                .long("evidences")
                .action(ArgAction::Set)
                .default_value("evidences.md")
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
            Arg::new("OUTPUT_DIRECTORY")
                .help("Emit all output files to directory <OUTPUT_DIRECTORY>.")
                .short('o')
                .long("output-dir")
                .action(ArgAction::Set)
                .default_value(".")
                .conflicts_with("CHECKONLY")
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
    let mut common_ancestors = find_common_ancestors_in_paths(
        &inputs.iter().map(PathBuf::from).collect::<Vec<PathBuf>>(),
    )?;
    let cwd = PathBuf::from(".").canonicalize()?;
    if common_ancestors.starts_with(&cwd) {
        common_ancestors = common_ancestors.strip_prefix(cwd)?.to_path_buf();
    }
    let all_inputs = prepare_input_paths(inputs)?;
    read_inputs(&all_inputs, &mut nodes, &mut modules, &mut diags)?;

    // Validate
    validate_and_check(&mut nodes, &modules, &mut diags, &excluded_modules, &layers);

    if diags.errors == 0 && !matches.get_flag("CHECKONLY") {
        let render_options = RenderOptions::from(&matches);
        // Output views
        print_outputs(nodes, &modules, &render_options, common_ancestors)?;
    }
    // Output diagnostic messages
    output_messages(&diags)
}

///
/// Read inputs
///
///
fn read_inputs(
    inputs: &[(String, String)],
    nodes: &mut BTreeMap<String, GsnNode>,
    modules: &mut HashMap<String, Module>,
    diags: &mut Diagnostics,
) -> Result<()> {
    for (input, relative_input) in inputs {
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
        let meta: ModuleInformation = match n.remove_entry(MODULE_INFORMATION_NODE) {
            Some((_, GsnDocumentNode::ModuleInformation(x))) => x,
            _ => {
                let module_name = escape_text(relative_input);
                ModuleInformation {
                    name: module_name.to_owned(),
                    brief: None,
                    extends: None,
                    additional: BTreeMap::new(),
                }
            }
        };
        // Add filename and module name to module list
        let module = meta.name.to_owned();

        if let std::collections::hash_map::Entry::Vacant(e) = modules.entry(module.to_owned()) {
            e.insert(Module {
                relative_module_path: relative_input.to_owned().to_owned(),
                meta,
            });
        } else {
            diags.add_error(
                Some(&module),
                format!(
                    "C06: Module name {} in {} was already present in {}.",
                    module,
                    input,
                    modules.get(&module).unwrap().relative_module_path // unwrap is ok, otherwise Entry would not have been Vacant
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
    common_ancestors: PathBuf,
) -> Result<()> {
    let output_path = &render_options.output_directory;
    if !render_options.skip_argument {
        for (module_name, module) in modules {
            let output_path = translate_to_output_path(
                output_path,
                &PathBuf::from(&module.relative_module_path),
            )?
            .with_extension("svg");
            let mut output_file = Box::new(File::create(&output_path).context(format!(
                "Failed to open output file {}",
                &output_path.display()
            ))?) as Box<dyn std::io::Write>;

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
        if let Some(architecture_filename) = &render_options.architecture_filename {
            let mut arch_output = PathBuf::from(&common_ancestors);
            arch_output.push(architecture_filename);

            let arch_output_path = translate_to_output_path(output_path, &arch_output)?;
            let mut output_file = File::create(&arch_output_path).context(format!(
                "Failed to open output file {}",
                &arch_output_path.display()
            ))?;
            let deps = crate::gsn::calculate_module_dependencies(&nodes);
            render::render_architecture(
                &mut output_file,
                modules,
                deps,
                render_options,
                &arch_output_path,
                output_path,
            )?;
        }
        if let Some(complete_filename) = &render_options.complete_filename {
            let mut comp_output = PathBuf::from(&common_ancestors);
            comp_output.push(complete_filename);

            let output_path = translate_to_output_path(output_path, &comp_output)?;
            let mut output_file = File::create(&output_path).context(format!(
                "Failed to open output file {}",
                output_path.display()
            ))?;
            render::render_complete(&mut output_file, &nodes, render_options)?;
        }
    }
    if let Some(evidences_filename) = &render_options.evidences_filename {
        let mut evidence_output = PathBuf::from(&common_ancestors);
        evidence_output.push(evidences_filename);

        let output_path = translate_to_output_path(output_path, &evidence_output)?;
        let mut output_file = File::create(&output_path).context(format!(
            "Failed to open output file {}",
            output_path.display()
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
    use std::path::PathBuf;

    use crate::{diagnostics::Diagnostics, find_common_ancestors_in_paths};
    use anyhow::Result;

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
    fn common_ancestor_many() -> Result<()> {
        let inputs = [
            PathBuf::from("examples/modular/sub1.gsn.yaml"),
            PathBuf::from("examples/modular/main.gsn.yaml"),
        ];
        let mut result = find_common_ancestors_in_paths(&inputs)?;
        let cwd = PathBuf::from(".").canonicalize()?;
        if result.starts_with(&cwd) {
            result = result.strip_prefix(cwd)?.to_path_buf();
        }
        assert_eq!(result, PathBuf::from("examples/modular"));
        Ok(())
    }

    #[test]
    fn common_ancestor_single() -> Result<()> {
        let inputs = [PathBuf::from("examples/example.gsn.yaml")];
        let result = find_common_ancestors_in_paths(&inputs)?;
        assert_eq!(result, PathBuf::from(""));
        Ok(())
    }
}
