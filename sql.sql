drop table player;
create table player (
	player_id serial primary key,
	username text unique,
	password text,
	created_on timestamp default now()
);