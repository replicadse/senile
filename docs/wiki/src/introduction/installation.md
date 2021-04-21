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

## The docker way

You can also install the program by utilizing docker.
```
docker pull senile
```
Note that the docker syntax to execute the program with a custom entrypoint is somewhat alien. In order to successfully execute it, run it with this syntax:
```
docker run --rm  -v $HOST_PATH:/app/targets/$FOLDER:ro replicadse/senile:latest collect -p=/app/targets --format="// TODO!(,):" [...]
```
You essentially need specify the command and all the args for the program _after_ specifying the image which is to be run (`replicadse/senile:latest`).

