use anyhow::{Context, Error};
use std::{env, fs, path::PathBuf, sync::Arc};
use swc_common::{chain, sync::Lrc, FileName, SourceMap};
use swc_ecma_codegen::{text_writer::JsWriter, Emitter};
use swc_ecma_minifier::optimize;
use swc_ecma_parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax, TsConfig};
use swc_ecma_transforms::{fixer, resolver_with_mark};
use swc_ecma_visit::FoldWith;

fn main() -> Result<(), Error> {
    // Set up the source map and environment
    let cm: Lrc<SourceMap> = Default::default();

    // Collect JavaScript and TypeScript files from the command line
    let args: Vec<String> = env::args().collect();
    let files: Vec<PathBuf> = args.iter().skip(1).map(PathBuf::from).collect();

    for file in files {
        let fm = cm.load_file(&file).context("Failed to load file")?;

        let syntax = if file.extension().map_or(false, |ext| ext == "ts" || ext == "tsx") {
            Syntax::Typescript(TsConfig {
                tsx: true,
                dynamic_import: true,
                decorators: true,
                ..Default::default()
            })
        } else {
            Syntax::Es(EsConfig {
                jsx: true,
                dynamic_import: true,
                ..Default::default()
            })
        };

        // Parse the file
        let lexer = Lexer::new(
            syntax,
            EsConfig::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);
        let mut module = parser.parse_module().expect("Failed to parse module");

        // Apply custom transformations (e.g., removing console statements)
        let mut passes = chain!(resolver_with_mark(), fixer(None));
        module = module.fold_with(&mut passes);

        // Minify the module
        let minified_module = optimize(
            module.clone(),
            cm.clone(),
            None,
            None,
            &Default::default(),
            &Default::default(),
        );

        // Convert the minified AST back to JavaScript code
        let mut buf = vec![];
        {
            let mut emitter = Emitter {
                cfg: swc_ecma_codegen::Config { minify: true },
                cm: cm.clone(),
                comments: None,
                wr: Box::new(JsWriter::new(cm.clone(), "\n", &mut buf, None)),
            };
            minified_module.emit_with(&mut emitter).expect("Failed to emit JS code");
        }

        let minified_code = String::from_utf8(buf)?;

        // Write the minified code to an output file
        let output_path = file.with_extension("min.js");
        fs::write(&output_path, minified_code).context("Failed to write output file")?;
        println!("Minified file written to: {}", output_path.display());
    }
    
    Ok(())
}