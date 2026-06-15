
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
docker run -it -v $(pwd)/rust-solana-launchpad-task:/rust-solana-launchpad-task -w /rust-solana-launchpad-task launchpad-runtime bash
 ```
