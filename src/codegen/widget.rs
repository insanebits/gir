use std::io::{Result, Write};

use analysis;
use env::Env;
use super::{function, general};
use super::general::tabs;

pub fn generate<W: Write>(w: &mut W, env: &Env, analysis: &analysis::object::Info) -> Result<()>{
    let type_ = analysis.type_(&env.library);

    try!(general::start_comments(w, &env.config));
    try!(general::uses(w, &analysis.imports, &env.config.library_name, env.config.min_cfg_version));
    try!(general::objects_child_type(w, &analysis.name, &type_.c_type));
    try!(general::impl_parents(w, &analysis.name, &analysis.parents));
    try!(general::impl_interfaces(w, &analysis.name, &analysis.implements));

    if generate_inherent(analysis) {
        try!(writeln!(w, ""));
        try!(writeln!(w, "impl {} {{", analysis.name));
        for func_analysis in &analysis.constructors() {
            try!(function::generate(w, env, func_analysis, false, false, 1));
        }

        if !generate_trait(analysis) {
            for func_analysis in &analysis.methods() {
                try!(function::generate(w, env, func_analysis, false, false, 1));
            }
        }

        for func_analysis in &analysis.functions() {
            try!(function::generate(w, env, func_analysis, false, false, 1));
        }
        try!(writeln!(w, "}}"));
    }
    try!(general::impl_static_type(w, &analysis.name, &type_.glib_get_type));

    if generate_trait(analysis) {
        try!(writeln!(w, ""));
        try!(writeln!(w, "pub trait {}Ext {{", analysis.name));
        for func_analysis in &analysis.methods() {
            try!(function::generate(w, env, func_analysis, true, true, 1));
        }
        try!(writeln!(w, "}}"));

        try!(writeln!(w, ""));
        try!(writeln!(w, "impl<O: Upcast<{}>> {}Ext for O {{", analysis.name, analysis.name));
        for func_analysis in &analysis.methods() {
            try!(function::generate(w, env, func_analysis, true, false, 1));
        }
        try!(writeln!(w, "}}"));
    }

    Ok(())
}

fn generate_inherent(analysis: &analysis::object::Info) -> bool {
    analysis.has_constructors || analysis.has_functions || !analysis.has_children
}

fn generate_trait(analysis: &analysis::object::Info) -> bool {
    analysis.has_children
}

pub fn generate_reexports(env: &Env, analysis: &analysis::object::Info, module_name: &str,
        contents: &mut Vec<String>, traits: &mut Vec<String>) {
    let version_cfg = general::version_condition_string(&env.config.library_name,
        env.config.min_cfg_version, analysis.version, false, 0);
    let (cfg, cfg_1) = match version_cfg {
        Some(s) => (format!("{}\n", s), format!("{}{}\n", tabs(1), s)),
        None => ("".into(), "".into()),
    };
    contents.push(format!(""));
    contents.push(format!("{}mod {};", cfg, module_name));
    contents.push(format!("{}pub use self::{}::{};", cfg, module_name, analysis.name));
    if generate_trait(analysis) {
        contents.push(format!("{}pub use self::{}::{}Ext;", cfg, module_name, analysis.name));
        traits.push(format!("{}{}pub use super::{}Ext;", cfg_1, tabs(1), analysis.name));
    }
}
