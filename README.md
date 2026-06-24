
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

Поэтому запускаем в docker.

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
## Настройка валидатора
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
## Собираем so для деплоя в валидатор
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
## Выполняем деплой в локальную тестовую сеть
```sh 
# solana airdrop 100
anchor deploy --program-name sol_usd_oracle --provider.cluster http://172.17.0.1:8899 # Idl account created: amr1tUCauWM2VkdzoTNqFC7FDSbUF1djuijqiE4andX
anchor deploy --program-name token_minter --provider.cluster http://172.17.0.1:8899 # Idl account created: GraXfBYuPthVgJQVa9JpL2cDxQSiutnFrAfWrtfPKcMx
```
## Запускаем js-тесты
```sh
yarn install
 yarn run ts-mocha -p ./tsconfig.json -t 1000000 "tests/**/*.ts"
```

## Инициалиазция аккаунтов
Через тест создаем аккаунты программ:
```sh
anchor test  --provider.cluster http://172.17.0.1:8899
```
Проверка аккаунтов:
```sh
 cat target/idl/sol_usd_oracle.json | jq '.instructions[] | {name: .name, accounts: .accounts[].name}'
cat target/idl/sol_usd_oracle.json | jq '.instructions[0]'
```
Тесты что-то не скоздают аккаунтов и PDA. 
Поэтому попробуем так:
```sh
npm install @coral-xyz/anchor@0.32.1
cd program
#node initialize_oracle.js
 RPC_URL=http://172.17.0.1:8899 node scripts/init-local.js
```
Результат:
```
Initializing oracle...
  tx: NS4Vdmh4VPvkSGmDuaNkrqTt9Ju9eNnRMfevvLSmqcaYG2rpeBsATTfuxoJhtV9AJ2MtQYVv2iEcwMoYGfT8ehD
Setting initial price...
  tx: 63H8y6v8jtwfkSxbPR6BrzHRfzkKEkpTNAR3tEBJ3yDVUdmXTqVT5cPfpEYbMsjumRg6aWGSzTzjhzEfHfqAuDrC
Initializing minter (treasury = wallet)...
  tx: 3UFe4NDTgcbu5b2fAS1sMMjP61VoYcX3dvBcLBM3F83LR47irjX68wM78WQgdhLt68fwJWwP7prZ5pW1joHU12yA
Done. Add to backend/.env:
ORACLE_STATE_PUBKEY=Athcok1p9jdYUrsVs4g4S2kAy5ZQe7jGHg49t3pHQUHr
```
## Запускаем бекженд
```sh
cd backend
. .env
cargo run
```
## Запускаем фронтэнд.
Воспользовавшись `npm run dev` получаем ошибки компиляции старого пакета `inotify`
```
npm error /root/.cache/node-gyp/24.16.0/include/node/v8-value.h:399:44: note:   candidate expects 1 argument, 0 provided
npm error make: *** [inotify.target.mk:111: Release/obj.target/inotify/src/bindings.o] Error 1
npm error gyp ERR! build error 
npm error gyp ERR! stack Error: `make` failed with exit code: 2
```

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

Получаем приватную часть ключа для baskpack:
```sh
python3 -m venv ./.venv
 chmod +x ./.venv/bin/activate
. ./.venv/bin/activate
pip install base58 solders
python3 ./cut_private_key.py 
```
Отминтим токен в UI:

![Результат минта](./artifacts/mod7_proof_test_net_ui.png)

Проверка информации о токене:
```sh
spl-token accounts
solana account GfFRvKjyHHn2GWJLyTwGkEa8a1PWnKMiNYLRipJVUM1B
```
![Проверка токена](./artifacts/mod7_proof_spl_token.png)

# Шаг 4-6: Деплой в devnet
## Перенастройка на devnet
Переключаемся на devnet:
```sh
solana config set --url https://api.devnet.solana.com 
```

