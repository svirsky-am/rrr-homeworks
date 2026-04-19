# PostgreSQL
```sh
sudo apt update
sudo apt install postgresql-14 postgresql-client-14
```
# Запуск и автозапуск
```sh
sudo systemctl start postgresql
#sudo systemctl enable postgresql
```
# Создать БД и задать пароль суперпользователю postgres
```sh
sudo -u postgres psql -c "CREATE DATABASE practice_db;"

sudo -u postgres psql -c "DROP DATABASE bank_api;"





export CUR_USER=$USER
sudo -u postgres psql -c "CREATE ROLE $CUR_USER WITH LOGIN;";
sudo -u postgres psql -c "ALTER ROLE $CUR_USER WITH SUPERUSER;"
sudo -u postgres psql -c "ALTER ROLE $CUR_USER CREATEDB;"
psql -c "CREATE USER blog_admin WITH PASSWORD 'blog_pass'";
sudo -u postgres psql -c "ALTER ROLE blog_admin CREATEDB;"

GPASSWORD=blog_pass psql -U blog_admin  -h localhost -d postgres -c "CREATE DATABASE bank_api;"

sudo systemctl reload postgresql


```
Запуск сервера с пересозданием базы:
```sh
make mod3_build_all
```

```sh 
export HOST=127.0.0.1
export PORT=8080
export JWT_SECRET=dev_super_secret_change_me_please
export  CORS_ORIGINS=http://localhost:3000
export DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/bank_api

```
Проверка 
Регистрация:
```sh
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "secure123"}'
```
Логин:
```sh 
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "secure123"}'

```
Получаем JWT-токен и Создание счёта с JWT:
```sh
export TOKEN=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJmODYyNjI0Yy1mZTJkLTQxOTgtOWJmYy03ODUyYWNiYzIwMWEiLCJleHAiOjE3NzY2Mzg3MzcsImlhdCI6MTc3NjYzNTEzN30.kmrpjgEhvmLyXEm-cpZ6VPc9hfD-2JrGfC7TftF8K8s
```
c# 4. Создание счёта с JWT
```sh
curl -X POST http://localhost:8080/accounts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"id": 5, "initial": 1000}'
# Ответ: {"id": 1}
```
# 5. Проверка баланса
```sh
curl -X GET http://localhost:8080/accounts/5 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN"
# Ответ: {"id": 1, "balance": 1000}
```
# 6. Попытка доступа без токена (должна вернуть 401)

```sh
curl -X GET http://localhost:8080/api/accounts/1
# Ответ: {"error": "missing bearer"} 
```