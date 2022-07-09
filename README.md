# VRGit

A TUI for staging, committing and pushing code in git repositories.

## Goal

The goal with VRGit is to replicate the functionality of [Magit](https://magit.vc/) as a standalone program using keybindings inspired by VIM. 

The idea came to me while exploring evil distributions like [spacemacs](https://www.spacemacs.org/) and [Doom Emacs](https://github.com/doomemacs/doomemacs). The Emacs way of integrating every imaginable functionality into the text editor wasn't for me, but smooth experience of using Magit really stuck with me nonetheless.

I don't have any intention of supporting the more complicated git operations such as diff, rebasing, etc. For now the goal is just to establish a concrete framework for the modal controls, configuration of those controls and management of the most common git operations I use in my workflow.

## Supported operations
- [x] Stage
- [x] Commit
- [x] Push
- [ ] Checkout
- [ ] Pull

## Operating system
For now the program only works on Linux and OSX as I can't mange to compile the [ncurses-crate](https://crates.io/crates/ncurses) on windows. This is obviously something that should be changed in the future.
