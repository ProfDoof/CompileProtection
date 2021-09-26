# Compile Protection

This crate defines a single macro that is a brainfunct
compile-time interpreter. One example is as follows 
```rust
#![recursion_limit = "18446744073709551615"]
#![feature(const_mut_refs)]

use compile_protection::brainfunct_protect;

fn main() {
    brainfunct_protect!("/,./+@", "1", "1");
}
```

`brainfunct_protect!` generates `const` functions that 
can be consumed at compile time to run the brain funct program
with the provided input `"1"`, and it expects the output `"1"`.

The syntax is as follows: 
```rust
brainfunct_protect!(
    "brainfunct legal program", 
    "input to give to the program", 
    "the output to expect from the program"
);
```

If the program given the input produces the given output,
then the rest of the rust program will compile. However,
if the program produces the incorrect output, it will recursively
loop infinitely or until the user cancels the compile. 

This is trivial to beat if there is nothing keeping them from 
deleting the macro from the source. But is nontrivial otherwise.

> The recursion limit being set so high prevents the recursive 
const functions that are being generated from accidentally
going over the recursion limit. 

