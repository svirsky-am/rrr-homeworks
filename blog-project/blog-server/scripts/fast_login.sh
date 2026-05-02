#!/usr/bin
# set -ex

curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "checker", "email": "user@example.com", "password": "secure123"}'


# 1. Логин и сохранение токена в переменную
export TOKEN_CLIENT=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "checker", "email": "user@example.com", "password": "secure123"}' \
  | jq -r '.token')

# 2. Проверка, что токен не пустой
if [ -z "$TOKEN_CLIENT" ] || [ "$TOKEN_CLIENT" = "null" ]; then
  echo "Ошибка: Токен не получен. Проверьте логин/пароль."
  # exit 1
fi
echo "Токен получен (первые 20 символов): ${TOKEN_CLIENT:0:20}..."

curl -v -X GET http://localhost:8080/api/debug \
  -H "Authorization: Bearer $TOKEN_CLIENT" 


#  3. Создание поста  
curl -v -X POST http://localhost:8080/api/posts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_CLIENT" \
  -d '{"title": "test_data", "content": "test_content" "initial": 1000}'


#  3. Просмотр постов 
curl -v -X GET http://localhost:8080/api/posts/list \
  -H "Authorization: Bearer $TOKEN_CLIENT"



# Работа с банковскими выписками

# # 3. Запрос создания счета
curl -v -X POST http://localhost:8080/api/protected/accounts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_CLIENT" \
  -d '{"id": 1, "initial": 1000}'



# 5. Проверка баланса
curl -X GET http://localhost:8080/api/protected/accounts/1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_CLIENT"


# 5. Проверка баланса
curl -X GET http://localhost:8080/api/protected/accounts/1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_CLIENT"



# curl -v -X get http://localhost:8080/api/test \
#   -H "Content-Type: application/json" \
#   -H "Authorization: Bearer $TOKEN_CLIENT" \
#   -d '{"initial": 1000}'