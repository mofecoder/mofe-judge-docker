# cafecoder-docker-rs

## docker
イメージ + タグ
```
cafecoder_docker:2104
```

build
```console
$ docker-compose build
```

start
```console
$ docker-compose up -d
```

## ioi/isolate サンドボックスの準備

[ioi/isolate](https://github.com/ioi/isolate) の指示に従って、サンドボックスのインストールをしてください。

- `apt install libcap-dev` が必要
- `default.cf` を設定したあとインストールする（設定したあと再度 `sudo make install` しても良い）

## 実行

```
cargo run
```

## テスト

テストはサイドエフェクトがあるため、`--test-threads=1` にしなければならない。

```command
cargo test -- --test-threads=1
```