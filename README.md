# kak-recase

Replace multiple selections while preserving capitalization of the replaced text.

## Installation

Build or grab the `kak-recase` binary from a release and put it in your path.
Add `eval %sh{kak-recase --generate-config}` to your kakrc to define get the `recase` command.

## Usage

Create a bunch of selections, then run the `recase` command.
Enter a string to replace with,
and watch the selections be replaced with the entered string,
but with the capitalization of the original selections.

I recommend binding the command to a user key, like `map global user c ":recase<ret>"`.
