//! # 计算器工具
//!
//! 数学表达式求值工具，使用递归下降解析器实现。
//! 支持基本运算（+, -, *, /, %, ^）、括号和常用数学函数。

use crate::{Tool, ToolError, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::debug;

// ============================================================
// 词法分析器
// ============================================================

/// 词法单元类型
#[derive(Clone, Debug, PartialEq)]
enum Token {
    /// 数字字面量
    Number(f64),
    /// 加号
    Plus,
    /// 减号
    Minus,
    /// 乘号
    Star,
    /// 除号
    Slash,
    /// 取模
    Percent,
    /// 幂运算
    Caret,
    /// 左括号
    LParen,
    /// 右括号
    RParen,
    /// 逗号（函数参数分隔）
    Comma,
    /// 标识符（函数名）
    Ident(String),
}

/// 词法分析器 — 将表达式字符串分割为词法单元序列
struct Lexer;

impl Lexer {
    /// 对输入表达式进行词法分析
    fn tokenize(input: &str) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            match chars[i] {
                // 跳过空白字符
                c if c.is_whitespace() => {
                    i += 1;
                }
                // 数字（包括小数点）
                c if c.is_ascii_digit() || c == '.' => {
                    let start = i;
                    let mut has_dot = c == '.';

                    i += 1;
                    while i < chars.len()
                        && (chars[i].is_ascii_digit() || (chars[i] == '.' && !has_dot))
                    {
                        if chars[i] == '.' {
                            has_dot = true;
                        }
                        i += 1;
                    }

                    let num_str: String = chars[start..i].iter().collect();
                    let num = num_str
                        .parse::<f64>()
                        .map_err(|_| format!("无效的数字: {}", num_str))?;
                    tokens.push(Token::Number(num));
                }
                // 标识符（函数名）
                c if c.is_ascii_alphabetic() || c == '_' => {
                    let start = i;
                    i += 1;
                    while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    let ident: String = chars[start..i].iter().collect();
                    tokens.push(Token::Ident(ident));
                }
                '+' => {
                    tokens.push(Token::Plus);
                    i += 1;
                }
                '-' => {
                    tokens.push(Token::Minus);
                    i += 1;
                }
                '*' => {
                    tokens.push(Token::Star);
                    i += 1;
                }
                '/' => {
                    tokens.push(Token::Slash);
                    i += 1;
                }
                '%' => {
                    tokens.push(Token::Percent);
                    i += 1;
                }
                '^' => {
                    tokens.push(Token::Caret);
                    i += 1;
                }
                '(' => {
                    tokens.push(Token::LParen);
                    i += 1;
                }
                ')' => {
                    tokens.push(Token::RParen);
                    i += 1;
                }
                ',' => {
                    tokens.push(Token::Comma);
                    i += 1;
                }
                other => {
                    return Err(format!("未知字符: '{}'", other));
                }
            }
        }

        Ok(tokens)
    }
}

// ============================================================
// 递归下降解析器
// ============================================================

