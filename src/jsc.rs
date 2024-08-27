use regex::Regex;

fn main() {
    let code = r#"
        // This is a comment
        let x = 5;
        const y = 10;
        var z = x + y;

        function add(a, b = 5) {
            return a + b;
        }

        const obj = { name: "John", age: 30, greet() { return `Hello, ${this.name}`; } };
        const arr = [1, 2, ...[3, 4]];

        class Person {
            constructor(name) {
                this.name = name;
            }
            greet() {
                return `Hello, ${this.name}`;
            }
        }

        class Student extends Person {
            constructor(name, grade) {
                super(name);
                this.grade = grade;
            }
        }

        async function fetchData() {
            try {
                let response = await fetch('https://api.example.com/data');
                let data = await response.json();
                return data;
            } catch (e) {
                console.log(e.message);
            }
        }

        // Array Methods
        const nums = [1, 2, 3, 4];
        const doubled = nums.map(n => n * 2);
        const evens = nums.filter(n => n % 2 === 0);
        const sum = nums.reduce((acc, n) => acc + n, 0);

        // Object Methods
        const keys = Object.keys(obj);
        const values = Object.values(obj);

        // Promise Handling
        const promise = new Promise((resolve, reject) => {
            if (x > 0) resolve("Success");
            else reject("Failure");
        });

        promise.then(result => console.log(result))
               .catch(error => console.error(error));

        // Optional Chaining
        const length = obj.name?.length;

        // Nullish Coalescing
        const name = obj.name ?? "Unknown";

        // Dynamic Imports
        import('module-name').then(module => {
            console.log(module);
        });

        // Modules
        import { func } from './module.js';
        export const value = 42;

        // Enhanced Object Literals
        const person = {
            name,
            greet() { return `Hello, ${this.name}`; }
        };

        // Async Iteration
        async function processAsync() {
            for await (const item of asyncIterable) {
                console.log(item);
            }
        }

        // Symbol Literals
        const sym = Symbol('description');

        // WeakMap and WeakSet
        const weakMap = new WeakMap();
        const weakSet = new WeakSet();
    "#;

    let compiled_code = compile_js(code);
    println!("{}", compiled_code);
}

