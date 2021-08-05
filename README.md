# Vault
This is a simple terminal program that helps you store important files into an 
encrypted archive. It utilizes OpenSSL's AES256 encryption to keep the archive
secure on your disk. 

Two things to note however:

1. This does not provide any kind of compression. The size of the files you add
will not magically shrink when put into one of these archives. There are other
tools for that.
2. This is probably vulnerable to various types of attacks. At the moment, I 
have no plans of really securing the hell out of this program, and by the time 
you need really serious protection from trained prying eyes, you should be 
looking much further than this piece of crap.


## Building and Usage
To build this program, you will need three things:
1. A Linux machine (its possible this works under other patforms but I don't 
have the effort or the resources to test it on every platform so ye be warned)
2. The OpenSSL library installed
3. Rust's `cargo` system and a Rust version >= 1.49

To run this program, simply open a terminal in this repository and run

``` 
cargo run --release -- --help
```

You can also find the actual executable in the `target` directory (run `cargo 
build --release` first!)
