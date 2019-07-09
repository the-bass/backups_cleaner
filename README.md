# Backups Cleaner

Provides various strategies to free your storage from superfluous backups.

The project was last tested using `rustc 1.36.0 (a53f9df32 2019-07-03)`.

## Library

This repository contains a Rust library, that provides adapters for different storage providers, such as AWS S3, and various pruning strategies. You can open the documentation by running

```sh
cargo doc --open
```

## Command line utility

It also provides a ready-to-use command line utility, for pruning AWS S3 buckets using the `OlderThanButKeepOnePerMonth` strategy.

Run

```sh
cargo build --release
```

to build the utility. You can now find it at `target/release/prune_backups`.

Assuming your `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` are set, you can now begin pruning using

```sh
target/release/prune_backups \
    --region=eu-central-1 \
    --bucket=chav.com \
    --prefix=database_backups/ \
    --keep_all_within=14 \
    --one_per_month_within=1460
```

The above example considers all files in directory `database_backups/`, in bucket `chav.com`, in region `eu-central-1`.

## Development with Docker

From the root of this repository, bash into a container using

```sh
docker run -it --rm \
    -w /backups_cleaner \
    -v `pwd`:/backups_cleaner \
    -e AWS_ACCESS_KEY_ID=ABCDEFG \
    -e AWS_SECRET_ACCESS_KEY=1234567 \
    rust:1.36 \
    bash
```

to start developing.

Note, that you have to use a valid `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY`, if you want to test on real AWS S3 buckets.
