use crate::ast::*;
use std::collections::HashMap;

pub struct CodeGen {
    output: String,
    temp_count: i32,
    label_count: i32,
    string_constants: HashMap<String, String>,
    symbol_table: HashMap<String, (String, Type)>, // Maps variable names to (IR register/pointer name, Type)
}

impl CodeGen {
    pub fn new() -> Self {
        CodeGen {
            output: String::new(),
            temp_count: 0,
            label_count: 0,
            string_constants: HashMap::new(),
            symbol_table: HashMap::new(),
        }
    }

    fn new_temp(&mut self) -> String {
        let name = format!("%t{}", self.temp_count);
        self.temp_count += 1;
        name
    }

    fn new_label(&mut self, prefix: &str) -> String {
        let name = format!("{}{}", prefix, self.label_count);
        self.label_count += 1;
        name
    }

    fn emit(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    pub fn generate(&mut self, program: &Program) -> String {
        self.emit("; ModuleID = 'c-script'");
        self.emit("source_filename = \"c-script\"");
        self.emit("target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"");
        
        // Determine target triple (simplified)
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        self.emit("target triple = \"x86_64-pc-linux-gnu\"");
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        self.emit("target triple = \"x86_64-apple-darwin\"");
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        self.emit("target triple = \"aarch64-apple-darwin\"");

        self.emit("");
        
        // Declare runtime functions
        self.emit("declare void @cscript_print_int(i32)");
        self.emit("declare void @cscript_print_float(float)");
        self.emit("declare void @cscript_print_string(i8*)");
        self.emit("declare i32 @cscript_fopen(i8*, i8*)");
        self.emit("declare i32 @cscript_fwrite(i32, i8*)");
        self.emit("declare i32 @cscript_fclose(i32)");
        self.emit("declare i8* @cscript_fread(i32, i32)");
        self.emit("declare i32 @cscript_system(i8*)");
        self.emit("declare i8* @cscript_getenv(i8*)");
        self.emit("");

        // Separate functions, imports, and statements
        let mut functions = Vec::new();
        let mut imports = Vec::new();
        let mut statements = Vec::new();

        for stmt in &program.statements {
            match stmt {
                Statement::FunctionDef(..) => functions.push(stmt),
                Statement::Import(..) => imports.push(stmt),
                _ => statements.push(stmt),
            }
        }

        // Process imports (currently just placeholders in IR if needed, but runtime is already declared)
        for _imp in imports {
            // In the python version, this conditionally declared runtime funcs.
            // We declared them all above for simplicity.
        }

        // Generate user functions
        for func in functions {
            if let Statement::FunctionDef(name, params, ret_type, body) = func {
                self.gen_function(name, params, ret_type, body);
            }
        }

        // Generate main function for top-level statements
        if !statements.is_empty() {
            self.emit("define i32 @main() {");
            self.emit("entry:");
            
            for stmt in statements {
                self.gen_statement(stmt);
            }

            self.emit("  ret i32 0");
            self.emit("}");
        }

        // Emit string constants at the top (or bottom, doesn't matter for LLVM IR)
        let mut header = String::new();
        for (s, var_name) in &self.string_constants {
            let len = s.len() + 1; // +1 for null terminator
            // Escape string for LLVM IR
            let mut escaped = String::new();
            for b in s.bytes() {
                if b < 32 || b > 126 || b == 34 || b == 92 {
                    escaped.push_str(&format!("\\{:02X}", b));
                } else {
                    escaped.push(b as char);
                }
            }
            header.push_str(&format!("{} = private unnamed_addr constant [{}_x_i8] c\"{}\\00\", align 1\n", var_name, len, escaped).replace("_x_", " x "));
        }
        
        self.output.clone() + &header
    }

    fn gen_function(&mut self, name: &str, params: &[(Type, String)], ret_type: &Type, body: &[Statement]) {
        let llvm_ret_type = self.get_llvm_type(ret_type);
        let param_str = params.iter().map(|(t, _)| self.get_llvm_type(t)).collect::<Vec<_>>().join(", ");
        
        self.emit(&format!("define {} @{}({}) {{", llvm_ret_type, name, param_str));
        self.emit("entry:");

        // Clear symbol table for new scope (simplified, no nested scopes yet)
        // Note: Global variables not supported yet in this simple version
        self.symbol_table.clear();

        // Allocate space for parameters and store them
        for (i, (ty, param_name)) in params.iter().enumerate() {
            let llvm_ty = self.get_llvm_type(ty);
            let _alloc_name = format!("%{}", param_name); // Shadowing arg with alloc
            // Actually, args are %0, %1... unless named. Let's assume we can name them in definition?
            // LLVM IR allows named args.
            // But to be safe and match Python logic:
            // Args come in as %0, %1 etc if we don't name them in the signature.
            // Let's name them in signature?
            // "define i32 @foo(i32 %a, i32 %b)"
            
            // Re-emit function header with named args?
            // Let's just use %0, %1 and store to alloca
            
            let alloca = self.new_temp(); // Allocate stack space
            self.emit(&format!("  {} = alloca {}, align 4", alloca, llvm_ty));
            self.symbol_table.insert(param_name.clone(), (alloca.clone(), ty.clone()));
            
            self.emit(&format!("  store {} %{}, {}* {}, align 4", llvm_ty, i, llvm_ty, alloca));
        }

        for stmt in body {
            self.gen_statement(stmt);
        }

        // Add implicit return for void functions or if missing
        if llvm_ret_type == "void" {
            self.emit("  ret void");
        } else if llvm_ret_type == "i32" {
             // Basic fallback
             self.emit("  ret i32 0");
        }

        self.emit("}");
        self.emit("");
    }

    fn get_llvm_type(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "i32".to_string(),
            Type::Float => "float".to_string(),
            Type::Char => "i8".to_string(),
            Type::Pointer(inner) => format!("{}*", self.get_llvm_type(inner)),
        }
    }

    fn gen_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::VarDecl(ty, name, init) => {
                let llvm_ty = self.get_llvm_type(ty);
                let ptr = self.new_temp();
                self.emit(&format!("  {} = alloca {}, align 4", ptr, llvm_ty));
                self.symbol_table.insert(name.clone(), (ptr.clone(), ty.clone()));

                if let Some(expr) = init {
                    let (val, val_ty) = self.gen_expression(expr);
                    // Simple type check/cast could go here
                    self.emit(&format!("  store {} {}, {}* {}, align 4", val_ty, val, llvm_ty, ptr));
                } else {
                    // Default init to 0
                     self.emit(&format!("  store {} 0, {}* {}, align 4", llvm_ty, llvm_ty, ptr));
                }
            }
            Statement::Assign(lhs, rhs) => {
                let (val, val_ty) = self.gen_expression(rhs);
                
                // Determine target pointer
                let (ptr, ptr_ty) = match lhs {
                    Expression::Identifier(name) => {
                        if let Some((p, var_type)) = self.symbol_table.get(name) {
                             let llvm_ty = self.get_llvm_type(var_type);
                             (p.clone(), format!("{}*", llvm_ty))
                        } else {
                            panic!("Unknown variable: {}", name);
                        }
                    }
                    Expression::ArrayAccess(name, index) => {
                         let (array_ptr, array_type) = self.symbol_table.get(name).expect("Unknown array").clone();
                         let (idx_val, _) = self.gen_expression(index);
                         let elem_ptr = self.new_temp();
                         // Extract element type from pointer type
                         let elem_type = if let Type::Pointer(inner) = array_type {
                             self.get_llvm_type(&inner)
                         } else {
                             panic!("Array variable is not a pointer type");
                         };
                         self.emit(&format!("  {} = getelementptr inbounds {}, {}* {}, i32 {}", elem_ptr, elem_type, elem_type, array_ptr, idx_val));
                         (elem_ptr, format!("{}*", elem_type))
                    }
                    _ => panic!("Invalid assignment target"),
                };

                self.emit(&format!("  store {} {}, {} {}, align 4", val_ty, val, ptr_ty, ptr));
            }
            Statement::Expression(expr) => {
                self.gen_expression(expr);
            }
            Statement::If(cond, then_block, else_block) => {
                let (cond_val, _) = self.gen_expression(cond);
                let cmp = self.new_temp();
                self.emit(&format!("  {} = icmp ne i32 {}, 0", cmp, cond_val));
                
                let label_then = self.new_label("then");
                let label_else = self.new_label("else");
                let label_merge = self.new_label("merge");
                
                let target_else = if else_block.is_some() { &label_else } else { &label_merge };

                self.emit(&format!("  br i1 {}, label %{}, label %{}", cmp, label_then, target_else));

                self.emit(&format!("{}:", label_then));
                for s in then_block.iter() {
                    self.gen_statement(s);
                }
                self.emit(&format!("  br label %{}", label_merge));

                if let Some(block) = else_block {
                    self.emit(&format!("{}:", label_else));
                    for s in block.iter() {
                        self.gen_statement(s);
                    }
                    self.emit(&format!("  br label %{}", label_merge));
                }

                self.emit(&format!("{}:", label_merge));
            }
            Statement::While(cond, body) => {
                let label_cond = self.new_label("whilecond");
                let label_body = self.new_label("whilebody");
                let label_end = self.new_label("whileend");

                self.emit(&format!("  br label %{}", label_cond));
                self.emit(&format!("{}:", label_cond));
                
                let (cond_val, _) = self.gen_expression(cond);
                let cmp = self.new_temp();
                self.emit(&format!("  {} = icmp ne i32 {}, 0", cmp, cond_val));
                self.emit(&format!("  br i1 {}, label %{}, label %{}", cmp, label_body, label_end));

                self.emit(&format!("{}:", label_body));
                for s in body.iter() {
                    self.gen_statement(s);
                }
                self.emit(&format!("  br label %{}", label_cond));

                self.emit(&format!("{}:", label_end));
            }
            Statement::For(init, cond, update, body) => {
                self.gen_statement(init);
                
                let label_cond = self.new_label("forcond");
                let label_body = self.new_label("forbody");
                let label_end = self.new_label("forend");

                self.emit(&format!("  br label %{}", label_cond));
                self.emit(&format!("{}:", label_cond));
                
                let (cond_val, _) = self.gen_expression(cond);
                let cmp = self.new_temp();
                self.emit(&format!("  {} = icmp ne i32 {}, 0", cmp, cond_val));
                self.emit(&format!("  br i1 {}, label %{}, label %{}", cmp, label_body, label_end));

                self.emit(&format!("{}:", label_body));
                for s in body.iter() {
                    self.gen_statement(s);
                }
                self.gen_statement(update);
                self.emit(&format!("  br label %{}", label_cond));

                self.emit(&format!("{}:", label_end));
            }
            Statement::Return(expr) => {
                let (val, val_ty) = self.gen_expression(expr);
                self.emit(&format!("  ret {} {}", val_ty, val));
            }
            Statement::Import(_) => {} // Handled at top level
            Statement::FunctionDef(..) => {} // Handled at top level
            Statement::ArrayDecl(ty, name, size) => {
                let llvm_ty = self.get_llvm_type(ty);
                let ptr = self.new_temp();
                // Array allocation
                self.emit(&format!("  {} = alloca [{} x {}], align 4", ptr, size, llvm_ty));
                // Store the array pointer with its element type wrapped in Pointer
                self.symbol_table.insert(name.clone(), (ptr.clone(), Type::Pointer(Box::new(ty.clone()))));
            }
        }
    }

    fn gen_expression(&mut self, expr: &Expression) -> (String, String) {
        match expr {
            Expression::Number(n) => (n.to_string(), "i32".to_string()),
            Expression::String(s) => {
                let name = format!("@.str.{}", self.string_constants.len());
                if !self.string_constants.contains_key(s) {
                    self.string_constants.insert(s.clone(), name.clone());
                } else {
                    // Find existing name (inefficient search but fine for now)
                    // Actually I need to retrieve the name if it exists.
                    // Let's just create new ones for now or fix the map logic.
                    // The map is String -> Name.
                    // So:
                    // let name = self.string_constants.get(s).unwrap().clone();
                }
                let var_name = self.string_constants.get(s).unwrap().clone();
                let ptr = self.new_temp();
                let len = s.len() + 1;
                self.emit(&format!("  {} = getelementptr inbounds [{} x i8], [{} x i8]* {}, i64 0, i64 0", ptr, len, len, var_name));
                (ptr, "i8*".to_string())
            }
            Expression::Identifier(name) => {
                if let Some((ptr, var_type)) = self.symbol_table.get(name).cloned() {
                    let val = self.new_temp();
                    let llvm_ty = self.get_llvm_type(&var_type);
                    self.emit(&format!("  {} = load {}, {}* {}, align 4", val, llvm_ty, llvm_ty, ptr));
                    (val, llvm_ty)
                } else {
                    panic!("Unknown variable: {}", name);
                }
            }
            Expression::BinOp(lhs, op, rhs) => {
                let (l_val, _) = self.gen_expression(lhs);
                let (r_val, _) = self.gen_expression(rhs);
                let res = self.new_temp();
                
                let op_str = match op {
                    BinOpType::Add => "add",
                    BinOpType::Sub => "sub",
                    BinOpType::Mul => "mul",
                    BinOpType::Div => "sdiv",
                    BinOpType::Less => "icmp slt",
                    BinOpType::Greater => "icmp sgt",
                    BinOpType::LessEq => "icmp sle",
                    BinOpType::GreaterEq => "icmp sge",
                    BinOpType::Eq => "icmp eq",
                    BinOpType::NotEq => "icmp ne",
                };

                if op_str.starts_with("icmp") {
                    self.emit(&format!("  {} = {} i32 {}, {}", res, op_str, l_val, r_val));
                    let bool_res = self.new_temp();
                    self.emit(&format!("  {} = zext i1 {} to i32", bool_res, res));
                    (bool_res, "i32".to_string())
                } else {
                    self.emit(&format!("  {} = {} i32 {}, {}", res, op_str, l_val, r_val));
                    (res, "i32".to_string())
                }
            }
            Expression::FuncCall(name, args) => {
                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(self.gen_expression(arg));
                }
                
                let res = self.new_temp();
                let arg_str = arg_vals.iter().map(|(v, t)| format!("{} {}", t, v)).collect::<Vec<_>>().join(", ");
                
                // Determine return type (hacky)
                let ret_ty = if name == "cscript_fopen" || name == "cscript_fwrite" || name == "cscript_fclose" || name == "cscript_system" {
                    "i32"
                } else if name == "cscript_fread" || name == "cscript_getenv" {
                    "i8*"
                } else if name == "print" {
                    // print maps to different runtime funcs
                    let (v, t) = &arg_vals[0];
                    if t == "i32" {
                        self.emit(&format!("  call void @cscript_print_int(i32 {})", v));
                    } else if t == "float" {
                        self.emit(&format!("  call void @cscript_print_float(float {})", v));
                    } else {
                        self.emit(&format!("  call void @cscript_print_string(i8* {})", v));
                    }
                    return ("0".to_string(), "i32".to_string()); // void return
                } else {
                    "i32" // Default user func return
                };

                if ret_ty == "void" {
                     self.emit(&format!("  call void @{}({})", name, arg_str));
                     ("0".to_string(), "i32".to_string())
                } else {
                     self.emit(&format!("  {} = call {} @{}({})", res, ret_ty, name, arg_str));
                     (res, ret_ty.to_string())
                }
            }
            Expression::ArrayAccess(name, index) => {
                 let (array_ptr, array_type) = self.symbol_table.get(name).expect("Unknown array").clone();
                 let (idx_val, _) = self.gen_expression(index);
                 let elem_ptr = self.new_temp();
                 // Extract element type from pointer type
                 let elem_type = if let Type::Pointer(inner) = array_type {
                     self.get_llvm_type(&inner)
                 } else {
                     panic!("Array variable is not a pointer type");
                 };
                 // Note: We still need array size info for proper GEP. For now, use pointer arithmetic.
                 // This works if the array has decayed to a pointer or we use simplified GEP.
                 self.emit(&format!("  {} = getelementptr inbounds {}, {}* {}, i32 {}", elem_ptr, elem_type, elem_type, array_ptr, idx_val));
                 
                 let val = self.new_temp();
                 self.emit(&format!("  {} = load {}, {}* {}, align 4", val, elem_type, elem_type, elem_ptr));
                 (val, elem_type)
            }
            _ => ("0".to_string(), "i32".to_string()), // Unimplemented
        }
    }
}
