# Neural How

This is a simple project to help you in your CLI Adventures.

## Installation

```bash
cargo install --git https://github.com/elikoga/neural-how
```

## Usage

Prepare your token by setting the HOW_TOKEN environment variable.

You can see a example value in `.env.sample` and some other schemes in `token_mappings.sample.json`.

Then just ask your terminal `how` questions.

```bash
~/Dev/neural-how$ how do i use a here document
cat << EOF > file.txt
hello
world
EOF
```
