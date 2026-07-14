
use core::panic;
use std::{collections::{HashMap, HashSet}, env::args};
use crate::{environment::scope::Scope, parser::ast::{Program, expr::{ ExprNode, SilkAssignment, SilkOperator}, stmt::StmtNode}};
use crate::lexer::Lexer;
use crate::parser::Parser;
use colored_text::Colorize;

use super::value::SilkValue;

type SilkType = usize;

const SILK_EXIT_OK: i32 = 0;
const SILK_EXIT_ERROR: i32 = 1;

pub enum SilkHandle {
    HeapAllocated(usize),
    StackAllocated(usize),
    HeapElement(usize, usize),
    GlobalValue(String),
}

pub struct VirtualMachine {
    
    pub heap: HashMap<usize, SilkValue>,
    
    next_heap_ptr: usize,
    
    stack: Vec<SilkValue>,
    
    pub globals: HashMap<String, SilkValue>,
    
    scope: Scope,
    
    pub modules: HashMap<String, HashMap<String, SilkValue>>,
    
    o_ptr: usize,
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
        }
    }

    pub fn stringify_value(&self, value: &SilkValue) -> String {
        match value {
            SilkValue::Pointer(ptr) => {
                if let Some(actual_val) = self.heap.get(ptr) {
                    
                    
                    match actual_val {
                        SilkValue::String(inner_str) => inner_str.clone(),
                        _ => self.stringify_value(actual_val),
                    }
                } else {
                    "null".to_string()
                }
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
        self.scope.variables.insert(id, idx);
        self.stack_push(v);
        SilkHandle::StackAllocated(idx)
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

        let Some(SilkValue::String(str)) = self.heap.get(&pointer) else {
            return None;
        };

        return Some(str.clone());
    }

    pub fn heap_get_list(&mut self, v: SilkValue) -> Option<Vec<SilkValue>> {
        let SilkValue::Pointer(pointer) = v else {
            return None;
        };

        let Some(SilkValue::List(ls)) = self.heap.get(&pointer) else {
            return None;
        };

        return Some(ls.clone());
    }

    pub fn execute(&mut self, program: Program, import_mode: bool) -> i32 {
        if !import_mode {
            self.scope = self.scope.child();
        }

        for stmt in program.statements {
            let err_code = self.evaluate_statement(&stmt);
            if let Some(error_msg) = err_code {
                println!("{}: {}", "[Silk Runtime Error]".red(), error_msg.yellow());
                return 1;
            }
        }
        
        if !import_mode {
            let variables_declared = self.scope.variables.len();
            for i in 0..variables_declared {
                self.stack_pop();
            }
            self.scope = self.scope.pop();
            self.clear_garbage();
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

    pub fn evaluate_statement(&mut self, statement: &StmtNode) -> Option<String> {

        match statement {
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
                    
                    match std::fs::read_to_string(module_name) {
                        Ok(src) => {
                            let mut lexer = Lexer::new(&src);
                            let tokens = lexer.tokenize();
                            let mut parser = Parser::new(tokens);
                            let program = parser.parse();

                            
                            let exit_code = self.execute(program, true);
                            if exit_code != SILK_EXIT_OK {
                                return Some(format!("Error occurred while importing silk file '{}'", module_name));
                            }
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
            _ => {Some(format!("Statement evaluation for {} has not been implemented", statement))}
        }
    }

    pub fn stmt_var_decl(&mut self, identifier: &String, initializer: &ExprNode) -> Option<String> {
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

    pub fn stmt_func_decl(&mut self, id: &String, args: &Vec<String>, body: &Vec<StmtNode>) -> Option<String> {
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

    pub fn stmt_if(&mut self, condition: &ExprNode, truthy: &Vec<StmtNode>, falsey: &Vec<StmtNode>) -> Option<String> {
        let result = self.evaluate_expression(condition);
        match result {
            Ok(value) => {
                if value.is_truthy() {
                    self.scope = self.scope.child();
                    for stmt in truthy {
                        self.evaluate_statement(stmt);
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
                        self.evaluate_statement(stmt);
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

    pub fn evaluate_expression_statement(&mut self, expression: ExprNode) -> Option<String> {
        let result = self.evaluate_expression(&expression);
        match result {
            Ok(v) => Option::None,
            Err(e) => Some(e.clone())
        }
    }

    pub fn evaluate_global_statement(&mut self, statement: &Box<StmtNode>) -> Option<String> {
        match statement.as_ref() {
            StmtNode::VarDecl(name, init) => {
                let does_exist = self.scope.retrieve(name);
                if let Some(global) = does_exist {
                    return Some(format!("identifier '{}' already exists in scope", name));
                }

                let result = self.evaluate_expression(init);
                match result {
                    Ok(initial_val) => {
                        let global_idx = self.globals.len();
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
                    let global_idx = self.globals.len();
                    self.globals.insert(name.clone(), SilkValue::Pointer(ptr));

                    Option::None
                } else {
                    unreachable!()
                }
            }
            StmtNode::Import(module_name, _) => {
                if module_name.ends_with(".silk") {
                    
                    match std::fs::read_to_string(module_name) {
                        Ok(src) => {
                            let mut lexer = Lexer::new(&src);
                            let tokens = lexer.tokenize();
                            let mut parser = Parser::new(tokens);
                            let program = parser.parse();

                            
                            self.execute(program, true);
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
            _ => {Some(format!("Statement {} cannot be evaluated as global", statement))}
        }
    }

    pub fn stmt_struct_decl(&mut self, id: &String, data: &Vec<StmtNode>) -> Option<String> {
        if self.scope.retrieve(id).is_some() {
            return Some(format!("identifier '{}' already exists in scope", id));
        }
        let SilkHandle::HeapAllocated(ptr) = self.heap_allocate(SilkValue::ObjectDefinition(data.clone())) else {
            unreachable!()
        };

        self.stack_push_variable(id.clone(), SilkValue::Pointer(ptr));

        None
    }

    pub fn evaluate_expression(&mut self, expression: &ExprNode) -> Result<SilkValue, String> {
        
        match expression {
            ExprNode::IntLiteral(num) => Ok(SilkValue::Int(*num)),
            ExprNode::FloatLiteral(num) => Ok(SilkValue::Float(*num)),
            ExprNode::BoolLiteral(truthy) => Ok(SilkValue::Bool(*truthy)),
            ExprNode::NullLiteral => Ok(SilkValue::Null),
            ExprNode::ArrayLiteral(arr) => self.expr_array_lit(arr),
            ExprNode::StringLiteral(str) => self.expr_str_lit(str),
            ExprNode::Var(id) => self.expr_var(id),
            ExprNode::IndexAccess(container, idx) => self.expr_index_access(container, idx),
            ExprNode::Op(lhs, rhs, op) => self.expr_op(lhs, rhs, op),
            ExprNode::AssignmentOp(lhs, rhs, op) => self.expr_assignment_op(lhs, rhs, op),
            ExprNode::FuncCall(func, args) => self.expr_call(func, args),
            ExprNode::Unary(expr) => {
                let result = self.evaluate_expression(expr);
                match result {
                    Ok(v) => {
                        match v {
                            SilkValue::Bool(b) => Ok(SilkValue::Bool(!b)),
                            SilkValue::Float(num) => Ok(SilkValue::Float(-num)),
                            SilkValue::Int(num) => Ok(SilkValue::Int(-num)),
                            _ => Err(format!("Unary operation is unavailble for expression {}", expr)),
                        }
                    }
                    Err(e) => Err(e)
                }
            }
            ExprNode::DotAccess(c, accessee) => self.expr_dot(c, accessee),
            _ => {Err(format!("Expression evaluation for {} has not been implemented", expression))}
        }
    }

    pub fn expr_array_lit(&mut self, arr: &Vec<ExprNode>) -> Result<SilkValue, String> {
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
        let result = &self.scope.retrieve(id);
        if let Some(idx) = result.clone() {
            let v = &self.stack[idx];
            return Ok(v.clone())
        }
        else {
            if let Some(v) = self.globals.get(id) {
                return Ok(v.clone())
            }
            Result::Err(format!("Variable '{}' was not found in the scope", id))
        }
        
    }

    pub fn expr_index_access(&mut self, container: &Box<ExprNode>, idx: &Box<ExprNode>) -> Result<SilkValue, String> {
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

    pub fn expr_op(&mut self, lhs: &Box<ExprNode>, rhs: &Box<ExprNode>, op: &SilkOperator) -> Result<SilkValue, String> {
        
        let l_value = self.evaluate_expression(lhs)?;
        let r_value = self.evaluate_expression(rhs)?;

        match op {
            SilkOperator::Plus => match (l_value, r_value) {
                (SilkValue::Int(a), SilkValue::Int(b)) => Ok(SilkValue::Int(a + b)),
                (SilkValue::Float(a), SilkValue::Float(b)) => Ok(SilkValue::Float(a + b)),
                _ => Err("Type mismatch: Expected numeric types for addition".to_string()),
            },
            SilkOperator::Minus => match (l_value, r_value) {
                (SilkValue::Int(a), SilkValue::Int(b)) => Ok(SilkValue::Int(a - b)),
                (SilkValue::Float(a), SilkValue::Float(b)) => Ok(SilkValue::Float(a - b)),
                _ => Err("Type mismatch: Expected numeric types for subtraction".to_string()),
            },
            SilkOperator::Multiply => match (l_value, r_value) {
                (SilkValue::Int(a), SilkValue::Int(b)) => Ok(SilkValue::Int(a * b)),
                (SilkValue::Float(a), SilkValue::Float(b)) => Ok(SilkValue::Float(a * b)),
                _ => Err("Type mismatch: Expected numeric types for multiplication".to_string()),
            },
            SilkOperator::Divide => match (l_value, r_value) {
                (SilkValue::Int(a), SilkValue::Int(b)) => {
                    if b == 0 { return Err("Division by zero error".to_string()); }
                    Ok(SilkValue::Int(a / b))
                }
                (SilkValue::Float(a), SilkValue::Float(b)) => {
                    if b == 0.0 { return Err("Division by zero error".to_string()); }
                    Ok(SilkValue::Float(a / b))
                }
                _ => Err("Type mismatch: Expected numeric types for division".to_string()),
            },
            SilkOperator::Mod => match (l_value, r_value) {
                (SilkValue::Int(a), SilkValue::Int(b)) => {
                    if b == 0 { return Err("Modulo by zero error".to_string()); }
                    Ok(SilkValue::Int(a % b))
                }
                _ => Err("Type mismatch: Modulo operations require Integer types".to_string()),
            },
            SilkOperator::Equality => {
                
                Ok(SilkValue::Bool(l_value.equals(&r_value)))
            }
            SilkOperator::GreaterThan => match (l_value, r_value) {
                (SilkValue::Int(a), SilkValue::Int(b)) => Ok(SilkValue::Bool(a > b)),
                (SilkValue::Float(a), SilkValue::Float(b)) => Ok(SilkValue::Bool(a > b)),
                _ => Err("Cannot apply relative comparison to non-numeric types".to_string())
            },
            SilkOperator::LesserThan => match (l_value, r_value) {
                (SilkValue::Int(a), SilkValue::Int(b)) => Ok(SilkValue::Bool(a < b)),
                (SilkValue::Float(a), SilkValue::Float(b)) => Ok(SilkValue::Bool(a < b)),
                _ => Err("Cannot apply relative comparison to non-numeric types".to_string())
            },
            SilkOperator::GreaterThanEq => match (l_value, r_value) {
                (SilkValue::Int(a), SilkValue::Int(b)) => Ok(SilkValue::Bool(a >= b)),
                (SilkValue::Float(a), SilkValue::Float(b)) => Ok(SilkValue::Bool(a >= b)),
                _ => Err("Cannot apply relative comparison to non-numeric types".to_string())
            },
            SilkOperator::LesserThanEq => match (l_value, r_value) {
                (SilkValue::Int(a), SilkValue::Int(b)) => Ok(SilkValue::Bool(a <= b)),
                (SilkValue::Float(a), SilkValue::Float(b)) => Ok(SilkValue::Bool(a <= b)),
                _ => Err("Cannot apply relative comparison to non-numeric types".to_string())
            },
            SilkOperator::And => Ok(SilkValue::Bool(l_value.is_truthy() && r_value.is_truthy())),
            SilkOperator::Or => Ok(SilkValue::Bool(l_value.is_truthy() || r_value.is_truthy())),
        }
    }

    pub fn expr_assignment_op(&mut self, lhs: &Box<ExprNode>, rhs: &Box<ExprNode>, op: &SilkAssignment) -> Result<SilkValue, String> {
        
        let l_handle = self.evaluate_expression_as_mut(lhs)?;
        let r_value = self.evaluate_expression(rhs)?;

        
        let current_lhs_value = match &l_handle {
            SilkHandle::StackAllocated(idx) => self.stack[*idx].clone(),
            SilkHandle::HeapAllocated(ptr) => self.heap.get(ptr).cloned().ok_or("Invalid heap pointer reference")?,
            SilkHandle::HeapElement(ptr, idx) => {
                if let Some(SilkValue::List(arr)) = self.heap.get(ptr) {
                    arr[*idx].clone()
                } else {
                    return Err("Target element context is not inside an indexable list".to_string());
                }
            }
            SilkHandle::GlobalValue(id) => self.globals.get(id).cloned().ok_or("Invalid Global ID")?,
        };

        
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

        
        match l_handle {
            SilkHandle::StackAllocated(idx) => {
                self.stack[idx] = final_value.clone();
            }
            SilkHandle::HeapAllocated(ptr) => {
                self.heap.insert(ptr, final_value.clone());
            }
            SilkHandle::HeapElement(ptr, idx) => {
                if let Some(SilkValue::List(arr)) = self.heap.get_mut(&ptr) {
                    arr[idx] = final_value.clone();
                }
            }
            SilkHandle::GlobalValue(id) => {
                self.globals.insert(id, final_value.clone());
            }
        }

        
        Ok(final_value)
    }

    pub fn expr_call(&mut self, function: &Box<ExprNode>, args: &Vec<ExprNode>) -> Result<SilkValue, String> {
        
        let v_ptr = self.evaluate_expression(function)?;
        
        let mut v_args = Vec::with_capacity(args.len());
        for arg in args {
            v_args.push(self.evaluate_expression(arg)?);
        }

        if self.o_ptr != 0 {
            v_args.insert(0, SilkValue::Pointer(self.o_ptr));
            self.o_ptr = 0;
        }

        let SilkValue::Pointer(ptr) = v_ptr else {
            return Err("Cannot call a non heap allocated type".to_string());
        };

        let ptr_val = self.heap.get(&ptr).cloned()
            .ok_or_else(|| format!("function reference was not found in the heap"))?;

        match ptr_val {
            SilkValue::Function(f_args, body) => {
                if args.len() != f_args.len() {
                    return Err("Mismatched argument size!".to_string());
                }

                self.scope = self.scope.child();

                for (param_name, arg_value) in f_args.iter().zip(v_args) {
                    self.stack_push(arg_value);
                    self.scope.variables.insert(param_name.clone(), self.stack.len() - 1); 
                }

                let mut return_val = SilkValue::Null;
                for stmt in body {
                    self.evaluate_statement(&stmt); 
                }

                let variables_declared = self.scope.variables.len();
                for _ in 0..variables_declared {
                    self.stack_pop();
                }
                self.scope = self.scope.pop();

                Ok(return_val)
            }
            SilkValue::NativeFn(native) => {
                Ok(native(self, &v_args))
            }
            SilkValue::ObjectDefinition(def) => {
                self.scope = self.scope.child();

                for stmt in &def {
                    self.evaluate_statement(stmt);
                }

                let variables_declared = self.scope.variables.len();
                let mut struct_map = HashMap::new();
                for (name, ptr) in &self.scope.variables {
                    struct_map.insert(name.clone(), self.stack.get(*ptr).expect("stack underflow").clone());
                }

                for idx in 0..variables_declared {
                    self.stack_pop();
                }
                
                self.scope = self.scope.pop();
                return Ok(SilkValue::Object(struct_map));
            }
            _ => Err(format!("Cannot call on a non-function value! ({})", ptr_val))
        }
    }

    pub fn expr_dot(&mut self, object: &Box<ExprNode>, accessee: &Box<ExprNode>) -> Result<SilkValue, String> {
        
        let o_object = self.evaluate_expression(object)?;
        let SilkValue::Pointer(ptr) = o_object else {
            return Err(String::from("Only heap allocated values can have accessables"));
        };
        
        
        let v_object = self.heap.get(&ptr).unwrap();
        self.o_ptr = ptr;
        match v_object {
            SilkValue::String(str) => {
                let string_lib = self.modules.get("string").unwrap().clone();
                self.scope = self.scope.child();
                for (id, v) in string_lib {
                    let ptr = self.heap_allocate(v);
                    match ptr {
                        SilkHandle::HeapAllocated(p) => { self.stack_push_variable(id, SilkValue::Pointer(p)); }
                        _ => unreachable!()
                    }
                }

                let result = self.evaluate_expression(&accessee);
                
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

                let result = self.evaluate_expression(&accessee);
                
                let variables_created = self.scope.variables.len();
                for _ in 0..variables_created {
                    self.stack_pop();
                }
                self.scope = self.scope.pop();
                result
            },
            _ => Err(format!("Dot access cannot be implemented for object type: {}", v_object))
        }
    }

    pub fn evaluate_expression_as_mut(&mut self, expression: &ExprNode) -> Result<SilkHandle, String> {
        match expression {
            ExprNode::Var(id) => self.expr_var_as_mut(id),
            ExprNode::IndexAccess(container, idx) => self.expr_index_access_as_mut(container, idx),
            _ => Err("Cannot evaluate an expression of this type as mutable".to_string())
        }
    }

    pub fn expr_var_as_mut(&mut self, id: &String) -> Result<SilkHandle, String> {
        let result = &self.scope.retrieve(id);
        if let Some(idx) = result.clone() {
            return Ok(SilkHandle::StackAllocated(idx))
        }
        else {
            let is_global = &self.globals.get(id);
            if let Some(v) = is_global {
                return Ok(SilkHandle::GlobalValue(id.clone()));
            }
            Result::Err(format!("Variable '{}' was not found in the scope", id))
        }
    }

    pub fn expr_index_access_as_mut(&mut self, container: &Box<ExprNode>, idx: &Box<ExprNode>) -> Result<SilkHandle, String> {
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

}
