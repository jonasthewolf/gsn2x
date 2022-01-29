use crate::gsn::{get_levels, GsnNode, ModuleDependency};
use crate::tera::{Pad, Ralign, WordWrap};
use crate::yaml_fix::MyMap;
use ::tera::Tera;
use anyhow::Context;
use std::collections::BTreeMap;
use std::io::Write;

pub enum View {
    Argument,
    Architecture,
    Complete,
    Evidences,
}

pub struct StaticRenderContext<'a> {
    pub modules: &'a [String],
    pub input_files: &'a [&'a str],
    pub layers: &'a Option<Vec<&'a str>>,
    pub stylesheet: Option<&'a str>,
}

///
/// Use Tera to create dot-file.
/// Templates are inlined in executable.
///
///
pub fn render_view(
    module: &str,
    nodes: &MyMap<String, GsnNode>,
    dependencies: Option<&BTreeMap<String, BTreeMap<String, ModuleDependency>>>,
    output: &mut impl Write,
    view: View,
    ctx: &StaticRenderContext,
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
    context.insert("modules", ctx.modules);
    context.insert("dependencies", &dependencies);
    context.insert("nodes", &nodes);
    context.insert("layers", &ctx.layers);
    context.insert("levels", &get_levels(nodes));
    context.insert("stylesheet", &ctx.stylesheet);
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
