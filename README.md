# libpapyrus

**libpapyrus** is meant to be a fully featured toolchain for working with
[papyrus](https://www.creationkit.com/fallout4/index.php?title=Category:Papyrus),
the scripting language created by *Bethesda* for the *Elder Scrolls* and *Fallout*
games series.

## Goals

The main aim of this project is to provide the maximum amount of comfort to
developers. This can be achieved by giving them some more tools that help save
both time and mental energy.

A formatter, **papyrusfmt**, is an essential tool for any development environment
in that in enables the programmer to truly focus only on what is absolutely
necessary - making sure the code does what it is supposed to - and let the formatter
handle the boring task of making sure everything is readable.

A linter helps catching all the simple errors and typos that are often overlooked
when reviewing code. This tool gives you an immediate feedback on bugs, errors
and other potentially dangerous things present in your work.

Lastly, a compiler, **papyrusc**, is the essential tool that enables you to
actually make use of your code. This will be written completely from scratch
making it 100% independent from the original compiler. This allows us to
extend the language with extra features and optimizations that are not present in
the original language.

Aditionally, all the source used in producing the tools above will be published
as a Rust Crate so that others may make use of this project in making their own
projects. While the binary tools will always be the main goal of this project
there will be an effort to keep the backend library presentable and documented.

## Roadmap

<!--- ✔ 🚀 ❌ --->

The current roadmap is below. Keep in mind this is subject to change during
development.

### 🚀 Creating the formatter
  1. ❌ Build the Lexer
  2. ❌ Build the Syntax Parser
  3. ❌ Create the *papyrusfmt* binary

### 🚀 Creating the linter
  1. ❌ Build the Lexical Parser
  2. ❌ Decide on a name for the linter
  3. ❌ Create the *insert_name_here* binary

### 🚀 Creating the compiler
  1. ❌ Build the AST
  2. ❌ Convert to `.pas` assembly-like language
  3. ❌ ???
  4. ❌ Create the *papyrusc* binary
