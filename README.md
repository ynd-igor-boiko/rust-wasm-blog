# Blog

Простая система блога: HTTP + gRPC сервер, клиентская библиотека, CLI и WASM-фронтенд.

## Что внутри

- `blog-server` — actix-web на :3000, tonic на :50051
- `blog-client` — библиотека для HTTP/gRPC
- `blog-cli` — консольный клиент
- `blog-wasm` — фронт для браузера

## Быстрый старт

```bash
# postgres
createdb blog_db

# .env в blog-server
DATABASE_URL=postgres://user:pass@localhost/blog_db
JWT_SECRET=какой-нибудь-длинный-секрет-минимум-32-символа

# запуск
cargo run -p blog-server
```

## CLI

```bash
# собрать
cargo build -p blog-cli --release

# использовать
./target/release/blog-cli register -u ivan -e ivan@test.com -p secret123
./target/release/blog-cli login -u ivan -p secret123
./target/release/blog-cli create -t "Первый пост" -c "Текст поста"
./target/release/blog-cli list
./target/release/blog-cli get 1
./target/release/blog-cli update 1 -t "Новый заголовок" -c "Новый текст"
./target/release/blog-cli delete 1

# через gRPC
./target/release/blog-cli --grpc list
```

Токен сохраняется в `~/.blog_token`.

## WASM

```bash
cd blog-wasm
wasm-pack build --target web
python3 -m http.server 8000
# открыть localhost:8000
```

## API

### Auth
- `POST /api/auth/register` — `{username, email, password}`
- `POST /api/auth/login` — `{username, password}`

### Posts
- `GET /api/posts?limit=10&offset=0` — список
- `GET /api/posts/:id` — один пост
- `POST /api/posts` — создать (нужен токен)
- `PUT /api/posts/:id` — обновить (нужен токен, только автор)
- `DELETE /api/posts/:id` — удалить (нужен токен, только автор)

Токен передавать в `Authorization: Bearer <token>`.

## curl примеры

```bash
# регистрация
curl -X POST localhost:3000/api/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"test","email":"test@test.com","password":"12345678"}'

# логин
curl -X POST localhost:3000/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"test","password":"12345678"}'

# создать пост
curl -X POST localhost:3000/api/posts \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer <token>' \
  -d '{"title":"Test","content":"Hello"}'

# список
curl localhost:3000/api/posts
```

## Стек

- actix-web, tonic, sqlx (postgres)
- JWT (jsonwebtoken), argon2
- clap, wasm-bindgen, gloo-net