/// 解析器 — 递归下降法求值数学表达式
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// 查看当前词法单元（不消费）
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    /// 消费当前词法单元并前进
    fn advance(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }

    /// 解析表达式（最低优先级：加减法）
    fn parse_expression(&mut self) -> Result<f64, String> {
        let mut left = self.parse_term()?;

        while let Some(token) = self.peek() {
            match token {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_term()?;
                    left += right;
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_term()?;
                    left -= right;
                }
                _ => break,
            }
        }

        Ok(left)
    }

    /// 解析项（乘除法和取模）
    fn parse_term(&mut self) -> Result<f64, String> {
        let mut left = self.parse_power()?;

        while let Some(token) = self.peek() {
            match token {
                Token::Star => {
                    self.advance();
                    let right = self.parse_power()?;
                    left *= right;
                }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_power()?;
                    if right == 0.0 {
                        return Err("除以零错误".to_string());
                    }
                    left /= right;
                }
                Token::Percent => {
                    self.advance();
                    let right = self.parse_power()?;
                    if right == 0.0 {
                        return Err("除以零错误".to_string());
                    }
                    left %= right;
                }
                _ => break,
            }
        }

        Ok(left)
    }

    /// 解析幂运算（右结合）
    fn parse_power(&mut self) -> Result<f64, String> {
        let base = self.parse_unary()?;

        if let Some(Token::Caret) = self.peek() {
            self.advance();
            let exp = self.parse_power()?;
            Ok(base.powf(exp))
        } else {
            Ok(base)
        }
    }

    /// 解析一元运算符（正负号）
    fn parse_unary(&mut self) -> Result<f64, String> {
        match self.peek() {
            Some(Token::Minus) => {
                self.advance();
                let val = self.parse_primary()?;
                Ok(-val)
            }
            Some(Token::Plus) => {
                self.advance();
                self.parse_primary()
            }
            _ => self.parse_primary(),
        }
    }

    /// 解析基本单元（数字、括号、函数调用）
    fn parse_primary(&mut self) -> Result<f64, String> {
        match self.advance() {
            Some(Token::Number(n)) => Ok(n),
            Some(Token::LParen) => {
                let val = self.parse_expression()?;
                match self.advance() {
                    Some(Token::RParen) => Ok(val),
                    _ => Err("缺少右括号".to_string()),
                }
            }
            Some(Token::Ident(name)) => {
                // 检查是否为常量
                match name.to_lowercase().as_str() {
                    "pi" => return Ok(std::f64::consts::PI),
                    "e" => return Ok(std::f64::consts::E),
                    _ => {}
                }

                // 函数调用
                match self.peek() {
                    Some(Token::LParen) => {
                        self.advance(); // 消费左括号
                        let arg = self.parse_expression()?;
                        match self.advance() {
                            Some(Token::RParen) => {}
                            _ => return Err(format!("函数 {} 缺少右括号", name)),
                        }
                        self.eval_function(&name, arg)
                    }
                    _ => Err(format!("未知标识符: {}", name)),
                }
            }
            Some(other) => Err(format!("意外的词法单元: {:?}", other)),
            None => Err("表达式不完整".to_string()),
        }
    }

    /// 求值内置数学函数
    fn eval_function(&self, name: &str, arg: f64) -> Result<f64, String> {
        match name.to_lowercase().as_str() {
            "sqrt" => {
                if arg < 0.0 {
                    Err("sqrt 参数不能为负数".to_string())
                } else {
                    Ok(arg.sqrt())
                }
            }
            "sin" => Ok(arg.sin()),
            "cos" => Ok(arg.cos()),
            "tan" => Ok(arg.tan()),
            "abs" => Ok(arg.abs()),
            "log" | "log10" => {
                if arg <= 0.0 {
                    Err("log 参数必须为正数".to_string())
                } else {
                    Ok(arg.log10())
                }
            }
            "ln" => {
                if arg <= 0.0 {
                    Err("ln 参数必须为正数".to_string())
                } else {
                    Ok(arg.ln())
                }
            }
            "ceil" => Ok(arg.ceil()),
            "floor" => Ok(arg.floor()),
            "round" => Ok(arg.round()),
            other => Err(format!("未知函数: {}", other)),
        }
    }
}

// ============================================================
// CalcTool — 计算器工具
// ============================================================

/// 计算器工具 — 数学表达式求值
#[derive(Debug)]
pub struct CalcTool;

impl CalcTool {
    /// 求值数学表达式
    pub fn evaluate(expr: &str) -> Result<f64, String> {
        let expr = expr.trim();
        if expr.is_empty() {
            return Err("表达式不能为空".to_string());
        }

        let tokens = Lexer::tokenize(expr)?;
        if tokens.is_empty() {
            return Err("表达式不能为空".to_string());
        }

        let mut parser = Parser::new(tokens);
        let result = parser.parse_expression()?;

        // 检查是否所有词法单元都已消费
        if parser.pos < parser.tokens.len() {
            return Err(format!(
                "表达式解析不完整，剩余词法单元: {:?}",
                &parser.tokens[parser.pos..]
            ));
        }

        Ok(result)
    }

    /// 格式化结果
    ///
    /// 整数值去掉小数点，非整数值保留有效数字。
    pub fn format_result(value: f64) -> String {
        if value.fract() == 0.0 && value.abs() < i64::MAX as f64 {
            format!("{}", value as i64)
        } else {
            // 去掉末尾多余的零
            let s = format!("{}", value);
            s
        }
    }
}

#[async_trait]
impl Tool for CalcTool {
    fn name(&self) -> &str {
        "calc"
    }

