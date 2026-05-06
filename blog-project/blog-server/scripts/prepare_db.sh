#!/bin/bash 
export POSTRES_WORKDIR=.postgres_workdir/pgdata
export LOG_DIR=$(realpath .logs)
export DATABASE_URL=${DATABASE_URL:-postgres://blog_admin:blog_pass@127.0.0.1:8432/r_blog_base}

psql --version 2>/dev/null | grep -q "14\." && echo "✅ psql 14 найден" || exit 1
mkdir -p $LOG_DIR
pkill -9 postgres | true
rm -rf $POSTRES_WORKDIR
echo ${POSTRES_WORKDIR}
/usr/lib/postgresql/14/bin/initdb -D $POSTRES_WORKDIR
cp -f blog-project/blog-server/scripts/* $POSTRES_WORKDIR/
/usr/lib/postgresql/14/bin/pg_ctl -D $POSTRES_WORKDIR -l $LOG_DIR/logfile_pg.log start
psql -d postgres -p 8432 -c "CREATE USER blog_admin WITH PASSWORD 'blog_pass'" -h /tmp | true;
psql -d postgres -p 8432 -c "CREATE DATABASE r_blog_base; " -h /tmp
psql -d postgres -p 8432 -c "GRANT ALL PRIVILEGES ON DATABASE r_blog_base TO blog_admin;" -h /tmp | true;