fn compile_js(code: &str) -> String {
    let mut result = String::new();

    // Regex patterns
    let var_pattern = Regex::new(r"(let|const|var)\s+(\w+)\s*=\s*(.+);").unwrap();
    let function_pattern = Regex::new(r"function\s+(\w+)\s*\(([^)]*)\)\s*\{\s*([^}]*)\s*\}").unwrap();
    let if_pattern = Regex::new(r"if\s*\(([^)]*)\)\s*\{\s*([^}]*)\s*\}\s*else\s*\{\s*([^}]*)\s*\}").unwrap();
    let for_pattern = Regex::new(r"for\s*\(([^)]*)\)\s*\{\s*([^}]*)\s*\}").unwrap();
    let while_pattern = Regex::new(r"while\s*\(([^)]*)\)\s*\{\s*([^}]*)\s*\}").unwrap();
    let do_while_pattern = Regex::new(r"do\s*\{\s*([^}]*)\s*\}\s*while\s*\(([^)]*)\);").unwrap();
    let switch_pattern = Regex::new(r"switch\s*\(([^)]*)\)\s*\{\s*([^}]*)\s*\}").unwrap();
    let class_pattern = Regex::new(r"class\s+(\w+)\s*\{\s*(.*?)\s*\}").unwrap();
    let comment_pattern = Regex::new(r"//.*").unwrap();
    let obj_pattern = Regex::new(r"\{[^}]*\}").unwrap();
    let arr_pattern = Regex::new(r"\[[^\]]*\]").unwrap();
    let arrow_function_pattern = Regex::new(r"(\w+)\s*=\s*\(([^)]*)\)\s*=>\s*\{([^}]*)\}").unwrap();
    let throw_pattern = Regex::new(r"throw\s+([^;]+);").unwrap();
    let return_pattern = Regex::new(r"return\s+([^;]+);").unwrap();
    let break_continue_pattern = Regex::new(r"\b(break|continue)\b;").unwrap();
    let function_call_pattern = Regex::new(r"(\w+)\s*\(([^)]*)\)").unwrap();

    // Additional regex patterns
    let array_methods_pattern = Regex::new(r"(\w+)\.(map|filter|reduce)\s*\(([^)]*)\)").unwrap();
    let object_methods_pattern = Regex::new(r"Object\.(keys|values)\s*\(([^)]*)\)").unwrap();
    let promise_handling_pattern = Regex::new(r"new\s+Promise\s*\(\s*(\w+)\s*\)\s*\.(then|catch)\s*\(([^)]*)\)").unwrap();
    let template_literals_pattern = Regex::new(r"`([^`]*)`").unwrap();
    let set_map_literals_pattern = Regex::new(r"new\s+(Set|Map)\s*\(\[([^\]]*)\]\)").unwrap();
    let destructuring_array_pattern = Regex::new(r"\[\s*([^]]*)\s*\]").unwrap();
    let optional_chaining_pattern = Regex::new(r"(\w+)\?\.(\w+)").unwrap();
    let nullish_coalescing_pattern = Regex::new(r"(\w+)\s*\?\?\s*(\w+)").unwrap();
    let dynamic_import_pattern = Regex::new(r"import\s*\(([^)]*)\)").unwrap();
    let module_pattern = Regex::new(r"import\s+(\{[^}]*\})\s+from\s+(['\"][^'\"]*['\"])").unwrap();
    let default_params_pattern = Regex::new(r"(\w+)\s*=\s*(\w+)").unwrap();
    let enhanced_obj_liter_pattern = Regex::new(r"\{\s*(\w+)\s*:\s*(\w+),\s*(\w+)\s*:\s*\(\w+\)\s*=>\s*\{([^}]*)\}\s*\}").unwrap();
    let async_iteration_pattern = Regex::new(r"for\s+await\s+of\s*\(\s*(\w+)\s*\)").unwrap();
    let symbol_liter_pattern = Regex::new(r"Symbol\s*\(\s*['\"][^'\"]*['\"]\s*\)").unwrap();
    let weak_map_weak_set_pattern = Regex::new(r"new\s+(WeakMap|WeakSet)\s*\(\)").unwrap();

    // Remove comments
    result = comment_pattern.replace_all(code, "").to_string();

    // Replace variable declarations
    result = var_pattern.replace_all(&result, |caps: &regex::Captures| {
        let var_type = &caps[1];
        let var_name = &caps[2];
        let value = &caps[3];
        format!("{} {} = {};", var_type, var_name, value)
    }).to_string();

    // Replace function declarations
    result = function_pattern.replace_all(&result, |caps: &regex::Captures| {
        let func_name = &caps[1];
        let params = &caps[2];
        let body = &caps[3];
        format!("function {}({}) {{\n{}\n}}", func_name, params, body)
    }).to_string();

    // Replace if statements
    result = if_pattern.replace_all(&result, |caps: &regex::Captures| {
        let condition = &caps[1];
        let true_block = &caps[2];
        let false_block = &caps[3];
        format!("if ({}) {{\n{}\n}} else {{\n{}\n}}", condition, true_block, false_block)
    }).to_string();

    // Replace for loops
    result = for_pattern.replace_all(&result, |caps: &regex::Captures| {
        let init = &caps[1];
        let body = &caps[2];
        format!("for ({}) {{\n{}\n}}", init, body)
    }).to_string();

    // Replace while loops
    result = while_pattern.replace_all(&result, |caps: &regex::Captures| {
        let condition = &caps[1];
        let body = &caps[2];
        format!("while ({}) {{\n{}\n}}", condition, body)
    }).to_string();

    // Replace do-while loops
    result = do_while_pattern.replace_all(&result, |caps: &regex::Captures| {
        let body = &caps[1];
        let condition = &caps[2];
        format!("do {{\n{}\n}} while ({})", body, condition)
    }).to_string();

    // Replace switch statements
    result = switch_pattern.replace_all(&result, |caps: &regex::Captures| {
        let condition = &caps[1];
        let cases = &caps[2];
        format!("switch ({}) {{\n{}\n}}", condition, cases)
    }).to_string();

    // Replace class declarations
    result = class_pattern.replace_all(&result, |caps: &regex::Captures| {
        let class_name = &caps[1];
        let body = &caps[2];
        format!("class {} {{\n{}\n}}", class_name, body)
    }).to_string();

    // Replace array methods
    result = array_methods_pattern.replace_all(&result, |caps: &regex::Captures| {
        let array_name = &caps[1];
        let method = &caps[2];
        let args = &caps[3];
        format!("{}.{}({})", array_name, method, args)
    }).to_string();

    // Replace object methods
    result = object_methods_pattern.replace_all(&result, |caps: &regex::Captures| {
        let method = &caps[1];
        let obj = &caps[2];
        format!("Object.{}({})", method, obj)
    }).to_string();

    // Replace promise handling
    result = promise_handling_pattern.replace_all(&result, |caps: &regex::Captures| {
        let promise = &caps[1];
        let method = &caps[2];
        let handler = &caps[3];
        format!("{}.{}({})", promise, method, handler)
    }).to_string();

    // Replace template literals
    result = template_literals_pattern.replace_all(&result, |caps: &regex::Captures| {
        let content = &caps[1];
        format!("`{}`", content)
    }).to_string();

    // Replace set/map literals
    result = set_map_literals_pattern.replace_all(&result, |caps: &regex::Captures| {
        let type_name = &caps[1];
        let items = &caps[2];
        format!("new {}([{}])", type_name, items)
    }).to_string();

    // Replace array destructuring
    result = destructuring_array_pattern.replace_all(&result, |caps: &regex::Captures| {
        let items = &caps[1];
        format!("[{}]", items)
    }).to_string();

    // Replace optional chaining
    result = optional_chaining_pattern.replace_all(&result, |caps: &regex::Captures| {
        let obj = &caps[1];
        let prop = &caps[2];
        format!("{}?.{}", obj, prop)
    }).to_string();

    // Replace nullish coalescing
    result = nullish_coalescing_pattern.replace_all(&result, |caps: &regex::Captures| {
        let left = &caps[1];
        let right = &caps[2];
        format!("{} ?? {}", left, right)
    }).to_string();

    // Replace dynamic imports
    result = dynamic_import_pattern.replace_all(&result, |caps: &regex::Captures| {
        let module = &caps[1];
        format!("import({})", module)
    }).to_string();

    // Replace modules
    result = module_pattern.replace_all(&result, |caps: &regex::Captures| {
        let imports = &caps[1];
        let module_path = &caps[2];
        format!("import {} from {}", imports, module_path)
    }).to_string();

    // Replace default parameters
    result = default_params_pattern.replace_all(&result, |caps: &regex::Captures| {
        let param = &caps[1];
        let default_value = &caps[2];
        format!("{} = {}", param, default_value)
    }).to_string();

    // Replace enhanced object literals
    result = enhanced_obj_liter_pattern.replace_all(&result, |caps: &regex::Captures| {
        let key1 = &caps[1];
        let value1 = &caps[2];
        let key2 = &caps[3];
        let value2 = &caps[4];
        format!("{{ {}: {}, {}: ({}) => {{ {} }} }}", key1, value1, key2, key2, value2)
    }).to_string();

    // Replace async iteration
    result = async_iteration_pattern.replace_all(&result, |caps: &regex::Captures| {
        let iterable = &caps[1];
        format!("for await (const item of {})", iterable)
    }).to_string();

    // Replace symbol literals
    result = symbol_liter_pattern.replace_all(&result, |caps: &regex::Captures| {
        let description = &caps[1];
        format!("Symbol({})", description)
    }).to_string();

    // Replace WeakMap/WeakSet
    result = weak_map_weak_set_pattern.replace_all(&result, |caps: &regex::Captures| {
        let type_name = &caps[1];
        format!("new {}()", type_name)
    }).to_string();

    result
}