Запросим sol  на https://faucet.solana.com/ для `867EYq4TfPM8SQjGoe7RW4j5DJzSG1kb31HwabcudRh5`.
И потом еще 5. Итого 10.
Деполим приложения в devnet:
```sh
pushd program
  anchor deploy --program-name sol_usd_oracle --provider.cluster devnet
  anchor deploy --program-name token_minter --provider.cluster devnet
  RPC_URL=https://api.devnet.solana.com node scripts/init-local.js
popd
```
Перезапускаем бекэнд для devnet:
```sh
pushd backend
  export SOLANA_RPC_HTTP=https://api.devnet.solana.com
  export SOLANA_RPC_WS=wss://api.devnet.solana.com
  cargo run
popd
```
Перезапускаем фронт для devnet:
```sh
pushd backend
  export SOLANA_RPC_HTTP=https://api.devnet.solana.com
  export SOLANA_RPC_WS=wss://api.devnet.solana.com
  cargo run
popd
```


Транзакция повисла. Проверим статус:
```sh
solana transaction-history $(solana address)
solana confirm -v 2ihhje5bPT1Y1wTyzsgDPQRPuLr5EZG7ZWYkq4Yb9zQHj5B95pJvwQ3oeT6wzVaEvsFJuWrZKhJh4xXaBCcWdJjM


solana transaction-history DJpBzQRWtuhnqFpjS9xVELb29wCkRNVeT7ZKnq8uHEyp
solana confirm -v 3K8yUFmhf2wCmvkNHH3Qm5AEfYTPLhymiUBDwN86Dujp2pyYCtpkGw5wA28Z2BRGa7KpTqgMVjryT7xaPsnaX55q


solana transaction-history Athcok1p9jdYUrsVs4g4S2kAy5ZQe7jGHg49t3pHQUHr
solana confirm -v CQyamQWBdv23uEQMrrW9b7m5kZcz9ntwKkwery1k6BcSTpHfieQVebM24MNX9oFR9c6o2uD4WafMAUVMBnDv5NB
```

Что-то не проходит.
Меняем адрес в backpack на `https://api.devnet.solana.com`.
Пробуем заминтится еще раз:
![Успешный минт](./artifacts/mod7_proof_mint_token_on_dev_net.png)

Адрес минта: `D6WZLez4yNwm4XmVZWbfqKNeJb5Wve27AbzRrnTnQDcR`
ID транзации: `4EdwrWJEosvdyyajbENHiYimwJiCpJ8kM6kifUQX2i9EWk75rf6o9ZYHHtQhKMFydEbVAmyc7arqummYJSFoRh2H`


пруф 1: https://explorer.solana.com/tx/4EdwrWJEosvdyyajbENHiYimwJiCpJ8kM6kifUQX2i9EWk75rf6o9ZYHHtQhKMFydEbVAmyc7arqummYJSFoRh2H?cluster=devnet

![Транзакция](./artifacts/mod7_transaction.png)

пруф 2: https://explorer.solana.com/address/D6WZLez4yNwm4XmVZWbfqKNeJb5Wve27AbzRrnTnQDcR?cluster=devnet
![токен](./artifacts/mod7_token.png)

## Минт токена 2 и 3
токен 2
https://explorer.solana.com/tx/67WdXb4CMcqNgrtki5g1x98ws43vefxvRQRFWdBFa8xZ9nU4q7gYVE7Svi86fDJndJcUjmGCfEcffjr2LeCAjzhs?cluster=devnet
https://explorer.solana.com/address/BuRhg5xBNG1S3Ctk8y5abMQcwr3R9CXCwSbCUBLxFhMY?cluster=devnet
токен 3
https://explorer.solana.com/tx/5Wekoov1jpuqEMDxTD94azi2M5DXttxB3Wrhfk414RXpm5iVL1NbRLRQGyUxP1fM1b19MTbjhC7oh1osZeCjYabW?cluster=devnet

К сожалениею api dev net solana плохо работает из под vpn и с местными провайдерами (
```
(.venv) root@f36117dbc305:/rust-solana-launchpad-task/program# spl-token accounts
Error: Error { request: Some(GetTokenAccountsByOwner), kind: Reqwest(reqwest::Error { kind: Request, url: "https://api.devnet.solana.com/", source: hyper_util::client::legacy::Error(Connect, ConnectError("dns error", Custom { kind: Uncategorized, error: "failed to lookup address information: Temporary failure in name resolution" })) }) }

```

<!-- https://emojio.ru/images/apple-b/1f921.png -->
