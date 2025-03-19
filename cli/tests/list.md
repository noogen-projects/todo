# List

## Prepare projects

```sh
$ todo new "test A"
    Creating `test A` project
```

```sh
$ todo new "test B"
    Creating `test B` project
```

```sh
$ todo new --with-manifest "test A/test C"
    Creating `test C` project under `${current_dir_path}/test A`
```

```sh
$ todo new "test A/test D"
    Creating `test D` project under `${current_dir_path}/test A`
```

## List empty projects

```sh
$ todo list
Error: could not find `Project.toml` or `*.manifest.md` in `${current_dir_path}` or any parent directory
```

```sh
$ todo list .
List steps of 4 projects

[test A]: 0

[test A/test C]: 0

[test A/test D]: 0

[test B]: 0
```

```sh
$ todo list "test A"
List steps of 1 project

[test A]: 0
```

```sh
$ cd "test A"
$ todo list
List steps of 3 projects

[test A]: 0

[test A/test C]: 0

[test A/test D]: 0
```

```sh
$ cd "test A/test C"
$ todo list
List steps of 1 project

[test C]: 0
```

## List non empty projects

```sh
$ cd "test A"
$ todo add "task 1"
    Adding `task 1` issue to `test A` project
```

```sh
$ todo list .
List steps of 4 projects

[test A]: 1
- task 1

[test A/test C]: 0

[test A/test D]: 0

[test B]: 0
```
