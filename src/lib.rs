use proc_macro::TokenStream;
use proc_macro2::{Punct, Span};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Expr, LitStr, Token};

struct InputStruct {
    program: LitStr,
    input: LitStr,
    expected: LitStr,
}

impl ToTokens for InputStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.program.to_tokens(tokens);
        tokens.append(Punct::new(',', proc_macro2::Spacing::Alone));
        self.input.to_tokens(tokens);
        tokens.append(Punct::new(',', proc_macro2::Spacing::Alone));
        self.expected.to_tokens(tokens);
    }
}

impl Parse for InputStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let p: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let i: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let e: LitStr = input.parse()?;

        // unimplemented!()
        Ok(InputStruct {
            program: p,
            input: i,
            expected: e,
        })
    }
}

struct BrainfunctProgram {
    operations: Vec<u8>,
    functions: Vec<usize>,
    main_index: usize,
    input: Vec<u8>,
    output: Vec<u8>,
}

impl Parse for BrainfunctProgram {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let p: LitStr = input.parse()?;
        let p_string = p.value();
        input.parse::<Token![,]>()?;
        let input_str: LitStr = input.parse()?;
        let input_string = input_str.value();
        input.parse::<Token![,]>()?;
        let output: LitStr = input.parse()?;
        let output_string = output.value();
        let mut temp = BrainfunctProgram {
            operations: Vec::with_capacity(p_string.len()),
            functions: vec![usize::MAX],
            main_index: usize::MAX,
            input: Vec::with_capacity(input_string.len()),
            output: Vec::with_capacity(output_string.len()),
        };
        let mut func_index = 0;
        for (index, c) in p.value().bytes().enumerate() {
            temp.operations.push(match c {
                b'>' | b'<' | b'+' | b'-' | b'.' | b',' | b'@' => c,
                b'/' => {
                    temp.functions.push(temp.main_index);
                    temp.main_index = index;
                    func_index += 1;
                    c
                }
                _ => {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "You had an invalid character in your program",
                    ))
                }
            });

            // The value is 256 because you can have 255 functions (and they start indexing at 1)
            // and a main function which will cause the len to be 256 in the greatest case
            if temp.functions.len() > 256 {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "You had more functions than are able to be called",
                ));
            }
        }

        if func_index == 0 {
            return Err(syn::Error::new(
                Span::call_site(),
                "You didn't define any functions at all",
            ));
        }

        for c in input_string.bytes() {
            temp.input.push(c);
        }

        for c in output_string.bytes() {
            temp.output.push(c);
        }

        Result::Ok(temp)
    }
}

struct BrainfunctArray {
    value: [u8; u16::MAX as usize],
}

impl Parse for BrainfunctArray {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![,]>()?;
        let array_str: LitStr = input.parse()?;
        let mut temp = BrainfunctArray {
            value: [0u8; u16::MAX as usize],
        };
        for (index, c) in array_str.value().bytes().enumerate() {
            temp.value[index] = c
        }

        Result::Ok(temp)
    }
}

