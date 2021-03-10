# Installation via `cargo`


## The rust way

In order to install or update, simply run the following command.
```
cargo install senile --force
```
Note that the `--force` flag is actually just necessary when updating.

## The arch way

Install the program from the Arch User Repository `AUR`. It is recommended to use the pacman wrapper `yay`.
```
yay -S senile
```
Note that this package will build the program locally (as cargo install does) so it depends on the `rust`+`rustup`+`cargo` packages being installed.

