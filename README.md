# cafecoder-docker-rs

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