#[proc_macro]
pub fn fast_brainfunct_protect(input: TokenStream) -> TokenStream {
    let program = parse_macro_input!(input as BrainfunctProgram);
    let o = program.operations.iter();
    let o_size = program.operations.len();
    let f = program.functions.iter();
    let f_size = program.functions.len();
    let m = program.main_index;
    let i = program.input.iter();
    let i_size = program.input.len();
    let out = program.output.iter();
    let out_size = program.output.len();
    let call_const = quote! {
        const fn run_fast_brainfunct_protect(
            operations: &[u8],
            op_size: usize,
            func_map: &[u8],
            func_size: u8,
            main_index: usize,
            input: &[u8],
            i_size: usize,
            output: &[u8],
            o_size: usize,
        ) {
            const fn die() {
                die()
            }

            let mut tape = [0u8; u16::MAX as usize];
            let mut tape_ptr = 0usize;
            let mut input_ptr = 0usize;
            let mut output_ptr = 0usize;

            let mut stack = [usize::MAX; u16::MAX as usize];
            let mut stack_head = 0i32;
            let mut current_index = main_index + 1usize;
            loop {
                if current_index >= op_size {
                    if stack_head != -1 {
                        // panic!("Your call stack wasn't completely emptied even though you have reached the end of the main function");
                        die()
                    }
                    break;
                }
                match operations[current_index] {
                    b'>' => tape_ptr += 1,
                    b'<' => tape_ptr -= 1,
                    b'+' => tape[tape_ptr] += 1,
                    b'-' => tape[tape_ptr] -= 1,
                    b',' => {
                        if input_ptr >= i_size {
                            die();
                        }
                        tape[tape_ptr] = input[input_ptr];
                        input_ptr += 1
                    }
                    b'.' => {
                        if output[output_ptr] != tape[tape_ptr] || output_ptr >= o_size {
                            die()
                        }
                        output_ptr += 1
                    }
                    b'@' => {
                        if tape[tape_ptr] >= func_size {
                            // panic!("Undefined function {}", tape[tape_ptr])
                            die();
                        }
                        stack[stack_head as usize] = current_index;
                        stack_head += 1;
                        current_index = (func_map[tape[tape_ptr] as usize] + 1) as usize;
                    }
                    b'/' => {
                        current_index = stack[stack_head as usize];
                        stack_head -= 1;
                    }
                    _ => die(),
                }
                current_index += 1
            }
        }

        run_fast_brainfunct_protect([#(#o),*], #o_size, [#(#f),*], #f_size, #m, [#(#i),*], #i_size, [#(#out),*], #out_size);
    };
    call_const.into()
}

#[proc_macro]
pub fn brainfunct_protect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as InputStruct);
    let mut funcs = Vec::new();
    for c in input.program.value().bytes() {
        match c {
            b'/' => funcs.push(Vec::new()),
            op => funcs.last_mut().unwrap().push(op),
        }
    }

    let indices = 1..(funcs.len());
    let names = indices.clone().map(|i| format_ident!("func{}", i));
    let call_function_tokens = quote! {
        const fn c(state: &mut BrainFunctState) -> &mut BrainFunctState {
            match state.tape[state.tape_ptr as usize] as usize {
                #(#indices => #names(state)),*,
                _ => state
            }
        }
    };

    let names = (1..funcs.len() + 1).map(|i| format_ident!("func{}", i));
    let defs = funcs.iter().map(|func| {
        let mut inner = vec!["state".to_string()];
        for op in func {
            match op {
                b'>' => {
                    inner.insert(0, "r(".into());
                    inner.push(")".into());
                }
                b'<' => {
                    inner.insert(0, "l(".into());
                    inner.push(")".into());
                }
                b'+' => {
                    inner.insert(0, "i(".into());
                    inner.push(")".into());
                }
                b'-' => {
                    inner.insert(0, "d(".into());
                    inner.push(")".into());
                }
                b'.' => {
                    inner.insert(0, "output(".into());
                    inner.push(")".into());
                }
                b',' => {
                    inner.insert(0, "input(".into());
                    inner.push(")".into());
                }
                b'@' => {
                    inner.insert(0, "c(".into());
                    inner.push(")".into());
                }
                c => panic!("You've used an illegal character: {}", c),
            }
        }
        let merged = inner.join("");
        syn::parse_str::<Expr>(&merged).unwrap()
    });

    let expected = input.expected.value();
    let expected_bytes = expected.bytes();
    let indices = 0..expected.len();
    let input_str = input.input.value();
    let input_iter = input_str
        .bytes()
        .chain(std::iter::repeat(0))
        .take(u16::MAX as usize);

    let main_func = format_ident!("func{}", funcs.len());
    let tokens = quote! {
        struct BrainFunctState {
            tape: [u8; u16::MAX as usize],
            tape_ptr: u16,
            input: [u8; u16::MAX as usize],
            input_ptr: u16,
            output: [u8; u16::MAX as usize],
            output_ptr: u16,
        }

        const fn r(state: &mut BrainFunctState) -> &mut BrainFunctState {
            state.tape_ptr = state.tape_ptr.wrapping_add(1);
            state
        }

        const fn l(state: &mut BrainFunctState) -> &mut BrainFunctState {
            state.tape_ptr = state.tape_ptr.wrapping_sub(1);
            state
        }

        const fn i(state: &mut BrainFunctState) -> &mut BrainFunctState {
            state.tape[state.tape_ptr as usize] = state.tape[state.tape_ptr as usize].wrapping_add(1);
            state
        }

        const fn d(state: &mut BrainFunctState) -> &mut BrainFunctState {
            state.tape[state.tape_ptr as usize] = state.tape[state.tape_ptr as usize].wrapping_sub(1);
            state
        }

        const fn output(state: &mut BrainFunctState) -> &mut BrainFunctState {
            state.output[state.output_ptr as usize] = state.tape[state.tape_ptr as usize];
            state.output_ptr += 1;
            state
        }

        const fn input(state: &mut BrainFunctState) -> &mut BrainFunctState {
            state.tape[state.tape_ptr as usize] = state.input[state.input_ptr as usize];
            state.input_ptr += 1;
            state
        }

        #call_function_tokens

        #(const fn #names(state: &mut BrainFunctState) -> &mut BrainFunctState {
            #defs
        }
        )*

        static S: () = {
            const fn die() { die() }
            let input = [#(#input_iter),*];
            let mut state = BrainFunctState {
                tape: [0u8; u16::MAX as usize],
                tape_ptr: 0u16,
                input,
                input_ptr: 0u16,
                output: [0u8; u16::MAX as usize],
                output_ptr: 0u16,
            };
            #main_func(&mut state);
            if !(#(state.output[#indices] == #expected_bytes)&&*) {
                die()
            }
        };
    };
    tokens.into()
}
