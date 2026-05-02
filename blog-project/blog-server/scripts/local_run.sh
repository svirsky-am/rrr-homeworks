
#apt install postgresql-14 
#mkdir -p ./blog-project/blog-server/pgdata

#initdb -D ./blog-project/blog-server/pgdata


#pg_ctl -D ./blog-project/blog-server/pgdata -l ./blog-project/blog-server/pgdata/logfile.log start


/usr/lib/postgresql/14/bin/initdb -D ./blog-project/blog-server/pgdata

mkdir .logs
/usr/lib/postgresql/14/bin/pg_ctl -D ./blog-project/blog-server/pgdata -l .logs/logfile_pg.log start


psql -d postgres -c "CREATE DATABASE bank_api; " -h /tmp
