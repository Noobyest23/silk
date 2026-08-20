
use core::panic;
use std::{collections::{HashMap, HashSet}, process::exit};
use crate::{
    environment::{scope::Scope, value::SilkValue::Pointer}, parser::ast::{
        Program, ProgramExpression, ProgramStatement,
        expr::{ExprNode::{self}, SilkAssignment, SilkOperator},
        stmt::StmtNode,
    },
};
use crate::lexer::Lexer;
use crate::parser::Parser;
use colored_text::Colorize;

use super::value::SilkValue;

type SilkType = usize;

const SILK_EXIT_OK: i32 = 0;
const SILK_EXIT_ERROR: i32 = 1;

// Supporting enum for numeric coercion
    enum EitherNumbers {
        Int(i32, i32),     // Adjust integer type (i32/i64) to match your SilkValue implementation
        Float(f32, f32),   // Adjust float type (f32/f64) to match your SilkValue implementation
    }

#[derive(Clone)]
pub enum SilkHandle {
    HeapAllocated(usize),
    StackAllocated(usize),
    HeapElement(usize, usize),
    GlobalValue(String),
    ObjectField(Box<SilkHandle>, String),
}

pub struct VirtualMachine {
    
    pub heap: HashMap<usize, SilkValue>,
    
    pub next_heap_ptr: usize,
    
    stack: Vec<SilkValue>,
    
    pub globals: HashMap<String, SilkValue>,
    
    scope: Scope,
    
    pub modules: HashMap<String, HashMap<String, SilkValue>>,
    
    o_ptr: usize,

    trace_stack: Vec<(SilkValue, u32, u32)>
}

impl VirtualMachine {
    pub fn new() -> Self {
        let mut heap: HashMap<usize, SilkValue> = HashMap::new();
        heap.insert(0 as usize, SilkValue::Null);

        Self {
            heap: heap,
            next_heap_ptr: 1,
            stack: Vec::new(),
            globals: HashMap::new(),
            scope: Scope::new(),
            modules: HashMap::new(),
            o_ptr: 0,
            trace_stack: Vec::new()
        }
    }

    pub fn error(&self, msg: String) {
        println!("{}: {}", "[Runtime Error]".red(), msg);
        println!("    Stack Trace: ");
        for call in &self.trace_stack {
            println!("        -{}, line: {}, column: {}", call.0, call.1, call.2)
        }
        exit(-1);
    }

