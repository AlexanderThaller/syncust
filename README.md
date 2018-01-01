# Syncust

Sync tool written in rust that can keep different folders in sync but also
supports a partial view on the data. So you can have an archive server tha keeps
all the data and a checkout on your local laptop that only has some of the
files. Its inspired by `syncthing` and `git annex`.

# Commands

```
syncust 0.1.0
Alexander Thaller <alexander.thaller@trivago.com>
Sync tool written in rust that can keep different folders in sync but also
supports a partial view on the data. So you can have an archive server tha keeps
all the data and a checkout on your local laptop that only has some of the
files. Its inspired by `syncthing` and `git annex`.

USAGE:
    syncust [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help
            Prints help information

    -V, --version
            Prints version information


OPTIONS:
    -L, --log_level <level>
            Loglevel to run under [env: SYNCUST_LOG_LEVEL=]  [default: info]  [values: trace, debug, info, warn, error]

    -R, --repo_path <path>
            Path to the repository that should be managed [default: .]


SUBCOMMANDS:
    clone
            Clone a remote repository

    drop
            Remove local data from the repository

    get
            Get data from a remote server

    help
            Prints this message or the help of the given subcommand(s)

    init
            Initialize a new syncfolder and add all existing files to the repository

    remote
            Add a remote repository to keep in sync with

    sync
            Add/Remove all files that are not yet tracked by syncust

    type
            Change the type of the local repository can be `auto` or `manual`

    watch
            Watch local and remote repositories and sync changes

```

# Goals

I like `git annex` especially the capability of having a "partial" checkout of
the data but still be able to view the structure of all the data. I dislike the
complexity that probably also stems that its based on git and I also want a bit
more performance as I have a lot of data in git annex. It should not have
feature parity with `git annex` as I only use a part of the features. I mostly
care about having multiple remotes so my data is backed up and I can have
locality (like with an external harddrive) and I want the partial checkout so I
don't olverload my local harddrive.

# Design

Probably I will have just a view of the data that is in the remote repositories
and the local repository that is saved and then just make it so that files get
synced around.
