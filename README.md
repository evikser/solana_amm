# Solana AMM (Automated Market Maker)

## Run tests
```shell
cargo test-bpf -- --nocapture
```

## Accounts

`AMM data acccount` - аккаунт с данными AMM (x_amount, y_amount, x_mint, y_mint).

`X token vault`, `Y token vault` - аккаунты, которые хранят токены, используемые AMM.

`User account` - аккаунт пользователя, который подписывает транзакию.

`X token user account`, `Y token user account` - аккаунты пользователя с токенами.

`X token temp account`, `Y token temp account` - временные аккаунты с токенами, используемые для передачи токена в AMM.

## Diagram

![Exchange](/media/exchange.png)

## Exchange steps

1. Пользователь создаёт временный аккаунт и переводит туда `X токены`, которые хочет обменять.
2. Пользователь передаёт в AMM временный аккаунт, с которого заберут `X токены`, и аккаунт, на который хочет получить `Y токены`.
3. Действия AMM после вызова метода Exchange:
    1. AMM переводит `X токены` с временного аккаунта пользователя в `X token vault`.
    2. AMM считает количество `Y токенов`, которое нужно отправть пользователю для сохранения константы `K = X * Y`.
    3. AMM отправляет нужное количество токенов с `Y token vault` на указанный пользователем адрес.
    4. Данные о новом состоянии записываются в `AMM data account`.
