### What?

The good old game [Pong](https://en.wikipedia.org/wiki/Pong),
written in lambda calculus, and a thin layer of Rust.

### Why?

I was bored.

### No, seriously, why?

Everyone keeps saying that lambda calculus and Turing machines are Turing
complete and therefore could, _theoretically_, be used for any computation
that can be done. What a bore! I wanted to show it in practice.

### Why the Rust layer? (Or, why not do everything in lambda calculus?)

Two reasons, mostly.

First, while lambda calculus is Turing complete, it doesn't have any way of
doing I/O, let alone having a GUI.

Second, lambda calculus can, given a state, compute the next one, but
-- since it's a pure functional language -- cannot store the state, which
is necessary for the game to work.

Also, you'll notice that a native Rust implementation of Pong has also been
provided; it's meant for comparing both the code and the programs' performance.

### Running

First, install the `lambda_calc` interpreter with

```
$ cargo install lambda_calc
```

By default, cargo will install the binary to `$HOME/.cargo/bin`. Make sure
to add that directory to the PATH environment variable, or install
somwehere else with the `--root` option.

Clone this repository, then, from this repository's root,
run the lambda calculus pong with

```
$ cargo run --release -- -l lambda/pong.txt
```

And the native Rust implementation with

```
$ cargo run --release -- -n
```

The `--release` flag instructs Rust to optimize the resulting program.
(It's slow enough with that, let alone without it...)

### How?

In a nutshell:

The main program spawns a lambda calculus interpreter process, which
parses the definitions from a source file, and keeps waiting for input.
The lambda calculus source must define the following symbols, which compute

- `initState`: the first state;

- `nextState`: the next game state, given its current state and the user input;

- `gameOver`: whether the game is over, given its state;

- `getScreenRects`: the list of rectangles that must be rendered, given the game state.

The main program then begins to supply input to the lambda calculus
interpreter process.
At the very first frame, the first state is obtained with `initState`.
Then, every frame,

- the next state is computed with `nextState`;

- it's decided whether to close the window, with `gameOver`'s result;

- the rectangles given by `getScreenRects` are rendered.

The game state is simply stored and never parsed in any way; only the lambda
calculus functions are required to understand its representation.

However, the results of `gameOver` and `getScreenRects` must be parsed,
and so must be in an encoding understood both by the main program and the
lambda calculus code.
Booleans use [Church enconding](https://en.wikipedia.org/wiki/Church_encoding),
while the list of rectangles is a Church list,
where each rectangle is a 4-tuple containing the integers (x, y, w, h),
[whose meanings can be seen here](https://wiki.libsdl.org/SDL_Rect).
In turn, each integer uses a custom encoding.

### Performance

In a modern i5, the lambda calculus implementation (`-l`) takes a bit more than
20 seconds to start, but has an ok-ish frame rate and is actually playable.