    fn description(&self) -> &str {
        "数学表达式求值，支持基本运算和常用数学函数"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "要求值的数学表达式（例如：2 + 3 * 4、sqrt(16)）"
                }
            },
            "required": ["expression"]
        })
    }

    async fn execute(&self, params: Value) -> ToolResult<String> {
        let expr = params
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParams("缺少必要参数: expression".to_string()))?;

        debug!("计算表达式: {}", expr);

        match Self::evaluate(expr) {
            Ok(value) => Ok(Self::format_result(value)),
            Err(e) => Err(ToolError::ExecutionError(format!("计算错误: {}", e))),
        }
    }
}

// ============================================================
// 单元测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// 测试基本加法
    #[test]
    fn test_basic_addition() {
        let result = CalcTool::evaluate("2 + 3").unwrap();
        assert!((result - 5.0).abs() < f64::EPSILON);
    }

    /// 测试基本减法
    #[test]
    fn test_basic_subtraction() {
        let result = CalcTool::evaluate("10 - 3").unwrap();
        assert!((result - 7.0).abs() < f64::EPSILON);
    }

    /// 测试乘法
    #[test]
    fn test_multiplication() {
        let result = CalcTool::evaluate("4 * 5").unwrap();
        assert!((result - 20.0).abs() < f64::EPSILON);
    }

    /// 测试除法
    #[test]
    fn test_division() {
        let result = CalcTool::evaluate("10 / 3").unwrap();
        assert!((result - 3.333333333333333).abs() < 1e-10);
    }

    /// 测试取模运算
    #[test]
    fn test_modulo() {
        let result = CalcTool::evaluate("10 % 3").unwrap();
        assert!((result - 1.0).abs() < f64::EPSILON);
    }

    /// 测试幂运算
    #[test]
    fn test_power() {
        let result = CalcTool::evaluate("2 ^ 10").unwrap();
        assert!((result - 1024.0).abs() < f64::EPSILON);
    }

    /// 测试括号
    #[test]
    fn test_parentheses() {
        let result = CalcTool::evaluate("(2 + 3) * 4").unwrap();
        assert!((result - 20.0).abs() < f64::EPSILON);
    }

    /// 测试嵌套括号
    #[test]
    fn test_nested_parens() {
        let result = CalcTool::evaluate("((2 + 3) * (4 - 1))").unwrap();
        assert!((result - 15.0).abs() < f64::EPSILON);
    }

    /// 测试 sqrt 函数
    #[test]
    fn test_sqrt() {
        let result = CalcTool::evaluate("sqrt(16)").unwrap();
        assert!((result - 4.0).abs() < f64::EPSILON);
    }

    /// 测试 abs 函数
    #[test]
    fn test_abs() {
        let result = CalcTool::evaluate("abs(-5)").unwrap();
        assert!((result - 5.0).abs() < f64::EPSILON);
    }

    /// 测试除以零报错
    #[test]
    fn test_division_by_zero() {
        let result = CalcTool::evaluate("10 / 0");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("零"));
    }

    /// 测试无效表达式报错
    #[test]
    fn test_invalid_expression() {
        // 不完整的表达式应报错
        let _result = CalcTool::evaluate("2 + + 3");
        let result = CalcTool::evaluate("2 +");
        assert!(result.is_err());
    }

    /// 测试格式化整数结果
    #[test]
    fn test_format_result_integer() {
        assert_eq!(CalcTool::format_result(5.0), "5");
    }

    /// 测试格式化小数结果
    #[test]
    fn test_format_result_decimal() {
        let formatted = CalcTool::format_result(std::f64::consts::PI);
        assert!(formatted.starts_with("3.14159"));
    }

    /// 测试复杂表达式
    #[test]
    fn test_complex_expression() {
        // sqrt(2^2 + 3^2) = sqrt(4 + 9) = sqrt(13) ≈ 3.6055
        let result = CalcTool::evaluate("sqrt(2^2 + 3^2)").unwrap();
        assert!((result - 13.0_f64.sqrt()).abs() < 1e-10);
    }

    /// 测试参数 Schema 结构
    #[test]
    fn test_parameter_schema() {
        let tool = CalcTool;
        let schema = tool.parameters_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["expression"].is_object());

        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("expression")));
    }

    /// 测试工具名称
    #[test]
    fn test_tool_name() {
        let tool = CalcTool;
        assert_eq!(tool.name(), "calc");
        assert!(!tool.description().is_empty());
    }
}