    pub fn stringify_value(&self, value: &SilkValue) -> String {
        match value {
            SilkValue::Object(map) => {
                let mut result = "{".to_string();
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&format!("{} : {}", key, self.stringify_value(value)));
                }
                result.push('}');
                result
            }
            SilkValue::Pointer(ptr) => {
                self.stringify_value(&self.get_value_from_handle(&SilkHandle::HeapAllocated(*ptr)).expect("Invalid Pointer"))
            }
            SilkValue::List(elements) => {
                let mut result = "[".to_string();
                for i in 0..elements.len() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    let element = &elements[i];
                    
                    result.push_str(&self.stringify_value(element));
                }
                result.push(']');
                result
            }
            _ => format!("{}", value)
        }
    }

    pub fn stack_push(&mut self, v: SilkValue) -> SilkHandle {
        self.stack.push(v);
        SilkHandle::StackAllocated(self.stack.len() - 1)
    }

    pub fn stack_push_variable(&mut self, id: String, v: SilkValue) -> SilkHandle {
        let idx = self.stack.len();
        let handle = SilkHandle::StackAllocated(idx);
        self.scope.variables.insert(id, handle.clone());
        self.stack_push(v);
        handle
    }

    pub fn stack_pop(&mut self) -> SilkValue {
        self.stack.pop().expect("Stack Underflow!")
    }

    pub fn heap_allocate(&mut self, v: SilkValue) -> SilkHandle {
        let ptr = self.next_heap_ptr;
        self.heap.insert(ptr, v);
        self.next_heap_ptr += 1;
        SilkHandle::HeapAllocated(ptr)
    }

    pub fn heap_free(&mut self, h: SilkHandle) {
        match h {
            SilkHandle::HeapAllocated(ptr) => {
                self.heap.remove(&ptr);
            }
            _ => {
                panic!("Attempted to free a non heap allocated value!")
            }
        }
    }

    pub fn heap_get_string(&mut self, v: SilkValue) -> Option<String> {
        let SilkValue::Pointer(pointer) = v else {
            return None;
        };

        let Some(SilkValue::String(str)) = self.get_value_from_pointer(pointer).ok() else {
            return None;
        };

        return Some(str.as_str().to_string());
    }

    pub fn heap_get_list(&mut self, v: SilkValue) -> Option<Vec<SilkValue>> {
        let SilkValue::Pointer(pointer) = v else {
            return None;
        };

        let Some(SilkValue::List(ls)) = self.get_value_from_pointer(pointer).ok() else {
            return None;
        };

        return Some(ls);
    }

    pub fn heap_get_object(&mut self, v: SilkValue) -> Option<HashMap<String, SilkValue>> {
        let SilkValue::Pointer(pointer) = v else {
            return None;
        };

        let Some(SilkValue::Object(map)) = self.get_value_from_pointer(pointer).ok() else {
            return None;
        };

        return Some(map);
    }

    fn get_value_from_pointer(&self, ptr: usize) -> Result<SilkValue, String> {
        self.heap.get(&ptr).cloned().ok_or_else(|| format!("Invalid heap pointer reference: {}", ptr))
    }

    fn get_value_from_handle(&self, handle: &SilkHandle) -> Result<SilkValue, String> {
        match handle {
            SilkHandle::StackAllocated(idx) => Ok(self.stack[*idx].clone()),
            SilkHandle::HeapAllocated(ptr) => self.get_value_from_pointer(*ptr),
            SilkHandle::HeapElement(ptr, idx) => {
                if let Some(SilkValue::List(arr)) = self.heap.get(ptr) {
                    Ok(arr[*idx].clone())
                } else {
                    Err("Target element context is not inside an indexable list".to_string())
                }
            }
            SilkHandle::GlobalValue(id) => self.globals.get(id).cloned().ok_or("Invalid Global ID".to_string()),
            SilkHandle::ObjectField(parent, field) => {
                let parent_value = self.get_value_from_handle(parent)?;
                match parent_value {
                    SilkValue::Object(map) => map.get(field).cloned().ok_or_else(|| format!("Field '{}' not found on object", field)),
                    SilkValue::Pointer(ptr) => match self.get_value_from_pointer(ptr) {
                        Ok(SilkValue::Object(map)) => map.get(field).cloned().ok_or_else(|| format!("Field '{}' not found on object", field)),
                        _ => Err("Cannot access field on a non-object value".to_string()),
                    },
                    _ => Err("Cannot access field on a non-object value".to_string()),
                }
            }
        }
    }

    fn set_value_in_handle(&mut self, handle: &SilkHandle, value: SilkValue) -> Result<(), String> {
        match handle {
            SilkHandle::StackAllocated(idx) => {
                self.stack[*idx] = value;
                Ok(())
            }
            SilkHandle::HeapAllocated(ptr) => {
                self.heap.insert(*ptr, value);
                Ok(())
            }
            SilkHandle::HeapElement(ptr, idx) => {
                if let Some(SilkValue::List(arr)) = self.heap.get_mut(ptr) {
                    arr[*idx] = value;
                    Ok(())
                } else {
                    Err("Target element context is not inside an indexable list".to_string())
                }
            }
            SilkHandle::GlobalValue(id) => {
                self.globals.insert(id.clone(), value);
                Ok(())
            }
            SilkHandle::ObjectField(parent, field) => {
                let parent_value = self.get_value_from_handle(parent)?;
                let new_parent_value = match parent_value {
                    SilkValue::Object(mut map) => {
                        map.insert(field.clone(), value);
                        SilkValue::Object(map)
                    }
                    SilkValue::Pointer(ptr) => {
                        let Some(SilkValue::Object(mut map)) = self.heap.get(&ptr).cloned() else {
                            return Err("Cannot assign to a field on a non-object value".to_string());
                        };
                        map.insert(field.clone(), value);
                        self.heap.insert(ptr, SilkValue::Object(map));
                        return Ok(());
                    }
                    _ => return Err("Cannot assign to a field on a non-object value".to_string()),
                };
                self.set_value_in_handle(parent, new_parent_value)
            }
        }
    }

    fn is_conditional_valid(&self, v: SilkValue) -> Result<bool, String> {
        if v.is_type(&SilkValue::Bool(false)) {
            return Ok(v.is_truthy());
        }
        
        Err("Conditional does not evaluate to boolean".to_string())
    }

    pub fn execute(&mut self, program: Program, import_mode: bool, trace_stack_start: String) -> i32 {
        if !import_mode {
            self.scope = self.scope.child();
            self.trace_stack.push((SilkValue::String(trace_stack_start), 0, 0));
        }

        for stmt in program.statements {
            let err_code = self.evaluate_statement(&stmt);
            if let Some(error_msg) = err_code {
                self.error(error_msg);
                return 1;
            }
        }
        
        if !import_mode {
            let stack_var_count = self.scope.variables.values().filter(|handle| matches!(handle, SilkHandle::StackAllocated(_))).count();
            for _ in 0..stack_var_count {
                self.stack_pop();
            }
            self.scope = self.scope.pop();
            self.clear_garbage();
            self.trace_stack.pop();
        }

        0
    }

    pub fn clear_garbage(&mut self) {
        
        let mut visited: HashSet<usize> = HashSet::new();
        let mut queue: Vec<usize> = Vec::new();

        
        for value in &self.stack {
            if let SilkValue::Pointer(ptr) = value {
                queue.push(*ptr);
            }
        }

        
        for value in self.globals.values() {
            if let SilkValue::Pointer(ptr) = value {
                queue.push(*ptr);
            }
        }

        
        for module in self.modules.values() {
            for value in module.values() {
                if let SilkValue::Pointer(ptr) = value {
                    queue.push(*ptr);
                }
            }
        }

        
        while let Some(ptr) = queue.pop() {
            if visited.insert(ptr) {
                
                if let Some(heap_val) = self.heap.get(&ptr) {
                    match heap_val {
                        
                        SilkValue::List(elements) => {
                            for item in elements {
                                if let SilkValue::Pointer(inner_ptr) = item {
                                    queue.push(*inner_ptr);
                                }
                            }
                        }
                        
                        SilkValue::Object(map) => {
                            for v in map.values() {
                                if let SilkValue::Pointer(inner_ptr) = v {
                                    queue.push(*inner_ptr);
                                }
                            }
                        }
                        
                        
                        _ => {}
                    }
                }
            }
        }

        
        
        let all_keys: Vec<usize> = self.heap.keys().cloned().collect();

        for key in all_keys {
            if key != 0 && !visited.contains(&key) {
                
                self.heap_free(SilkHandle::HeapAllocated(key));
            }
        }
    }

    fn attach_location(&self, err: String, line: u32, column: u32) -> String {
        if err.contains("(line:") {
            err
        } else {
            format!("{} (line: {}, column: {})", err, line, column)
        }
    }

    pub fn evaluate_statement(&mut self, statement: &ProgramStatement) -> Option<String> {
        let res = match statement.node.as_ref() {
            StmtNode::VarDecl(id, initializer) => self.stmt_var_decl(id, initializer),
            StmtNode::FuncDecl(id, args, body) => self.stmt_func_decl(id, args, body),
            StmtNode::If(conditional, truthy, falsey) => self.stmt_if(conditional, truthy, falsey),
            StmtNode::StandaloneExpression(expr) => {
                let result = self.evaluate_expression(expr);
                match result {
                    Ok(_) => Option::None,
                    Err(e) => Some(e)
                }
            },
            StmtNode::Import(module_name, _) => {
                if module_name.ends_with(".silk") {
                    self.trace_stack.push((SilkValue::String("import ".to_owned() + module_name), statement.line, statement.column));
                    match std::fs::read_to_string(module_name) {
                        Ok(src) => {
                            let mut lexer = Lexer::new(&src);
                            let tokens = lexer.tokenize();
                            let mut parser = Parser::new(tokens);
                            let Some(program) = parser.parse() else {
                                return Some(format!("Failed to parse module '{}' (line: {}, column: {})", module_name, statement.line, statement.column));
                            };

                            let exit_code = self.execute(program, true, String::new());
                            if exit_code != SILK_EXIT_OK {
                                return Some(format!("Error occurred while importing silk file '{}' (line: {}, column: {})", module_name, statement.line, statement.column));
                            }
                            self.trace_stack.pop();
                            Option::None
                        }
                        Err(e) => Some(format!("Could not read silk file '{}': {}", module_name, e))
                    }
                } else {
                    
                    if let Some(module_items) = self.modules.get(module_name).cloned() {
                        for (func_name, native_val) in module_items {
                            let handle = self.heap_allocate(native_val);
                            if let SilkHandle::HeapAllocated(ptr) = handle {
                                self.stack_push_variable(func_name, SilkValue::Pointer(ptr));
                            }
                        }
                        Option::None
                    } else {
                        Some(format!("Standard module '{}' could not be resolved", module_name))
                    }
                }
            },
            StmtNode::Global(stmt) => self.evaluate_global_statement(stmt),
            StmtNode::StructDecl(id, data) => self.stmt_struct_decl(id, data),
            StmtNode::For(var, container, body) => self.stmt_for(var, container, body),
            StmtNode::While(conditional, body) => self.stmt_while(conditional, body),
            _ => {Some(format!("Statement evaluation for {:?} has not been implemented", statement.node))}
        };

        if let Some(err) = res {
            return Some(self.attach_location(err, statement.line, statement.column));
        }

        None
    }

    pub fn stmt_for(&mut self, var: &String, container: &ProgramExpression, body: &Vec<ProgramStatement>) -> Option<String> {
        if self.scope.variables.contains_key(var) {
            return Some(format!("Cannot declare for loop variable '{}' because it already exists in the scope!", var));
        }

        let mut v_container = self.evaluate_expression(container).ok()?;
        
        if let SilkValue::Pointer(ptr) = v_container {
            if let Some(v) = self.heap.get(&ptr) {
                v_container = v.clone();
            }
            else {
                return Some("invalid pointer as for loop container".to_string());
            }
        }

        match v_container {
            SilkValue::List(list) => {
                for v in list {
                    let ptr = self.next_heap_ptr;
                    self.scope = self.scope.child();
                    let stack_size = self.stack.len();

                    self.heap_allocate(v);
                    self.stack_push_variable(var.to_string(), Pointer(ptr));

                    for stmt in body {
                        if let Some(error) = self.evaluate_statement(stmt) {
                            return Some(error)
                        }
                    }

                    while self.stack.len() > stack_size {
                        self.stack_pop();
                    }
                    
                    self.scope = self.scope.pop();
                }
            }
            _ => {
                return Some(format!("Cannot iterate through type of {}", v_container));
            }
        }

        None
    }

    pub fn stmt_while(&mut self, conditional: &ProgramExpression, body: &Vec<ProgramStatement>) -> Option<String> {
        while {
            let value = self.evaluate_expression(conditional).ok()?;
            self.is_conditional_valid(value).ok()?
        } {
            self.scope = self.scope.child();
            let stack_size = self.stack.len();

            for stmt in body {
                if let Some(error) = self.evaluate_statement(stmt) {
                    return Some(error)
                }
            }

            while self.stack.len() > stack_size {
                self.stack_pop();
            }
            
            self.scope = self.scope.pop();

        }

        None
    }

    pub fn stmt_var_decl(&mut self, identifier: &String, initializer: &ProgramExpression) -> Option<String> {
        if self.scope.variables.contains_key(identifier) {
            return Some(format!("Cannot declare variable '{}' because it already exists in the scope!", identifier));
        }

        let v = self.evaluate_expression(initializer);
        match v {
            Ok(value) => {
                self.stack_push_variable(identifier.clone(), value);
                Option::None
            }
            Err(e) => Some(e)
        }
    }

    pub fn stmt_func_decl(&mut self, id: &String, args: &Vec<String>, body: &Vec<ProgramStatement>) -> Option<String> {
        if self.scope.variables.contains_key(id) {
            return Some(format!("Cannot declare variable '{}' because it already exists in the scope!", id));
        }

        let v = SilkValue::Function(args.clone(), body.clone());
        let handle = self.heap_allocate(v);
        match handle {
            SilkHandle::HeapAllocated(ptr) => {self.stack_push_variable(id.clone(), SilkValue::Pointer(ptr));},
            _ => unreachable!()
        }
        

        Option::None
    }

    pub fn stmt_if(&mut self, condition: &ProgramExpression, truthy: &Vec<ProgramStatement>, falsey: &Vec<ProgramStatement>) -> Option<String> {
        let result = self.evaluate_expression(condition);
        match result {
            Ok(value) => {
                if value.is_truthy() {
                    self.scope = self.scope.child();
                    for stmt in truthy {
                        if let Some(error) = self.evaluate_statement(stmt) {
                            return Some(error)
                        }
                    }
                    let variables_declared = self.scope.variables.len();
                    for _ in 0..variables_declared {
                        self.stack_pop();
                    }
                    self.scope = self.scope.pop();
                }
                else {
                    self.scope = self.scope.child();
                    for stmt in falsey {
                        if let Some(error) = self.evaluate_statement(stmt) {
                            return Some(error)
                        }
                    }
                    let variables_declared = self.scope.variables.len();
                    for _ in 0..variables_declared {
                        self.stack_pop();
                    }
                    self.scope = self.scope.pop();
                }
                Option::None
            },
            Err(e) => Some(e)
        }

        
    }

    pub fn evaluate_expression_statement(&mut self, expression: ProgramExpression) -> Option<String> {
        let result = self.evaluate_expression(&expression);
        match result {
            Ok(_v) => Option::None,
            Err(e) => Some(e.clone())
        }
    }

    pub fn evaluate_global_statement(&mut self, statement: &ProgramStatement) -> Option<String> {
        match statement.node.as_ref() {
            StmtNode::VarDecl(name, init) => {
                let does_exist = self.scope.retrieve(name);
                if let Some(_global) = does_exist {
                    return Some(format!("identifier '{}' already exists in scope", name));
                }

                let result = self.evaluate_expression(init);
                match result {
                    Ok(initial_val) => {
                        let _global_idx = self.globals.len();
                        self.globals.insert(name.clone(), initial_val);

                        Option::None
                    }
                    Err(e) => Some(e)
                }
            }
            StmtNode::FuncDecl(name, args, body) => {
                if self.scope.retrieve(name).is_some() {
                    return Some(format!("identifier '{}' already exists in scope", name));
                }

                let v = SilkValue::Function(args.clone(), body.clone());
                let handle = self.heap_allocate(v);
                if let SilkHandle::HeapAllocated(ptr) = handle {
                    let _global_idx = self.globals.len();
                    self.globals.insert(name.clone(), SilkValue::Pointer(ptr));

                    Option::None
                } else {
                    unreachable!()
                }
            }
            StmtNode::Import(module_name, _) => {
                if module_name.ends_with(".silk") {
                    self.trace_stack.push((SilkValue::String("import ".to_owned() + module_name), statement.line, statement.column));
                    match std::fs::read_to_string(module_name) {
                        Ok(src) => {
                            let mut lexer = Lexer::new(&src);
                            let tokens = lexer.tokenize();
                            let mut parser = Parser::new(tokens);
                            let Some(program) = parser.parse() else {
                                return Some(format!("Failed to parse module '{}'", module_name));
                            };

                            self.execute(program, true, String::new());
                            self.trace_stack.pop();
                            Option::None
                        }
                        Err(e) => Some(format!("Could not read silk file '{}': {}", module_name, e))
                    }
                } else {
                    
                    if let Some(module_items) = self.modules.get(module_name).cloned() {
                        for (func_name, native_val) in module_items {
                            let handle = self.heap_allocate(native_val);
                            if let SilkHandle::HeapAllocated(ptr) = handle {
                                self.stack_push_variable(func_name, SilkValue::Pointer(ptr));
                            }
                        }
                        Option::None
                    } else {
                        Some(format!("Standard module '{}' could not be resolved", module_name))
                    }
                }
            }
            _ => {Some(format!("Statement {} cannot be evaluated as global", statement.node))}
        }
    }

    pub fn stmt_struct_decl(&mut self, id: &String, data: &Vec<ProgramStatement>) -> Option<String> {
        if self.scope.retrieve(id).is_some() {
            return Some(format!("identifier '{}' already exists in scope", id));
        }
        let SilkHandle::HeapAllocated(ptr) = self.heap_allocate(SilkValue::ObjectDefinition(data.clone())) else {
            unreachable!()
        };

        self.stack_push_variable(id.clone(), SilkValue::Pointer(ptr));

        None
    }

    
    pub fn expr_array_lit(&mut self, arr: &Vec<ProgramExpression>) -> Result<SilkValue, String> {
        let mut v_arr: Vec<SilkValue> = vec![SilkValue::Null; arr.len()];
        for idx in 0..arr.len() {
            let result = self.evaluate_expression(&arr[idx]);
            match result {
                Ok(v) => v_arr[idx] = v,
                Err(e) => {return Err(e.clone());}
            }
        }

        let handle = self.heap_allocate(SilkValue::List(v_arr));
        if let SilkHandle::HeapAllocated(ptr) = handle {
            return Ok(SilkValue::Pointer(ptr))
        }
        else {
            unreachable!()
        }
    }

    pub fn expr_str_lit(&mut self, str: &String) -> Result<SilkValue, String> {
        let handle = self.heap_allocate(SilkValue::String(str.clone()));
        if let SilkHandle::HeapAllocated(ptr) = handle {
            return Ok(SilkValue::Pointer(ptr))
        }
        else {
            unreachable!()
        }
    }

    pub fn expr_var(&mut self, id: &String) -> Result<SilkValue, String> {
        if let Some(handle) = self.scope.retrieve(id) {
            return self.get_value_from_handle(&handle);
        }

        if let Some(v) = self.globals.get(id) {
            return Ok(v.clone())
        }

        Err(format!("Variable '{}' was not found in the scope", id))
    }

    pub fn expr_index_access(&mut self, container: &ProgramExpression, idx: &ProgramExpression) -> Result<SilkValue, String> {
        let v_container = self.evaluate_expression(container)?;
        let v_index = self.evaluate_expression(idx)?;
        
        let v_int = v_index.as_int().ok_or_else(|| "Array index must be an integer".to_string())?;

        match v_container {
            SilkValue::Pointer(ptr) => {
                
                match self.heap.get(&ptr) {
                    Some(SilkValue::List(v_array)) => {
                        if (v_int as usize) < v_array.len() {
                            Ok(v_array[v_int as usize].clone())
                        } else {
                            Err("Array index out of bounds".to_string())
                        }
                    }
                    _ => Err("Target pointer is not an indexable collection".to_string())
                }
            }
            _ => Err("Cannot index into a non-pointer type".to_string())
        }
    }

    pub fn expr_op(&mut self, lhs: &ProgramExpression, rhs: &ProgramExpression, op: &SilkOperator) -> Result<SilkValue, String> {
        let mut l_value = self.evaluate_expression(lhs)?;
        let mut r_value = self.evaluate_expression(rhs)?;

        // Dereference pointer values from the heap if present
        if let SilkValue::Pointer(ptr) = l_value {
            l_value = self.heap.get(&ptr).ok_or("lhs was not found in the heap")?.clone();
        }
        if let SilkValue::Pointer(ptr) = r_value {
            r_value = self.heap.get(&ptr).ok_or("rhs was not found in the heap")?.clone();
        }

        // Helper closure to convert mixed Int/Float pairs into either Int or Float operands
        let coerce_numeric = |l: SilkValue, r: SilkValue| -> Option<EitherNumbers> {
            match (l, r) {
                (SilkValue::Int(a), SilkValue::Int(b)) => Some(EitherNumbers::Int(a, b)),
                (SilkValue::Float(a), SilkValue::Float(b)) => Some(EitherNumbers::Float(a, b)),
                (SilkValue::Int(a), SilkValue::Float(b)) => Some(EitherNumbers::Float(a as f32, b)),
                (SilkValue::Float(a), SilkValue::Int(b)) => Some(EitherNumbers::Float(a, b as f32)),
                _ => None,
            }
        };

        match op {
            SilkOperator::Plus => match coerce_numeric(l_value, r_value) {
                Some(EitherNumbers::Int(a, b)) => Ok(SilkValue::Int(a + b)),
                Some(EitherNumbers::Float(a, b)) => Ok(SilkValue::Float(a + b)),
                None => Err("Type mismatch: Expected numeric types for addition".to_string()),
            },
            SilkOperator::Minus => match coerce_numeric(l_value, r_value) {
                Some(EitherNumbers::Int(a, b)) => Ok(SilkValue::Int(a - b)),
                Some(EitherNumbers::Float(a, b)) => Ok(SilkValue::Float(a + -b)),
                None => Err("Type mismatch: Expected numeric types for subtraction".to_string()),
            },
            SilkOperator::Multiply => match coerce_numeric(l_value, r_value) {
                Some(EitherNumbers::Int(a, b)) => Ok(SilkValue::Int(a * b)),
                Some(EitherNumbers::Float(a, b)) => Ok(SilkValue::Float(a * b)),
                None => Err("Type mismatch: Expected numeric types for multiplication".to_string()),
            },
            SilkOperator::Divide => match coerce_numeric(l_value, r_value) {
                Some(EitherNumbers::Int(a, b)) => {
                    if b == 0 { return Err("Division by zero error".to_string()); }
                    Ok(SilkValue::Int(a / b))
                }
                Some(EitherNumbers::Float(a, b)) => {
                    if b == 0.0 { return Err("Division by zero error".to_string()); }
                    Ok(SilkValue::Float(a / b))
                }
                None => Err("Type mismatch: Expected numeric types for division".to_string()),
            },
            SilkOperator::Mod => match (l_value, r_value) {
                (SilkValue::Int(a), SilkValue::Int(b)) => {
                    if b == 0 { return Err("Modulo by zero error".to_string()); }
                    Ok(SilkValue::Int(a % b))
                }
                _ => Err("Type mismatch: Modulo operations require Integer types".to_string()),
            },
            SilkOperator::NotEqual => {
                Ok(SilkValue::Bool(!l_value.equals(&r_value)))
            }
            SilkOperator::Equality => {
                Ok(SilkValue::Bool(l_value.equals(&r_value)))
            }
            SilkOperator::GreaterThan => match coerce_numeric(l_value, r_value) {
                Some(EitherNumbers::Int(a, b)) => Ok(SilkValue::Bool(a > b)),
                Some(EitherNumbers::Float(a, b)) => Ok(SilkValue::Bool(a > b)),
                None => Err("Cannot apply relative comparison to non-numeric types".to_string()),
            },
            SilkOperator::LesserThan => match coerce_numeric(l_value, r_value) {
                Some(EitherNumbers::Int(a, b)) => Ok(SilkValue::Bool(a < b)),
                Some(EitherNumbers::Float(a, b)) => Ok(SilkValue::Bool(a < b)),
                None => Err("Cannot apply relative comparison to non-numeric types".to_string()),
            },
            SilkOperator::GreaterThanEq => match coerce_numeric(l_value, r_value) {
                Some(EitherNumbers::Int(a, b)) => Ok(SilkValue::Bool(a >= b)),
                Some(EitherNumbers::Float(a, b)) => Ok(SilkValue::Bool(a >= b)),
                None => Err("Cannot apply relative comparison to non-numeric types".to_string()),
            },
            SilkOperator::LesserThanEq => match coerce_numeric(l_value, r_value) {
                Some(EitherNumbers::Int(a, b)) => Ok(SilkValue::Bool(a <= b)),
                Some(EitherNumbers::Float(a, b)) => Ok(SilkValue::Bool(a <= b)),
                None => Err("Cannot apply relative comparison to non-numeric types".to_string()),
            },
            SilkOperator::And => Ok(SilkValue::Bool(l_value.is_truthy() && r_value.is_truthy())),
            SilkOperator::Or => Ok(SilkValue::Bool(l_value.is_truthy() || r_value.is_truthy())),
        }
    }

    pub fn expr_assignment_op(&mut self, lhs: &ProgramExpression, rhs: &ProgramExpression, op: &SilkAssignment) -> Result<SilkValue, String> {
        
        let l_handle = self.evaluate_expression_as_mut(lhs)?;
        let r_value = self.evaluate_expression(rhs)?;

        
        let current_lhs_value = self.get_value_from_handle(&l_handle)?;

        
        let final_value = match op {
            SilkAssignment::Assignment => r_value,
            SilkAssignment::CompoundPlus => match (current_lhs_value, r_value) {
                (SilkValue::Int(a), SilkValue::Int(b)) => SilkValue::Int(a + b),
                (SilkValue::Float(a), SilkValue::Float(b)) => SilkValue::Float(a + b),
                _ => return Err("Invalid types for compound addition assignment".to_string()),
            },
            SilkAssignment::CompoundMinus => match (current_lhs_value, r_value) {
                (SilkValue::Int(a), SilkValue::Int(b)) => SilkValue::Int(a - b),
                (SilkValue::Float(a), SilkValue::Float(b)) => SilkValue::Float(a - b),
                _ => return Err("Invalid types for compound subtraction assignment".to_string()),
            },
            SilkAssignment::CompoundMultiply => match (current_lhs_value, r_value) {
                (SilkValue::Int(a), SilkValue::Int(b)) => SilkValue::Int(a * b),
                (SilkValue::Float(a), SilkValue::Float(b)) => SilkValue::Float(a * b),
                _ => return Err("Invalid types for compound multiplication assignment".to_string()),
            },
            SilkAssignment::CompoundDivide => match (current_lhs_value, r_value) {
                (SilkValue::Int(a), SilkValue::Int(b)) => {
                    if b == 0 { return Err("Division by zero error during compounding".to_string()); }
                    SilkValue::Int(a / b)
                }
                (SilkValue::Float(a), SilkValue::Float(b)) => {
                    if b == 0.0 { return Err("Division by zero error during compounding".to_string()); }
                    SilkValue::Float(a / b)
                }
                _ => return Err("Invalid types for compound division assignment".to_string()),
            },
            SilkAssignment::CompoundMod => match (current_lhs_value, r_value) {
                (SilkValue::Int(a), SilkValue::Int(b)) => {
                    if b == 0 { return Err("Modulo by zero error during compounding".to_string()); }
                    SilkValue::Int(a % b)
                }
                _ => return Err("Invalid types for compound modulo assignment".to_string()),
            },
        };

        
        self.set_value_in_handle(&l_handle, final_value.clone())?;

        Ok(final_value)
    }

    pub fn expr_call(&mut self, function: &ProgramExpression, args: &Vec<ProgramExpression>) -> Result<SilkValue, String> {
        let args_str = args
            .iter()
            .map(|arg| arg.node.as_ref().to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let func_decompile = format!("{}({})", function.node.as_ref(), args_str);
        self.trace_stack.push((SilkValue::String(func_decompile), function.line, function.column));
        self.o_ptr = 0;
        let v_ptr = self.evaluate_expression(function)?;
        
        let receiver = self.o_ptr;
        self.o_ptr = 0;

        let mut v_args = Vec::with_capacity(args.len());
        for arg in args {
            v_args.push(self.evaluate_expression(arg)?);
        }

        if receiver != 0 {
            v_args.insert(0, SilkValue::Pointer(receiver));
        }

        let fn_val = if let SilkValue::Pointer(ptr) = v_ptr { 
            self.heap.get(&ptr).cloned().ok_or_else(|| format!("function reference was not found in the heap"))?
        }
        else {
            v_ptr
        };
        

        match fn_val {
            SilkValue::Function(f_args, body) => {
                if args.len() != f_args.len() {
                    return Err("Mismatched argument size!".to_string());
                }

                self.scope = self.scope.child();

                for (param_name, arg_value) in f_args.iter().zip(v_args) {
                    self.stack_push_variable(param_name.clone(), arg_value);
                }

                if receiver != 0 {
                    let receiver_value = self.heap.get(&receiver).cloned()
                        .ok_or_else(|| format!("receiver reference was not found in the heap"))?;
                    if let SilkValue::Object(map) = receiver_value {
                        let receiver_handle = SilkHandle::HeapAllocated(receiver);
                        for (name, _) in map {
                            self.scope.variables.insert(
                                name.clone(),
                                SilkHandle::ObjectField(Box::new(receiver_handle.clone()), name.clone()),
                            );
                        }
                    }
                }

                let return_val = SilkValue::Null;
                for stmt in body {
                    if let Some(error) = self.evaluate_statement(&stmt) {
                        return Err(error)
                    }
                }

                let stack_var_count = self.scope.variables.values().filter(|handle| matches!(handle, SilkHandle::StackAllocated(_))).count();
                for _ in 0..stack_var_count {
                    self.stack_pop();
                }
                self.scope = self.scope.pop();
                self.trace_stack.pop();
                Ok(return_val)
            }
            SilkValue::NativeFn(native, _) => {
                let ret_val = Ok(native(self, &v_args));
                self.trace_stack.pop();
                ret_val
            }
            SilkValue::ObjectDefinition(def) => {
                self.scope = self.scope.child();

                for stmt in &def {
                    if let Some(error) = self.evaluate_statement(stmt) {
                            return Err(error)
                        }
                }

                if let ExprNode::Var(id) = function.node.as_ref() {
                    if self.scope.variables.contains_key(id) {
                        let constructor_callee = ProgramExpression::new(ExprNode::Var(id.clone()), 0, 0);
                        let constructor_call = ExprNode::FuncCall(constructor_callee, args.clone());
                        let _ = self.evaluate_expression(&ProgramExpression::new(constructor_call, 0, 0));
                    }
                }
                else {
                    return Err("Object definition must be called with a variable identifier".to_string());
                }

                let stack_var_count = self.scope.variables.values().filter(|handle| matches!(handle, SilkHandle::StackAllocated(_))).count();
                let mut struct_map = HashMap::new();
                for (name, handle) in &self.scope.variables {
                    if let SilkHandle::StackAllocated(idx) = handle {
                        if let Some(value) = self.stack.get(*idx) {
                            struct_map.insert(name.clone(), value.clone());
                        }
                    }
                }

                for _idx in 0..stack_var_count {
                    self.stack_pop();
                }
                
                self.scope = self.scope.pop();
                let handle = self.heap_allocate(SilkValue::Object(struct_map.clone()));
                if let SilkHandle::HeapAllocated(ptr) = handle {
                    self.trace_stack.pop();
                    return Ok(SilkValue::Pointer(ptr));
                }
                else {
                    unreachable!();
                }

    
            }
            _ => Err(format!("Cannot call on a non-function value! ({})", fn_val))
        }
    }

    pub fn expr_dot(&mut self, object: &ProgramExpression, accessee: &ProgramExpression) -> Result<SilkValue, String> {
        let o_object = self.evaluate_expression(object)?;

        let (_ptr, v_object) = match o_object {
            SilkValue::Pointer(ptr) => {
                self.o_ptr = ptr;
                let v_object = self.heap.get(&ptr).cloned().ok_or_else(|| "Object reference was not found in the heap".to_string())?;
                (Some(ptr), v_object)
            }
            value => {
                self.o_ptr = 0;
                (None, value)
            }
        };

        match v_object {
            SilkValue::String(_) => {
                let string_lib = self.modules.get("string").unwrap().clone();
                self.scope = self.scope.child();
                for (id, v) in string_lib {
                    let ptr = self.heap_allocate(v);
                    match ptr {
                        SilkHandle::HeapAllocated(p) => { self.stack_push_variable(id, SilkValue::Pointer(p)); }
                        _ => unreachable!()
                    }
                }

                let result = self.evaluate_expression(accessee);

                let variables_created = self.scope.variables.len();
                for _ in 0..variables_created {
                    self.stack_pop();
                }
                self.scope = self.scope.pop();
                result
            },
            SilkValue::List(_) => {
                let list_lib = self.modules.get("list").unwrap().clone();
                self.scope = self.scope.child();
                for (id, v) in list_lib {
                    let ptr = self.heap_allocate(v);
                    match ptr {
                        SilkHandle::HeapAllocated(p) => { self.stack_push_variable(id, SilkValue::Pointer(p)); }
                        _ => unreachable!()
                    }
                }

                let result = self.evaluate_expression(accessee);

                let variables_created = self.scope.variables.len();
                for _ in 0..variables_created {
                    self.stack_pop();
                }
                self.scope = self.scope.pop();
                result
            },
            SilkValue::Object(map) => {
                let field_name = match accessee.node.as_ref() {
                    ExprNode::Var(id) => id,
                    _ => return Err("Struct field access requires a field name".to_string()),
                };

                map.get(field_name).cloned().ok_or_else(|| format!("Field '{}' not found on object", field_name))
            },
            _ => Err(format!("Dot access cannot be implemented for object type: {}", v_object))
        }
    }

    pub fn evaluate_expression(&mut self, expression: &ProgramExpression) -> Result<SilkValue, String> {
        match expression.node.as_ref() {
            ExprNode::IntLiteral(num) => Ok(SilkValue::Int(*num)),
            ExprNode::FloatLiteral(num) => Ok(SilkValue::Float(*num)),
            ExprNode::BoolLiteral(truthy) => Ok(SilkValue::Bool(*truthy)),
            ExprNode::NullLiteral => Ok(SilkValue::Null),
            ExprNode::ArrayLiteral(arr) => self.expr_array_lit(arr).map_err(|e| self.attach_location(e, expression.line, expression.column)),
            ExprNode::StringLiteral(str) => self.expr_str_lit(str).map_err(|e| self.attach_location(e, expression.line, expression.column)),
            ExprNode::Var(id) => self.expr_var(id).map_err(|e| self.attach_location(e, expression.line, expression.column)),
            ExprNode::IndexAccess(container, idx) => self.expr_index_access(container, idx).map_err(|e| self.attach_location(e, expression.line, expression.column)),
            ExprNode::Op(lhs, rhs, op) => self.expr_op(lhs, rhs, op).map_err(|e| self.attach_location(e, expression.line, expression.column)),
            ExprNode::AssignmentOp(lhs, rhs, op) => self.expr_assignment_op(lhs, rhs, op).map_err(|e| self.attach_location(e, expression.line, expression.column)),
            ExprNode::FuncCall(func, args) => self.expr_call(func, args).map_err(|e| self.attach_location(e, expression.line, expression.column)),
            ExprNode::Unary(expr) => {
                let result = self.evaluate_expression(expr);
                match result {
                    Ok(v) => match v {
                        SilkValue::Bool(b) => Ok(SilkValue::Bool(!b)),
                        SilkValue::Float(num) => Ok(SilkValue::Float(-num)),
                        SilkValue::Int(num) => Ok(SilkValue::Int(-num)),
                        _ => Err(self.attach_location(format!("Unary operation is unavailble for expression {}", expr.node.as_ref()), expression.line, expression.column)),
                    },
                    Err(e) => Err(self.attach_location(e, expression.line, expression.column)),
                }
            }
            ExprNode::DotAccess(c, accessee) => self.expr_dot(c, accessee).map_err(|e| self.attach_location(e, expression.line, expression.column)),
            _ => Err(self.attach_location(format!("Expression evaluation for {} has not been implemented", expression.node.as_ref()), expression.line, expression.column)),
        }
    }


    
    pub fn evaluate_expression_as_mut(&mut self, expression: &ProgramExpression) -> Result<SilkHandle, String> {
        match expression.node.as_ref() {
            ExprNode::Var(id) => self.expr_var_as_mut(id),
            ExprNode::IndexAccess(container, idx) => self.expr_index_access_as_mut(container, idx),
            ExprNode::DotAccess(object, accessee) => self.expr_dot_as_mut(object, accessee),
            _ => Err("Cannot evaluate an expression of this type as mutable".to_string())
        }
    }

    pub fn expr_var_as_mut(&mut self, id: &String) -> Result<SilkHandle, String> {
        if let Some(handle) = self.scope.retrieve(id) {
            return Ok(handle);
        }

        if self.globals.contains_key(id) {
            return Ok(SilkHandle::GlobalValue(id.clone()));
        }

        Err(format!("Variable '{}' was not found in the scope", id))
    }

    pub fn expr_index_access_as_mut(&mut self, container: &ProgramExpression, idx: &ProgramExpression) -> Result<SilkHandle, String> {
        let v_container = self.evaluate_expression(container)?;
        let v_index = self.evaluate_expression(idx)?;
        
        let v_int = v_index.as_int().ok_or_else(|| "Array index must be an integer".to_string())?;
        
        match v_container {
            SilkValue::Pointer(ptr) => {
                
                match self.heap.get(&ptr) {
                    Some(SilkValue::List(v_array)) => {
                        if (v_int as usize) < v_array.len() {
                            Ok(SilkHandle::HeapElement(ptr, v_int as usize))
                        } else {
                            Err("Array index out of bounds".to_string())
                        }
                    }
                    _ => Err("Target is not mutable or indexable".to_string())
                }
            }
            _ => Err("Cannot evaluate expression as mutable".to_string())
        }
    }

    pub fn expr_dot_as_mut(&mut self, object: &ProgramExpression, accessee: &ProgramExpression) -> Result<SilkHandle, String> {
        let parent_handle = self.evaluate_expression_as_mut(object)?;
        match accessee.node.as_ref() {
            ExprNode::Var(field_name) => Ok(SilkHandle::ObjectField(Box::new(parent_handle), field_name.clone())),
            _ => Err("Struct field access as mutable requires a field name".to_string()),
        }
    }

    pub fn expr_dot_access_as_mut(&mut self, object: &ProgramExpression, accessee: &ProgramExpression) -> Result<SilkHandle, String> {
        self.expr_dot_as_mut(object, accessee)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receiver_member_variables_can_be_mutated_via_mut_handle() {
        let mut vm = VirtualMachine::new();
        let object_ptr = match vm.heap_allocate(SilkValue::Object(HashMap::from([("bar".to_string(), SilkValue::Int(10))]))) {
            SilkHandle::HeapAllocated(ptr) => ptr,
            _ => unreachable!(),
        };

        vm.scope.variables.insert(
            "bar".to_string(),
            SilkHandle::ObjectField(Box::new(SilkHandle::HeapAllocated(object_ptr)), "bar".to_string()),
        );

        let handle = vm.expr_var_as_mut(&"bar".to_string()).unwrap();
        vm.set_value_in_handle(&handle, SilkValue::Int(20)).unwrap();

        let value = vm.get_value_from_handle(&handle).unwrap();
        assert!(matches!(value, SilkValue::Int(20)));

        let Some(SilkValue::Object(map)) = vm.heap.get(&object_ptr).cloned() else {
            std::panic!("expected heap object to be preserved");
        };
        assert!(matches!(map.get("bar"), Some(SilkValue::Int(20))));
    }

    #[test]
    fn extracts_value_from_pointer() {
        let mut vm = VirtualMachine::new();
        let value = SilkValue::Int(42);
        let handle = vm.heap_allocate(value.clone());
        let ptr = match handle {
            SilkHandle::HeapAllocated(ptr) => ptr,
            _ => unreachable!(),
        };

        assert!(matches!(vm.get_value_from_pointer(ptr).unwrap(), SilkValue::Int(42)));
    }
}
