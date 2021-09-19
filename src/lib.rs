use proc_macro::{TokenStream};
use quote::{quote, ToTokens, TokenStreamExt, format_ident};
use syn::{LitStr, parse_macro_input, Token, Expr};
use syn::parse::{Parse, ParseStream};
use proc_macro2::Punct;

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

#[proc_macro]
pub fn brainfunct_protect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as InputStruct);
    let mut funcs = Vec::new();
    for c in input.program.value().bytes() {
        match c {
            b'/' => funcs.push(Vec::new()),
            op => funcs.last_mut().unwrap().push(op)
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
                c => panic!("You've used an illegal character: {}", c)
            }
        }
        let merged = inner.join("");
        syn::parse_str::<Expr>(&merged).unwrap()
    });

    let expected = input.expected.value();
    let expected_bytes = expected.bytes();
    let indices = 0..expected.len();
    let input_str = input.input.value();
    let input_iter = input_str.bytes().chain(std::iter::repeat(0)).take(u16::MAX as usize);

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
