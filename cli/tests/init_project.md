# Init project outside project dir

## Prepare directories

```sh
$ mkdir "test A"
$ mkdir "test B"
```

## Init project with config

```sh
$ todo init "test A"
    Initializing `test A` project
```

```sh
$ ls "test A"
Project.toml
```

```sh
$ cat "test A/Project.toml"
id = "test A"
name = "test A"
```

```sh
$ todo init "test A"
    Initializing `test A` project
Error: destination `${current_dir_path}/test A/Project.toml` already exists
```

```sh
$ todo init "test C"
    Initializing `test C` project
Error: destination `${current_dir_path}/test C` does not exists
```

## Init project with manifest

```sh
$ todo init --with-manifest "test B"
    Initializing `test B` project
```

```sh
$ ls "test B"
test B.manifest.md
```

````sh
$ cat "test B/test B.manifest.md"
# test B

```toml project
id = "test B"
name = "test B"
```
````

```sh
$ todo init --with-manifest "test A"
    Initializing `test A` project
Error: destination `${current_dir_path}/test A/Project.toml` already exists
```

```sh
$ todo init --with-manifest "test B"
    Initializing `test B` project
Error: destination `${current_dir_path}/test B/test B.manifest.md` already exists
```

```sh
$ todo init "test B"
    Initializing `test B` project
Error: destination `${current_dir_path}/test B/test B.manifest.md` already exists
```

# Init project inside project dir

## Prepare directories

```sh
$ mkdir "test A"
$ mkdir "test B"
```

## Init project with config

```sh
$ cd "test A"
$ todo init
    Initializing `test A` project
```

```sh
$ ls "test A"
Project.toml
```

```sh
$ cat "test A/Project.toml"
id = "test A"
name = "test A"
```

```sh
$ cd "test A"
$ todo init
    Initializing `test A` project
Error: destination `${current_dir_path}/Project.toml` already exists
```

## Init project with manifest

```sh
$ cd "test B"
$ todo init --with-manifest
    Initializing `test B` project
```

```sh
$ ls "test B"
test B.manifest.md
```

````sh
$ cat "test B/test B.manifest.md"
# test B

```toml project
id = "test B"
name = "test B"
```
````

```sh
$ cd "test A"
$ todo init --with-manifest
    Initializing `test A` project
Error: destination `${current_dir_path}/Project.toml` already exists
```

```sh
$ cd "test B"
$ todo init --with-manifest
    Initializing `test B` project
Error: destination `${current_dir_path}/test B.manifest.md` already exists
```

```sh
$ cd "test B"
$ todo init
    Initializing `test B` project
Error: destination `${current_dir_path}/test B.manifest.md` already exists
```

# Init subproject outside project dir

## Prepare parent projects

```sh
$ todo new "test A"
    Creating `test A` project
```

```sh
$ todo new --with-manifest "test B"
    Creating `test B` project
```

```sh
$ mkdir "test A/test B"
$ mkdir "test B/test B"
```

## Init subproject default

```sh
$ todo init "test A/test B"
    Initializing `test B` project under `${current_dir_path}/test A`
```

```sh
$ ls "test A"
Project.toml test B
```

```sh
$ ls "test A/test B"
Project.toml
```

```sh
$ cat "test A/Project.toml"
id = "test A"
name = "test A"
```

```sh
$ cat "test A/test B/Project.toml"
id = "test B"
name = "test B"
```

```sh
$ todo init "test A/test B"
    Initializing `test B` project under `${current_dir_path}/test A`
Error: destination `${current_dir_path}/test A/test B/Project.toml` already exists
```

## Init subproject with manifest

```sh
$ todo init --with-manifest "test B/test B"
    Initializing `test B` project under `${current_dir_path}/test B`
```

```sh
$ ls "test B"
test B.manifest.md test B
```

```sh
$ ls "test B/test B"
test B.manifest.md
```

````sh
$ cat "test B/test B.manifest.md"
# test B

```toml project
id = "test B"
name = "test B"
```
````

````sh
$ cat "test B/test B/test B.manifest.md"
# test B

```toml project
id = "test B"
name = "test B"
```
````

```sh
$ todo init --with-manifest "test B/test B"
    Initializing `test B` project under `${current_dir_path}/test B`
Error: destination `${current_dir_path}/test B/test B/test B.manifest.md` already exists
```

```sh
$ todo init "test B/test B"
    Initializing `test B` project under `${current_dir_path}/test B`
Error: destination `${current_dir_path}/test B/test B/test B.manifest.md` already exists
```
