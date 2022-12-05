# Cuzi

Cuzi (chuzi) is a little, tiny HTTP web server implementation written in C programming language. In the near future it'll be ported to Rust programming language.

I was trying to figure out how sockets and non-blocking I/O works. Absolutely best way to learn new things is use it on a real thing. And cuzi born by that way.

Currently I adding new features when I have time.

## Compile

### Linux

You must have `gcc` and `make` to compile source code. You can install necessary tools under Debian based systems by typing:

```bash
sudo apt install build-essential
```

This command will install the `gcc` and `make` tools to your system.
On Arch based systems probably you have `make` and `gcc`. But if you don't, you can install by using `pacman`:

```bash
sudo pacman -S gcc base-devel make
```

> Probably it was something like that. It has been so long from install my system. I don't remember.
>
> By the way I use Arch :)

### Windows

If you are using Windows, you can use **MinGW** or similar tools.

> Sorry I don't use Windows for coding. So I don't know how to compile on it :(

### Mac OS X

I don't have a Macbook :(

But I've already tried to compile cuzi under Mac OS X and it couldn't compile it via clang. I think if you use gcc instead of clang, it should compile.

> And it is up to you, how to install gcc :)

## Contribute

If you see a bug or want to add new feature, feel free to send pull requests. There are not any restriction to become a contributor yet.
