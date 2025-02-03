# New project

## New project default

```sh
$ todo new "test A"
    Creating `test A` project
```

```sh
$ ls "./test A"
Project.toml
```

```sh
$ cat "./test A/Project.toml"
id = "test A"
name = "test A"
```

```sh
$ todo new "test A"
    Creating `test A` project
Error: destination `${current_dir_path}/test A` already exists
```

## New project with manifest

```sh
$ todo new --manifest "test B"
    Creating `test B` project
```

```sh
$ ls "./test B"
test B.manifest.md
```

````sh
$ cat "./test B/test B.manifest.md"
# test B

```toml project
id = "test B"
name = "test B"
```
````

```sh
$ todo new --manifest "test A"
    Creating `test A` project
Error: destination `${current_dir_path}/test A` already exists
```

```sh
$ todo new --manifest "test B"
    Creating `test B` project
Error: destination `${current_dir_path}/test B` already exists
```

```sh
$ todo new "test B"
    Creating `test B` project
Error: destination `${current_dir_path}/test B` already exists
```

# New subproject

```sh
$ todo new "test A"
    Creating `test A` project
```

```sh
$ todo new --manifest "test B"
    Creating `test B` project
```

## New subproject default

```sh
$ todo new "test A/test B"
    Creating `test B` project under `${current_dir_path}/test A`
```

```sh
$ ls "./test A/test B"
Project.toml
```

```sh
$ cat "./test A/test B/Project.toml"
id = "test B"
name = "test B"
```

```sh
$ todo new "test A/test B"
    Creating `test B` project under `${current_dir_path}/test A`
Error: destination `${current_dir_path}/test A/test B` already exists
```

## New project with manifest

```sh
$ todo new --manifest "test B/test B"
    Creating `test B` project under `${current_dir_path}/test B`
```

```sh
$ ls "./test B/test B"
test B.manifest.md
```

````sh
$ cat "./test B/test B/test B.manifest.md"
# test B

```toml project
id = "test B"
name = "test B"
```
````

```sh
$ todo new --manifest "test B/test B"
    Creating `test B` project under `${current_dir_path}/test B`
Error: destination `${current_dir_path}/test B/test B` already exists
```

```sh
$ todo new "test B/test B"
    Creating `test B` project under `${current_dir_path}/test B`
Error: destination `${current_dir_path}/test B/test B` already exists
```
