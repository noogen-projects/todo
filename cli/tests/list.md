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

## List one step

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

```sh
$ todo list "test A"
List steps of 1 project

[test A]: 1
- task 1
```

```sh
$ cd "test A"
$ todo list
List steps of 3 projects

[test A]: 1
- task 1

[test A/test C]: 0

[test A/test D]: 0
```

## List multiple steps

```sh
$ cd "test A"
$ todo add "task 2"
    Adding `task 2` issue to `test A` project
```

```sh
$ cd "test A/test C"
$ todo add "task 1"
    Adding `task 1` issue to `test C` project
```

```sh
$ todo list .
List steps of 4 projects

[test A]: 2
- task 2
- task 1

[test A/test C]: 1
- task 1

[test A/test D]: 0

[test B]: 0
```

```sh
$ todo list "test A"
List steps of 1 project

[test A]: 2
- task 2
- task 1
```

```sh
$ cd "test A"
$ todo list
List steps of 3 projects

[test A]: 2
- task 2
- task 1

[test A/test C]: 1
- task 1

[test A/test D]: 0
```

```sh
$ cd "test A/test C"
$ todo list
List steps of 1 project

[test C]: 1
- task 1
```

```sh
$ cd "test A"
$ todo add "task 3"
    Adding `task 3` issue to `test A` project
```

```sh
$ cd "test A/test C"
$ todo add "task 2"
    Adding `task 2` issue to `test C` project
```

```sh
$ cd "test A/test D"
$ todo add "task D-1"
    Adding `task D-1` issue to `test D` project
```

```sh
$ todo list .
List steps of 4 projects

[test A]: 3
- task 3
- task 2
- task 1

[test A/test C]: 2
- task 2
- task 1

[test A/test D]: 1
- task D-1

[test B]: 0
```

```sh
$ todo list "test A"
List steps of 1 project

[test A]: 3
- task 3
- task 2
- task 1
```

```sh
$ cd "test A"
$ todo list
List steps of 3 projects

[test A]: 3
- task 3
- task 2
- task 1

[test A/test C]: 2
- task 2
- task 1

[test A/test D]: 1
- task D-1
```

```sh
$ cd "test A/test C"
$ todo list
List steps of 1 project

[test C]: 2
- task 2
- task 1
```

```sh
$ cd "test A/test D"
$ todo list
List steps of 1 project

[test D]: 1
- task D-1
```

```sh
$ cd "test B"
$ todo add "task B-1"
    Adding `task B-1` issue to `test B` project
```

```sh
$ cd "test B"
$ todo add --last "task B-2"
    Adding `task B-2` issue to `test B` project
```

```sh
$ todo list .
List steps of 4 projects

[test A]: 3
- task 3
- task 2
- task 1

[test A/test C]: 2
- task 2
- task 1

[test A/test D]: 1
- task D-1

[test B]: 2
- task B-1
- task B-2
```

```sh
$ cd "test B"
$ todo list
List steps of 1 project

[test B]: 2
- task B-1
- task B-2
```
