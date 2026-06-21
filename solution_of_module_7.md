
## Шаг 0. Подготовка репозитория
### Фикс совментимости glibc 
Шаблон для выполнения задания предполагает использоваиние  ubuntu 24/debian12, поэтому при выполнении `make build` возникает ошибка:
```
cd program && anchor build
/home/svirsky/.avm/bin/anchor-0.32.1: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.39' not found (required by /home/svirsky/.avm/bin/anchor-0.32.1)
make: *** [Makefile:18: build] Error 1
```
т.к., например, для ubuntu 22.04:
```
~/repos/hometask1_parser/rust-solana-launchpad-task$ ldd --version
ldd (Ubuntu GLIBC 2.35-0ubuntu3.13) 2.35
```

Поэтому запускаем в docker:
Соберем докер образ на базе ubuntu 24:

```sh
 docker image build rust-solana-launchpad-task/launchpad-image --tag launchpad-runtime
```
Запустим контейнер на базе полученного образа:
```sh
docker run -it -v -name launchpad $(pwd)/rust-solana-launchpad-task:/rust-solana-launchpad-task \
    -v ./rust-solana-launchpad-task/config_solana:/root/.config/solana  -w /rust-solana-launchpad-task \
    launchpad-runtime bash
```


<!-- Дополнительная сессия в контейнере 
```sh
#docker  -it -v $(pwd)/rust-solana-launchpad-task:/rust-solana-launchpad-task -w /rust-solana-launchpad-task launchpad-runtime bash

``` -->


# Шаг 1. Починить program
Обновили docker-файл , установив yarn - зависимости 
Внутри нового контейнера выполяем: 
```sh
cd program
anchor build
```
Для удобства работы с хоста выставим разрешениея для ключей приложения:
```sh
chmod 777 ./target/deploy/* -R 
```

С хоста проверяем:
```sh
solana-keygen pubkey rust-solana-launchpad-task/program/target/deploy/sol_usd_oracle-keypair.json
solana-keygen pubkey rust-solana-launchpad-task/program/target/deploy/token_minter-keypair.json
```
Подставляем `Program Id` и вычисляем PDA:
```sh
yarn add @solana/web3.js
yarn add @coral-xyz/anchor
node program/scripts/get-oracle-pda.cjs
# ORACLE_STATE_PUBKEY=Athcok1p9jdYUrsVs4g4S2kAy5ZQe7jGHg49t3pHQUHr
```
# Шаг 3. Подготовить свой контур деплоя
Печать токена в base64
```sh
docker exec -it launchpad-runtime bash
cat ~/.config/solana/id.json | node -e "console.log(Buffer.from(JSON.parse(require('fs').readFileSync(0))).toString('base64'))"
```
На хосте запускаем:
```sh
solana-test-validator 
```

Получаем адрес докер бриджа через traceroute:
```sh
traceroute 8.8.8.8 
```
Внутри контейнера указываем ip-адрес валидатора , развернутого на хосте:
Solana set config:
```sh
solana config set --url http://172.17.0.1:8899
```
Ключи уже закомичены. Приватный демо-ключ примонтирован в контейнер. Запросим sol на новом валидаторе:
```sh

# solana-keygen new -o /home/svirsky/.config/solana/id.json
solana airdrop 100
```
```sh 
docker exec -it launchpad bash 
```

Получаем ключи от приложений для сборки либ:
```sh 
solana address -k target/deploy/sol_usd_oracle-keypair.json
solana address -k target/deploy/sol_usd_oracle-keypair.json
```
Подменяем публичные ключи в *.so `rust-solana-launchpad-task/program/programs/token_minter/src/lib.rs`  и в `rust-solana-launchpad-task/program/programs/sol_usd_oracle/src/lib.rs` в секции `declare_id`:
```sh 
anchor build
#Program Id: DJpBzQRWtuhnqFpjS9xVELb29wCkRNVeT7ZKnq8uHEyp   token_minter
# Program Id: BkdrrqR4RrCB6m5YhMmWtjHpzuFPhdrvoA9x2tPFsWfj  sol_usd_oracle

```

Выполняем деплой в локальную тестовую сеть:
```sh 
anchor deploy --program-name sol_usd_oracle --provider.cluster http://172.17.0.1:8899 # Idl account created: amr1tUCauWM2VkdzoTNqFC7FDSbUF1djuijqiE4andX
anchor deploy --program-name token_minter --provider.cluster http://172.17.0.1:8899 # Idl account created: GraXfBYuPthVgJQVa9JpL2cDxQSiutnFrAfWrtfPKcMx
```
Запускаем js-тесты:
```sh
yarn install
 yarn run ts-mocha -p ./tsconfig.json -t 1000000 "tests/**/*.ts"
```


Через тест создаем аккаунты программ:
```sh

anchor test  --provider.cluster http://172.17.0.1:8899

```

Проверка аккаунтов:
```sh
 cat target/idl/sol_usd_oracle.json | jq '.instructions[] | {name: .name, accounts: .accounts[].name}'
cat target/idl/sol_usd_oracle.json | jq '.instructions[0]'

```

Но тесты, что-то не скоздают аккаунтов и PDA. Поэтому попробуем так:
Их создание:
```sh
npm install @coral-xyz/anchor@0.32.1
cd program
node initialize_oracle.js
```
Запускаем бекженд
```sh
cd backend
. .env
cargo run
```
Имеются ошибки компиляции старого пакета `inotify`
```
npm error /root/.cache/node-gyp/24.16.0/include/node/v8-value.h:399:44: note:   candidate expects 1 argument, 0 provided
npm error make: *** [inotify.target.mk:111: Release/obj.target/inotify/src/bindings.o] Error 1
npm error gyp ERR! build error 
npm error gyp ERR! stack Error: `make` failed with exit code: 2
```

Запускаем фронтэнд
 ```sh
cd frontend 
nvm install 22.13.1
nvm use 22.13.1
sudo rm -rf node_modules package-lock.json
rm -rf node_modules package-lock.json
rm -rf .cache .remix build
npm cache clean --force
npm install -g yarn 
yarn install
yarn run dev
 ```


# Шаг 4: Деплой в devnet
Переключаемся на devnet:
```sh

solana config set --url https://api.devnet.solana.com 
```


<!-- https://emojio.ru/images/apple-b/1f921.png -->