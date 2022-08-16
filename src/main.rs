use std::{process::exit, io, fs, path::Path, env};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum TokenKind {
    Int,
    Char,
    String,
    Function,
    Empty,
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,
    content: String,
    line: u64,
    pos: u64,
}

#[derive(Debug)]
enum Maybe<T> {
    Just(T),
    Error(String)
}

struct TokenColelction {
    tokens: Vec<Token>,
    pointer: usize
}

impl TokenColelction {
    fn new(tokens: Vec<Token>) -> TokenColelction {
        TokenColelction { tokens, pointer: 0 }
    }

    fn next(&mut self) {
        self.pointer += 1;
    }

    fn curent(&mut self) -> Option<&Token> {
        if self.pointer == self.tokens.len() {
            return None
        }
        Some(&self.tokens[self.pointer])
    }

    fn running(&mut self) -> bool {
        self.pointer < self.tokens.len()
    }

    fn set(&mut self, value: usize) {
        self.pointer = value;
    }
}

fn lex(content: &String) -> Maybe<Vec<Token>> {
    let mut result: Vec<Token> = Vec::new();
    result.push(Token { kind: TokenKind::Empty, content: String::new(), line: 0 , pos: 0 });
    let mut pos = 0;
    let mut line = 0;
    for character in content.chars() {
        let last = result.last_mut().unwrap();
        pos += 1;
        match character {
            '0'..='9' => {
                last.content.push(character);
                if last.kind == TokenKind::Empty {
                    last.kind = TokenKind::Int;
                }
            },
            '\'' => {
                if last.kind == TokenKind::Empty {
                    last.kind = TokenKind::Char;
                } else {
                    result.push(Token {kind: TokenKind::Empty, content: String::new(), line, pos});
                }
            },
            '"' => {

                if last.kind == TokenKind::Empty {
                    last.kind = TokenKind::String;
                } else if last.kind != TokenKind::String {
                    result.push(Token { kind: TokenKind::String, content: String::new(), line, pos });
                } else {
                    result.push(Token {kind: TokenKind::Empty, content: String::new(), line, pos});
                }
            },
            ' ' | '\n' => {
                if character == '\n' {
                    line += 1;
                    pos = 0;
                }
                match last.kind {
                    TokenKind::Int | TokenKind::Function => {
                        result.push(Token { kind: TokenKind::Empty, content: String::new(), line, pos});
                    }, 
                    TokenKind::Char => {
                        if last.content.len() == 1 {
                            return Maybe::Error("Bad char format".to_string());
                        }
                        last.content.push(character);
                    },
                    TokenKind::Empty => {},
                    _ => {
                        last.content.push(character);
                    }
                }
            },
            _ => {
                match last.kind {
                    TokenKind::Char => {
                        if last.content.len() == 1 {
                            return Maybe::Error("Bad char format".to_string());
                        }
                    },
                    TokenKind::Int => {
                        result.push(Token { kind: TokenKind::Function, content: character.to_string(), line, pos});
                        continue;
                    },
                    TokenKind::Empty => {
                        last.kind = TokenKind::Function;
                    },
                    _ => {}
                }

                last.content.push(character);
            }
        }
    }

    return Maybe::Just(result);
}

fn parse(tokens: Maybe<Vec<Token>>) {
    match tokens {
        Maybe::Just(tokens) => {
            evaluate(tokens);
        },
        Maybe::Error(error) => {
            println!("{}", &error);
        }
    }
}

fn evaluate(tokens: Vec<Token>) {
    let mut stack: Vec<i64> = Vec::new();
    let mut tokens = TokenColelction::new(tokens);
    while tokens.running() {
        evaluate_token(&mut tokens, &mut stack);
        tokens.next();
    }
}

fn evaluate_token(tokens: &mut TokenColelction, stack: &mut Vec<i64>) {
    match tokens.curent().unwrap().kind {
        TokenKind::Int => {
            stack.push(tokens.curent().unwrap().content.parse::<i64>().unwrap());
        },
        TokenKind::Char => {
            stack.push(tokens.curent().unwrap().content.chars().next().expect("Empty characters are not allowed.") as i64);
        },
        TokenKind::String => {
            stack.push(0);
            for character in tokens.curent().unwrap().content.chars().rev() {
                stack.push(character as i64);
            }
        },
        TokenKind::Function => {
            call(stack, tokens);
        },
        TokenKind::Empty => {}
    }
}

fn call(stack: &mut Vec<i64>, tokens: &mut TokenColelction) {
    match tokens.curent().unwrap().content.as_str() {
        "+" => {
            let first = stack.pop().expect("There is no number to be added.");
            let second = stack.pop().expect("There is no number to be added.");
            stack.push(first + second);
        },
        "-" => {
            let first = stack.pop().expect("There is no number to be subtracted.");
            let second = stack.pop().expect("There is no number to be substracted.");
            stack.push(second - first);
        },
        "pop" => {
            print!("{}", stack.pop().expect("There is no number to be printed"));
        },
        "printint" => {
             print!("{}", stack.last().expect("There is no number to be printed"));           
        },
        "putchar" => {
            print!("{}", stack.pop().expect("There is no character to be printed").to_string().parse::<u8>().expect("Number cant be negative to be printed as char.") as char)
        },
        "putstring" => {
            while *stack.last().expect("String didnt end with \\0.") != 0 {
                print!("{}", stack.pop().expect("There is no character to be printed").to_string().parse::<u8>().expect("Number cant be negative to be printed as char.") as char)
            }
            stack.pop();
        }, 
        "if" => {
            if *stack.last().expect("Cant perform if because stack is empty") == 0 {
                let mut unclosed = 0;
                tokens.next();
                while tokens.curent().expect("Unclosed if").content != "fi" || unclosed != 0 {
                    if tokens.curent().unwrap().content == "if" {
                        unclosed += 1
                    }
                    
                    if tokens.curent().unwrap().content == "fi" {
                        unclosed -= 1
                    }
                    tokens.next();
                }
            }
        },
        "fi" => {},
        "scanint" => {
            let mut result = String::new();
            io::stdin().read_line(&mut result).expect("No input");
            stack.push(result.trim().parse::<i64>().expect("Expected int"));
        },
        "scanstring" => {
            let mut result = String::new();
            io::stdin().read_line(&mut result).expect("No input");
            stack.push(0);
            for character in result.chars().rev() {
                stack.push(character as i64);
            }
        },
        "while" => {
            let start = tokens.pointer;
            while *stack.last().expect("Cant perform while since stack is empty") != 0 {
                while tokens.curent().expect("Unclosed while").content != "end" {
                    tokens.next();
                    evaluate_token(tokens, stack);
                }
                tokens.set(start);
            }
        },
        "end" => {},
        _ => {
            println!("Not implemented.");
            exit(1);
        }
    }
}

fn main() {
    let mut args = env::args();
    let filename = args.nth(1).expect("no file provided");

    let code = fs::read_to_string(filename)
        .expect("File is not readable");
    parse(lex(&code));
//    println!("{:?}", lex(&code));
}
