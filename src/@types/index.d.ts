declare module 'noxium' {
    // TypeScript Compiler Options
    interface TypeScriptCompilerOptions {
      inputFile: string;
      outputFile: string;
    }
  
    // Regular Expression Compiler Options
    interface RegexCompilerOptions {
      pattern: string;
      inputFile: string;
      outputFile: string;
    }
  
    // Code Minification Compiler Options
    interface MinifyCompilerOptions {
      inputFile: string;
      outputFile: string;
    }
  
    // Bundling Compiler Options
    interface BundleCompilerOptions {
      inputFiles: string[];
      outputFile: string;
    }
  
    // WebAssembly Transformation Compiler Options
    interface WasmTransformCompilerOptions {
      inputFile: string;
      outputFile: string;
    }
  
    // noxium API
    interface noxium {
      typescript(options: TypeScriptCompilerOptions): Promise<void>;
      regex(options: RegexCompilerOptions): Promise<void>;
      minify(options: MinifyCompilerOptions): Promise<void>;
      bundle(options: BundleCompilerOptions): Promise<void>;
      wasmTransform(options: WasmTransformCompilerOptions): Promise<void>;
    }
  
    const noxium: noxium;
    export default noxium;
  }