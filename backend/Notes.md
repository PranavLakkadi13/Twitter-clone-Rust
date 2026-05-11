
docker compose up -d 

docker compose down -v

sqlx migrate run 

docker exec -it backend-postgres-1 psql -U postgres -d twitter 