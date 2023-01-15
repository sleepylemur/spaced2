# Spaced2

Command-line spaced repetition flashcard reviewer

Flash cards live in the plain text files in "cards" directory

History is saved in "history"

### Run

Summary of decks

```
cargo run
```

Quiz a specific deck

```
cargo run deckname
```

Build

```
cargo build
```

### Shell alias

Replace `app` path with path to spaced2 executable

```
sp () {
  local app
  app='/Users/sleepylemur/rust/spaced2/target/debug/spaced2'
  "$app" "$("$app" | fzf -n 1 | awk '{print $1}')"
}
```

build and run

```
cargo build && sp
```
