# Oxide - a command line terminal and shell for the laid back developer
Terminal and shell rustily written. 

## Goals of Oxide
    - 100% Rust
    - 100% usable by experienced developer and newbie alike
    - 100% better than any existing terminal and shell
    - 50% less humility

## Current Features being Addressed

- [ ] Testing of functionality
- [ ] Refactoring main.rs into lib.rs
- [ ] Guided tutorial (in the style of vimtutor)

## Future Features

- [ ] Standalone program 
    - [ ] Command line - GUI interactivity
    - [ ] Fancy command colorization

## Features Completed
- [X] Command Piping
- [X] File redirection (>, >>, <)
- [X] File redirection within chained pipes (ls .. | sort -r >
  parent.txt | cat)
- [X] Tab Completion (of filenames)
- [X] Home-made state changing commands (cd, exit)
- [X] Calling of arbitrary programs (ls, mkdir, cat, anything else
  accessible to the environment)
